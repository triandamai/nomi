use crate::AppState;
use crate::common::identity;
use crate::common::repository::channel_repo;
use crate::feature::conversation::command::{
    get_help_command, process_generate_pairing, process_login, process_pairing, process_register,
};
use crate::feature::message_processor::v2_orchestrator::process_v2_message;
use crate::feature::{
    Conversation, FallBackPayload, InboundMessage, MessageSource, OutboundMessage, UnifiedMessage,
};
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
    let mut resolved_external_chat_id = if msg.is_group {
        msg.sender_id.clone()
    } else {
        msg.conversation_id.clone()
    };

    // ======== WhatsApp ID Strategy Refinement ===========//
    if msg.channel.starts_with("whatsapp") {
        // 1. Clean LID/JID (remove :xx if present)
        if let Some((id, rest)) = msg.sender_id.split_once(':') {
            if let Some((_, domain)) = rest.split_once('@') {
                msg.sender_id = format!("{}@{}", id, domain);
            }
        }

        // 2. Extract phone-based ID (phone@s.whatsapp.net) from original_meta
        let phone_id = msg
            .original_meta
            .as_ref()
            .and_then(|meta| meta.get("source"))
            .and_then(|source| source.get("sender_alt"))
            .and_then(|sender_alt| {
                let phone = sender_alt.get("user")?.as_str()?;
                let server = sender_alt.get("server")?.as_str()?;
                if phone.is_empty() || server.is_empty() {
                    None
                } else {
                    Some(format!("{}@{}", phone, server))
                }
            });

        // 3. For private chats, use phone_id as conversation_id for outbound
        if let Some(pid) = phone_id {
            resolved_external_chat_id = pid.clone();
            if !msg.is_group {
                msg.conversation_id = pid;
            }
        }
    }

    if msg.sender_id.contains(":") {
        info!("User {} still has ':' after cleaning, skipping", msg.sender_id);
        return Ok(());
    }

    info!(
        "INCOMING sender: {} chat: {}",
        msg.sender_id, msg.conversation_id
    );

    // 1. Resolve Identity and Channel Info EARLY (Needed for registration and thresholds)
    let (conversation_id, cumulative_tokens, max_token_usage) = if msg.is_group {
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

    // ======== Group Filtering & Registration Check ========//
    if msg.is_group {
        let registered = !conversation_id.is_nil();

        info!("group registered status {}", registered);
        if !registered {
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
    if text.to_uppercase().starts_with("/pair ") {
        return process_pairing(&state, &msg, &text).await;
    } else if text.starts_with("/linkapp") {
        return process_generate_pairing(&state, &msg).await;
    } else if text.starts_with("/login") {
        return process_login(&state, &msg).await;
    } else if text.to_lowercase() == "/help" {
        return get_help_command(&state, &msg).await;
    }

    // 2. Fetch Conversation Data (Including Thresholds)
    let conv_info = if !conversation_id.is_nil() {
        crate::common::repository::conversation_repo::get_conversation_info(
            &state.pool,
            &state.redis,
            conversation_id,
        )
        .await
        .ok()
    } else {
        None
    };

    let mut is_ambient = false;

    // ======== Interaction Gate & Ambient Soul (Pre-Filtering for Groups) ========//
    if msg.is_group && !msg.is_mentioned && !conversation_id.is_nil() {
        let gate = crate::services::interaction_gate::InteractionGateService::new(
            state.pool.clone(),
            state.gemini_api_key.clone(),
        );

        let mut is_reply_to_nomi = false;
        if let Some(q) = &msg.quoted_message {
            let quoted_msg_role: Option<String> = sqlx::query_scalar!(
                "SELECT role FROM messages WHERE conversation_id = $1 AND content = $2 LIMIT 1",
                conversation_id,
                q.text
            )
            .fetch_optional(&state.pool)
            .await
            .unwrap_or(None);

            if let Some(role) = quoted_msg_role {
                if role == "assistant" {
                    is_reply_to_nomi = true;
                    info!("Interaction Gate (Redis): Detected reply to Nomi (assistant message)");
                }
            }
        }

        let default_thresholds = json!({});
        let thresholds = conv_info.as_ref().map(|c| &c.gateway_thresholds).unwrap_or(&default_thresholds);
        match gate.should_respond_to_group_message(conversation_id, &text, is_reply_to_nomi, thresholds).await {
            Ok(true) => {
                info!("Interaction Gate: Passed, processing as active participation.");
            }
            Ok(false) => {
                info!(
                    "Interaction Gate: Dropping ambient message in group {}. Queuing for Ambient Soul.",
                    msg.conversation_id
                );
                is_ambient = true;
            }
            Err(e) => {
                error!("Interaction Gate: Error during evaluation: {}. Falling back.", e);
            }
        }
    }

    // Token Usage Guard
    if cumulative_tokens.to_i64().unwrap_or(0) >= max_token_usage.to_i64().unwrap_or(700000) {
        info!("Conversation {} has reached max token usage", conversation_id);
        let _ = state.publish_outbond(&OutboundMessage {
            is_group: msg.is_group,
            sender_id: "nomi_auth".to_string(),
            conversation_id: msg.conversation_id.clone(),
            text: format!("⚠️ **Token Usage Limit Reached** ⚠️\n\nThis conversation has reached its maximum token usage limit of **{:.0}** tokens.\n\nPlease contact Trian to increase the limit. ✨", max_token_usage),
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

    if conversation_id.is_nil() {
        info!("Conversation not found for {} via {}. Requesting registration.", msg.conversation_id, msg.channel);
        let _ = state.publish_outbond(&OutboundMessage {
            is_group: msg.is_group,
            sender_id: "nomi_auth".to_string(),
            conversation_id: msg.conversation_id.clone(),
            text: format!("Hello there! 👋 \n\nI'm **Nomi**, Trian's AI collaborator. To get started, please use one of these commands:\n\n**`/register`** — New user profile\n**`/login`** — Existing profile\n**`/help`** — Get assistance\n\nIt’s a pleasure to meet you! ✨"),
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

    // 3. Resolve Identity
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
        &resolved_external_chat_id,
        &msg.channel,
        msg.is_group,
        display_name.clone(),
    )
    .await;
    
    if let Err(err) = &user_id {
        info!("User resolution failed: {}", err);
        return Ok(());
    }
    let user_id = user_id?;
    info!("[USER]{}", user_id);

    let conv_info = match conv_info {
        Some(ci) => ci,
        None => return Err(anyhow::anyhow!("Workspace Conversation doesn't exist")),
    };

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
                "telegram" => MessageSource::Telegram { name: channel.clone() },
                "whatsapp" => MessageSource::WhatsApp { name: channel.clone() },
                _ => MessageSource::Other { name: channel.clone() },
            },
            quoted_message: msg.quoted_message,
            reply_to_id: None,
            v2: true,
        };

        let map_convo = Conversation::from(conv_info);

        if is_ambient {
            info!("Processing as AMBIENT SOUL message");
            
            // Save the incoming ambient user message to the database so Nomi does not lose context!
            let mut quoted_metadata = None;
            if let Some(q) = &unified_msg.quoted_message {
                let mut q_with_name = json!(q);
                let quoted_sender_name: Option<String> = sqlx::query_scalar!(
                    "SELECT u.display_name FROM users u JOIN channels c ON c.user_id = u.id WHERE c.external_id = $1 LIMIT 1",
                    q.sender_id
                )
                .fetch_optional(&state_clone.pool)
                .await
                .unwrap_or(None)
                .flatten();

                if let Some(name) = quoted_sender_name {
                    if let Some(obj) = q_with_name.as_object_mut() {
                        obj.insert("display_name".to_string(), json!(name));
                    }
                }
                quoted_metadata = Some(json!({ "quoted_message": q_with_name }));
            }

            let save_res = crate::common::repository::message_repo::save_message(
                &state_clone.pool,
                conversation_id,
                "user",
                &user_text,
                None,
                Some(user_id.id),
                0,
                0,
                0,
                unified_msg.image_url.clone(),
                unified_msg.video_url.clone(),
                unified_msg.audio_url.clone(),
                unified_msg.doc_url.clone(),
                unified_msg.sticker_url.clone(),
                quoted_metadata,
                unified_msg.reply_to_id,
                Some(&state_clone.redis),
            ).await;

            // Notify connected dashboard clients in real-time even for ambient messages!
            if let Ok(ref saved_msg) = save_res {
                let members = sqlx::query!(
                    "SELECT m.user_id FROM conversation_members as m WHERE m.conversation_id = $1",
                    conversation_id
                )
                .fetch_all(&state_clone.pool)
                .await
                .unwrap_or(Vec::new());

                for member in members.iter().map(|v| v.user_id) {
                    info!("notify user message saved (ambient) :{:?}", member);
                    let mut sse_msg = saved_msg.clone();
                    sse_msg.display_name = unified_msg.display_name.clone();
                    let _ = state_clone
                        .dispatch(crate::services::event_dispatcher::AppEvent::user(
                            member.to_string().as_str(),
                            "message",
                            sse_msg.to_sse_json(0),
                        ))
                        .await;
                }
            } else if let Err(e) = save_res {
                error!("Failed to save ambient user message to database: {}", e);
            }

            let ambient_soul = crate::services::ambient_soul::AmbientSoulService::new(
                state_clone.pool.clone(),
                state_clone.redis.clone(),
                state_clone.gemini.clone(),
                state_clone.gemini_api_key.clone(),
            );

            if let Err(e) = ambient_soul.process_ambient_memory(user_id.id, conversation_id, &user_text).await {
                error!("Ambient Soul Memory processing failed: {}", e);
            }

            match ambient_soul.evaluate_initiative(conversation_id, &user_text, 1.0).await {
                Ok(initiative) => {
                    if let Some(proactive_text) = initiative.response_text {
                        info!("Ambient Soul: Proactive initiative triggered!");
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
                            None,
                            None,
                            Some(&state_clone.redis),
                        ).await;

                        let _ = state_clone.publish_outbond(&OutboundMessage {
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
                        }).await;
                    }
                }
                Err(e) => error!("Ambient Soul Initiative evaluation failed: {}", e),
            }
        } else {
            if let Err(e) = process_v2_message(state_clone, map_convo, unified_msg).await {
                error!("Failed to process inbound message: {}", e);
            }
        }
    });

    Ok(())
}
