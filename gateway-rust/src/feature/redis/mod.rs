use crate::AppState;
use crate::common::identity;
use crate::common::repository::channel_repo;
use crate::common::repository::channel_repo::is_group_registered;
use crate::feature::conversation::command::{
    get_help_command, process_generate_pairing, process_login, process_pairing, process_register,
};
use crate::feature::message_processor::v2_orchestrator::process_v2_message;
use crate::feature::{
    Conversation, FallBackPayload, InboundMessage, MessageSource, OutboundMessage, UnifiedMessage,
};
use crate::services::event_dispatcher::AppEvent;
use rust_decimal::prelude::ToPrimitive;
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

    if msg.channel.starts_with("whatsapp") {
        let sender_chat_id = msg
            .original_meta
            .clone()
            .map_or(msg.sender_id.clone(), |original| {
                original
                    .get("source")
                    .map_or(msg.sender_id.clone(), |source| {
                        source
                            .get("sender_alt")
                            .map_or(msg.sender_id.clone(), |sender_alt| {
                                let phone = sender_alt
                                    .get("user")
                                    .map_or("", |v| v.as_str().unwrap_or(""));
                                let server = sender_alt
                                    .get("server")
                                    .map_or("", |v| v.as_str().unwrap_or(""));
                                if phone.is_empty() || server.is_empty() {
                                    msg.sender_id.clone()
                                } else {
                                    format!("{}@{}", phone, server)
                                }
                            })
                    })
            });

        msg.sender_id = sender_chat_id.clone();
        if !msg.is_group {
            msg.conversation_id = sender_chat_id;
        }
    }

    info!(
        "INCOMING sender: {} chat: {}",
        msg.sender_id, msg.conversation_id
    );
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

    let mut is_ambient = false;

    // ======== Interaction Gate & Ambient Soul (Pre-Filtering for Groups) ========//
    if msg.is_group && !msg.is_mentioned {
        let gate = crate::services::interaction_gate::InteractionGateService::new(
            state.pool.clone(),
            state.gemini_api_key.clone(),
        );

        match gate.should_respond_to_group_message(&text, false).await {
            Ok(true) => {
                info!("Interaction Gate: Passed, processing as active participation.");
                // Let it flow through as a normal message
            }
            Ok(false) => {
                info!(
                    "Interaction Gate: Dropping ambient message in group {}. Queuing for Ambient Soul.",
                    msg.conversation_id
                );
                is_ambient = true;
                // We don't return early here; we let it process minimally for ambient memory,
                // but we will flag it so it doesn't trigger the main orchestrator loop.
            }
            Err(e) => {
                error!(
                    "Interaction Gate: Error during evaluation: {}. Continuing as fallback.",
                    e
                );
            }
        }
    }

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
    if cumulative_tokens.to_i64().unwrap_or(0) >= max_token_usage.to_i64().unwrap_or(700000) {
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
        let _ = state.publish_outbond(&OutboundMessage {
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
        Some(meta) => meta.get("display_name").map_or_else(
            || msg.sender_id.clone(),
            |v| v.as_str().unwrap_or("").to_string(),
        ),
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
    info!("[USER]{}", user_id);
    let conv_info = sqlx::query!(
        "SELECT
                id,
                title,
                conversation_type,
                user_id,
                created_at,
                updated_at,
                soul_content,
                bootstrap_content
            FROM conversations
            WHERE id = $1",
        conversation_id
    )
    .fetch_one(&state.pool)
    .await;
    if let Err(err) = conv_info {
        info!("Conversation not exist:{}", err);
        let error_msg = "Workspace Conversation doesn exist".to_string();
        let _ = state
            .dispatch(AppEvent::user(
                user_id.id.to_string().as_str(),
                "error",
                json!({
                    "conversation_id": conversation_id,
                    "message": error_msg,
                }),
            ))
            .await;

        return Ok(());
    }

    let conv_info = conv_info?;

    // ================================== REGULAR CONVO ============================//
    // 5. Trigger Agentic Loop
    let state_clone = state.clone();
    let user_text = msg.text.clone();
    let channel = msg.channel.clone();
    let image_url = msg.image_url.clone();
    let audio_url = msg.audio_url.clone();
    let video_url = msg.video_url.clone();
    let document_url = msg.doc_url.clone();
    let sticker_url = msg.sticker_url.clone();

    tokio::spawn(async move {
        // B. Unified Processing (Contextual Image Classification if image present)
        let unified_msg = UnifiedMessage {
            is_group: msg.is_group,
            is_mentioned: msg.is_mentioned,
            display_name: Some(display_name),
            conversation_id,
            user_id: Some(user_id.id.clone()),
            text_content: user_text.clone(),
            image_url,
            audio_url,
            video_url,
            sticker_url,
            doc_url: document_url,
            source: match channel.as_str() {
                "telegram" => MessageSource::Telegram {
                    name: channel.clone(),
                },
                "whatsapp" => MessageSource::WhatsApp {
                    name: channel.clone(),
                },
                _ => MessageSource::Other {
                    name: channel.clone(),
                },
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

        if is_ambient {
            info!("Processing as AMBIENT SOUL message");
            let ambient_soul = crate::services::ambient_soul::AmbientSoulService::new(
                state_clone.pool.clone(),
                state_clone.redis.clone(),
                state_clone.gemini.clone(),
                state_clone.gemini_api_key.clone(),
            );

            // 1. Process Ambient Memory
            if let Err(e) = ambient_soul
                .process_ambient_memory(user_id.id.clone(), conversation_id, &user_text)
                .await
            {
                error!("Ambient Soul Memory processing failed: {}", e);
            }

            // 2. Evaluate Initiative (Proactive interaction)
            // We use a dummy interaction score of 1.0 for testing, in a real scenario
            // this would come from the IntentClassifier or InteractionGate
            match ambient_soul
                .evaluate_initiative(conversation_id, &user_text, 1.0)
                .await
            {
                Ok(initiative) => {
                    if let Some(proactive_text) = initiative.response_text {
                        info!("Ambient Soul: Proactive initiative triggered!");
                        // Save the proactive message to DB
                        let _ = crate::common::repository::message_repo::save_message(
                            &state_clone.pool,
                            conversation_id,
                            "assistant",
                            &proactive_text,
                            None,
                            None,
                            initiative.tokens.input_tokens as i32,
                            initiative.tokens.output_tokens as i32,
                            initiative.tokens.total_tokens as i32,
                            None,
                            None,
                            None,
                            None,
                            None,
                        )
                        .await;

                        // Send back to the channel
                        let _ = state_clone
                            .publish_outbond(&OutboundMessage {
                                is_group: true,
                                sender_id: "nomi_ambient".to_string(),
                                conversation_id: conversation_id.to_string(),
                                text: proactive_text,
                                channel: channel.clone(),
                                video_url: None,
                                image_url: None,
                                audio_url: None,
                                doc_url: None,
                                sticker_url: None,
                                metadata: None,
                            })
                            .await;
                    }
                }
                Err(e) => error!("Ambient Soul Initiative evaluation failed: {}", e),
            }
        } else {
            if let Err(e) = process_v2_message(state_clone, map_convo, unified_msg).await {
                error!("Failed to process inbound message: {}", e.backtrace());
            }
        }
    });

    Ok(())
}
