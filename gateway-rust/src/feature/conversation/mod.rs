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

    // 1. Check for Pairing Code
    let text = payload.text.trim();
    if text.to_uppercase().starts_with("PAIR ") || text.to_uppercase().starts_with("/PAIR ") {
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() >= 2 {
            let code = parts[1].to_uppercase();
            
            // Start Transaction for Pairing
            let mut tx = match state.pool.begin().await {
                Ok(tx) => tx,
                Err(e) => {
                    error!("Failed to start transaction: {}", e);
                    return ApiResponse::failed("Internal server error");
                }
            };

            // Try to find a valid pairing room
            let pairing_room = sqlx::query!(
                "SELECT id, conversation_id FROM pairing_rooms WHERE pairing_code = $1 AND expires_at > now() AND user_id IS NULL",
                code
            )
            .fetch_optional(&mut *tx)
            .await;

            if let Ok(Some(room)) = pairing_room {
                let conv_id = room.conversation_id;

                // Resolve/Create User
                let user_id = match sqlx::query!(
                    "INSERT INTO users (external_id, display_name) VALUES ($1, $2) ON CONFLICT (external_id) DO UPDATE SET display_name = EXCLUDED.display_name RETURNING id",
                    payload.sender_id,
                    payload.sender_id
                )
                .fetch_one(&mut *tx)
                .await {
                    Ok(row) => row.id,
                    Err(e) => {
                        error!("Failed to resolve user in tx: {}", e);
                        let _ = tx.rollback().await;
                        return ApiResponse::failed("Failed to resolve user");
                    }
                };

                // Mark pairing room as used
                if let Err(e) = sqlx::query!(
                    "UPDATE pairing_rooms SET user_id = $1 WHERE id = $2",
                    user_id,
                    room.id
                ).execute(&mut *tx).await {
                    error!("Failed to update pairing room: {}", e);
                    let _ = tx.rollback().await;
                    return ApiResponse::failed("Failed to link pairing room");
                }

                // Create channel entry
                if let Err(e) = sqlx::query!(
                    "INSERT INTO channels (channel_type, external_id, external_chat_id, conversation_id, user_id) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (channel_type, external_chat_id) DO UPDATE SET external_id = EXCLUDED.external_id, user_id = EXCLUDED.user_id",
                    payload.channel,
                    payload.sender_id,
                    payload.chat_id,
                    conv_id,
                    user_id
                ).execute(&mut *tx).await {
                    error!("Failed to link channel in tx: {}", e);
                    let _ = tx.rollback().await;
                    return ApiResponse::failed("Failed to link channel");
                }

                // Add member
                if let Err(e) = sqlx::query!(
                    "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    conv_id,
                    user_id
                ).execute(&mut *tx).await {
                    error!("Failed to add conversation member: {}", e);
                    let _ = tx.rollback().await;
                    return ApiResponse::failed("Failed to join conversation");
                }

                // Commit Transaction
                if let Err(e) = tx.commit().await {
                    error!("Failed to commit pairing transaction: {}", e);
                    return ApiResponse::failed("Failed to finalize pairing");
                }

                // Broadcast to SSE so Web UI shows pairing success
                let _ = state.sse.send(SseBuilder::new(
                    SseTarget::broadcast("pairing_success".to_string()),
                    serde_json::json!({
                        "conversation_id": conv_id,
                        "platform": payload.channel,
                        "user_id": user_id,
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

                // We need to insert this system message so the debouncer can process it
                let _ = sqlx::query!(
                    "INSERT INTO messages (conversation_id, role, content, thought, user_id, created_at) VALUES ($1, 'user', $2, '', $3, now())",
                    conv_id,
                    congrats_payload.text,
                    user_id
                ).execute(&state.pool).await;

                if let Err(e) = state.presence.channel_tx.send(DebounceEvent::NewMessage(conv_id, user_id)).await {
                    error!("Failed to send congrats trigger to debouncer: {}", e);
                }

                return ApiResponse::ok(serde_json::json!({ 
                    "status": "paired", 
                    "conversation_id": conv_id,
                    "message": "Pairing successful! This conversation is now linked." 
                }), "Pairing successful");
            } else {
                let _ = tx.rollback().await;
            }
        }
    }

    // 2. Resolve Conversation & User for existing channel messages
    let (conversation_id, user_id) = match sqlx::query!(
        "SELECT c.conversation_id, u.id as user_id 
         FROM channels c 
         JOIN users u ON u.external_id = c.external_id
         WHERE c.channel_type = $1 AND c.external_chat_id = $2",
        payload.channel,
        payload.chat_id
    )
    .fetch_optional(&state.pool)
    .await {
        Ok(Some(row)) => (row.conversation_id.unwrap(), row.user_id),
        Ok(None) => {
            // Start transaction for new conversation
            let mut tx = match state.pool.begin().await {
                Ok(tx) => tx,
                Err(e) => {
                    error!("Failed to start transaction: {}", e);
                    return ApiResponse::failed("Internal server error");
                }
            };

            // Resolve User
            let u_id = match sqlx::query!(
                "INSERT INTO users (external_id, display_name) VALUES ($1, $2) ON CONFLICT (external_id) DO UPDATE SET display_name = EXCLUDED.display_name RETURNING id",
                payload.sender_id,
                payload.sender_id
            ).fetch_one(&mut *tx).await {
                Ok(r) => r.id,
                Err(e) => {
                    error!("Failed to resolve user: {}", e);
                    let _ = tx.rollback().await;
                    return ApiResponse::failed("Failed to resolve user");
                }
            };

            // Create new conversation
            let conv_id = Uuid::new_v4();
            let title = format!("{} via {}", payload.chat_id, payload.channel);
            
            if let Err(e) = sqlx::query!("INSERT INTO conversations (id, title) VALUES ($1, $2)", conv_id, title).execute(&mut *tx).await {
                error!("Failed to create conversation: {}", e);
                let _ = tx.rollback().await;
                return ApiResponse::failed("Failed to create conversation");
            }

            if let Err(e) = sqlx::query!(
                "INSERT INTO channels (channel_type, external_id, external_chat_id, conversation_id, user_id) VALUES ($1, $2, $3, $4, $5)",
                payload.channel, payload.sender_id, payload.chat_id, conv_id, u_id
            ).execute(&mut *tx).await {
                error!("Failed to link channel: {}", e);
                let _ = tx.rollback().await;
                return ApiResponse::failed("Failed to link channel");
            }

            if let Err(e) = sqlx::query!(
                "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                conv_id, u_id
            ).execute(&mut *tx).await {
                error!("Failed to add member: {}", e);
                let _ = tx.rollback().await;
                return ApiResponse::failed("Failed to join conversation");
            }

            if let Err(e) = tx.commit().await {
                error!("Failed to commit new conversation: {}", e);
                return ApiResponse::failed("Failed to create conversation");
            }

            (conv_id, u_id)
        }
        Err(e) => {
            error!("Database error: {}", e);
            return ApiResponse::failed("Database error");
        }
    };

    // 3. Save incoming message immediately
    let save_res = sqlx::query!(
        "INSERT INTO messages (conversation_id, role, content, thought, user_id, created_at) VALUES ($1, 'user', $2, '', $3, now()) RETURNING id, role, content, thought, created_at, user_id",
        conversation_id,
        payload.text,
        user_id
    ).fetch_one(&state.pool).await;

    match save_res {
        Ok(m) => {
            info!(conversation_id = %conversation_id, message_id = %m.id, "Persisted channel message to DB");
            
            // Broadcast to SSE so Web UI shows it
            let _ = state.sse.send(SseBuilder::new(
                SseTarget::broadcast("message".to_string()),
                serde_json::json!({
                    "id": m.id,
                    "conversation_id": conversation_id,
                    "role": m.role,
                    "content": m.content,
                    "thought": m.thought,
                    "user_id": m.user_id,
                    "created_at": m.created_at.unwrap_or_else(Utc::now),
                })
            )).await;

            // 4. Send to Debouncer
            if let Err(e) = state.presence.channel_tx.send(DebounceEvent::NewMessage(conversation_id, user_id)).await {
                error!("Failed to send to debouncer: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to save message: {}", e);
            return ApiResponse::failed("Failed to save message");
        }
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
        SELECT id, conversation_id as "conversation_id!", role, content, thought, user_id, created_at as "created_at!"
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

    let state_clone = state.clone();
    let conversation_id = payload.conversation_id;
    let user_message = payload.message.clone();

    tokio::spawn(async move {
        // Resolve user_id from conversation if possible (for Web UI, usually we have a session)
        let user_id = sqlx::query!(
            "SELECT s.user_id FROM conversations c LEFT JOIN sessions s ON c.session_id = s.id WHERE c.id = $1",
            conversation_id
        )
        .fetch_one(&state_clone.pool)
        .await
        .ok()
        .and_then(|c| Some(c.user_id));

        let unified_msg = crate::feature::message_processor::UnifiedMessage {
            conversation_id,
            user_id,
            text_content: user_message,
            source: crate::feature::message_processor::MessageSource::Web,
        };

        if let Err(e) = crate::feature::message_processor::process_incoming_message(state_clone, unified_msg).await {
            error!("Failed to process web message: {}", e);
        }
    });

    ApiResponse::ok("Streaming started".to_string(), "Success")
}
