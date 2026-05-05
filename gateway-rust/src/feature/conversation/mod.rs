use crate::common::agent::agent_model::PromptActor;
use crate::common::agent::{function_call, send_prompt};
use crate::common::api_response::ApiResponse;
use crate::common::sse::sse_builder::{SseBuilder, SseTarget};
use crate::common::tools::ToolDispatcher;
use crate::feature::conversation::chat_model::{
    ChatRequest, ConversationResponse, CreateConversationRequest, MessageItem, MessageListParams,
    MessageListResponse, RestoreSoulRequest, RestoreSoulResponse, SoulHistoryResponse, UpdateConversationRequest,
};
use crate::{AppState, rag};
use axum::Json;
use axum::extract::{Path, State};
use chrono::Utc;
use serde_json::Value;
use sqlx::Row;
use tracing::{error, info};
use uuid::Uuid;

pub mod chat_model;

pub async fn handle_get_messages(
    State(state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
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
        SELECT id, conversation_id as "conversation_id!", role, content, thought, created_at as "created_at!"
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

pub async fn handle_get_conversations(
    State(state): State<AppState>,
) -> ApiResponse<Vec<ConversationResponse>> {
    info!("Fetching all conversations");

    let result = sqlx::query!(
        "SELECT id, title, session_id, created_at, updated_at FROM conversations ORDER BY updated_at DESC"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let convs = rows
                .into_iter()
                .map(|row| ConversationResponse {
                    id: row.id,
                    name: row.title.unwrap_or_default(),
                    session_id: row.session_id,
                    created_at: row.created_at.unwrap_or_else(Utc::now),
                    updated_at: row.updated_at.unwrap_or_else(Utc::now),
                })
                .collect();
            ApiResponse::ok(convs, "Conversations retrieved")
        }
        Err(e) => {
            error!("Failed to fetch conversations: {}", e);
            ApiResponse::failed("Failed to fetch conversations")
        }
    }
}

pub async fn handle_create_conversation(
    State(state): State<AppState>,
    Json(payload): Json<CreateConversationRequest>,
) -> ApiResponse<ConversationResponse> {
    info!("Creating new conversation");

    let id = Uuid::new_v4();
    let title = payload
        .name
        .or(payload.title)
        .unwrap_or_else(|| "New Conversation".to_string());

    let result = sqlx::query!(
        "INSERT INTO conversations (id, session_id, title, soul_content, bootstrap_content) VALUES ($1, $2, $3, $4, $5) RETURNING id, title, session_id, created_at, updated_at",
        id,
        payload.session_id,
        title,
        payload.soul_content,
        payload.bootstrap_content
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => {
            info!(conversation_id = %id, "Conversation created successfully");
            ApiResponse::ok(
                ConversationResponse {
                    id: row.id,
                    name: row.title.unwrap_or_default(),
                    session_id: row.session_id,
                    created_at: row.created_at.unwrap_or_else(Utc::now),
                    updated_at: row.updated_at.unwrap_or_else(Utc::now),
                },
                "Conversation created",
            )
        }
        Err(e) => {
            error!("Failed to create conversation: {}", e);
            ApiResponse::failed("Failed to create conversation")
        }
    }
}

pub async fn handle_update_conversation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateConversationRequest>,
) -> ApiResponse<ConversationResponse> {
    info!(conversation_id = %id, "Updating conversation");

    let result = sqlx::query!(
        "UPDATE conversations SET title = $1, updated_at = NOW() WHERE id = $2 RETURNING id, title, session_id, created_at, updated_at",
        payload.name,
        id
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => ApiResponse::ok(
            ConversationResponse {
                id: row.id,
                name: row.title.unwrap_or_default(),
                session_id: row.session_id,
                created_at: row.created_at.unwrap_or_else(Utc::now),
                updated_at: row.updated_at.unwrap_or_else(Utc::now),
            },
            "Conversation updated",
        ),
        Err(e) => {
            error!("Failed to update conversation: {}", e);
            ApiResponse::failed("Failed to update conversation")
        }
    }
}

pub async fn handle_get_soul_history(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Vec<SoulHistoryResponse>> {
    info!(conversation_id = %id, "Fetching soul history");

    let result = sqlx::query!(
        "SELECT id, version_number, change_reason, soul_content, created_at FROM soul_history WHERE conversation_id = $1 ORDER BY version_number DESC",
        id
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let history = rows
                .into_iter()
                .map(|row| SoulHistoryResponse {
                    id: row.id,
                    version: row.version_number,
                    change_reason: row.change_reason,
                    soul_content: row.soul_content,
                    created_at: row.created_at,
                })
                .collect();
            ApiResponse::ok(history, "Soul history retrieved")
        }
        Err(e) => {
            error!("Failed to fetch soul history: {}", e);
            ApiResponse::failed("Failed to fetch soul history")
        }
    }
}

pub async fn handle_restore_conversation_soul(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RestoreSoulRequest>,
) -> ApiResponse<RestoreSoulResponse> {
    info!(
        conversation_id = %id,
        version = %payload.version,
        "Restoring conversation soul"
    );

    let result: Result<Option<RestoreSoulResponse>, sqlx::Error> = async {
        let mut tx = state.pool.begin().await?;

        let history = sqlx::query(
            "SELECT soul_content FROM soul_history WHERE version_number = $1 AND conversation_id = $2",
        )
        .bind(payload.version)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(history) = history else {
            tx.rollback().await?;
            return Ok(None);
        };

        let soul_content: String = history.try_get("soul_content")?;
        let updated = sqlx::query(
            "UPDATE conversations SET soul_content = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(&soul_content)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        if updated.rows_affected() == 0 {
            tx.rollback().await?;
            return Ok(None);
        }

        tx.commit().await?;
        Ok(Some(RestoreSoulResponse {
            conversation_id: id,
            version: payload.version,
            soul_content,
        }))
    }
    .await;

    match result {
        Ok(Some(response)) => ApiResponse::ok(response, "Conversation soul restored"),
        Ok(None) => ApiResponse::not_found("Soul history not found for conversation"),
        Err(e) => {
            error!("Failed to restore conversation soul: {}", e);
            ApiResponse::failed("Failed to restore conversation soul")
        }
    }
}

pub async fn handle_delete_conversation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Value> {
    info!(conversation_id = %id, "Deleting conversation");

    // First delete messages
    let _ = sqlx::query!("DELETE FROM messages WHERE conversation_id = $1", id)
        .execute(&state.pool)
        .await;

    let result = sqlx::query!("DELETE FROM conversations WHERE id = $1", id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => ApiResponse::ok(serde_json::json!({}), "Conversation deleted"),
        Err(e) => {
            error!("Failed to delete conversation: {}", e);
            ApiResponse::failed("Failed to delete conversation")
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
            Some(conversation_id),
        );

        let conversation = sqlx::query!(
            "SELECT bootstrap_content, soul_content FROM conversations WHERE id = $1",
            conversation_id
        )
        .fetch_one(&state.pool)
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
                "-[{}] {}: {}.
",
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
            .join(
                "
---
",
            ); // Improved separator

        // C. Prepare Reasoning Loop

        let result = send_prompt(
            &state.gemini,
            None,
            PromptActor::User {
                history: history_text.clone(),
                memories: memories_text.clone(),
                message: user_message.clone(),
                system_prompt: system_prompt.clone(),
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
                system_prompt.clone(),
            )
            .await
            .map_or_else(|_| response, |r| r);

            // F. Finalization
            if let Ok(result) = sqlx::query!(
                "INSERT INTO messages (conversation_id, role, content, thought, created_at) VALUES ($1, 'user', $2, '', now()), ($1, 'assistant', $3, $4, now()) returning id, role, content, thought, created_at",
                conversation_id,
                user_message,
                function_result.1.content,
                function_result.1.thought
            )
            .fetch_all(&state.pool)
            .await
            {
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

                // Hierarchical Memory: Background Consolidation (Last Summarized ID Approach)
                let pool = state.pool.clone();
                let gemini = state.gemini.clone();
                let gemini_api_key = state.gemini_api_key.clone();

                tokio::spawn(async move {
                    // 1. Get the last summarized message ID from metadata
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
                    .await
                    .unwrap_or_default();

                    let last_msg_id = last_summary
                        .and_then(|r| r.last_message_id)
                        .and_then(|id| Uuid::parse_str(&id).ok());

                    // 2. Fetch messages created after that ID
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
                    .await
                    .unwrap_or_default();

                    // 3. Threshold check (10 or more new messages)
                    if new_messages.len() >= 10 {
                        info!(conversation_id = %conversation_id, "Triggering memory consolidation (new messages: {})", new_messages.len());

                        let last_processed_id = new_messages.last().map(|m| m.id).unwrap();
                        let mut summary_input = String::new();
                        for msg in new_messages {
                            summary_input.push_str(&format!("{}: {}
", msg.role, msg.content));
                        }

                        // 4. Summarize and extract graph
                        let summarizer_prompt = format!(
                            "Analyze the following conversation and return a JSON object with:
1. 'summary': A concise summary of permanent facts and project context.
2. 'nodes': An array of entities ({{'id': 'unique_id', 'label': 'Entity Name', 'node_type': 'Technology|Project|Person|Organization'}}).
3. 'edges': An array of relationships ({{'source': 'node_id', 'target': 'node_id', 'relationship': 'Description'}}).

Rules:
- Reuse IDs for the same entities across different parts of the conversation.
- 'id' should be lowercase and snake_case (e.g., 'rust', 'axum_framework').
- Focus on technical stack, project goals, and preferences.

Conversation:
{}
",
                            summary_input
                        );

                        let summary_res = gemini
                            .generate_content()
                            .with_user_message(summarizer_prompt)
                            .execute()
                            .await;

                        if let Ok(resp) = summary_res {
                            let raw_json = resp.text();

                            // Try to parse JSON from the response
                            let parsed_data: serde_json::Value =
                                if let Some(start) = raw_json.find('{') {
                                    if let Some(end) = raw_json.rfind('}') {
                                        serde_json::from_str(&raw_json[start..=end]).unwrap_or(
                                            serde_json::json!({
                                                "summary": raw_json,
                                                "nodes": [],
                                                "edges": []
                                            }),
                                        )
                                    } else {
                                        serde_json::json!({"summary": raw_json, "nodes": [], "edges": []})
                                    }
                                } else {
                                    serde_json::json!({"summary": raw_json, "nodes": [], "edges": []})
                                };

                            let summary_text = parsed_data["summary"]
                                .as_str()
                                .unwrap_or(&raw_json)
                                .to_string();

                            // 5. Embed and Save with last_message_id and graph metadata
                            if let Ok(embedding) =
                                rag::get_embedding(&gemini_api_key, &summary_text).await
                            {
                                let metadata = serde_json::json!({
                                    "type": "summary",
                                    "conversation_id": conversation_id.to_string(),
                                    "last_message_id": last_processed_id.to_string(),
                                    "graph": {
                                        "nodes": parsed_data["nodes"],
                                        "links": parsed_data["edges"]
                                    }
                                });

                                let save_result = rag::save_to_knowledge_base(
                                    &pool,
                                    &summary_text,
                                    embedding,
                                    Some(metadata),
                                )
                                .await;
                                info!("Memory consolidation complete conversation_id={} with={:?}", conversation_id, save_result);
                            }
                        }
                    }
                });
            }
        }
    });

    ApiResponse::ok("Streaming started".to_string(), "Success")
}
