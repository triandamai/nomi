use crate::AppState;
use crate::common::api_response::ApiResponse;
use crate::feature::conversation::auth::Claims;
use crate::feature::conversation::model::MessageItem;
use crate::feature::{InboundMessage, MessageSource};
use axum::extract::Multipart;
use axum::{
    Json,
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
    response::Response,
};
use log::info;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::QueryBuilder;
use tracing::error;
use uuid::Uuid;
use crate::feature::message_processor::v2_orchestrator::send_message_to_subscriber;

#[derive(Deserialize)]
pub struct ExploreQuery {
    pub prefix: Option<String>,
}

#[derive(Deserialize)]
pub struct DeleteQuery {
    pub path: String,
}

#[derive(Deserialize)]
pub struct UploadQuery {
    pub prefix: Option<String>,
}

#[derive(Deserialize)]
pub struct MoneyHistoryQuery {
    pub page: Option<i64>,
    pub query: Option<String>,
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct AdminConversationsQuery {
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct UserListQuery {
    pub cursor: Option<String>,
    pub limit: Option<i64>,
    pub query: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserListItem {
    pub id: Uuid,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub is_verified: Option<bool>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct UserListResponse {
    pub items: Vec<UserListItem>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct AdminConversationItem {
    pub id: Uuid,
    pub title: Option<String>,
    pub cumulative_tokens: Option<i32>,
    pub max_token_usage: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct AdminConversationsResponse {
    pub items: Vec<AdminConversationItem>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct MoneyHistoryItem {
    pub id: Uuid,
    pub merchant_name: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub total_amount: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub items: sqlx::types::JsonValue,
    pub user_display_name: Option<String>,
    pub conversation_title: Option<String>,
}

#[derive(Serialize)]
pub struct MoneyHistoryResponse {
    pub items: Vec<MoneyHistoryItem>,
    pub total_count: i64,
}

#[derive(Deserialize)]
pub struct UpdateMoneyRequest {
    pub amount: Option<Decimal>,
    pub merchant_name: Option<String>,
    pub category: Option<String>,
}

#[derive(Serialize)]
pub struct UserDetailResponse {
    pub user: UserListItem,
    pub channels: Vec<UserChannelItem>,
    pub conversations: Vec<UserConversationMemberItem>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserChannelItem {
    pub id: Uuid,
    pub channel_type: String,
    pub external_id: String,
    pub external_chat_id: String,
    pub conversation_title: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserConversationMemberItem {
    pub conversation_id: Uuid,
    pub title: Option<String>,
    pub joined_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub is_verified: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateConversationRequest {
    pub max_token_usage: Option<i32>,
    pub title: Option<String>,
}

pub async fn handle_get_user_detail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user: UserListItem = match sqlx::query_as!(
        UserListItem,
        "SELECT id, name, display_name, email, role, is_verified, created_at FROM users WHERE id = $1",
        id
    )
        .fetch_one(&state.pool)
        .await
    {
        Ok(u) => u,
        Err(e) => {
            error!("Error fetching user: {}", e);
            return Json(ApiResponse::create(404, None::<()>, "User not found")).into_response();
        }
    };

    let channels: Vec<UserChannelItem> = sqlx::query_as!(
        UserChannelItem,
        r#"
        SELECT c.id as "id!", c.channel_type as "channel_type!", c.external_id as "external_id!", c.external_chat_id as "external_chat_id!", conv.title as conversation_title
        FROM channels c
        LEFT JOIN conversations conv ON c.conversation_id = conv.id
        WHERE conv.user_id = $1 OR c.conversation_id IN (SELECT conversation_id FROM conversation_members WHERE user_id = $1)
        "#,
        id
    )
        .fetch_all(&state.pool)
        .await.unwrap_or_else(|e| {
        error!("Error fetching channels: {}", e);
        Vec::new()
    });

    let conversations: Vec<UserConversationMemberItem> = sqlx::query_as!(
        UserConversationMemberItem,
        r#"
        SELECT cm.conversation_id as "conversation_id!", conv.title, cm.joined_at as "joined_at!"
        FROM conversation_members cm
        JOIN conversations conv ON cm.conversation_id = conv.id
        WHERE cm.user_id = $1
        "#,
        id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_else(|e| {
        error!("Error fetching conversations: {}", e);
        Vec::new()
    });

    Json(ApiResponse::ok(
        UserDetailResponse {
            user,
            channels,
            conversations,
        },
        "Success",
    ))
    .into_response()
}

pub async fn handle_update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        r#"
        UPDATE users
        SET 
            display_name = COALESCE($1, display_name),
            name = COALESCE($2, name),
            email = COALESCE($3, email),
            role = COALESCE($4, role),
            is_verified = COALESCE($5, is_verified)
        WHERE id = $6
        "#,
        req.display_name,
        req.name,
        req.email,
        req.role,
        req.is_verified,
        id
    )
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => Json(ApiResponse::ok((), "User updated successfully")).into_response(),
        Err(e) => {
            error!("Error updating user: {}", e);
            Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response()
        }
    }
}

pub async fn handle_delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<String> {
    // Note: This might need careful handling of foreign keys if not set to CASCADE
    let tx = state.pool.begin().await;
    if let Err(err) = tx {
        info!("Error delete user: {}", err);
        return ApiResponse::failed("Error deleting user, trx failed");
    }
    let mut tx = tx.unwrap();
    let delete_messages = sqlx::query!("DELETE FROM messages WHERE user_id = $1", id)
        .execute(&mut *tx)
        .await;
    if let Err(e) = delete_messages {
        let _ = tx.rollback().await;
        info!("Error deleting messages: {}", e);
        return ApiResponse::failed("Error deleting user, trx message failed");
    }
    let delete_channels = sqlx::query!("DELETE FROM channels WHERE user_id = $1", id)
        .execute(&mut *tx)
        .await;
    if let Err(e) = delete_channels {
        let _ = tx.rollback().await;
        info!("Error deleting channels: {}", e);
        return ApiResponse::failed("Error deleting user, trx channels failed");
    }

    let delete_members = sqlx::query!("DELETE FROM conversation_members WHERE user_id = $1", id)
        .execute(&mut *tx)
        .await;
    if let Err(e) = delete_members {
        let _ = tx.rollback().await;
        info!("Error deleting members: {}", e);
        return ApiResponse::failed("Error deleting user, trx members failed");
    }

    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&mut *tx)
        .await;

    if let Err(e) = result {
        let _ = tx.rollback().await;
        error!("Error deleting user: {}", e);
        return ApiResponse::failed("Error deleting user, trx failed");
    }

    let _ = tx.commit().await;

    ApiResponse::ok("Success".to_string(), "User deleted successfully")
}

pub async fn admin_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let admin_id = claims.sub.clone();
    let admin_id = Uuid::parse_str(&admin_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Check if user is admin
    let is_admin: Result<bool, sqlx::Error> =
        sqlx::query_scalar("SELECT role = 'admin' FROM users WHERE id = $1")
            .bind(admin_id)
            .fetch_one(&state.pool)
            .await;

    match is_admin {
        Ok(true) => Ok(next.run(req).await),
        Ok(false) => Err(StatusCode::FORBIDDEN),
        Err(e) => {
            error!("Database error checking admin role: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_explore_storage(
    State(state): State<AppState>,
    Query(query): Query<ExploreQuery>,
) -> impl IntoResponse {
    let result = state.storage.explore_storage(query.prefix).await;
    match result {
        Ok(items) => Json(ApiResponse::ok(items, "Success")).into_response(),
        Err(e) => {
            error!("Error exploring storage: {}", e);
            Json(ApiResponse::create(500, None::<()>, &e)).into_response()
        }
    }
}

pub async fn handle_delete_storage(
    State(state): State<AppState>,
    Query(query): Query<DeleteQuery>,
) -> impl IntoResponse {
    let path = query.path;
    let mut parts = path.splitn(2, '/');
    let bucket_name = parts.next().unwrap_or("");
    let file_path = parts.next().unwrap_or("");

    if bucket_name.is_empty() || file_path.is_empty() {
        return Json(ApiResponse::create(400, None::<()>, "Invalid path")).into_response();
    }

    let result = state
        .storage
        .delete_file(file_path.to_string(), bucket_name.to_string())
        .await;
    match result {
        Ok(msg) => Json(ApiResponse::ok(msg, "Success")).into_response(),
        Err(e) => {
            error!("Error deleting file: {}", e);
            Json(ApiResponse::create(500, None::<()>, &e)).into_response()
        }
    }
}

pub async fn handle_upload_to_storage(
    State(state): State<AppState>,
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let prefix = query.prefix.unwrap_or_default();
    let mut parts = prefix.splitn(2, '/');
    let bucket_name = parts.next().unwrap_or("");
    let obj_prefix = parts.next().unwrap_or("");

    if bucket_name.is_empty() {
        return Json(ApiResponse::create(
            400,
            None::<()>,
            "Bucket name is required in prefix",
        ))
        .into_response();
    }

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("file").to_string();
        if name == "file" {
            let file_name = field.file_name().unwrap_or("unnamed").to_string();
            let data = match field.bytes().await {
                Ok(b) => b.to_vec(),
                Err(e) => {
                    return Json(ApiResponse::create(500, None::<()>, &e.to_string()))
                        .into_response();
                }
            };

            let full_path = if obj_prefix.is_empty() {
                file_name
            } else {
                format!("{}/{}", obj_prefix.trim_end_matches('/'), file_name)
            };

            match state
                .storage
                .upload_byte(bucket_name.to_string(), full_path, data)
                .await
            {
                Ok(path) => return Json(ApiResponse::ok(path, "Success")).into_response(),
                Err(e) => return Json(ApiResponse::create(500, None::<()>, &e)).into_response(),
            }
        }
    }

    Json(ApiResponse::create(400, None::<()>, "No file found")).into_response()
}
pub async fn handle_get_admin_conversations(
    State(state): State<AppState>,
    Query(q): Query<AdminConversationsQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(20).max(1).min(100);

    let mut qb = QueryBuilder::new(
        r#"SELECT id, title, cumulative_tokens, max_token_usage, created_at FROM conversations WHERE 1=1 "#,
    );

    if let Some(cursor) = q.cursor {
        if let Ok(cursor_dt) = chrono::DateTime::parse_from_rfc3339(&cursor) {
            qb.push(" AND created_at < ");
            qb.push_bind(cursor_dt.with_timezone(&chrono::Utc));
        }
    }

    qb.push(" ORDER BY created_at DESC LIMIT ");
    qb.push_bind(limit);

    let items: Vec<AdminConversationItem> = match qb.build_query_as().fetch_all(&state.pool).await {
        Ok(items) => items,
        Err(e) => {
            error!("Database error fetching admin conversations: {}", e);
            return Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response();
        }
    };

    let next_cursor = items.last().map(|i| i.created_at.to_rfc3339());

    Json(ApiResponse::ok(
        AdminConversationsResponse { items, next_cursor },
        "Success",
    ))
    .into_response()
}

pub async fn handle_update_admin_conversation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateConversationRequest>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        r#"
        UPDATE conversations 
        SET 
            max_token_usage = COALESCE($1, max_token_usage),
            title = COALESCE($2, title)
        WHERE id = $3
        "#,
        req.max_token_usage,
        req.title,
        id
    )
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => Json(ApiResponse::ok((), "Conversation updated successfully")).into_response(),
        Err(e) => {
            error!("Error updating conversation: {}", e);
            Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response()
        }
    }
}

pub async fn handle_get_users(
    State(state): State<AppState>,
    Query(q): Query<UserListQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(20).max(1).min(100);

    let mut qb = QueryBuilder::new(
        "SELECT id, name, display_name, email, role, is_verified, created_at FROM users WHERE 1=1 ",
    );

    if let Some(ref search) = q.query {
        if !search.trim().is_empty() {
            let search_term = format!("%{}%", search);
            qb.push(" AND (display_name ILIKE ");
            qb.push_bind(search_term.clone());
            qb.push(" OR email ILIKE ");
            qb.push_bind(search_term.clone());
            qb.push(" OR name ILIKE ");
            qb.push_bind(search_term);
            qb.push(") ");
        }
    }

    if let Some(cursor) = q.cursor {
        if let Ok(cursor_dt) = chrono::DateTime::parse_from_rfc3339(&cursor) {
            qb.push(" AND created_at < ");
            qb.push_bind(cursor_dt.with_timezone(&chrono::Utc));
        }
    }

    qb.push(" ORDER BY created_at DESC LIMIT ");
    qb.push_bind(limit);

    let items: Vec<UserListItem> = match qb.build_query_as().fetch_all(&state.pool).await {
        Ok(items) => items,
        Err(e) => {
            error!("Database error fetching admin users: {}", e);
            return Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response();
        }
    };

    let next_cursor = items
        .last()
        .and_then(|i| i.created_at.map(|dt| dt.to_rfc3339()));

    Json(ApiResponse::ok(
        UserListResponse { items, next_cursor },
        "Success",
    ))
    .into_response()
}

#[derive(Deserialize)]
pub struct RedisInboundRequest {
    pub event: String,
    pub channel: String,
    pub payload: InboundMessage,
}

#[derive(Deserialize)]
pub struct RedisOutboundRequest {
    pub channel: String,
    pub event: String,
    pub conversation_id: Option<Uuid>,
    pub role: String,
    pub content: String,
}

pub async fn handle_outbound_redis(
    State(state): State<AppState>,
    Json(req): Json<RedisOutboundRequest>,
) -> ApiResponse<String> {
    if let None = req.conversation_id {
        info!("Published a redis conversation failed, convo id null");
        return ApiResponse::bad_request("conversation_id is not valid");
    }
    let members = sqlx::query!(
        "SELECT conversation_id FROM conversation_members WHERE conversation_id = $1",
        req.conversation_id
    )
    .fetch_all(&state.pool)
    .await;

    if let Err(err) = members {
        info!("Database error publishing conversation: {}", err);
        return ApiResponse::failed("Failed to fetch conversation");
    }
    let members = members
        .unwrap()
        .iter()
        .map(|m| m.conversation_id)
        .collect::<Vec<_>>();
    let conversation_id = req.conversation_id.unwrap();
    let outbound = MessageItem {
        id: Uuid::new_v4(),
        conversation_id,
        display_name:None,
        role: req.role.clone(),
        content: req.content.clone(),
        total_tokens: None,
        answer_tokens: None,
        prompt_tokens: None,
        thought: None,
        image_url: None,
        video_url: None,
        audio_url: None,
        document_url: None,
        sticker_url: None,
        user_id: None,
        created_at: Default::default(),
        metadata: None,
    };
    let _ = send_message_to_subscriber(
        &state,
        members,
        conversation_id,
        match req.channel.as_str() {
            "whatsapp" => MessageSource::WhatsApp {
                name: req.channel.to_string(),
            },
            "telegram" => MessageSource::Telegram {
                name: req.channel.to_string(),
            },
            "web" => MessageSource::Web {
                name: req.channel.to_string(),
            },
            _ => MessageSource::Other {
                name: "other".to_string(),
            },
        },
        json!({}),
        outbound,
    );

    ApiResponse::ok("".to_string(), "Published a redis conversation")
}

pub async fn handle_inbound_redis(
    State(state): State<AppState>,
    Json(req): Json<RedisInboundRequest>,
) -> impl IntoResponse {
    info!(
        "Published a redis event:{} \n payload {}\n",
        req.channel, req.payload
    );
    match state.redis.publish_event(&req.event, &req.payload).await {
        Ok(_) => Json(ApiResponse::ok((), "Event published successfully")).into_response(),
        Err(e) => {
            error!("Failed to publish to redis: {}", e);
            Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response()
        }
    }
}
