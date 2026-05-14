use crate::AppState;
use crate::common::identity;
use crate::common::repository::channel_repo;
use crate::common::repository::channel_repo::is_group_registered;
use crate::feature::conversation::command::{
    get_help_command, process_generate_pairing, process_login, process_pairing, process_register,
};
use crate::feature::message_processor::model::{MessageSource, UnifiedMessage};
use crate::feature::message_processor::v2_orchestrator::process_v2_message;
use crate::feature::{FallBackPayload, InboundMessage, OutboundMessage};
use serde_json::json;
use tokio_stream::StreamExt;
use tracing::{error, info};
use uuid::Uuid;

pub async fn start_redis_listener(state: AppState) -> anyhow::Result<()> {
    let mut pubsub = state.redis.get_pubsub().await?;
    pubsub.subscribe("nomi:inbound").await?;
    pubsub.subscribe("nomi:channel-fallback").await?;

    info!("Redis listener started for nomi:inbound");

    let mut stream = pubsub.on_message();

    while let Some(msg) = stream.next().await {
        match msg.get_channel_name().to_string().as_str() {
            "nomi:channel-fallback" => {
                let payload: String = msg.get_payload()?;
                let fallback: FallBackPayload = match serde_json::from_str(&payload) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Failed to parse inbound message: {}", e);
                        continue;
                    }
                };
                info!("incoming nomi:channel-fallback \n data:{:?}\n", fallback);
            }
            "nomi:inbound" => {
                let payload: String = msg.get_payload()?;
                let inbound: InboundMessage = match serde_json::from_str(&payload) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Failed to parse inbound message: {}", e);
                        continue;
                    }
                };

                info!("inbound \n data:{:?}\n", inbound);
                let state_clone = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_inbound_message(state_clone, inbound).await {
                        error!("Error handling inbound message: {}", e);
                    }
                });
            }
            _ => {
                info!("unknown channel {}", msg.get_channel_name());
            }
        }
    }

    Ok(())
}

async fn handle_inbound_message(state: AppState, msg: InboundMessage) -> anyhow::Result<()> {
    info!(
        "Handling inbound from {} in chat {}: {}",
        msg.sender_id, msg.conversation_id, msg.text
    );

    let text = msg.text.trim();

    // 1. Group Filtering & Registration Check
    if msg.is_group {
        let registered = is_group_registered(&state.pool, &msg.conversation_id, &msg.channel).await;

        info!("group registered status {}", registered);
        if !registered {
            // Only allow registration command in unregistered groups
            if text.starts_with("/register") {
                return process_register(&state, &msg).await;
            }
            info!(
                "Group {} not registered, ignoring message",
                msg.conversation_id
            );
            return Ok(());
        }
    }

    // 2. Resolve Identity and Channel Info
    let conversation_id = if msg.is_group {
        // For groups, we look up the channel_group table instead of the regular channels table
        match channel_repo::get_channel_group_info(&state.pool, &msg.channel, &msg.conversation_id)
            .await
        {
            Ok(value) => value.map_or_else(|| Uuid::nil(), |v| v.conversation_id),
            Err(_) => Uuid::nil(),
        }
    } else {
        match channel_repo::get_channel_info(&state.pool, &msg.channel, &msg.conversation_id).await
        {
            Ok(value) => value.map_or_else(|| Uuid::nil(), |v| v.conversation_id),
            Err(_) => Uuid::nil(),
        }
    };

    info!("Conversation id {}",conversation_id);
    let display_name = match &msg.metadata {
        None => msg.sender_id.clone(),
        Some(meta) => meta
            .get("display_name")
            .map_or_else(|| msg.sender_id.clone(), |v| v.to_string()),
    };
    let user_id = identity::resolve_identity(
        &state,
        &msg.sender_id,
        &msg.conversation_id,
        &msg.channel,
        msg.is_group.clone(),
        display_name,
    )
    .await;
    if let Err(err) = &user_id {
        info!("User not exist:{}", err);
    }

    let user_id = user_id.unwrap().id;
    // ================================== BEGIN COMMAND ============================//
    // 3. Check for Pairing/Register/Login
    if text.to_uppercase().starts_with("/pair ") {
        return process_pairing(&state, &msg, text, user_id.clone()).await;
    } else if text.starts_with("/linkapp") {
        return process_generate_pairing(&state, &msg, user_id.clone()).await;
    } else if text.starts_with("/register") {
        return process_register(&state, &msg).await;
    } else if text.starts_with("/login") {
        return process_login(&state, &msg).await;
    } else if text.starts_with("/help") {
        return get_help_command(&state, &msg).await;
    }

    // ================================== NOT REGISTERED STOP HERE ============================//
    // 3. Resolve/Create Conversation
    if conversation_id.is_nil() {
        info!("{} via {}", msg.conversation_id, msg.channel);
        info!(
            "Unfortunately user doesnt associate with any conversation, stop here will not sent to llm"
        );
        let _ = state.publish_outbond(&OutboundMessage{
            is_group: msg.is_group,
            sender_id: "nomi_auth".to_string(),
            conversation_id: msg.conversation_id.clone(),
            text: format!("Hello there! 👋 \n
            I'm **Nomi**, Trian's AI collaborator. I help him manage his projects, track his adventures on the road, and keep his digital ecosystem running smoothly. \n
            If you're a friend of Trian's, I'd love to get to know you! To get started and secure your access to our chat, could you please use one of the commands below?\n
                {} — If this is your first time chatting with me, use this to set up your profile. \n
                {} — If we've spoken before, use this to jump right back into our conversation.\n
                {} — Get help?.\n
            It’s a pleasure to meet you, and I look forward to assisting you once you're signed in! ✨\n",
                "**`/register`**",
                "**`/login`**",
                "**`/help`**",
            ),
            channel: msg.channel.clone(),
            video_url: None,
            image_url: None,
            audio_url: None,
            doc_url: None,
            sticker_url: None,
            metadata: msg.metadata.clone(),
        });
        return Ok(());
    }

    // ================================== REGULAR CONVO ============================//
    // 5. Trigger Agentic Loop
    let state_clone = state.clone();
    let user_text = msg.text.clone();
    let sender_id = msg.sender_id.clone();
    let chat_id = msg.conversation_id.clone();
    let channel = msg.channel.clone();
    let image_url = msg.image_url.clone();
    let audio_url = msg.audio_url.clone();
    let video_url = msg.video_url.clone();
    let document_url = msg.doc_url.clone();
    let sticker_url = msg.sticker_url.clone();

    tokio::spawn(async move {
        // A. Presence
        let _ = state_clone
            .send_presence_to_user(
                user_id.to_string().as_str(),
                json! ({
                    "conversation_id": conversation_id,
                    "is_typing": true,
                    "user_id": "nomi"
                }),
                &crate::feature::PresenceMessage {
                    sender_id: sender_id.clone(),
                    chat_id: chat_id.clone(),
                    channel: channel.clone(),
                    status: "typing".to_string(),
                },
            )
            .await;

        // B. Unified Processing (Contextual Image Classification if image present)
        let unified_msg = UnifiedMessage {
            is_group: msg.is_group,
            is_mentioned: msg.is_mentioned,
            conversation_id,
            user_id: Some(user_id),
            text_content: user_text,
            image_url,
            audio_url,
            video_url,
            sticker_url,
            doc_url: document_url,
            source: match msg.channel.as_str() {
                "telegram" => MessageSource::Telegram { name: msg.channel },
                "whatsapp" => MessageSource::WhatsApp { name: msg.channel },
                _ => MessageSource::Other { name: msg.channel },
            },
            v2: true,
        };

        if let Err(e) = process_v2_message(state, unified_msg).await {
            error!("Failed to process inbound message: {}", e);
        }
    });

    Ok(())
}
