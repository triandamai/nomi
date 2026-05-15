use crate::common::identity;
use crate::common::repository::channel_repo;
use crate::common::repository::channel_repo::is_group_registered;
use crate::feature::conversation::command::{
    get_help_command, process_generate_pairing, process_login, process_pairing, process_register,
};
use crate::feature::conversation::model::ChatStreamChunk;
use crate::feature::message_processor::v2_orchestrator::process_v2_message;
use crate::feature::{FallBackPayload, InboundMessage, MessageSource, OutboundMessage, UnifiedMessage};
use crate::models::Conversation;
use crate::AppState;
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
                info!("incoming nomi:channel-fallback \n data:{}\n", fallback);
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

                info!("inbound \n data:{}\n", inbound);
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

async fn handle_inbound_message(state: AppState, mut msg: InboundMessage) -> anyhow::Result<()> {
    info!(
        "Handling inbound from {} in chat {}: {}",
        msg.sender_id, msg.conversation_id, msg.text
    );

    // ======== WA CHANNEL GUARD ===========//
    let mut text = msg.text.trim().to_string();
    if text.contains("@42078516064356") {
        info!("native mentioned detected {}", text);
        text = text.replace("@42078516064356", "Nomi");
        msg.is_mentioned = true;
    }
    if let Some((id, rest)) = msg.sender_id.split_once(':') {
        if let Some((_, domain)) = rest.split_once('@') {
            let clean_id = format!("{}@{}", id, domain);
            msg.sender_id = clean_id;
        }
    }

    if msg.sender_id.contains(":") {
        info!("User {} has ':' skiped", msg.sender_id);
        return Ok(());
    }
    // ======== END WA CHANNEL GUARD ===========//

    // ======== Group Filtering & Registration Check ========//
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


    // ================================== BEGIN COMMAND ============================//
    // 3. Check for Pairing/Register/Login
    if text.to_uppercase().starts_with("/pair ") {
        return process_pairing(&state, &msg, &text).await;
    } else if text.starts_with("/linkapp") {
        return process_generate_pairing(&state, &msg).await;
    } else if text.starts_with("/register") {
        return process_register(&state, &msg).await;
    } else if text.starts_with("/login") {
        return process_login(&state, &msg).await;
    } else if text.starts_with("/help") {
        return get_help_command(&state, &msg).await;
    }

    // ================================== NOT REGISTERED STOP HERE ============================//


    // 2. Resolve Identity and Channel Info
    let (conversation_id, cumulative_tokens, max_token_usage) = if msg.is_group {
        // For groups, we look up the channel_group table instead of the regular channels table
        match channel_repo::get_channel_group_info(&state.pool, &msg.channel, &msg.conversation_id)
            .await
        {
            Ok(value) => value.map_or_else(
                || (Uuid::nil(), 0, 700000.0),
                |v| (v.conversation_id, v.cumulative_tokens, v.max_token_usage),
            ),
            Err(_) => (Uuid::nil(), 0, 700000.0),
        }
    } else {
        match channel_repo::get_channel_info(&state.pool, &msg.channel, &msg.conversation_id).await
        {
            Ok(value) => value.map_or_else(
                || (Uuid::nil(), 0, 700000.0),
                |v| (v.conversation_id, v.cumulative_tokens, v.max_token_usage),
            ),
            Err(_) => (Uuid::nil(), 0, 700000.0),
        }
    };
    info!("Conversation id {}", conversation_id);
    if cumulative_tokens as f64 >= max_token_usage {
        info!(
            "Conversation {} has reached max token usage",
            conversation_id
        );
        let _ = state.publish_outbond(&OutboundMessage {
            is_group: msg.is_group,
            sender_id: "nomi_auth".to_string(),
            conversation_id: msg.conversation_id.clone(),
            text: format!("⚠️ **Token Usage Limit Reached** ⚠️\n\nThis conversation has reached its maximum token usage limit of **{:.0}** tokens (Current: **{}**).\n\nPlease contact Trian to increase the limit or start a new conversation. Thank you!", max_token_usage, cumulative_tokens),
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

    // ============== START AUTHENTICATED USER ============= /
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
        display_name.clone(),
    )
    .await;
    if let Err(err) = &user_id {
        info!("User not exist:{}", err);
        return Ok(());
    }
    let user_id = user_id?;
    let conv_info = sqlx::query!("SELECT * FROM conversations WHERE id = $1", conversation_id)
        .fetch_optional(&state.pool)
        .await;

    if let Ok(c) = &conv_info {
        if let None = c {
            let error_msg = format!(
                "Token limit reached ({}/{:.0})",
                cumulative_tokens, max_token_usage
            );
            let _ = state
                .send_sse_to_user(
                    user_id.to_string().as_str(),
                    "message",
                    json!(ChatStreamChunk {
                        content: "No workspace detected.".to_string(),
                        thought: "".to_string(),
                        code_block: "".to_string(),
                        tool_call: None,
                        prompt_tokens: 0,
                        answer_tokens: 0,
                        total_tokens: 0,
                        finish_reason: Some("error".to_string()),
                        error: Some(error_msg.clone()),
                    }),
                )
                .await;

            return Ok(());
        }
    }
    let conv_info = conv_info?.unwrap();

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
            user_id: Some(user_id.id.clone()),
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

        let map_convo = Conversation {
            id: conv_info.id,
            session_id: conv_info.user_id,
            title: conv_info.title,
            soul_content: conv_info.soul_content,
            bootstrap_content: conv_info.bootstrap_content,
            created_at: conv_info.created_at,
            updated_at: conv_info.updated_at,
        };
        if let Err(e) = process_v2_message(state, map_convo, unified_msg).await {
            error!("Failed to process inbound message: {}", e);
        }
    });

    Ok(())
}
