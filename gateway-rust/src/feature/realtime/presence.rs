use crate::AppState;
use crate::common::agent::agent_model::PromptActor;
use crate::common::sse::sse_builder::{SseBuilder, SseTarget};
use crate::common::tools::ToolDispatcher;
use crate::feature::conversation::chat_model::MessageItem;
use crate::feature::conversation::internal_model::InboundMessage;
use crate::rag;
use chrono::Utc;
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};
use tracing::{error, info};
use uuid::Uuid;

pub struct PresenceManager {
    // conversation_id -> last_activity
    pub debouncers: DashMap<Uuid, Instant>,
    pub channel_tx: mpsc::Sender<DebounceEvent>,
}

#[derive(Debug)]
pub enum DebounceEvent {
    NewMessage(Uuid, InboundMessage),
}

impl PresenceManager {
    pub fn new(state: AppState) -> Arc<Self> {
        let (tx, mut rx) = mpsc::channel::<DebounceEvent>(100);
        let manager = Arc::new(Self {
            debouncers: DashMap::new(),
            channel_tx: tx,
        });

        let manager_clone = manager.clone();
        tokio::spawn(async move {
            let mut pending_messages: DashMap<Uuid, (InboundMessage, Instant)> = DashMap::new();

            loop {
                tokio::select! {
                    Some(event) = rx.recv() => {
                        match event {
                            DebounceEvent::NewMessage(conv_id, msg) => {
                                pending_messages.insert(conv_id, (msg, Instant::now() + Duration::from_secs(2)));
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(500)) => {
                        let now = Instant::now();
                        let mut to_process = Vec::new();

                        for entry in pending_messages.iter() {
                            if now >= entry.value().1 {
                                to_process.push(*entry.key());
                            }
                        }

                        for conv_id in to_process {
                            if let Some((_, (msg, _))) = pending_messages.remove(&conv_id) {
                                let state_inner = state.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = process_debounced_message(state_inner, conv_id, msg).await {
                                        error!("Error processing debounced message: {}", e);
                                    }
                                });
                            }
                        }
                    }
                }
            }
        });

        manager_clone
    }
}

async fn process_debounced_message(
    state: AppState,
    conversation_id: Uuid,
    msg: InboundMessage,
) -> anyhow::Result<()> {
    info!(conversation_id = %conversation_id, "Processing debounced message");

    // 1. Notify channel-rust to start typing
    let channel_rust_url = std::env::var("CHANNEL_RUST_URL").unwrap_or_else(|_| "http://localhost:8001".to_string());
    let client = reqwest::Client::new();
    
    // Attempt to send typing heartbeat
    let _ = client.post(format!("{}/api/presence/typing", channel_rust_url))
        .json(&serde_json::json!({
            "chat_id": msg.chat_id,
            "channel": msg.channel,
            "is_typing": true
        }))
        .send()
        .await;

    // Broadcast to SSE
    let _ = state.sse.send(SseBuilder::new(
        SseTarget::broadcast("presence".to_string()),
        serde_json::json!({
            "conversation_id": conversation_id,
            "is_typing": true,
            "user_id": "nomi" // Nomi is typing
        }),
    )).await;

    // 2. Start Agentic Loop (similar to handle_chat_stream but with persistence)
    // For now, let's reuse logic or trigger handle_chat_stream logic
    // We'll implement the full flow here for persistence
    
    let user_message = msg.text.clone();
    let gemini_api_key = state.gemini_api_key.clone();

    let dispatcher = ToolDispatcher::new(
        state.pool.clone(),
        std::env::current_dir().unwrap_or_default(),
        Some(conversation_id),
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
            combined.push_str("

### Current Personality/Soul
");
            combined.push_str(&soul);
        }
        combined
    };

    // Fetch history
    let history = sqlx::query!(
        "SELECT created_at, role, content FROM messages WHERE conversation_id = $1 ORDER BY created_at DESC LIMIT 10",
        conversation_id
    )
    .fetch_all(&state.pool)
    .await.unwrap_or_default();

    let mut history_text = String::new();
    for msg in history.into_iter().rev() {
        history_text.push_str(&format!(
            "-[{}] {}: {}.
",
            msg.created_at.unwrap_or(Utc::now()).to_rfc3339(),
            msg.role,
            msg.content
        ));
    }

    // RAG
    let embedding = rag::get_embedding(&gemini_api_key, &user_message).await.unwrap_or_default();
    let memories_text = if !embedding.is_empty() {
        rag::search_similar_with_summaries(&state.pool, embedding, 5)
            .await?
            .iter()
            .map(|r| r.content.clone())
            .collect::<Vec<String>>()
            .join("
---
")
    } else {
        String::new()
    };

    let mut current_actor = PromptActor::User {
        history: history_text.clone(),
        memories: memories_text.clone(),
        message: user_message.clone(),
        system_prompt: system_prompt.clone(),
    };

    let mut loop_count = 0;
    let max_loops = 5;
    let mut final_response = None;
    let mut previous_calls = Vec::new();

    while loop_count < max_loops {
        loop_count += 1;
        let result = crate::common::agent::send_prompt(&state.gemini, current_actor).await;

        match result {
            Ok((response, chunk)) => {
                // Emit thought
                if !chunk.thought.is_empty() {
                    let _ = state.sse.send(SseBuilder::new(
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
                    &user_message,
                    Some(state.sse.clone()),
                ).await;

                current_actor = PromptActor::MultiTool {
                    history: history_text.clone(),
                    memories: memories_text.clone(),
                    message: user_message.clone(),
                    system_prompt: system_prompt.clone(),
                    tool_results,
                    previous_calls: previous_calls.clone(),
                };
            }
            Err(e) => {
                error!("Loop error: {}", e);
                break;
            }
        }
    }

    if let Some((_, function_result)) = final_response {
        // Save to DB
        let result = sqlx::query!(
            "INSERT INTO messages (conversation_id, role, content, thought, created_at) VALUES ($1, 'assistant', $2, $3, now()) RETURNING id, role, content, thought, created_at",
            conversation_id,
            function_result.content,
            function_result.thought
        )
        .fetch_one(&state.pool)
        .await?;

        // Broadcast to SSE
        let _ = state.sse.send(SseBuilder::new(
            SseTarget::broadcast("message".to_string()),
            MessageItem {
                id: result.id,
                conversation_id,
                role: result.role,
                content: result.content.clone(),
                thought: result.thought,
                created_at: result.created_at.unwrap_or_else(Utc::now),
            },
        )).await;

        // Send to channel-rust outbound
        let _ = client.post(format!("{}/api/outbound", channel_rust_url))
            .json(&serde_json::json!({
                "chat_id": msg.chat_id,
                "channel": msg.channel,
                "text": function_result.content
            }))
            .send()
            .await;
            
        // Stop typing
        let _ = client.post(format!("{}/api/presence/typing", channel_rust_url))
            .json(&serde_json::json!({
                "chat_id": msg.chat_id,
                "channel": msg.channel,
                "is_typing": false
            }))
            .send()
            .await;

        let _ = state.sse.send(SseBuilder::new(
            SseTarget::broadcast("presence".to_string()),
            serde_json::json!({
                "conversation_id": conversation_id,
                "is_typing": false,
                "user_id": "nomi"
            }),
        )).await;
    }

    Ok(())
}
