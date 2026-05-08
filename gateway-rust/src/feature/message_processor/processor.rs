use crate::AppState;
use crate::common::agent::agent_model::PromptActor;
use crate::common::agent::execute_tools;
use crate::common::tools::ToolDispatcher;
use crate::feature::message_processor::model::UnifiedMessage;
use crate::feature::{OutboundMessage, PresenceMessage};
use crate::rag;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::common::repository::message_repo::save_message;
use tracing::{error, info};

pub async fn process_incoming_message(state: AppState, msg: UnifiedMessage) -> anyhow::Result<()> {
    let conversation_id = msg.conversation_id;
    let user_id = msg.user_id;
    let text_content = msg.text_content;

    info!(
        conversation_id = %conversation_id,
        user_id = ?user_id,
        source = ?msg.source,
        "Processing unified message"
    );

    // 1. Immediate Save
    let m = sqlx::query!(
        "INSERT INTO messages (conversation_id, role, content, thought, user_id, created_at) VALUES ($1, 'user', $2, '', $3, now()) RETURNING id, role, content, thought, created_at, user_id",
        conversation_id,
        text_content,
        user_id
    ).fetch_one(&state.pool).await?;

    // Broadcast user message to SSE
    let payload = json!({
        "id": m.id,
        "conversation_id": conversation_id,
        "role": m.role,
        "content": m.content,
        "thought": m.thought,
        "user_id": m.user_id,
        "created_at": m.created_at.unwrap_or_else(Utc::now),
    });
    let _ = match user_id {
        None => state.broadcast_sse("message", payload).await,
        Some(ref id) => {
            state
                .send_sse_to_user(id.to_string().as_str(), "message", payload)
                .await
        }
    };

    // 2. Start Typing / Presence
    let presence_payload = json!({
        "conversation_id": conversation_id,
        "is_typing": true,
        "user_id": "nomi"
    });
    let _ = match user_id {
        None => state.broadcast_presence_sse(presence_payload).await,
        Some(ref id) => {
            state
                .send_presence_sse_to_user(id.to_string().as_str(), presence_payload)
                .await
        }
    };

    // Broadcast presence to Redis for channels
    if let Ok(channel_info) = sqlx::query!(
        "SELECT c.channel_type, c.external_id, c.external_chat_id FROM channels c JOIN conversation_members cm ON c.user_id = cm.user_id WHERE cm.conversation_id = $1",
        conversation_id
    ).fetch_all(&state.pool).await {
        for channel in channel_info {
            let presence = PresenceMessage {
                sender_id: channel.external_id.clone(),
                chat_id: channel.external_chat_id.clone(),
                channel: channel.channel_type.clone(),
                status: "typing".to_string(),
            };
            if let Ok(redis_url) = std::env::var("REDIS_URL") {
                if let Ok(client) = redis::Client::open(redis_url) {
                    if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                        use redis::AsyncCommands;
                        let payload = serde_json::to_string(&presence).unwrap();
                        let _ = conn.publish::<&str, String, ()>("nomi:presence", payload).await;
                    }
                }
            }
        }
    }

    // 3. Prepare AI Context
    let dispatcher = ToolDispatcher::new(
        state.pool.clone(),
        std::env::current_dir().unwrap_or_default(),
        user_id.clone(),
        Some(conversation_id),
        state.gemini.clone(),
        state.gemini_api_key.clone(),
        state.sse.clone(),
    );

    let conversation = sqlx::query!(
        "SELECT bootstrap_content, soul_content FROM conversations WHERE id = $1",
        conversation_id
    )
    .fetch_one(&state.pool)
    .await?;

    let system_prompt = {
        let boot = conversation.bootstrap_content.unwrap_or_default();
        let soul = conversation.soul_content.unwrap_or_default();
        let mut combined = boot;
        if !soul.is_empty() {
            combined.push_str("\n\n### Current Personality/Soul\n");
            combined.push_str(&soul);
        }
        combined
    };

    // History Retrieval
    let history = sqlx::query!(
        "SELECT users.display_name as display_name, messages.created_at, messages.role, messages.content FROM messages LEFT JOIN users ON users.id = messages.user_id WHERE conversation_id = $1 ORDER BY created_at DESC LIMIT 15",
        conversation_id
    )
    .fetch_all(&state.pool)
    .await?;

    let mut history_text = String::new();
    for msg in history.into_iter().rev() {
        let role_label = match msg.role.as_str() {
            "user" => match msg.display_name {
                None => "User",
                Some(ref user) => &user,
            },
            "assistant" => "Nomi",
            _ => "System",
        };
        history_text.push_str(&format!(
            "-[{}] {}: {}.\n",
            msg.created_at
                .unwrap_or(Utc::now())
                .format("%Y-%m-%d %H:%M")
                .to_string(),
            role_label,
            msg.content
        ));
    }

    // RAG Context
    let embedding = rag::get_embedding(&state.gemini_api_key, &text_content)
        .await
        .unwrap_or_default();
    let memories_text = if !embedding.is_empty() {
        crate::utils::rag::hybrid_retrieve(
            &state.pool,
            &text_content,
            embedding,
            Some(conversation_id),
        )
        .await
        .unwrap_or_default()
        .join("\n---\n")
    } else {
        String::new()
    };

    // 4. LLM Execution Loop
    let mut loop_count = 0;
    let max_loops = 5;
    let mut current_actor = PromptActor::User {
        history: history_text.clone(),
        memories: memories_text.clone(),
        message: text_content.clone(),
        system_prompt: system_prompt.clone(),
    };

    let mut final_response = None;
    let mut previous_calls = Vec::new();

    while loop_count < max_loops {
        loop_count += 1;
        info!("Loop count iterate(N): N({})", loop_count);
        let result = crate::common::agent::send_prompt(state.gemini.as_ref(), current_actor).await;

        match result {
            Ok((response, chunk)) => {
                if !chunk.thought.is_empty() {
                    let payload =
                        json!({ "thought": chunk.thought, "conversation_id": conversation_id });
                    let _ = match user_id {
                        None => state.broadcast_sse("thought", payload).await,
                        Some(ref id) => {
                            state
                                .send_sse_to_user(id.to_string().as_str(), "thought", payload)
                                .await
                        }
                    };
                }

                let tool_calls = response.function_calls();
                if tool_calls.is_empty() {
                    final_response = Some((response, chunk));
                    break;
                }

                let current_calls: Vec<_> = tool_calls.into_iter().map(|c| c.clone()).collect();
                previous_calls.extend(current_calls.clone());

                let tool_results = execute_tools(
                    &dispatcher,
                    current_calls.clone(),
                    &text_content,
                    Some(state.sse.clone()),
                )
                .await;

                current_actor = PromptActor::MultiTool {
                    history: history_text.clone(),
                    memories: memories_text.clone(),
                    message: text_content.clone(),
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

    // 5. Post-Process & Final Save
    if let Some((_, function_result)) = final_response {
        if let Ok(record) = save_message(
            &state.pool,
            conversation_id,
            "assistant",
            &function_result.content,
            Some(function_result.thought.as_str()),
            None,
            function_result.prompt_tokens,
            function_result.answer_tokens,
            function_result.total_tokens,
        )
        .await
        {
            // Broadcast assistant message to SSE
            let payload = json!({
                        "id": record.id,
                        "conversation_id":conversation_id,
                        "role": record.role,
                        "content": record.content.clone(),
                        "thought": record.thought,
                        "user_id": record.user_id,
                        "total_token": function_result.total_tokens,
                        "created_at": record.created_at
            });

            let _ = match user_id {
                None => state.broadcast_sse("message", payload).await,
                Some(ref id) => {
                    state
                        .send_sse_to_user(id.to_string().as_str(), "message", payload)
                        .await
                }
            };

            // Outbound Routing for Channels
            let channel_info = sqlx::query!(
            "SELECT c.channel_type, c.external_id, c.external_chat_id FROM channels c JOIN conversation_members cm ON c.user_id = cm.user_id WHERE cm.conversation_id = $1",
            conversation_id
        ).fetch_all(&state.pool).await.unwrap_or_default();

            for channel in channel_info {
                let outbound = OutboundMessage {
                    is_group: false,
                    sender_id: channel.external_id.clone(),
                    chat_id: channel.external_chat_id.clone(),
                    text: record.content.clone(),
                    channel: channel.channel_type.clone(),
                    metadata: None,
                };

                if let Ok(redis_url) = std::env::var("REDIS_URL") {
                    if let Ok(client) = redis::Client::open(redis_url) {
                        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                            use redis::AsyncCommands;
                            let payload = serde_json::to_string(&outbound).unwrap();
                            let _ = conn
                                .publish::<&str, String, ()>("nomi:outbound", payload)
                                .await;
                        }
                    }
                }
            }
        }
        // Memory Consolidation Trigger (Background)
        let pool = state.pool.clone();
        let gemini = state.gemini.clone();
        let gemini_api_key = state.gemini_api_key.clone();
        tokio::spawn(async move {
            let _ =
                trigger_memory_consolidation(pool, gemini, gemini_api_key, conversation_id).await;
        });
    }

    // Stop Typing
    let payload = json!({
        "conversation_id": conversation_id,
        "is_typing": false,
        "user_id": "nomi"
    });
    let _ = match user_id {
        None => state.broadcast_presence_sse(payload).await,
        Some(ref id) => {
            state
                .send_presence_sse_to_user(id.to_string().as_str(), payload)
                .await
        }
    };

    // Presence Outbound (Stop Typing)
    if let Ok(channel_info) = sqlx::query!(
        "SELECT c.channel_type, c.external_id, c.external_chat_id FROM channels c JOIN conversation_members cm ON c.user_id = cm.user_id WHERE cm.conversation_id = $1",
        conversation_id
    ).fetch_all(&state.pool).await {
        for channel in channel_info {
            let presence = PresenceMessage {
                sender_id: channel.external_id.clone(),
                chat_id: channel.external_chat_id.clone(),
                channel: channel.channel_type.clone(),
                status: "idle".to_string(),
            };
            if let Ok(redis_url) = std::env::var("REDIS_URL") {
                if let Ok(client) = redis::Client::open(redis_url) {
                    if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                        use redis::AsyncCommands;
                        let payload = serde_json::to_string(&presence).unwrap();
                        let _ = conn.publish::<&str, String, ()>("nomi:presence", payload).await;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn trigger_memory_consolidation(
    pool: sqlx::PgPool,
    gemini: std::sync::Arc<gemini_rust::Gemini>,
    gemini_api_key: String,
    conversation_id: Uuid,
) -> anyhow::Result<()> {
    // 1. Get the last summarized message ID
    let last_summary = sqlx::query!(
        r#"
        SELECT metadata->>'last_message_id' as last_message_id
        FROM knowledge_base
        WHERE metadata->>'type' = 'summary' 
        AND metadata->>'conversation_id' = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        conversation_id.to_string()
    )
    .fetch_optional(&pool)
    .await?;

    let last_msg_id = last_summary
        .and_then(|r| r.last_message_id)
        .and_then(|id| Uuid::parse_str(&id).ok());

    // 2. Fetch new messages
    let new_messages = sqlx::query!(
        r#"
        SELECT id, role, content 
        FROM messages 
        WHERE conversation_id = $1 
        AND ($2::uuid IS NULL OR created_at > (SELECT created_at FROM messages WHERE id = $2))
        ORDER BY created_at ASC
        "#,
        conversation_id,
        last_msg_id
    )
    .fetch_all(&pool)
    .await?;

    // 3. Threshold check
    if new_messages.len() >= 10 {
        info!(conversation_id = %conversation_id, "Memory consolidation triggered ({} new messages)", new_messages.len());

        let last_processed_id = new_messages.last().map(|m| m.id).unwrap();
        let mut summary_input = String::new();
        for msg in new_messages {
            summary_input.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }

        let summarizer_prompt = format!(
            "Analyze the following conversation and return a JSON object with:
1. 'summary': A concise summary of permanent facts and project context.
2. 'nodes': An array of entities ({{'id': 'unique_id', 'label': 'Entity Name', 'node_type': 'Technology|Project|Person|Organization'}}).
3. 'edges': An array of relationships ({{'source': 'node_id', 'target': 'node_id', 'relationship': 'Description'}}).

Rules:
- NEVER create a node with id 'summary' or that represents the conversation summary itself.
- Extract individual entities.
- Reuse IDs.
- 'id' should be snake_case.

Conversation:
{}
",
            summary_input
        );

        let summary_res = gemini
            .generate_content()
            .with_user_message(summarizer_prompt)
            .execute()
            .await?;

        let raw_json = summary_res.text();
        let parsed_data: serde_json::Value = if let Some(start) = raw_json.find('{') {
            if let Some(end) = raw_json.rfind('}') {
                serde_json::from_str(&raw_json[start..=end])
                    .unwrap_or(json!({ "summary": raw_json, "nodes": [], "edges": [] }))
            } else {
                json!({ "summary": raw_json, "nodes": [], "edges": [] })
            }
        } else {
            json!({ "summary": raw_json, "nodes": [], "edges": [] })
        };

        let summary_text = parsed_data["summary"]
            .as_str()
            .unwrap_or(&raw_json)
            .to_string();

        if let Ok(embedding) = rag::get_embedding(&gemini_api_key, &summary_text).await {
            let metadata = json!({
                "type": "summary",
                "conversation_id": conversation_id.to_string(),
                "last_message_id": last_processed_id.to_string(),
                "graph": {
                    "nodes": parsed_data["nodes"],
                    "links": parsed_data["edges"]
                }
            });

            rag::save_to_knowledge_base(
                &pool,
                &summary_text,
                embedding,
                Some(metadata),
                Some(conversation_id.clone()),
            )
            .await?;
            info!(conversation_id = %conversation_id, "Memory consolidation complete");
        }
    }

    Ok(())
}
