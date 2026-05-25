use std::collections::HashMap;
use crate::common::api_response::ApiResponse;
use crate::common::identity::auth_model::{AuthResponse, UserProfile};
use crate::feature::conversation::model::{
    ChannelStatus, ChatRequest, ConversationResponse, CreateConversationRequest, MessageItem,
    MessageListParams, MessageListResponse, PairingRequest, PairingResponse, RestoreSoulRequest,
    RestoreSoulResponse, SoulHistoryResponse, UpdateConversationRequest,
};
use crate::feature::message_processor::v2_orchestrator::process_v2_message;
use crate::feature::{Conversation, MessageSource, UnifiedMessage};
use crate::services::event_dispatcher::AppEvent;
use crate::{AppState, common};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::Row;
use tracing::{error, info};
use uuid::Uuid;

pub mod auth;
pub mod command;
pub mod model;
pub mod srp_factory;

pub async fn handle_get_model_info(
    State(state): State<AppState>,
    axum::extract::Extension(_claims): axum::extract::Extension<auth::Claims>,
) -> ApiResponse<crate::common::agent::agent_model::ModelInfo> {
    ApiResponse::ok(state.model_info.clone(), "Model info retrieved")
}

#[derive(serde::Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub intents: Vec<String>,
}

#[derive(serde::Serialize)]
pub enum SkillType {
    System,
    Dynamic,
}

#[derive(serde::Serialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub intents: Vec<String>,
    pub skill_type: SkillType,
    pub script_code: Option<String>,
    pub schema_json: Option<serde_json::Value>,
    pub creator_name: Option<String>,
}

pub async fn handle_get_public_skills(
    State(state): State<AppState>,
) -> ApiResponse<Vec<SkillInfo>> {
    let dispatcher = crate::common::tools::ToolDispatcher::new(
        state.pool.clone(),
        std::path::PathBuf::from("."),
        None,
        None,
        state.gemini.clone(),
        state.gemini_api_key.clone(),
        state.storage.clone(),
        state.clone(),
    );

    let mut skills = Vec::new();

    // 1. Static System Plugins
    for (name, plugin) in &dispatcher.plugins {
        let schema = plugin.schema();
        let description = schema["description"].as_str().unwrap_or("").to_string();
        let intents = plugin
            .matching_intents()
            .iter()
            .map(|i| i.to_string())
            .collect();

        skills.push(SkillInfo {
            name: name.to_string(),
            description,
            intents,
            skill_type: SkillType::System,
            script_code: None,
            schema_json: Some(schema),
            creator_name: Some("System".to_string()),
        });
    }

    // 2. Dynamic Edge Plugins
    let dynamic_plugins = sqlx::query!(
        "SELECT ef.slug, ef.description, ef.intents, ef.script_code, ef.schema_json, u.display_name \
         FROM edge_functions ef \
         LEFT JOIN users u ON ef.user_id = u.id"
    )
    .fetch_all(&state.pool)
    .await;

    if let Ok(plugins) = dynamic_plugins {
        for p in plugins {
            skills.push(SkillInfo {
                name: p.slug,
                description: p.description,
                intents: p.intents,
                skill_type: SkillType::Dynamic,
                script_code: Some(p.script_code),
                schema_json: Some(p.schema_json),
                creator_name: p.display_name,
            });
        }
    }

    // Sort by name for consistent UI
    skills.sort_by(|a, b| a.name.cmp(&b.name));

    ApiResponse::ok(skills, "Public skills retrieved")
}

pub async fn handle_get_available_tools(
    State(state): State<AppState>,
    axum::extract::Extension(_claims): axum::extract::Extension<auth::Claims>,
) -> ApiResponse<Vec<ToolInfo>> {
    let dispatcher = crate::common::tools::ToolDispatcher::new(
        state.pool.clone(),
        std::path::PathBuf::from("."),
        None,
        None,
        state.gemini.clone(),
        state.gemini_api_key.clone(),
        state.storage.clone(),
        state.clone(),
    );

    let mut tools = Vec::new();
    for (name, plugin) in &dispatcher.plugins {
        let schema = plugin.schema();
        let description = schema["description"].as_str().unwrap_or("").to_string();
        let intents = plugin
            .matching_intents()
            .iter()
            .map(|i| i.to_string())
            .collect();

        tools.push(ToolInfo {
            name: name.to_string(),
            description,
            intents,
        });
    }

    // Sort by name for consistent UI
    tools.sort_by(|a, b| a.name.cmp(&b.name));

    ApiResponse::ok(tools, "Available tools retrieved")
}

#[derive(serde::Deserialize)]
pub struct CreatePatternRequest {
    pub content: String,
}

pub async fn handle_get_guardrail_patterns(
    State(state): State<AppState>,
    axum::extract::Extension(_claims): axum::extract::Extension<auth::Claims>,
) -> ApiResponse<Vec<crate::services::guardrail::PatternInfo>> {
    let service = crate::services::guardrail::GuardrailService::new(
        state.pool.clone(),
        state.gemini_api_key.clone(),
    );

    match service.get_all_patterns().await {
        Ok(patterns) => ApiResponse::ok(patterns, "Guardrail patterns retrieved"),
        Err(e) => {
            error!("Failed to fetch guardrail patterns: {}", e);
            ApiResponse::failed("Failed to fetch guardrail patterns")
        }
    }
}

#[derive(serde::Deserialize)]
pub struct WebSkillRequest {
    pub plugin_name: String,
    pub args: serde_json::Value,
    pub conversation_id: Option<Uuid>,
    pub script_code: Option<String>,
}

pub async fn handle_get_skill_schemas(
    State(state): State<AppState>,
    axum::extract::Extension(_claims): axum::extract::Extension<auth::Claims>,
) -> ApiResponse<Vec<Value>> {
    let dispatcher = crate::common::tools::ToolDispatcher::new(
        state.pool.clone(),
        std::path::PathBuf::from("."),
        None,
        None,
        state.gemini.clone(),
        state.gemini_api_key.clone(),
        state.storage.clone(),
        state.clone(),
    );

    let mut schemas = Vec::new();
    // 1. Static plugin schemas
    for plugin in dispatcher.plugins.values() {
        schemas.push(plugin.schema());
    }

    // 2. Dynamic edge function schemas
    let edge_fns = sqlx::query!("SELECT slug, name, description, schema_json FROM edge_functions")
        .fetch_all(&state.pool)
        .await;

    if let Ok(records) = edge_fns {
        for r in records {
            let mut schema = r.schema_json.clone();
            if let Some(obj) = schema.as_object_mut() {
                obj.insert("name".to_string(), serde_json::Value::String(r.slug));
                obj.insert(
                    "description".to_string(),
                    serde_json::Value::String(r.description),
                );
            }
            schemas.push(schema);
        }
    }

    // Sort by name for consistent UI
    schemas.sort_by(|a, b| {
        let name_a = a["name"].as_str().unwrap_or("");
        let name_b = b["name"].as_str().unwrap_or("");
        name_a.cmp(name_b)
    });

    ApiResponse::ok(schemas, "Skill schemas retrieved")
}

pub async fn handle_get_readme(
    State(_state): State<AppState>,
    axum::extract::Extension(_claims): axum::extract::Extension<auth::Claims>,
) -> ApiResponse<String> {
    // Try current directory first (Docker/Prod), then parent (Local Dev)
    let paths = ["./README.md", "../README.md"];

    for path in paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            return ApiResponse::ok(content, "README retrieved successfully");
        }
    }

    error!("Failed to read README.md from any expected location");
    ApiResponse::failed("Failed to read documentation")
}

pub async fn handle_get_skills_readme(
    State(_state): State<AppState>,
    axum::extract::Extension(_claims): axum::extract::Extension<auth::Claims>,
) -> ApiResponse<String> {
    // Try current directory first (Docker/Prod), then parent (Local Dev)
    let paths = ["./docs/SKILLS.md", "../docs/SKILLS.md"];

    for path in paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            return ApiResponse::ok(content, "Skills documentation retrieved successfully");
        }
    }

    error!("Failed to read docs/SKILLS.md from any expected location");
    ApiResponse::failed("Failed to read skills documentation")
}

pub async fn handle_execute_skill(
    State(state): State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<auth::Claims>,
    Json(payload): Json<WebSkillRequest>,
) -> ApiResponse<Value> {
    let user_id_str = claims.sub.clone();
    let user_id = match Uuid::parse_str(&user_id_str) {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    // 🚀 OPTION A: DIRECT SANDBOX EXECUTION (Dry Run)
    if let Some(code) = payload.script_code {
        let executor = crate::common::tools::edge_runner::BunEdgeExecutor {
            slug: payload.plugin_name.clone(),
            script_code: code,
        };

        let bridge_token = "TEMP_SKILL_TEST_TOKEN";
        let api_base_url = "http://localhost:8000";

        let incoming = serde_json::json!({
            "is_group": false,
            "is_mentioned": true,
            "sender_id": user_id_str,
            "conversation_id": payload.conversation_id.unwrap_or(Uuid::nil()),
            "text": "Skill test execution",
            "channel": "web"
        });

        let workspace = serde_json::json!({
            "id": payload.conversation_id.unwrap_or(Uuid::nil()),
            "title": "Skill Test Workspace"
        });

        let env:HashMap<String, String> = HashMap::new();
        return match executor
            .run(
                bridge_token,
                api_base_url,
                payload.args,
                incoming,
                workspace,
                env
            )
            .await
        {
            Ok(exec_result) => ApiResponse::ok(
                serde_json::json!({
                    "result": exec_result.result,
                    "logs": exec_result.logs
                }),
                "Execution successful",
            ),
            Err(e) => {
                error!("Edge execution failed: {}", e);
                ApiResponse::failed(&format!("{}", e))
            }
        };
    }

    // 🚀 OPTION B: STANDARD PRODUCTION EXECUTION
    let dispatcher = crate::common::tools::ToolDispatcher::new(
        state.pool.clone(),
        std::path::PathBuf::from("."),
        payload.conversation_id,
        user_id,
        state.gemini.clone(),
        state.gemini_api_key.clone(),
        state.storage.clone(),
        state.clone(),
    );

    let plugin = match dispatcher.plugins.get(payload.plugin_name.as_str()) {
        Some(p) => p,
        None => return ApiResponse::failed("Skill plugin not found"),
    };

    match plugin.execute(&dispatcher, payload.args).await {
        Ok(result) => {
            // Asynchronous token telemetry logging (Manual execution)
            let pool_clone = state.pool.clone();
            let conv_id = payload.conversation_id;
            let u_id = user_id;
            let log_type = "skill_test".to_string();

            tokio::spawn(async move {
                let _ = crate::services::ambient_soul::AmbientSoulService::log_token_transaction(
                    &pool_clone,
                    conv_id,
                    None,
                    u_id,
                    &log_type,
                    "system",
                    0,
                    0,
                    0,
                )
                .await;
            });

            ApiResponse::ok(json!(result), "Skill executed successfully")
        }
        Err(e) => {
            error!("Skill execution failed: {}", e);
            ApiResponse::failed(&format!("Skill execution error: {}", e))
        }
    }
}

pub async fn handle_insert_guardrail_pattern(
    State(state): State<AppState>,
    axum::extract::Extension(_claims): axum::extract::Extension<auth::Claims>,
    Json(payload): Json<CreatePatternRequest>,
) -> ApiResponse<()> {
    let service = crate::services::guardrail::GuardrailService::new(
        state.pool.clone(),
        state.gemini_api_key.clone(),
    );

    match service.insert_pattern(&payload.content).await {
        Ok(_) => ApiResponse::ok((), "Security pattern inserted successfully"),
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("exists") {
                ApiResponse::failed(&err_msg)
            } else {
                error!("Failed to insert guardrail pattern: {}", e);
                ApiResponse::failed("Failed to insert security pattern")
            }
        }
    }
}

pub async fn handle_delete_guardrail_pattern(
    State(state): State<AppState>,
    axum::extract::Extension(_claims): axum::extract::Extension<auth::Claims>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    let service = crate::services::guardrail::GuardrailService::new(
        state.pool.clone(),
        state.gemini_api_key.clone(),
    );

    match service.delete_pattern(id).await {
        Ok(_) => ApiResponse::ok((), "Security pattern deleted successfully"),
        Err(e) => {
            error!("Failed to delete guardrail pattern: {}", e);
            ApiResponse::failed("Failed to delete security pattern")
        }
    }
}

pub async fn handle_get_user_channels(
    State(state): State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<auth::Claims>,
) -> ApiResponse<Vec<ChannelStatus>> {
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
            let platforms = vec![
                "telegram".to_string(),
                "whatsapp".to_string(),
                "mobile".to_string(),
            ];
            let mut channels = Vec::new();

            let linked_platforms: std::collections::HashSet<String> =
                rows.into_iter().map(|r| r.channel_type).collect();

            for p in platforms {
                channels.push(ChannelStatus {
                    paired: linked_platforms.contains(&p),
                    platform: p,
                });
            }

            ApiResponse::ok(channels, "User channels retrieved")
        }
        Err(e) => {
            error!("Failed to fetch user channels: {}", e);
            ApiResponse::failed("Failed to fetch user channels")
        }
    }
}

pub async fn handle_create_pairing(
    State(state): State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<auth::Claims>,
    Path(conversation_id): Path<Uuid>,
) -> ApiResponse<PairingResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiResponse::failed("Invalid user ID in token"),
    };

    match common::repository::pairing_repo::create_pairing_code(&state, conversation_id, user_id)
        .await
    {
        Ok(pairing) => ApiResponse::ok(pairing, "Created pairing code"),
        Err(err) => ApiResponse::failed(err.to_string().as_str()),
    }
}

pub async fn handle_pairing_handshake(
    State(state): State<AppState>,
    Json(payload): Json<PairingRequest>,
) -> impl IntoResponse {
    let redis_key = format!("pairing:{}", payload.pairing_code);

    let data_str = match state.redis.get(&redis_key).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<AuthResponse>::failed(
                    "Pairing code expired or invalid",
                )),
            )
                .into_response();
        }
        Err(e) => {
            error!("Redis error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AuthResponse>::failed("Server error")),
            )
                .into_response();
        }
    };

    let (user_id, _conv_id) = match serde_json::from_str::<Value>(&data_str) {
        Ok(v) => {
            let uid = v["user_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
            let cid = v["conversation_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok());
            match (uid, cid) {
                (Some(u), Some(c)) => (u, c),
                _ => {
                    // Fallback for old simple string format if any
                    if let Ok(u) = Uuid::parse_str(&data_str) {
                        (u, Uuid::nil())
                    } else {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<AuthResponse>::failed("Invalid user data")),
                        )
                            .into_response();
                    }
                }
            }
        }
        Err(_) => {
            // Fallback for old simple string format
            if let Ok(u) = Uuid::parse_str(&data_str) {
                (u, Uuid::nil())
            } else {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<AuthResponse>::failed("Invalid user data")),
                )
                    .into_response();
            }
        }
    };

    // Single-use token: delete immediately
    let _ = state.redis.del(&redis_key).await;

    // Issue JWT
    let user_row = match sqlx::query!(
        "SELECT id, role, display_name FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AuthResponse>::failed("User not found")),
            )
                .into_response();
        }
    };

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(30)) // Mobile usually longer session
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = auth::Claims {
        sub: user_id.to_string(),
        role: user_row.role.clone().unwrap_or_else(|| "user".to_string()),
        exp: expiration,
    };

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let token = match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(e) => {
            error!("JWT encoding error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AuthResponse>::failed(
                    "Token generation error",
                )),
            )
                .into_response();
        }
    };

    // Fetch Profile
    let profile = UserProfile {
        id: user_id.to_string(),
        display_name: user_row.display_name,
        avatar_url: None,
        role: user_row.role,
    };

    // Fetch Channels
    let channel_rows = sqlx::query!(
        "SELECT DISTINCT channel_type FROM channels WHERE user_id = $1",
        user_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let platforms = vec!["telegram".to_string(), "whatsapp".to_string()];
    let linked_platforms: std::collections::HashSet<String> =
        channel_rows.into_iter().map(|r| r.channel_type).collect();

    let channels = platforms
        .into_iter()
        .map(|p| ChannelStatus {
            paired: linked_platforms.contains(&p),
            platform: p,
        })
        .collect::<Vec<_>>();

    // Fetch Conversations
    let conv_rows = sqlx::query!(
        r#"
        SELECT c.id, c.title, c.created_at, c.updated_at,c.max_token_usage, c.cumulative_tokens
        FROM conversations c
        INNER JOIN conversation_members cm ON c.id = cm.conversation_id
        WHERE cm.user_id = $1
        ORDER BY c.updated_at DESC
        LIMIT 50
        "#,
        user_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let conversations = conv_rows
        .into_iter()
        .map(|row| ConversationResponse {
            id: row.id,
            cumulative_tokens: row.cumulative_tokens,
            max_token_usage: row.max_token_usage,
            name: row.title.unwrap_or_default(),
            created_at: row.created_at.unwrap_or_else(Utc::now),
            updated_at: row.updated_at.unwrap_or_else(Utc::now),
        })
        .collect::<Vec<_>>();

    (
        StatusCode::OK,
        Json(ApiResponse::ok(
            AuthResponse {
                access_token: token,
                user_id: user_id.to_string(),
                profile: Some(profile),
                channels: Some(channels),
                conversations: Some(conversations),
            },
            "Pairing successful",
        )),
    )
        .into_response()
}

pub async fn handle_get_messages(
    State(state): State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<auth::Claims>,
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
        Ok(None) => {
            return ApiResponse::failed("Forbidden: You are not a member of this conversation");
        }
        Err(e) => {
            error!("Failed to verify membership: {}", e);
            return ApiResponse::failed("Internal server error");
        }
    }

    let limit = params.limit.unwrap_or(20);
    let cursor = params.cursor.unwrap_or_else(Utc::now);

    // info!(
    //     conversation_id = %conversation_id,
    //     user_id = %user_id,
    //     cursor = %cursor,
    //     limit = limit,
    //     "Fetching messages"
    // );

    let rows = sqlx::query(
        r#"
        SELECT m.id,
               m.conversation_id as conversation_id,
               u.display_name,
               m.role,
               m.content,
               m.thought,
               m.user_id,
               m.created_at as created_at,
               m.total_tokens,
               m.answer_tokens,
               m.prompt_tokens,
               m.image_url,
               m.video_url,
               m.audio_url,
               m.document_url,
               m.sticker_url,
               m.metadata,
               m.reply_to_id,
               CASE WHEN m.reply_to_id IS NOT NULL THEN
                 jsonb_build_object(
                   'id', rm.id,
                   'role', rm.role,
                   'content', rm.content,
                   'display_name', ru.display_name
                 )
               ELSE NULL END as replied_message
        FROM messages as m
        LEFT JOIN users AS u ON u.id = m.user_id
        LEFT JOIN messages AS rm ON rm.id = m.reply_to_id
        LEFT JOIN users AS ru ON ru.id = rm.user_id
        WHERE m.conversation_id = $1 AND m.created_at < $2
        ORDER BY m.created_at DESC
        LIMIT $3
        "#,
    )
    .bind(conversation_id)
    .bind(cursor)
    .bind(limit)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let messages: Vec<MessageItem> = rows.into_iter().map(|r| {
                let replied: Option<serde_json::Value> = r.get("replied_message");
                let replied_message = replied.and_then(|v| serde_json::from_value(v).ok());
                
                MessageItem {
                    id: r.get("id"),
                    conversation_id: r.get("conversation_id"),
                    display_name: r.get("display_name"),
                    role: r.get("role"),
                    content: r.get("content"),
                    thought: r.get("thought"),
                    user_id: r.get("user_id"),
                    created_at: r.get("created_at"),
                    total_tokens: r.get("total_tokens"),
                    answer_tokens: r.get("answer_tokens"),
                    prompt_tokens: r.get("prompt_tokens"),
                    image_url: r.get("image_url"),
                    video_url: r.get("video_url"),
                    audio_url: r.get("audio_url"),
                    document_url: r.get("document_url"),
                    sticker_url: r.get("sticker_url"),
                    metadata: r.get("metadata"),
                    reply_to_id: r.get("reply_to_id"),
                    replied_message,
                }
            }).collect();

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
) -> impl IntoResponse {
    let bucket = "conversations";
    match state
        .storage
        .get_file(bucket.to_string(), filename.clone())
        .await
    {
        Ok(data) => {
            let mime = mime_guess::from_path(&filename)
                .first_or_octet_stream()
                .to_string();

            (
                [
                    (axum::http::header::CONTENT_TYPE, mime),
                    (
                        axum::http::header::CACHE_CONTROL,
                        "public, max-age=31536000".to_string(),
                    ),
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

pub async fn handle_get_path_file(
    State(state): State<AppState>,
    Path((path, filename)): Path<(String, String)>,
) -> impl axum::response::IntoResponse {
    let bucket = "conversations";
    match state
        .storage
        .get_file(bucket.to_string(), format!("{}/{}", path, filename))
        .await
    {
        Ok(data) => {
            let mime = mime_guess::from_path(&filename)
                .first_or_octet_stream()
                .to_string();

            info!("mime : {}", mime);
            (
                [
                    (axum::http::header::CONTENT_TYPE, mime),
                    (
                        axum::http::header::CACHE_CONTROL,
                        "public, max-age=31536000".to_string(),
                    ),
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
    axum::extract::Extension(claims): axum::extract::Extension<
        crate::feature::conversation::auth::Claims,
    >,
) -> ApiResponse<Vec<ConversationResponse>> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiResponse::failed("Invalid user ID in token"),
    };

    info!(user_id = %user_id, "Fetching user conversations");

    let result = sqlx::query!(
        r#"
        SELECT c.id, c.title, c.created_at,c.max_token_usage, c.updated_at,c.cumulative_tokens
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
                    cumulative_tokens: row.cumulative_tokens,
                    max_token_usage: row.max_token_usage,
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
    axum::extract::Extension(claims): axum::extract::Extension<auth::Claims>,
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

        let conv_type = payload.conversation_type.unwrap_or_else(|| "private".to_string());

        let row = sqlx::query!(
            "INSERT INTO conversations (id, title, soul_content, bootstrap_content, cumulative_tokens, conversation_type) VALUES ($1, $2, $3, $4, 0, $5) RETURNING id, title,max_token_usage, created_at, updated_at, cumulative_tokens",
            id,
            title,
            payload.soul_content,
            payload.bootstrap_content,
            conv_type
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
            max_token_usage:row.max_token_usage,
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
    axum::extract::Extension(claims): axum::extract::Extension<
        crate::feature::conversation::auth::Claims,
    >,
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
        Ok(None) => {
            return ApiResponse::failed("Forbidden: You are not a member of this conversation");
        }
        Err(e) => {
            error!("Failed to verify membership: {}", e);
            return ApiResponse::failed("Internal server error");
        }
    }

    info!(conversation_id = %id, user_id = %user_id, "Updating conversation");

    let result = sqlx::query!(
        "UPDATE conversations SET title = $1, updated_at = NOW() WHERE id = $2 RETURNING id, title, created_at, updated_at,max_token_usage,cumulative_tokens",
        payload.name,
        id
    )
    .fetch_one(&state.pool)
    .await;

    if result.is_ok() {
        crate::common::repository::conversation_repo::invalidate_conversation_cache(
            &state.redis,
            id
        ).await;
    }

    match result {
        Ok(row) => ApiResponse::ok(
            ConversationResponse {
                id: row.id,
                cumulative_tokens: row.cumulative_tokens,
                max_token_usage: row.max_token_usage,
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
    axum::extract::Extension(claims): axum::extract::Extension<
        crate::feature::conversation::auth::Claims,
    >,
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
        Ok(None) => {
            return ApiResponse::failed("Forbidden: You are not a member of this conversation");
        }
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
    axum::extract::Extension(claims): axum::extract::Extension<
        crate::feature::conversation::auth::Claims,
    >,
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
        Ok(None) => {
            return ApiResponse::failed("Forbidden: You are not a member of this conversation");
        }
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
    axum::extract::Extension(claims): axum::extract::Extension<
        crate::feature::conversation::auth::Claims,
    >,
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
        Ok(None) => {
            return ApiResponse::failed("Forbidden: You are not a member of this conversation");
        }
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
    axum::extract::Extension(claims): axum::extract::Extension<auth::Claims>,
    Json(payload): Json<ChatRequest>,
) -> ApiResponse<MessageItem> {
    info!(conversation_id = %payload.conversation_id,user_id= %claims.sub, "Received chat stream request");

    // Resolve user_id from JWT claims
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    let conv_info = crate::common::repository::conversation_repo::get_conversation_info(
        &state.pool,
        &state.redis,
        payload.conversation_id,
    )
    .await;

    if let Ok(row) = &conv_info {
        let cumulative_tokens = row.cumulative_tokens;
        let max_token_usage = row.max_token_usage;
        if cumulative_tokens as f64 >= max_token_usage as f64 {
            let error_msg = format!(
                "Token limit reached ({}/{:.0})",
                cumulative_tokens, max_token_usage
            );
            if let Some(uid) = user_id {
                let _ = state
                    .dispatch(AppEvent::user(
                        uid.to_string().as_str(),
                        "error",
                        json!({
                            "conversation_id": payload.conversation_id,
                            "message": error_msg,
                        }),
                    ))
                    .await;
            }
            return ApiResponse::custom(1000, &error_msg);
        }
    }

    let conv_info = conv_info.unwrap();
    let state_clone = state.clone();
    let conversation_id = payload.conversation_id;
    let user_message = payload.message.clone();

    let unified_msg = UnifiedMessage {
        is_group: conv_info.conversation_type != "private",
        is_mentioned: true,
        display_name: Some(conv_info.title.clone().unwrap_or_else(|| "User".to_string())),
        conversation_id,
        user_id,
        text_content: user_message,
        image_url: payload.image_url,
        audio_url: payload.audio_url,
        video_url: payload.video_url,
        sticker_url: None,
        doc_url: payload.doc_url,
        source: MessageSource::Web {
            name: "web".to_string(),
        },
        quoted_message: None,
        reply_to_id: payload.reply_to_id,
        v2: true,
    };

    let map_convo = Conversation::from(conv_info);
    let message = process_v2_message(state_clone, map_convo, unified_msg).await;
    if let Err(e) = message {
        error!("Failed to process web message: {}", e);
        return ApiResponse::failed(format!("Failed to process web message {}", e).as_str());
    }

    ApiResponse::ok(message.unwrap(), "Success")
}

#[derive(serde::Deserialize)]
pub struct SrpTestRequest {
    pub slug: String,
    pub text: String,
}

pub async fn handle_get_srp_state(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResponse<Value> {
    let res = sqlx::query(
        "SELECT enriched_description, additional_rules, learned_phrases FROM static_plugin_reinforcements WHERE plugin_slug = $1"
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await;

    match res {
        Ok(Some(row)) => {
            use sqlx::Row;
            let data = json!({
                "slug": slug,
                "enriched_description": row.get::<String, _>("enriched_description"),
                "additional_rules": row.get::<Vec<String>, _>("additional_rules"),
                "learned_phrases": row.get::<Vec<String>, _>("learned_phrases"),
            });
            ApiResponse::ok(data, "SRP state retrieved")
        }
        Ok(None) => ApiResponse::not_found("No reinforcement found for this plugin"),
        Err(e) => ApiResponse::failed(&e.to_string()),
    }
}

pub async fn handle_test_srp(
    State(state): State<AppState>,
    Json(payload): Json<SrpTestRequest>,
) -> ApiResponse<Value> {
    // 🌟 Simulation Pass: Temporarily bypass real tool execution to check alignment
    let dispatcher = crate::common::tools::ToolDispatcher::new(
        state.pool.clone(),
        std::path::PathBuf::from("."),
        None,
        None,
        state.gemini.clone(),
        state.gemini_api_key.clone(),
        state.storage.clone(),
        state.clone(),
    );

    if let Some(plugin) = dispatcher.plugins.get(payload.slug.as_str()) {
        let schema = plugin.schema();
        let base_desc = schema["description"].as_str().unwrap_or_default();
        let outcome = format!(
            "Simulated alignment for tool [{}]: Phrasing '{}' was evaluated against base description '{}'. Reinforcement logic would suggest expanding vocabulary to include context-specific keywords.",
            payload.slug, payload.text, base_desc
        );
        ApiResponse::ok(json!({"outcome": outcome}), "Simulation successful")
    } else {
        ApiResponse::not_found("Plugin not found")
    }
}

pub async fn handle_get_available_plugins(
    State(state): State<AppState>,
) -> ApiResponse<Vec<String>> {
    let dispatcher = crate::common::tools::ToolDispatcher::new(
        state.pool.clone(),
        std::path::PathBuf::from("."),
        None,
        None,
        state.gemini.clone(),
        state.gemini_api_key.clone(),
        state.storage.clone(),
        state.clone(),
    );
    let mut slugs: Vec<String> = dispatcher.plugins.keys().map(|&k| k.to_string()).collect();

    // 🌟 DYNAMIC DISCOVERY: Fetch slugs from edge_functions to include in the registry
    if let Ok(dynamic_slugs) = sqlx::query_scalar!("SELECT slug FROM edge_functions")
        .fetch_all(&state.pool)
        .await
    {
        for slug in dynamic_slugs {
            if !slugs.contains(&slug) {
                slugs.push(slug);
            }
        }
    }

    slugs.sort();

    ApiResponse::ok(slugs, "Available plugins retrieved")
}
