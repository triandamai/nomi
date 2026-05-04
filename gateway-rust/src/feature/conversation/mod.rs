use crate::common::agent::agent_model::PromptActor;
use crate::common::agent::{function_call, send_prompt};
use crate::common::api_response::ApiResponse;
use crate::common::sse::sse_builder::{SseBuilder, SseTarget};
use crate::common::tools::ToolDispatcher;
use crate::feature::conversation::chat_model::{
    ChatRequest, CreateConversationRequest, CreateConversationResponse, MessageItem,
    MessageListParams, MessageListResponse,
};
use crate::{rag, AppState};
use axum::extract::State;
use axum::Json;
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use sqlx::Row;
use tracing::{error, info};
use uuid::Uuid;

pub mod chat_model;

pub async fn handle_get_messages(
    State(state): State<AppState>,
    axum::extract::Path(conversation_id): axum::extract::Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<MessageListParams>,
) -> ApiResponse<MessageListResponse> {
    let limit = params.limit.unwrap_or(20);
    let cursor = params.cursor.unwrap_or_else(Utc::now);

    info!(
        conversation_id = %conversation_id,
        cursor = %cursor,
        limit = limit,
        "Fetching messages"
    );

    let messages_result = sqlx::query_as!(
        MessageItem,
        r#"
        SELECT id, conversation_id as "conversation_id!", role, content,thought, created_at as "created_at!"
        FROM messages
        WHERE conversation_id = $1 AND created_at < $2
        ORDER BY created_at DESC
        LIMIT $3
        "#,
        conversation_id,
        cursor,
        limit
    )
    .fetch_all(&state.pool)
    .await;

    match messages_result {
        Ok(messages) => {
            let next_cursor = messages.last().map(|m| m.created_at);
            ApiResponse::ok(
                MessageListResponse {
                    messages,
                    next_cursor,
                },
                "Messages retrieved",
            )
        }
        Err(e) => {
            error!("Failed to fetch messages: {}", e);
            ApiResponse::failed("Failed to fetch messages")
        }
    }
}

pub async fn handle_create_conversation(
    State(state): State<AppState>,
    Json(payload): Json<CreateConversationRequest>,
) -> ApiResponse<CreateConversationResponse> {
    info!("Creating new conversation");

    let id = Uuid::new_v4();
    let result = sqlx::query!(
        "INSERT INTO conversations (id, session_id, title, soul_content, bootstrap_content) VALUES ($1, $2, $3, $4, $5)",
        id,
        payload.session_id,
        payload.title,
        payload.soul_content,
        payload.bootstrap_content
    )
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => {
            info!(conversation_id = %id, "Conversation created successfully");
            ApiResponse::ok(CreateConversationResponse { id }, "Conversation created")
        }
        Err(e) => {
            error!("Failed to create conversation: {}", e);
            ApiResponse::failed("Failed to create conversation")
        }
    }
}

pub async fn handle_chat_stream(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> ApiResponse<String> {
    info!(conversation_id = %payload.conversation_id, "Received chat stream request");

    let gemini_api_key = state.gemini_api_key.clone();
    let conversation_id = payload.conversation_id;
    let user_message = payload.message.clone();

    tokio::spawn(async move {
        let dispatcher = ToolDispatcher::new(
            state.pool.clone(),
            std::env::current_dir().unwrap_or_default(),
        );

        // A. Fetch last 10 messages for short-term history
        let history = sqlx::query!(
            "SELECT created_at, role, content FROM messages WHERE conversation_id = $1 ORDER BY created_at DESC LIMIT 10",
            conversation_id
        )
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

        let mut history_text = String::new();
        for msg in history.into_iter().rev() {
            history_text.push_str(&format!(
                "-[{}] {}: {}.\n",
                msg.created_at.unwrap_or(Utc::now()).to_rfc3339(),
                msg.role,
                msg.content
            ));
        }

        // B. Context Retrieval (RAG)
        let embedding = rag::get_embedding(&gemini_api_key, &user_message)
            .await
            .unwrap_or_default();
        let context_results = if !embedding.is_empty() {
            // Updated search to prioritize summaries in the future if we had a more complex query, 
            // but for now we'll just fetch more and filter or rely on the similarity.
            // The prompt asked to filter/prioritize entries where metadata->>'type' = 'summary'.
            rag::search_similar_with_summaries(&state.pool, embedding, 5)
                .await
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let memories_text = context_results
            .iter()
            .map(|r| r.content.clone())
            .collect::<Vec<String>>()
            .join("\n---\n"); // Improved separator

        // C. Prepare Reasoning Loop

        let result = send_prompt(
            &state.gemini,
            None,
            PromptActor::User {
                history: history_text.clone(),
                memories: memories_text.clone(),
                message: user_message.clone(),
            },
        )
        .await;

        if let Err(error) = result {
            error!("Got error {}", error);
            return;
        }

        if let Ok(response) = result {
            history_text.push_str(response.0.text().as_str());

            let function_result = function_call(
                &state.gemini,
                dispatcher,
                response.0.clone(),
                user_message.clone(),
                history_text.clone(),
                memories_text.clone(),
            )
            .await
            .map_or_else(|_| response, |r| r);

            // F. Finalization
            if let Ok(result) = sqlx::query!(
                "INSERT INTO messages (conversation_id, role, content,thought) VALUES ($1, 'user', $2,''), ($1, 'assistant', $3, $4) returning id,role,content,thought,created_at",
                conversation_id,
                user_message,
                function_result.1.content,
                function_result.1.thought
            )
                .fetch_all(&state.pool)
                .await {
                let data = result.iter().find(|m| m.role == "assistant");
                if let Some(record) = data {
                    let _ = &state.sse
                        .send(SseBuilder::new(
                            SseTarget::broadcast("message".to_string()),
                            MessageItem {
                                id: record.id.clone(),
                                conversation_id,
                                role: record.role.clone(),
                                content: record.content.clone(),
                                thought: record.thought.clone(),
                                created_at: Default::default(),
                            },
                        ))
                        .await;
                }

                // Hierarchical Memory: Background Consolidation
                let pool = state.pool.clone();
                let gemini = state.gemini.clone();
                let gemini_api_key = state.gemini_api_key.clone();
                
                tokio::spawn(async move {
                    let count_result = sqlx::query!(
                        "SELECT COUNT(*) as count FROM messages WHERE conversation_id = $1",
                        conversation_id
                    ).fetch_one(&pool).await;

                    if let Ok(row) = count_result {
                        let count = row.count.unwrap_or(0);
                        if count > 0 && count % 10 == 0 {
                            info!(conversation_id = %conversation_id, "Triggering memory consolidation (count: {})", count);
                            
                            // 1. Fetch last 10 messages
                            let last_10 = sqlx::query!(
                                "SELECT role, content FROM messages WHERE conversation_id = $1 ORDER BY created_at DESC LIMIT 10",
                                conversation_id
                            ).fetch_all(&pool).await.unwrap_or_default();

                            let mut summary_input = String::new();
                            for msg in last_10.into_iter().rev() {
                                summary_input.push_str(&format!("{}: {}\n", msg.role, msg.content));
                            }

                            // 2. Summarize
                            let summarizer_prompt = format!(
                                "Summarize the following 10 messages into a concise set of 'Permanent Facts & Project Context'. Focus on key decisions, technical details, and Trian's preferences. Be extremely concise.\n\n{}",
                                summary_input
                            );

                            let summary_res = gemini.generate_content()
                                .with_user_message(summarizer_prompt)
                                .execute()
                                .await;

                            if let Ok(resp) = summary_res {
                                let summary_text = resp.text();
                                
                                // 3. Embed
                                if let Ok(embedding) = rag::get_embedding(&gemini_api_key, &summary_text).await {
                                    // 4. Save to knowledge_base
                                    let metadata = serde_json::json!({
                                        "type": "summary",
                                        "conversation_id": conversation_id.to_string()
                                    });
                                    
                                    let save_to_summary = rag::save_to_knowledge_base(&pool, &summary_text, embedding, Some(metadata)).await;
                                    info!("Memory consolidation complete conversation_id={} with={:?}",conversation_id,save_to_summary);
                                }
                            }
                        }
                    }
                });
            }
        }
    });

    ApiResponse::ok("Streaming started".to_string(), "Success")
}
