use crate::feature::{InboundMessage, OutboundMessage};
use crate::AppState;
use redis::AsyncCommands;
use tokio_stream::StreamExt;
use tracing::{error, info};

pub async fn start_redis_listener(state: AppState) -> anyhow::Result<()> {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let client = redis::Client::open(redis_url)?;
    let mut pubsub = client.get_async_pubsub().await?;

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
    info!("Handling inbound from {}: {}", msg.external_id, msg.content);
    
    // 1. Get or Create User
    let user_id = sqlx::query_scalar!(
        r#"
        INSERT INTO users (external_id, display_name)
        VALUES ($1, $2)
        ON CONFLICT (external_id) DO UPDATE SET display_name = EXCLUDED.display_name
        RETURNING id
        "#,
        msg.external_id,
        msg.display_name
    )
    .fetch_one(&state.pool)
    .await?;

    // 2. Get or Create Conversation (Simplified for now, assuming 1:1)
    let conversation_id = sqlx::query_scalar!(
        r#"
        SELECT conversation_id 
        FROM conversation_members 
        WHERE user_id = $1 
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?;

    let conversation_id = match conversation_id {
        Some(id) => id,
        None => {
            let session_id = sqlx::query_scalar!(
                "INSERT INTO sessions (user_id) VALUES ($1) RETURNING id",
                user_id
            )
            .fetch_one(&state.pool)
            .await?;

            let conv_id = sqlx::query_scalar!(
                "INSERT INTO conversations (session_id, title) VALUES ($1, $2) RETURNING id",
                session_id,
                format!("Chat with {}", msg.display_name.unwrap_or_else(|| "Unknown".into()))
            )
            .fetch_one(&state.pool)
            .await?;

            sqlx::query!(
                "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2)",
                conv_id,
                user_id
            )
            .execute(&state.pool)
            .await?;

            conv_id
        }
    };

    // 3. Save User Message
    let user_msg_id = sqlx::query_scalar!(
        "INSERT INTO messages (conversation_id, role, content) VALUES ($1, 'user', $2) RETURNING id",
        conversation_id,
        msg.content
    )
    .fetch_one(&state.pool)
    .await?;

    // Broadcast to SSE (Web UI)
    let _ = state.sse.send(crate::common::sse::sse_builder::SseBuilder::new(
        crate::common::sse::sse_builder::SseTarget::broadcast("message".to_string()),
        crate::feature::conversation::chat_model::MessageItem {
            id: user_msg_id,
            conversation_id,
            role: "user".to_string(),
            content: msg.content.clone(),
            thought: Some("".to_string()),
            created_at: chrono::Utc::now(),
        }
    )).await;

    // 4. Trigger Gemini (Reuse existing logic by calling the stream logic or similar)
    // For simplicity, we trigger the agentic loop here.
    let state_clone = state.clone();
    let user_message = msg.content.clone();
    let external_id = msg.external_id.clone();
    let platform = msg.platform.clone();

    tokio::spawn(async move {
        // Start typing in Web UI
        let _ = state_clone.sse.send(crate::common::sse::sse_builder::SseBuilder::new(
            crate::common::sse::sse_builder::SseTarget::broadcast("presence".to_string()),
            serde_json::json!({
                "conversation_id": conversation_id,
                "is_typing": true,
                "user_id": "nomi"
            }),
        )).await;

        // Execute agentic loop (simplified for this context)
        // In a real scenario, you'd refactor handle_chat_stream to be more modular.
        
        let _dispatcher = crate::common::tools::ToolDispatcher::new(
            state_clone.pool.clone(),
            std::env::current_dir().unwrap_or_default(),
            Some(conversation_id),
            state_clone.gemini.clone(),
            state_clone.gemini_api_key.clone(),
            state_clone.sse.clone(),
        );

        let conversation = sqlx::query!(
            "SELECT bootstrap_content, soul_content FROM conversations WHERE id = $1",
            conversation_id
        )
        .fetch_one(&state_clone.pool)
        .await;

        let system_prompt = match conversation {
            Ok(c) => {
                let boot = c.bootstrap_content.unwrap_or_default();
                let soul = c.soul_content.unwrap_or_default();
                let mut combined = boot;
                if !soul.is_empty() {
                    combined.push_str("\n\n### Current Personality/Soul\n");
                    combined.push_str(&soul);
                }
                combined
            }
            Err(_) => String::new(),
        };

        let result = crate::common::agent::send_prompt(
            &state_clone.gemini,
            crate::common::agent::agent_model::PromptActor::User {
                history: String::new(), // Fetch history for better results
                memories: String::new(), // RAG results
                message: user_message.clone(),
                system_prompt,
            }
        ).await;

        if let Ok((_response, chunk)) = result {
            // Save Assistant Message
            let assistant_msg_id = sqlx::query_scalar!(
                "INSERT INTO messages (conversation_id, role, content, thought) VALUES ($1, 'assistant', $2, $3) RETURNING id",
                conversation_id,
                chunk.content,
                chunk.thought
            )
            .fetch_one(&state_clone.pool)
            .await.unwrap();

            // Broadcast to SSE
            let _ = state_clone.sse.send(crate::common::sse::sse_builder::SseBuilder::new(
                crate::common::sse::sse_builder::SseTarget::broadcast("message".to_string()),
                crate::feature::conversation::chat_model::MessageItem {
                    id: assistant_msg_id,
                    conversation_id,
                    role: "assistant".to_string(),
                    content: chunk.content.clone(),
                    thought: Some(chunk.thought.clone()),
                    created_at: chrono::Utc::now(),
                }
            )).await;

            // Stop typing
            let _ = state_clone.sse.send(crate::common::sse::sse_builder::SseBuilder::new(
                crate::common::sse::sse_builder::SseTarget::broadcast("presence".to_string()),
                serde_json::json!({"conversation_id": conversation_id, "is_typing": false}),
            )).await;

            // Publish to Redis for channel-rust
            let outbound = OutboundMessage {
                external_id: external_id,
                platform: platform,
                content: chunk.content,
                thought: Some(chunk.thought),
                conversation_id,
            };

            if let Ok(redis_url) = std::env::var("REDIS_URL") {
                if let Ok(client) = redis::Client::open(redis_url) {
                    if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                        let payload = serde_json::to_string(&outbound).unwrap();
                        let _ = conn.publish::<&str, String, ()>("nomi:outbound", payload).await;
                    }
                }
            }
        }
    });

    Ok(())
}
