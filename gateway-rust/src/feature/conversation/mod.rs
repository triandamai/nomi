use crate::common::api_response::ApiResponse;
use crate::feature::conversation::chat_model::{
    ChatRequest, ConversationResponse, CreateConversationRequest, MessageItem, MessageListParams,
    MessageListResponse, PairingResponse, RestoreSoulRequest, RestoreSoulResponse,
    SoulHistoryResponse, UpdateConversationRequest,
};
use crate::feature::conversation::internal_model::InboundMessage;
use crate::feature::realtime::presence::DebounceEvent;
use crate::{rag, AppState};
use axum::extract::{Path, State};
use axum::Json;
use chrono::Utc;

use rand::{rng, Rng, RngExt};
use rand::distr::Alphanumeric;
use serde_json::Value;
use sqlx::Row;
use tracing::{error, info};
use uuid::Uuid;
use crate::common::agent::agent_model::PromptActor;
use crate::common::agent::execute_tools;
use crate::common::sse::sse_builder::{SseBuilder, SseTarget};
use crate::common::tools::ToolDispatcher;

pub mod chat_model;
pub mod internal_model;

pub async fn handle_create_pairing(
    State(state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
) -> ApiResponse<PairingResponse> {
    // Generate a random 6-character alphanumeric code
    let pairing_code: String = rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>()
        .to_uppercase();

    let expires_at = Utc::now() + chrono::Duration::minutes(10);

    match sqlx::query!(
        "INSERT INTO pairing_rooms (conversation_id, pairing_code, expires_at) VALUES ($1, $2, $3) RETURNING pairing_code, expires_at",
        conversation_id,
        pairing_code,
        expires_at
    )
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => ApiResponse::ok(
            PairingResponse {
                pairing_code: row.pairing_code,
                expires_at: row.expires_at.unwrap_or(expires_at),
            },
            "Pairing code generated",
        ),
        Err(e) => {
            error!("Failed to create pairing room: {}", e);
            ApiResponse::failed("Failed to create pairing room")
        }
    }
}

pub async fn handle_internal_inbound(
    State(state): State<AppState>,
    Json(payload): Json<InboundMessage>,
) -> ApiResponse<Value> {
    info!(
        sender_id = %payload.sender_id,
        chat_id = %payload.chat_id,
        channel = %payload.channel,
        "Received internal inbound message"
    );

    // 1. Resolve/Create User
    let user_id = match sqlx::query!(
        "INSERT INTO users (external_id, display_name) VALUES ($1, $2) ON CONFLICT (external_id) DO UPDATE SET display_name = EXCLUDED.display_name RETURNING id",
        payload.sender_id,
        payload.sender_id // Default display name to sender_id if not provided
    )
    .fetch_one(&state.pool)
    .await {
        Ok(row) => row.id,
        Err(e) => {
            error!("Failed to resolve user: {}", e);
            return ApiResponse::failed("Failed to resolve user");
        }
    };

    // 2. Check for Pairing Code
    // format: /pair CODE or PAIR CODE
    let text = payload.text.trim();
    if text.to_uppercase().starts_with("PAIR ") || text.to_uppercase().starts_with("/PAIR ") {
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() >= 2 {
            let code = parts[1].to_uppercase();
            
            // Try to find a valid pairing room
            let pairing_room = sqlx::query!(
                "SELECT id, conversation_id FROM pairing_rooms WHERE pairing_code = $1 AND expires_at > now() AND user_id IS NULL",
                code
            )
            .fetch_optional(&state.pool)
            .await;

            if let Ok(Some(room)) = pairing_room {
                let conv_id = room.conversation_id;

                // Mark pairing room as used
                let _ = sqlx::query!(
                    "UPDATE pairing_rooms SET user_id = $1 WHERE id = $2",
                    user_id,
                    room.id
                ).execute(&state.pool).await;

                // Create channel entry if it doesn't exist
                let _ = sqlx::query!(
                    "INSERT INTO channels (channel_type, external_id, external_chat_id, conversation_id) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
                    payload.channel,
                    payload.sender_id,
                    payload.chat_id,
                    conv_id
                ).execute(&state.pool).await;

                // Add member
                let _ = sqlx::query!(
                    "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    conv_id,
                    user_id
                ).execute(&state.pool).await;

                // Broadcast to SSE so Web UI shows pairing success
                let _ = state.sse.send(SseBuilder::new(
                    SseTarget::broadcast("pairing_success".to_string()),
                    serde_json::json!({
                        "conversation_id": conv_id,
                        "platform": payload.channel,
                        "message": format!("Successfully paired with {}!", payload.channel)
                    })
                )).await;

                // Trigger LLM to send a congratulatory message
                let congrats_payload = InboundMessage {
                    sender_id: payload.sender_id.clone(),
                    chat_id: payload.chat_id.clone(),
                    channel: payload.channel.clone(),
                    text: format!("(System: The user has successfully paired their {} account. Please congratulate them briefly and confirm they can now chat from here.)", payload.channel),
                };

                if let Err(e) = state.presence.channel_tx.send(DebounceEvent::NewMessage(conv_id, congrats_payload)).await {
                    error!("Failed to send congrats trigger to debouncer: {}", e);
                }

                // Send success response back to the platform
                return ApiResponse::ok(serde_json::json!({ 
                    "status": "paired", 
                    "conversation_id": conv_id,
                    "message": "Pairing successful! This conversation is now linked." 
                }), "Pairing successful");
            }
        }
    }

    // 3. Resolve/Create Conversation (Regular flow)
    let conversation_id = match sqlx::query!(
        "SELECT conversation_id FROM channels WHERE channel_type = $1 AND external_chat_id = $2",
        payload.channel,
        payload.chat_id
    )
    .fetch_optional(&state.pool)
    .await {
        Ok(Some(row)) => row.conversation_id.unwrap(),
        Ok(None) => {
            // Create new conversation
            let conv_id = Uuid::new_v4();
            let title = format!("{} via {}", payload.chat_id, payload.channel);
            
            if let Err(e) = sqlx::query!(
                "INSERT INTO conversations (id, title) VALUES ($1, $2)",
                conv_id,
                title
            ).execute(&state.pool).await {
                error!("Failed to create conversation: {}", e);
                return ApiResponse::failed("Failed to create conversation");
            }

            if let Err(e) = sqlx::query!(
                "INSERT INTO channels (channel_type, external_id, external_chat_id, conversation_id) VALUES ($1, $2, $3, $4)",
                payload.channel,
                payload.sender_id,
                payload.chat_id,
                conv_id
            ).execute(&state.pool).await {
                error!("Failed to link channel: {}", e);
                return ApiResponse::failed("Failed to link channel");
            }

            // Add member
            let _ = sqlx::query!(
                "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                conv_id,
                user_id
            ).execute(&state.pool).await;

            conv_id
        }
        Err(e) => {
            error!("Database error: {}", e);
            return ApiResponse::failed("Database error");
        }
    };

    // 3. Save incoming message immediately
    let save_res = sqlx::query!(
        "INSERT INTO messages (conversation_id, role, content, thought, created_at) VALUES ($1, 'user', $2, '', now()) RETURNING id, role, content, thought, created_at",
        conversation_id,
        payload.text
    ).fetch_one(&state.pool).await;

    if let Ok(m) = save_res {
        // Broadcast to SSE so Web UI shows it
        let _ = state.sse.send(SseBuilder::new(
            SseTarget::broadcast("message".to_string()),
            MessageItem {
                id: m.id,
                conversation_id,
                role: m.role,
                content: m.content,
                thought: m.thought,
                created_at: m.created_at.unwrap_or_else(Utc::now),
            }
        )).await;
    }

    // 4. Send to Debouncer
    if let Err(e) = state.presence.channel_tx.send(DebounceEvent::NewMessage(conversation_id, payload)).await {
        error!("Failed to send to debouncer: {}", e);
    }

    ApiResponse::ok(serde_json::json!({ "status": "received", "conversation_id": conversation_id }), "Message received")
}

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
        // Start typing
        let _ = state.sse.send(SseBuilder::new(
            SseTarget::broadcast("presence".to_string()),
            serde_json::json!({
                "conversation_id": conversation_id,
                "is_typing": true,
                "user_id": "nomi"
            }),
        )).await;

        let dispatcher = ToolDispatcher::new(
            state.pool.clone(),
            std::env::current_dir().unwrap_or_default(),
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
        let mut loop_count = 0;
        let max_loops = 5;
        let mut current_actor = PromptActor::User {
            history: history_text.clone(),
            memories: memories_text.clone(),
            message: user_message.clone(),
            system_prompt: system_prompt.clone(),
        };

        let mut final_response = None;
        let mut previous_calls = Vec::new();

        while loop_count < max_loops {
            loop_count += 1;
            info!(loop_count = loop_count, "Starting agentic loop iteration");

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
                        info!("No tool calls requested, finishing loop");
                        final_response = Some((response, chunk));
                        break;
                    }

                    info!(count = tool_calls.len(), "Executing parallel tool calls");
                    
                    // Track calls for history
                    let current_calls: Vec<_> = tool_calls.into_iter().map(|c| c.clone()).collect();
                    previous_calls.extend(current_calls.clone());

                    let tool_results = execute_tools(
                        &dispatcher,
                        current_calls.clone(),
                        &user_message,
                        Some(state.sse.clone()),
                    ).await;

                    // Update actor for next turn
                    current_actor = PromptActor::MultiTool {
                        history: history_text.clone(),
                        memories: memories_text.clone(),
                        message: user_message.clone(),
                        system_prompt: system_prompt.clone(),
                        tool_results,
                        previous_calls: previous_calls.clone(),
                    };
                }
                Err(error) => {
                    error!("Agentic loop error: {}", error);
                    break;
                }
            }
        }

        if let Some((_, function_result)) = final_response {
            // F. Finalization
            if let Ok(result) = sqlx::query!(
                "INSERT INTO messages (conversation_id, role, content, thought, created_at) VALUES ($1, 'user', $2, '', now()), ($1, 'assistant', $3, $4, now()) returning id, role, content, thought, created_at",
                conversation_id,
                user_message,
                function_result.content,
                function_result.thought
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

                // Stop typing
                let _ = state.sse.send(SseBuilder::new(
                    SseTarget::broadcast("presence".to_string()),
                    serde_json::json!({
                        "conversation_id": conversation_id,
                        "is_typing": false,
                        "user_id": "nomi"
                    }),
                )).await;

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
- NEVER create a node with id 'summary' or that represents the conversation summary itself.
- Extract individual entities (specific technologies, names, project names).
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
