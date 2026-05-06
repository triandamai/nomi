use crate::common::api_response::ApiResponse;
use crate::feature::conversation::chat_model::{ChannelStatus, ChatRequest, ConversationResponse, CreateConversationRequest, MessageItem, MessageListParams, MessageListResponse, PairingResponse, RestoreSoulRequest, RestoreSoulResponse, SoulHistoryResponse, UpdateConversationRequest, UserChannelsResponse};
use crate::feature::conversation::internal_model::InboundMessage;
use crate::feature::realtime::presence::DebounceEvent;
use crate::{rag, AppState};
use axum::extract::{Path, State, Request};
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
pub mod auth;

pub async fn handle_get_user_channels(
    State(state): State<AppState>,
) -> ApiResponse<UserChannelsResponse> {
    // Note: In a real app, we'd get user_id from session. 
    // For this prototype/current state, we might need a default or use the session_id if available.
    // Based on register_public_sse, user_id is passed as a query param there.
    // Here we'll check all channels to see if any are linked.
    // Since we don't have Auth middleware yet, let's look for channels linked to the current active conversations.
    
    let result = sqlx::query!(
        "SELECT DISTINCT channel_type FROM channels"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let platforms = vec!["telegram".to_string(), "whatsapp".to_string()];
            let mut channels = Vec::new();
            
            let linked_platforms: std::collections::HashSet<String> = rows.into_iter().map(|r| r.channel_type).collect();

            for p in platforms {
                channels.push(ChannelStatus {
                    paired: linked_platforms.contains(&p),
                    platform: p,
                });
            }

            ApiResponse::ok(UserChannelsResponse { channels }, "User channels retrieved")
        }
        Err(e) => {
            error!("Failed to fetch user channels: {}", e);
            ApiResponse::failed("Failed to fetch user channels")
        }
    }
}

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
        "SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC"
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
        "INSERT INTO conversations (id, title, soul_content, bootstrap_content) VALUES ($1, $2, $3, $4) RETURNING id, title, created_at, updated_at",
        id,
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
        "UPDATE conversations SET title = $1, updated_at = NOW() WHERE id = $2 RETURNING id, title, created_at, updated_at",
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
    axum::extract::Extension(claims): axum::extract::Extension<crate::feature::conversation::auth::Claims>,
    Json(payload): Json<ChatRequest>,
) -> ApiResponse<String> {
    info!(conversation_id = %payload.conversation_id, "Received chat stream request");

    let state_clone = state.clone();
    let conversation_id = payload.conversation_id;
    let user_message = payload.message.clone();

    // Resolve user_id from JWT claims
    let user_id = Uuid::parse_str(&claims.sub).ok();

    tokio::spawn(async move {
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
