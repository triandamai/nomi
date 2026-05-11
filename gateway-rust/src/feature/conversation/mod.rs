use crate::AppState;
use crate::common::api_response::ApiResponse;
use crate::feature::conversation::chat_model::{
    ChannelStatus, ChatRequest, ConversationResponse, CreateConversationRequest, MessageItem,
    MessageListParams, MessageListResponse, PairingResponse, RestoreSoulRequest,
    RestoreSoulResponse, SoulHistoryResponse, UpdateConversationRequest, UserChannelsResponse,
};
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use chrono::Utc;

use rand::distr::Alphanumeric;
use rand::{ rng, RngExt};
use serde_json::Value;
use sqlx::Row;
use tracing::{error, info};
use uuid::Uuid;

pub mod auth;
pub mod chat_model;
pub mod reminder;

pub async fn handle_get_user_channels(
    State(state): State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<auth::Claims>,
) -> ApiResponse<UserChannelsResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiResponse::failed("Invalid user ID in token"),
    };

    let result = sqlx::query!(
        "SELECT DISTINCT channel_type FROM channels WHERE user_id = $1",
        user_id
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let platforms = vec!["telegram".to_string(), "whatsapp".to_string()];
            let mut channels = Vec::new();

            let linked_platforms: std::collections::HashSet<String> =
                rows.into_iter().map(|r| r.channel_type).collect();

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
    axum::extract::Extension(claims): axum::extract::Extension<crate::feature::conversation::auth::Claims>,
    Path(conversation_id): Path<Uuid>,
) -> ApiResponse<PairingResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiResponse::failed("Invalid user ID in token"),
    };

    // Verify membership
    let membership = sqlx::query!(
        "SELECT 1 as one FROM conversation_members WHERE conversation_id = $1 AND user_id = $2",
        conversation_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await;

    match membership {
        Ok(Some(_)) => (),
        Ok(None) => return ApiResponse::failed("Forbidden: You are not a member of this conversation"),
        Err(e) => {
            error!("Failed to verify membership: {}", e);
            return ApiResponse::failed("Internal server error");
        }
    }

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
    axum::extract::Extension(claims): axum::extract::Extension<crate::feature::conversation::auth::Claims>,
    Path(conversation_id): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<MessageListParams>,
) -> ApiResponse<MessageListResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiResponse::failed("Invalid user ID in token"),
    };

    // Verify membership
    let membership = sqlx::query!(
        "SELECT 1 as one FROM conversation_members WHERE conversation_id = $1 AND user_id = $2",
        conversation_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await;

    match membership {
        Ok(Some(_)) => (),
        Ok(None) => return ApiResponse::failed("Forbidden: You are not a member of this conversation"),
        Err(e) => {
            error!("Failed to verify membership: {}", e);
            return ApiResponse::failed("Internal server error");
        }
    }

    let limit = params.limit.unwrap_or(20);
    let cursor = params.cursor.unwrap_or_else(Utc::now);

    info!(
        conversation_id = %conversation_id,
        user_id = %user_id,
        cursor = %cursor,
        limit = limit,
        "Fetching messages"
    );

    let messages_result = sqlx::query_as!(
        MessageItem,
        r#"
        SELECT id,
               conversation_id as "conversation_id!",
               role,
               content,
               thought,
               user_id,
               created_at as "created_at!",
               total_tokens,
               answer_tokens,
               prompt_tokens,
               image_url,
               video_url,
               audio_url,
               document_url,
               sticker_url
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

pub async fn handle_get_file(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> impl axum::response::IntoResponse {
    let bucket = "conversations";
    match state.storage.get_file(bucket.to_string(), filename.clone()).await {
        Ok(data) => {
            let mime = mime_guess::from_path(&filename)
                .first_or_octet_stream()
                .to_string();
            
            (
                [
                    (axum::http::header::CONTENT_TYPE, mime),
                    (axum::http::header::CACHE_CONTROL, "public, max-age=31536000".to_string()),
                ],
                data.to_vec(),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to get file from storage: {}", e);
            (axum::http::StatusCode::NOT_FOUND, "File not found").into_response()
        }
    }
}

pub async fn handle_get_conversations(
    State(state): State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<crate::feature::conversation::auth::Claims>,
) -> ApiResponse<Vec<ConversationResponse>> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiResponse::failed("Invalid user ID in token"),
    };

    info!(user_id = %user_id, "Fetching user conversations");

    let result = sqlx::query!(
        r#"
        SELECT c.id, c.title, c.created_at, c.updated_at,c.cumulative_tokens
        FROM conversations c
        INNER JOIN conversation_members cm ON c.id = cm.conversation_id
        WHERE cm.user_id = $1
        ORDER BY c.updated_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let convs = rows
                .into_iter()
                .map(|row| ConversationResponse {
                    id: row.id,
                    cumulative_tokens:row.cumulative_tokens,
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
    axum::extract::Extension(claims): axum::extract::Extension<crate::feature::conversation::auth::Claims>,
    Json(payload): Json<CreateConversationRequest>,
) -> ApiResponse<ConversationResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiResponse::failed("Invalid user ID in token"),
    };

    info!(user_id = %user_id, "Creating new conversation");

    let id = Uuid::new_v4();
    let title = payload
        .name
        .or(payload.title)
        .unwrap_or_else(|| "New Conversation".to_string());

    let result: Result<ConversationResponse, sqlx::Error> = async {
        let mut tx = state.pool.begin().await?;

        let row = sqlx::query!(
            "INSERT INTO conversations (id, title, soul_content, bootstrap_content,cumulative_tokens) VALUES ($1, $2, $3, $4,0) RETURNING id, title, created_at, updated_at,cumulative_tokens",
            id,
            title,
            payload.soul_content,
            payload.bootstrap_content
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2)",
            id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(ConversationResponse {
            id: row.id,
            cumulative_tokens: row.cumulative_tokens,
            name: row.title.unwrap_or_default(),
            created_at: row.created_at.unwrap_or_else(Utc::now),
            updated_at: row.updated_at.unwrap_or_else(Utc::now),
        })
    }
    .await;

    match result {
        Ok(response) => {
            info!(conversation_id = %id, user_id = %user_id, "Conversation created successfully");
            ApiResponse::ok(response, "Conversation created")
        }
        Err(e) => {
            error!("Failed to create conversation: {}", e);
            ApiResponse::failed("Failed to create conversation")
        }
    }
}

pub async fn handle_update_conversation(
    State(state): State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<crate::feature::conversation::auth::Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateConversationRequest>,
) -> ApiResponse<ConversationResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiResponse::failed("Invalid user ID in token"),
    };

    // Verify membership
    let membership = sqlx::query!(
        "SELECT 1 as one FROM conversation_members WHERE conversation_id = $1 AND user_id = $2",
        id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await;

    match membership {
        Ok(Some(_)) => (),
        Ok(None) => return ApiResponse::failed("Forbidden: You are not a member of this conversation"),
        Err(e) => {
            error!("Failed to verify membership: {}", e);
            return ApiResponse::failed("Internal server error");
        }
    }

    info!(conversation_id = %id, user_id = %user_id, "Updating conversation");

    let result = sqlx::query!(
        "UPDATE conversations SET title = $1, updated_at = NOW() WHERE id = $2 RETURNING id, title, created_at, updated_at,cumulative_tokens",
        payload.name,
        id
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => ApiResponse::ok(
            ConversationResponse {
                id: row.id,
                cumulative_tokens: row.cumulative_tokens,
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
    axum::extract::Extension(claims): axum::extract::Extension<crate::feature::conversation::auth::Claims>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Vec<SoulHistoryResponse>> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiResponse::failed("Invalid user ID in token"),
    };

    // Verify membership
    let membership = sqlx::query!(
        "SELECT 1 as one FROM conversation_members WHERE conversation_id = $1 AND user_id = $2",
        id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await;

    match membership {
        Ok(Some(_)) => (),
        Ok(None) => return ApiResponse::failed("Forbidden: You are not a member of this conversation"),
        Err(e) => {
            error!("Failed to verify membership: {}", e);
            return ApiResponse::failed("Internal server error");
        }
    }

    info!(conversation_id = %id, user_id = %user_id, "Fetching soul history");

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
    axum::extract::Extension(claims): axum::extract::Extension<crate::feature::conversation::auth::Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RestoreSoulRequest>,
) -> ApiResponse<RestoreSoulResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiResponse::failed("Invalid user ID in token"),
    };

    // Verify membership
    let membership = sqlx::query!(
        "SELECT 1 as one FROM conversation_members WHERE conversation_id = $1 AND user_id = $2",
        id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await;

    match membership {
        Ok(Some(_)) => (),
        Ok(None) => return ApiResponse::failed("Forbidden: You are not a member of this conversation"),
        Err(e) => {
            error!("Failed to verify membership: {}", e);
            return ApiResponse::failed("Internal server error");
        }
    }

    info!(
        conversation_id = %id,
        user_id = %user_id,
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
    axum::extract::Extension(claims): axum::extract::Extension<crate::feature::conversation::auth::Claims>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Value> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiResponse::failed("Invalid user ID in token"),
    };

    // Verify membership
    let membership = sqlx::query!(
        "SELECT 1 as one FROM conversation_members WHERE conversation_id = $1 AND user_id = $2",
        id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await;

    match membership {
        Ok(Some(_)) => (),
        Ok(None) => return ApiResponse::failed("Forbidden: You are not a member of this conversation"),
        Err(e) => {
            error!("Failed to verify membership: {}", e);
            return ApiResponse::failed("Internal server error");
        }
    }

    info!(conversation_id = %id, user_id = %user_id, "Deleting conversation");

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

pub async fn handle_upload_file(
    State(state): State<AppState>,
    mut multipart: axum::extract::Multipart,
) -> ApiResponse<String> {
    info!("Handling file upload...");
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        let file_name = field.file_name().unwrap_or_default().to_string();
        let _content_type = field.content_type().unwrap_or_default().to_string();

        if name == "file" {
            let data = match field.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    error!("Failed to read multipart bytes: {}", e);
                    return ApiResponse::failed(&format!("Failed to read file data: {}", e));
                }
            };

            let bucket = "conversations";
            let unique_name = format!("{}_{}", Uuid::new_v4(), file_name);

            match state
                .storage
                .upload_byte(bucket.to_string(), unique_name.clone(), data.to_vec())
                .await
            {
                Ok(_) => {
                    info!("File uploaded successfully: {}", unique_name);
                    return ApiResponse::ok(unique_name, "File uploaded successfully");
                }
                Err(e) => {
                    error!("Storage upload error: {}", e);
                    return ApiResponse::failed(&format!("Storage error: {}", e));
                }
            }
        }
    }

    ApiResponse::failed("No file field found in request or multipart parsing error occurred")
}
pub async fn handle_chat_stream(
    State(state): State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<
        crate::feature::conversation::auth::Claims,
    >,
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
            image_url: payload.image_url,
            audio_url: payload.audio_url,
            video_url: payload.video_url,
            sticker_url: None,
            doc_url: payload.doc_url,
            source: crate::feature::message_processor::MessageSource::Web,
            v2: true,
        };

        if let Err(e) =
            crate::feature::message_processor::process_incoming_message(state_clone, unified_msg)
                .await
        {
            error!("Failed to process web message: {}", e);
        }
    });

    ApiResponse::ok("Streaming started".to_string(), "Success")
}
