use crate::feature::InboundMessage;
use crate::AppState;
use tokio_stream::StreamExt;
use tracing::{error, info};
use crate::common::repository::{message_repo, pairing_repo, channel_repo};
use crate::common::identity;
use crate::common::sse::sse_builder::{SseBuilder, SseTarget};

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
    info!("Handling inbound from {} in chat {}: {}", msg.sender_id, msg.chat_id, msg.text);

    // 1. Resolve Identity and Channel Info
    let channel_info = channel_repo::get_channel_info(&state.pool, &msg.channel, &msg.chat_id).await?;
    
    let (user_id, mut conversation_id) = if let Some(ci) = channel_info {
        (ci.user_id, ci.conversation_id)
    } else {
        let identity = identity::resolve_identity(&state.pool, &msg.sender_id, &msg.channel).await?;
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
                channel_repo::link_channel(&state.pool, &msg.channel, &msg.sender_id, &msg.chat_id, conv_id, user_id).await?;

                let _ = state.sse.send(SseBuilder::new(
                    SseTarget::broadcast("pairing_success".to_string()),
                    serde_json::json!({
                        "conversation_id": conv_id,
                        "platform": msg.channel,
                        "message": format!("Successfully paired with {}!", msg.channel)
                    })
                )).await;

                state.redis.publish_event("nomi:outbound", &crate::feature::OutboundMessage {
                    sender_id: msg.sender_id.clone(),
                    chat_id: msg.chat_id.clone(),
                    text: "Pairing successful! This conversation is now linked.".to_string(),
                    channel: msg.channel.clone(),
                }).await?;

                return Ok(());
            }
        }
    }

    // 3. Resolve/Create Conversation
    if conversation_id.is_nil() {
        let conv_id = uuid::Uuid::new_v4();
        let title = format!("{} via {}", msg.chat_id, msg.channel);
        
        sqlx::query!("INSERT INTO conversations (id, title) VALUES ($1, $2)", conv_id, title)
            .execute(&state.pool).await?;

        channel_repo::link_channel(&state.pool, &msg.channel, &msg.sender_id, &msg.chat_id, conv_id, user_id).await?;
        conversation_id = conv_id;
    }

    // 4. Save User Message
    let user_message = message_repo::save_message(
        &state.pool, 
        conversation_id, 
        "user", 
        &msg.text, 
        None, 
        Some(user_id)
    ).await?;

    let _ = state.sse.send(SseBuilder::new(
        SseTarget::broadcast("message".to_string()),
        user_message
    )).await;

    // 5. Trigger Agentic Loop
    let state_clone = state.clone();
    let user_text = msg.text.clone();
    let sender_id = msg.sender_id.clone();
    let chat_id = msg.chat_id.clone();
    let channel = msg.channel.clone();

    tokio::spawn(async move {
        let _ = state_clone.sse.send(SseBuilder::new(
            SseTarget::broadcast("presence".to_string()),
            serde_json::json!({
                "conversation_id": conversation_id,
                "is_typing": true,
                "user_id": "nomi"
            }),
        )).await;

        let _ = state_clone.redis.publish_event("nomi:presence", &crate::feature::PresenceMessage {
            sender_id: sender_id.clone(),
            chat_id: chat_id.clone(),
            channel: channel.clone(),
            status: "typing".to_string(),
        }).await;

        let conversation = sqlx::query!(
            "SELECT bootstrap_content, soul_content FROM conversations WHERE id = $1",
            conversation_id
        ).fetch_one(&state_clone.pool).await;

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
            crate::utils::rag::hybrid_retrieve(&state_clone.pool, &user_text, embedding)
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
            let result = crate::common::agent::send_prompt(&state_clone.gemini, current_actor).await;

            match result {
                Ok((response, chunk)) => {
                    // Emit thought
                    if !chunk.thought.is_empty() {
                        let _ = state_clone.sse.send(SseBuilder::new(
                            SseTarget::broadcast("thought".to_string()),
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
                    ).await;

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
                None
            ).await.unwrap();

            let _ = state_clone.sse.send(SseBuilder::new(
                SseTarget::broadcast("message".to_string()),
                assistant_message
            )).await;

            let _ = state_clone.sse.send(SseBuilder::new(
                SseTarget::broadcast("presence".to_string()),
                serde_json::json!({"conversation_id": conversation_id, "is_typing": false, "user_id": "nomi"}),
            )).await;

            let _ = state_clone.redis.publish_event("nomi:presence", &crate::feature::PresenceMessage {
                sender_id: sender_id.clone(),
                chat_id: chat_id.clone(),
                channel: channel.clone(),
                status: "idle".to_string(),
            }).await;

            let _ = state_clone.redis.publish_event("nomi:outbound", &crate::feature::OutboundMessage {
                sender_id: sender_id,
                chat_id: chat_id,
                text: chunk.content,
                channel: channel,
            }).await;
        }
    });

    Ok(())
}
