use crate::AppState;
use crate::common::identity;
use crate::common::repository::{channel_repo, message_repo, pairing_repo};
use crate::feature::{InboundMessage, OutboundMessage};
use rand::RngExt;
use serde_json::json;
use tokio_stream::StreamExt;
use tracing::{error, info};
use uuid::Uuid;

pub async fn start_redis_listener(state: AppState) -> anyhow::Result<()> {
    let mut pubsub = state.redis.get_pubsub().await?;
    pubsub.subscribe("nomi:inbound").await?;

    info!("Redis listener started for nomi:inbound");

    let mut stream = pubsub.on_message();

    while let Some(msg) = stream.next().await {
        let payload: String = msg.get_payload()?;
        let inbound: InboundMessage = match serde_json::from_str(&payload) {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to parse inbound message: {}", e);
                continue;
            }
        };

        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_inbound_message(state_clone, inbound).await {
                error!("Error handling inbound message: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_inbound_message(state: AppState, msg: InboundMessage) -> anyhow::Result<()> {
    info!(
        "Handling inbound from {} in chat {}: {}",
        msg.sender_id, msg.conversation_id, msg.text
    );

    // 1. Resolve Identity and Channel Info
    let channel_info =
        channel_repo::get_channel_info(&state.pool, &msg.channel, &msg.conversation_id).await?;

    let (user_id, external_conversation_id) = if let Some(ci) = channel_info {
        (ci.user_id, ci.conversation_id)
    } else {
        let identity =
            identity::resolve_identity(&state.pool, &msg.sender_id, &msg.channel).await?;
        (identity.id, Uuid::nil())
    };

    // ================================== BEGIN COMMAND ============================//
    // 2. Check for Pairing Code
    let text = msg.text.trim();
    if text.to_uppercase().starts_with("PAIR ") || text.to_uppercase().starts_with("/PAIR ") {
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() >= 2 {
            let code = parts[1].to_uppercase();
            if let Some(conv_id) = pairing_repo::validate_pairing_code(&state.pool, &code).await? {
                let display_name = match msg.metadata.clone() {
                    None => None,
                    Some(meta) => meta
                        .get("display_name")
                        .map_or_else(|| None, |v| Some(v.to_string())),
                };

                pairing_repo::complete_pairing(&state.pool, &code, user_id).await?;
                channel_repo::link_channel(
                    &state.pool,
                    &msg.channel,
                    &msg.sender_id,
                    &msg.conversation_id,
                    conv_id,
                    user_id,
                    display_name,
                )
                .await?;

                let _ = state
                    .send_to_user(
                        user_id.to_string().as_str(),
                        "pairing_success",
                        serde_json::json!({
                            "conversation_id": conv_id,
                            "platform": msg.channel,
                            "message": format!("Successfully paired with {}!", msg.channel)
                        }),
                        &OutboundMessage {
                            is_group: msg.is_group,
                            sender_id: msg.sender_id.clone(),
                            conversation_id: msg.conversation_id.clone(),
                            text: "Pairing successful! This conversation is now linked."
                                .to_string(),
                            channel: msg.channel.clone(),
                            video_url: None,
                            image_url: None,
                            audio_url: None,
                            doc_url: None,
                            sticker_url: None,
                            metadata: msg.metadata.clone(),
                        },
                    )
                    .await;

                return Ok(());
            }
        }
    } else if text.starts_with("/register") {
        info!(
            "start registering from channel {} sender_id {}",
            msg.channel, msg.sender_id
        );
        let channel_exists = sqlx::query!(
                "SELECT u.id as user_id FROM channels c JOIN users u ON u.id = c.user_id WHERE c.channel_type = $1 AND c.external_chat_id = $2",
                msg.channel,
                msg.conversation_id
            )
            .fetch_optional(&state.pool)
            .await;
        if let Err(err) = channel_exists {
            info!("failed register because error getting information: {}", err);
            let _ = state
                .publish_outbond(&crate::feature::OutboundMessage {
                    is_group: msg.is_group,
                    sender_id: msg.sender_id.clone(),
                    conversation_id: msg.conversation_id.clone(),
                    text: "We having trouble, meanwhile we on fixing, you can try again later."
                        .to_string(),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                })
                .await;
            return Ok(());
        }
        let channel_result = channel_exists?;
        if let Some(value) = channel_result {
            info!("failed register because user exist: {}", value.user_id);
            let _ = state
                .publish_outbond(&crate::feature::OutboundMessage {
                    is_group: msg.is_group,
                    sender_id: msg.sender_id.clone(),
                    conversation_id: msg.conversation_id.clone(),
                    text: "Account already exists. Use /login.".to_string(),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                })
                .await;

            return Ok(());
        }

        let mut tx = match state.pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!("Failed to start transaction: {}", e);
                let _ = state
                    .publish_outbond(&OutboundMessage {
                        is_group: msg.is_group,
                        sender_id: msg.sender_id.clone(),
                        conversation_id: msg.conversation_id.clone(),
                        text: "Internal server error".to_string(),
                        channel: msg.channel.clone(),
                        video_url: None,
                        image_url: None,
                        audio_url: None,
                        doc_url: None,
                        sticker_url: None,
                        metadata: msg.metadata.clone(),
                    })
                    .await;
                return Ok(());
            }
        };

        info!("begin create user \n");

        let display_name = match msg.metadata.clone() {
            None => msg.sender_id.clone(),
            Some(meta) => meta
                .get("display_name")
                .map_or_else(|| msg.sender_id.clone(), |v| v.to_string()),
        };

        let u_id = match sqlx::query!(
            "INSERT INTO users (external_id, display_name) VALUES ($1, $2) ON CONFLICT (external_id) DO UPDATE SET display_name = EXCLUDED.display_name RETURNING id",
            msg.sender_id,
            display_name
        ).fetch_one(&mut *tx).await {
            Ok(r) => r.id,
            Err(e) => {
                error!("Failed to resolve user: {}", e);
                let _ = tx.rollback().await;
                let _ = state.publish_outbond(&crate::feature::OutboundMessage {
                    is_group: msg.is_group,
                    sender_id: msg.sender_id.clone(),
                    conversation_id: msg.conversation_id.clone(),
                    text: "Failed to resolve user".to_string(),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                }).await;

                return Ok(());
            }
        };

        info!("begin create conversation \n");
        // Create new conversation
        let conv_id = Uuid::new_v4();
        let title = format!("{} via {}", msg.conversation_id, msg.channel);

        if let Err(e) = sqlx::query!(
            "INSERT INTO conversations (id, title) VALUES ($1, $2)",
            conv_id,
            title
        )
        .execute(&mut *tx)
        .await
        {
            error!("Failed to create conversation: {}", e);
            let _ = tx.rollback().await;
            let _ = state
                .publish_outbond(&crate::feature::OutboundMessage {
                    is_group: msg.is_group,
                    sender_id: msg.sender_id.clone(),
                    conversation_id: msg.conversation_id.clone(),
                    text: "Failed to create conversation".to_string(),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                })
                .await;

            return Ok(());
        }

        info!("begin create channels");
        if let Err(e) = sqlx::query!(
            "INSERT INTO channels (channel_type, external_id, external_chat_id, conversation_id, user_id) VALUES ($1, $2, $3, $4, $5)",
            msg.channel,
            msg.sender_id,
            msg.conversation_id,
            conv_id,
            u_id
        ).execute(&mut *tx).await {
            error!("Failed to link channel: {}", e);
            let _ = tx.rollback().await;

            let _ = state.publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Failed to link channel".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            }).await;

            return Ok(());
        }

        if let Err(e) = sqlx::query!(
            "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            conv_id,
            u_id
        ).execute(&mut *tx).await {
            error!("Failed to add member: {}", e);
            let _ = tx.rollback().await;
            let _ = state.publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Failed to join conversation".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            }).await;

            return Ok(());
        }

        if let Err(e) = tx.commit().await {
            error!("Failed to commit registration: {}", e);
            let _ = state
                .publish_outbond(&crate::feature::OutboundMessage {
                    is_group: msg.is_group,
                    sender_id: msg.sender_id.clone(),
                    conversation_id: msg.conversation_id.clone(),
                    text: "Failed to register".to_string(),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                })
                .await;

            return Ok(());
        }

        let _ = state
            .publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Success register account, you can now /login for access dashboard"
                    .to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;
        return Ok(());
    } else if text.starts_with("/login") {
        info!(
            "start login from channel {} sender_id {}",
            msg.channel, msg.sender_id
        );
        // Check if user/channel exists
        let channel_exists = sqlx::query!(
            "SELECT u.id as user_id FROM channels c JOIN users u ON u.id = c.user_id WHERE c.channel_type = $1 AND c.external_chat_id = $2",
            msg.channel,
            msg.conversation_id
        ).fetch_optional(&state.pool).await;

        if let Err(err) = channel_exists {
            info!("failed get channel data: {}", err);
            let _ = state
                .publish_outbond(&crate::feature::OutboundMessage {
                    is_group: msg.is_group,
                    sender_id: msg.sender_id.clone(),
                    conversation_id: msg.conversation_id.clone(),
                    text: "We having trouble for getting information, meanwhile we fixing you can try again later.".to_string(),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                })
                .await;

            return Ok(());
        }
        if let Ok(None) = channel_exists {
            info!("channel doesnt exist:");
            let _ = state
                .publish_outbond(&crate::feature::OutboundMessage {
                    is_group: msg.is_group,
                    sender_id: msg.sender_id.clone(),
                    conversation_id: msg.conversation_id.clone(),
                    text: "Channel not registered, Use /register for new user use, if you already had account, get pairing code from dashboard and use /pair <PAIRING CODE>".to_string(),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                })
                .await;

            return Ok(());
        }

        let channel_data = channel_exists.unwrap().unwrap();
        let user_id = channel_data.user_id;

        // Generate OTP
        let otp_code: u32 = rand::rng().random_range(100000..999999);
        let otp_str = otp_code.to_string();
        let redis_key = format!("otp:{}", user_id);

        if let Err(e) = state.redis.set_ex(&redis_key, &otp_str, 300).await {
            error!("Failed to store OTP in Redis: {}", e);
            let _ = state
                .publish_outbond(&crate::feature::OutboundMessage {
                    is_group: msg.is_group,
                    sender_id: msg.sender_id.clone(),
                    conversation_id: msg.conversation_id.clone(),
                    text: "Database error".to_string(),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                })
                .await;

            return Ok(());
        }

        let app_url =
            std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
        let login_url = format!("{}/login?id={}", app_url, user_id);

        let outbound_text = format!(
            "Your verification code is: {}\n\nClick here to login: {}",
            otp_str, login_url
        );

        let outbound = crate::feature::OutboundMessage {
            is_group: msg.is_group,
            sender_id: "nomi_auth".to_string(),
            conversation_id: msg.conversation_id.clone(),
            text: outbound_text,
            channel: msg.channel.clone(),
            video_url: None,
            image_url: None,
            audio_url: None,
            doc_url: None,
            sticker_url: None,
            metadata: msg.metadata.clone(),
        };

        if let Err(e) = state.redis.publish_event("nomi:outbound", &outbound).await {
            error!("Failed to publish OTP to nomi:outbound: {}", e);
            return Ok(());
        }

        return Ok(());
    }

    // ================================== NOT REGISTERED STOP HERE ============================//
    // 3. Resolve/Create Conversation
    if external_conversation_id.is_nil() {
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
            It’s a pleasure to meet you, and I look forward to assisting you once you're signed in! ✨",
                "**`/register`**",
                "**`/login`**"
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

    // 4. Save User Message
    let user_message = message_repo::save_message(
        &state.pool,
        external_conversation_id,
        "user",
        &msg.text,
        None,
        Some(user_id),
        0,
        0,
        0,
    ).await?;

    let _ = state
        .send_sse_to_user(user_id.to_string().as_str(), "message", json!(user_message))
        .await;

    // 5. Trigger Agentic Loop
    let state_clone = state.clone();
    let user_text = msg.text.clone();
    let sender_id = msg.sender_id.clone();
    let chat_id = msg.conversation_id.clone();
    let channel = msg.channel.clone();
    let image_url = msg.image_url.clone();

    tokio::spawn(async move {
        // A. Presence
        let _ = state_clone
            .send_presence_to_user(
                user_id.to_string().as_str(),
                json! ({
                "conversation_id": external_conversation_id,
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
        let unified_msg = crate::feature::message_processor::UnifiedMessage {
            conversation_id: external_conversation_id,
            user_id: Some(user_id),
            text_content: user_text,
            image_url,
            source: match msg.channel.as_str() {
                "telegram" => crate::feature::message_processor::MessageSource::Telegram,
                "whatsapp" => crate::feature::message_processor::MessageSource::WhatsApp,
                _ => crate::feature::message_processor::MessageSource::Other(msg.channel),
            },
        };

        if let Err(e) =
            crate::feature::message_processor::process_incoming_message(state_clone, unified_msg)
                .await
        {
            error!("Failed to process inbound message: {}", e);
        }
    });

    Ok(())
}
