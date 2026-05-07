use crate::AppState;
use crate::common::identity;
use crate::common::repository::{channel_repo, message_repo, pairing_repo};
use crate::common::sse::sse_builder::{SseBuilder, SseTarget};
use crate::feature::InboundMessage;
use rand::{RngExt};
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
        msg.sender_id, msg.chat_id, msg.text
    );

    // 1. Resolve Identity and Channel Info
    let channel_info =
        channel_repo::get_channel_info(&state.pool, &msg.channel, &msg.chat_id).await?;

    let (user_id, mut conversation_id) = if let Some(ci) = channel_info {
        (ci.user_id, ci.conversation_id)
    } else {
        let identity =
            identity::resolve_identity(&state.pool, &msg.sender_id, &msg.channel).await?;
        (identity.id, uuid::Uuid::nil())
    };

    // 2. Check for Pairing Code
    let text = msg.text.trim();
    if text.to_uppercase().starts_with("PAIR ") || text.to_uppercase().starts_with("/PAIR ") {
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() >= 2 {
            let code = parts[1].to_uppercase();
            if let Some(conv_id) = pairing_repo::validate_pairing_code(&state.pool, &code).await? {
                pairing_repo::complete_pairing(&state.pool, &code, user_id).await?;
                channel_repo::link_channel(
                    &state.pool,
                    &msg.channel,
                    &msg.sender_id,
                    &msg.chat_id,
                    conv_id,
                    user_id,
                )
                .await?;

                let _ = state
                    .sse
                    .send(SseBuilder::new(
                        SseTarget::sent_to_user(user_id.to_string(), "pairing_success".to_string()),
                        serde_json::json!({
                            "conversation_id": conv_id,
                            "platform": msg.channel,
                            "message": format!("Successfully paired with {}!", msg.channel)
                        }),
                    ))
                    .await;

                state
                    .redis
                    .publish_event(
                        "nomi:outbound",
                        &crate::feature::OutboundMessage {
                            sender_id: msg.sender_id.clone(),
                            chat_id: msg.chat_id.clone(),
                            text: "Pairing successful! This conversation is now linked."
                                .to_string(),
                            channel: msg.channel.clone(), metadata: msg.metadata.clone(),
                        },
                    )
                    .await?;

                return Ok(());
            }
        }
    } else if text.starts_with("/login") || text.starts_with("/register") {
        info!("hit {}",text.to_uppercase());
        // Check if user/channel exists
        let channel_exists = sqlx::query!(
                "SELECT u.id as user_id FROM channels c JOIN users u ON u.id = c.user_id WHERE c.channel_type = $1 AND c.external_chat_id = $2",
                msg.channel,
                msg.chat_id
            )
            .fetch_optional(&state.pool)
            .await;

        let user_id = match channel_exists {
            Ok(Some(row)) => {
                if text == "/register" {
                    state
                        .redis
                        .publish_event(
                            "nomi:outbound",
                            &crate::feature::OutboundMessage {
                                sender_id: msg.sender_id.clone(),
                                chat_id: msg.chat_id.clone(),
                                text: "Account already exists. Use /login.".to_string(),
                                channel: msg.channel.clone(), metadata: msg.metadata.clone(),
                            },
                        )
                        .await?;
                    return Ok(());
                }
                row.user_id
            }
            Ok(None) => {
                if text == "/login" {
                    state
                        .redis
                        .publish_event(
                            "nomi:outbound",
                            &crate::feature::OutboundMessage {
                                sender_id: msg.sender_id.clone(),
                                chat_id: msg.chat_id.clone(),
                                text: "Account not found. Please type /register first.".to_string(),
                                channel: msg.channel.clone(), metadata: msg.metadata.clone(),
                            },
                        )
                        .await?;
                    return Ok(());
                }

                // Start Transaction for Registration
                let mut tx = match state.pool.begin().await {
                    Ok(tx) => tx,
                    Err(e) => {
                        error!("Failed to start transaction: {}", e);

                        state
                            .redis
                            .publish_event(
                                "nomi:outbound",
                                &crate::feature::OutboundMessage {
                                    sender_id: msg.sender_id.clone(),
                                    chat_id: msg.chat_id.clone(),
                                    text: "Internal server error".to_string(),
                                    channel: msg.channel.clone(), metadata: msg.metadata.clone(),
                                },
                            )
                            .await?;
                        return Ok(());
                    }
                };

                // Resolve/Create User
                let u_id = match sqlx::query!(
                        "INSERT INTO users (external_id, display_name) VALUES ($1, $2) ON CONFLICT (external_id) DO UPDATE SET display_name = EXCLUDED.display_name RETURNING id",
                        msg.sender_id,
                        msg.sender_id
                    ).fetch_one(&mut *tx).await {
                    Ok(r) => r.id,
                    Err(e) => {
                        error!("Failed to resolve user: {}", e);
                        let _ = tx.rollback().await;
                        state.redis.publish_event("nomi:outbound", &crate::feature::OutboundMessage {
                            sender_id: msg.sender_id.clone(),
                            chat_id: msg.chat_id.clone(),
                            text: "Failed to resolve user".to_string(),
                            channel: msg.channel.clone(), metadata: msg.metadata.clone(),
                        }).await?;
                        return Ok(())
                    }
                };

                // Create new conversation
                let conv_id = Uuid::new_v4();
                let title = format!("{} via {}", msg.chat_id, msg.channel);

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
                    state
                        .redis
                        .publish_event(
                            "nomi:outbound",
                            &crate::feature::OutboundMessage {
                                sender_id: msg.sender_id.clone(),
                                chat_id: msg.chat_id.clone(),
                                text: "Failed to create conversation".to_string(),
                                channel: msg.channel.clone(), metadata: msg.metadata.clone(),
                            },
                        )
                        .await?;
                    return Ok(());
                }

                if let Err(e) = sqlx::query!(
                        "INSERT INTO channels (channel_type, external_id, external_chat_id, conversation_id, user_id) VALUES ($1, $2, $3, $4, $5)",
                        msg.channel, msg.sender_id, msg.chat_id, conv_id, u_id
                    ).execute(&mut *tx).await {
                    error!("Failed to link channel: {}", e);
                    let _ = tx.rollback().await;
                    state.redis.publish_event("nomi:outbound", &crate::feature::OutboundMessage {
                        sender_id: msg.sender_id.clone(),
                        chat_id: msg.chat_id.clone(),
                        text: "Failed to link channel".to_string(),
                        channel: msg.channel.clone(), metadata: msg.metadata.clone(),
                    }).await?;
                    return Ok(())
                }

                if let Err(e) = sqlx::query!(
                        "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                        conv_id, u_id
                    ).execute(&mut *tx).await {
                    error!("Failed to add member: {}", e);
                    let _ = tx.rollback().await;
                    state.redis.publish_event("nomi:outbound", &crate::feature::OutboundMessage {
                        sender_id: msg.sender_id.clone(),
                        chat_id: msg.chat_id.clone(),
                        text: "Failed to join conversation".to_string(),
                        channel: msg.channel.clone(), metadata: msg.metadata.clone(),
                    }).await?;
                    return Ok(())
                }

                if let Err(e) = tx.commit().await {
                    error!("Failed to commit registration: {}", e);
                    state
                        .redis
                        .publish_event(
                            "nomi:outbound",
                            &crate::feature::OutboundMessage {
                                sender_id: msg.sender_id.clone(),
                                chat_id: msg.chat_id.clone(),
                                text: "Failed to register".to_string(),
                                channel: msg.channel.clone(), metadata: msg.metadata.clone(),
                            },
                        )
                        .await?;
                    return Ok(());
                }
                u_id
            }
            Err(e) => {
                error!("Database error: {}", e);
                state
                    .redis
                    .publish_event(
                        "nomi:outbound",
                        &crate::feature::OutboundMessage {
                            sender_id: msg.sender_id.clone(),
                            chat_id: msg.chat_id.clone(),
                            text: "Database error".to_string(),
                            channel: msg.channel.clone(), metadata: msg.metadata.clone(),
                        },
                    )
                    .await?;
                return Ok(());
            }
        };

        // Generate OTP
        let otp_code: u32 = rand::rng().random_range(100000..999999);
        let otp_str = otp_code.to_string();
        let redis_key = format!("otp:{}", user_id);

        if let Err(e) = state.redis.set_ex(&redis_key, &otp_str, 300).await {
            error!("Failed to store OTP in Redis: {}", e);
            state
                .redis
                .publish_event(
                    "nomi:outbound",
                    &crate::feature::OutboundMessage {
                        sender_id: msg.sender_id.clone(),
                        chat_id: msg.chat_id.clone(),
                        text: "Failed to generate OTP".to_string(),
                        channel: msg.channel.clone(), metadata: msg.metadata.clone(),
                    },
                )
                .await?;
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
            sender_id: "nomi_auth".to_string(),
            chat_id: msg.chat_id.clone(),
            text: outbound_text,
            channel: msg.channel.clone(), metadata: msg.metadata.clone(),
        };

        if let Err(e) = state.redis.publish_event("nomi:outbound", &outbound).await {
            error!("Failed to publish OTP to nomi:outbound: {}", e);
            return Ok(());
        }

        return Ok(());
    }

    // 3. Resolve/Create Conversation
    if conversation_id.is_nil() {
        let conv_id = uuid::Uuid::new_v4();
        let title = format!("{} via {}", msg.chat_id, msg.channel);

        sqlx::query!(
            "INSERT INTO conversations (id, title) VALUES ($1, $2)",
            conv_id,
            title
        )
        .execute(&state.pool)
        .await?;

        channel_repo::link_channel(
            &state.pool,
            &msg.channel,
            &msg.sender_id,
            &msg.chat_id,
            conv_id,
            user_id,
        )
        .await?;
        conversation_id = conv_id;
    }

    // 4. Save User Message
    let user_message = message_repo::save_message(
        &state.pool,
        conversation_id,
        "user",
        &msg.text,
        None,
        Some(user_id),
    )
    .await?;

    let _ = state
        .sse
        .send(SseBuilder::new(
            SseTarget::sent_to_user(user_id.to_string(), "message".to_string()),
            user_message,
        ))
        .await;

    // 5. Trigger Agentic Loop
    let state_clone = state.clone();
    let user_text = msg.text.clone();
    let sender_id = msg.sender_id.clone();
    let chat_id = msg.chat_id.clone();
    let channel = msg.channel.clone();

    tokio::spawn(async move {
        let _ = state_clone
            .sse
            .send(SseBuilder::new(
                SseTarget::sent_to_user(user_id.to_string(), "presence".to_string()),
                serde_json::json!({
                    "conversation_id": conversation_id,
                    "is_typing": true,
                    "user_id": "nomi"
                }),
            ))
            .await;

        let _ = state_clone
            .redis
            .publish_event(
                "nomi:presence",
                &crate::feature::PresenceMessage {
                    sender_id: sender_id.clone(),
                    chat_id: chat_id.clone(),
                    channel: channel.clone(),
                    status: "typing".to_string(),
                },
            )
            .await;

        let conversation = sqlx::query!(
            "SELECT bootstrap_content, soul_content FROM conversations WHERE id = $1",
            conversation_id
        )
        .fetch_one(&state_clone.pool)
        .await;

        let (system_prompt, _user_id_from_db) = match conversation {
            Ok(c) => {
                let boot = c.bootstrap_content.unwrap_or_default();
                let soul = c.soul_content.unwrap_or_default();
                let mut combined = boot;
                if !soul.is_empty() {
                    combined.push_str("\n\n### Current Personality/Soul\n");
                    combined.push_str(&soul);
                }
                (combined, Some(user_id)) // We already have user_id from resolution
            }
            Err(_) => (String::new(), Some(user_id)),
        };

        // A. Fetch last 15 messages for short-term history
        let history = sqlx::query!(
            "SELECT created_at, role, content FROM messages WHERE conversation_id = $1 ORDER BY created_at DESC LIMIT 15",
            conversation_id
        )
        .fetch_all(&state_clone.pool)
        .await
        .unwrap_or_default();

        let mut history_text = String::new();
        for msg in history.into_iter().rev() {
            let role_label = match msg.role.as_str() {
                "user" => "User",
                "assistant" => "Nomi",
                _ => "System",
            };
            history_text.push_str(&format!(
                "-[{}] {}: {}.\n",
                msg.created_at.unwrap_or_else(chrono::Utc::now).to_rfc3339(),
                role_label,
                msg.content
            ));
        }

        // B. Context Retrieval (RAG)
        let gemini_api_key = state_clone.gemini_api_key.clone();
        let embedding = crate::rag::get_embedding(&gemini_api_key, &user_text)
            .await
            .unwrap_or_default();

        let memories_text = if !embedding.is_empty() {
            crate::utils::rag::hybrid_retrieve(&state_clone.pool, &user_text, embedding, Some(conversation_id))
                .await
                .unwrap_or_default()
                .join("\n---\n")
        } else {
            String::new()
        };

        // C. Prepare Reasoning Loop
        let dispatcher = crate::common::tools::ToolDispatcher::new(
            state_clone.pool.clone(),
            std::env::current_dir().unwrap_or_default(),
            Some(conversation_id),
            state_clone.gemini.clone(),
            state_clone.gemini_api_key.clone(),
            state_clone.sse.clone(),
        );

        let mut loop_count = 0;
        let max_loops = 5;
        let mut current_actor = crate::common::agent::agent_model::PromptActor::User {
            history: history_text.clone(),
            memories: memories_text.clone(),
            message: user_text.clone(),
            system_prompt: system_prompt.clone(),
        };

        let mut final_response = None;
        let mut previous_calls = Vec::new();

        while loop_count < max_loops {
            loop_count += 1;
            let result =
                crate::common::agent::send_prompt(&state_clone.gemini, current_actor).await;

            match result {
                Ok((response, chunk)) => {
                    // Emit thought
                    if !chunk.thought.is_empty() {
                        let _ = state_clone.sse.send(SseBuilder::new(
                            SseTarget::sent_to_user(user_id.to_string(), "thought".to_string()),
                            serde_json::json!({ "thought": chunk.thought, "conversation_id": conversation_id }),
                        )).await;
                    }

                    let tool_calls = response.function_calls();
                    if tool_calls.is_empty() {
                        final_response = Some((response, chunk));
                        break;
                    }

                    let current_calls: Vec<_> = tool_calls.into_iter().map(|c| c.clone()).collect();
                    previous_calls.extend(current_calls.clone());

                    let tool_results = crate::common::agent::execute_tools(
                        &dispatcher,
                        current_calls.clone(),
                        &user_text,
                        Some(state_clone.sse.clone()),
                    )
                    .await;

                    current_actor = crate::common::agent::agent_model::PromptActor::MultiTool {
                        history: history_text.clone(),
                        memories: memories_text.clone(),
                        message: user_text.clone(),
                        system_prompt: system_prompt.clone(),
                        tool_results,
                        previous_calls: previous_calls.clone(),
                    };
                }
                Err(e) => {
                    error!("Agentic loop error: {}", e);
                    break;
                }
            }
        }

        if let Some((_, chunk)) = final_response {
            let assistant_message = message_repo::save_message(
                &state_clone.pool,
                conversation_id,
                "assistant",
                &chunk.content,
                Some(&chunk.thought),
                None,
            )
            .await
            .unwrap();

            let _ = state_clone
                .sse
                .send(SseBuilder::new(
                    SseTarget::sent_to_user(user_id.to_string(), "message".to_string()),
                    assistant_message,
                ))
                .await;

            let _ = state_clone.sse.send(SseBuilder::new(
                SseTarget::sent_to_user(user_id.to_string(), "presence".to_string()),
                serde_json::json!({"conversation_id": conversation_id, "is_typing": false, "user_id": "nomi"}),
            )).await;

            let _ = state_clone
                .redis
                .publish_event(
                    "nomi:presence",
                    &crate::feature::PresenceMessage {
                        sender_id: sender_id.clone(),
                        chat_id: chat_id.clone(),
                        channel: channel.clone(),
                        status: "idle".to_string(),
                    },
                )
                .await;

            let _ = state_clone
                .redis
                .publish_event(
                    "nomi:outbound",
                    &crate::feature::OutboundMessage {
                        sender_id: sender_id,
                        chat_id: chat_id,
                        text: chunk.content,
                        channel: channel.clone(), metadata: None,
                    },
                )
                .await;
        }
    });

    Ok(())
}
