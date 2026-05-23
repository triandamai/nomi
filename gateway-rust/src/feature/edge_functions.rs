use crate::AppState;
use crate::common::api_response::ApiResponse;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, info};

use crate::services::intent_classifier::IntentClassifierService;
use sqlx::FromRow;

use crate::feature::conversation::auth::Claims;
use axum::extract::Extension;

#[derive(Serialize, Deserialize, FromRow)]
pub struct EdgeFunction {
    pub id: uuid::Uuid,
    pub user_id: Option<uuid::Uuid>,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub schema_json: Value,
    pub rules_text: String,
    pub script_code: String,
    pub intents: Vec<String>,
    pub version: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub display_name: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CreateEdgeFunctionRequest {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub schema_json: Value,
    pub rules_text: String,
    pub script_code: String,
    pub intents: Vec<String>,
}

pub async fn handle_get_edge_functions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResponse<Vec<EdgeFunction>> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).unwrap_or_default();

    // Admins see all, users see only theirs
    let query = if claims.role == "admin" {
        "SELECT ef.id, ef.user_id, ef.slug, ef.name, ef.description, ef.schema_json, ef.rules_text, ef.script_code, ef.intents, ef.version, ef.created_at, u.display_name \
         FROM edge_functions ef \
         LEFT JOIN users u ON ef.user_id = u.id \
         ORDER BY ef.created_at DESC"
    } else {
        "SELECT ef.id, ef.user_id, ef.slug, ef.name, ef.description, ef.schema_json, ef.rules_text, ef.script_code, ef.intents, ef.version, ef.created_at, u.display_name \
         FROM edge_functions ef \
         LEFT JOIN users u ON ef.user_id = u.id \
         WHERE ef.user_id = $1 \
         ORDER BY ef.created_at DESC"
    };

    let functions = if claims.role == "admin" {
        sqlx::query_as::<_, EdgeFunction>(query)
            .fetch_all(&state.pool)
            .await
    } else {
        sqlx::query_as::<_, EdgeFunction>(query)
            .bind(user_id)
            .fetch_all(&state.pool)
            .await
    };

    match functions {
        Ok(f) => ApiResponse::ok(f, "Edge functions retrieved"),
        Err(e) => {
            error!("Failed to fetch edge functions: {}", e);
            ApiResponse::failed("Database error")
        }
    }
}

pub async fn handle_create_edge_function(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    axum::Json(payload): axum::Json<CreateEdgeFunctionRequest>,
) -> ApiResponse<EdgeFunction> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).unwrap_or_default();

    // 1. Check Limits for non-admins
    if claims.role != "admin" {
        #[derive(FromRow)]
        struct CountRow {
            count: Option<i64>,
        }

        let count_res = sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) as count FROM edge_functions WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&state.pool)
        .await;

        if let Ok(res) = count_res {
            if res.count.unwrap_or(0) >= 10 {
                return ApiResponse::failed(
                    "Plugin limit reached. Non-admin users are restricted to 10 dynamic plugins.",
                );
            }
        }
    }

    let mut tx = match state.pool.begin().await {
        Ok(t) => t,
        Err(e) => return ApiResponse::failed(&format!("Failed to start transaction: {}", e)),
    };

    let function_res = sqlx::query!(
        "INSERT INTO edge_functions (slug, name, description, schema_json, rules_text, script_code, intents, user_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, user_id, slug, name, description, schema_json, rules_text, script_code, intents, version, created_at",
        payload.slug,
        payload.name,
        payload.description,
        payload.schema_json,
        payload.rules_text,
        payload.script_code,
        &payload.intents,
        user_id
    )
    .fetch_one(&mut *tx)
    .await;

    match function_res {
        Ok(r) => {
            let f = EdgeFunction {
                id: r.id,
                user_id: r.user_id,
                slug: r.slug,
                name: r.name,
                description: r.description,
                schema_json: r.schema_json,
                rules_text: r.rules_text,
                script_code: r.script_code,
                intents: r.intents,
                version: r.version,
                created_at: r.created_at,
                display_name: None, // Will be hydrated on next GET
            };
            if let Err(e) = IntentClassifierService::sync_dynamic_plugin_intents_to_knowledge(
                &mut tx,
                &state.gemini_api_key,
                &payload,
                f.id,
            )
            .await
            {
                error!("Failed to sync plugin intents: {}", e);
                let _ = tx.rollback().await;
                return ApiResponse::failed("Failed to synchronize plugin capabilities");
            }

            if let Err(e) = tx.commit().await {
                error!("Failed to commit transaction: {}", e);
                return ApiResponse::failed("Database commit error");
            }

            ApiResponse::ok(f, "Edge function created")
        }
        Err(e) => {
            error!("Failed to create edge function: {}", e);
            let _ = tx.rollback().await;
            ApiResponse::failed("Database error (maybe slug already exists?)")
        }
    }
}

pub async fn handle_update_edge_function(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(slug): Path<String>,
    axum::Json(payload): axum::Json<CreateEdgeFunctionRequest>,
) -> ApiResponse<EdgeFunction> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).unwrap_or_default();

    let mut tx = match state.pool.begin().await {
        Ok(t) => t,
        Err(e) => return ApiResponse::failed(&format!("Failed to start transaction: {}", e)),
    };

    // Check ownership unless admin
    #[derive(FromRow)]
    struct UserRow {
        user_id: Option<uuid::Uuid>,
    }

    let existing =
        sqlx::query_as::<_, UserRow>("SELECT user_id FROM edge_functions WHERE slug = $1")
            .bind(&slug)
            .fetch_optional(&mut *tx)
            .await;

    match existing {
        Ok(Some(record)) => {
            if claims.role != "admin" && record.user_id != Some(user_id) {
                return ApiResponse::failed("Unauthorized: You do not own this plugin.");
            }
        }
        Ok(None) => return ApiResponse::failed("Edge function not found"),
        Err(e) => return ApiResponse::failed(&format!("Database error: {}", e)),
    }

    let function_res = sqlx::query!(
        "UPDATE edge_functions
         SET name = $1, description = $2, schema_json = $3, rules_text = $4, script_code = $5, intents = $6, version = version + 1
         WHERE slug = $7
         RETURNING id, user_id, slug, name, description, schema_json, rules_text, script_code, intents, version, created_at",
        payload.name,
        payload.description,
        payload.schema_json,
        payload.rules_text,
        payload.script_code,
        &payload.intents,
        slug
    )
    .fetch_one(&mut *tx)
    .await;

    match function_res {
        Ok(r) => {
            let f = EdgeFunction {
                id: r.id,
                user_id: r.user_id,
                slug: r.slug,
                name: r.name,
                description: r.description,
                schema_json: r.schema_json,
                rules_text: r.rules_text,
                script_code: r.script_code,
                intents: r.intents,
                version: r.version,
                created_at: r.created_at,
                display_name: None,
            };
            if let Err(e) = IntentClassifierService::sync_dynamic_plugin_intents_to_knowledge(
                &mut tx,
                &state.gemini_api_key,
                &payload,
                f.id,
            )
            .await
            {
                error!("Failed to sync plugin intents during update: {}", e);
                let _ = tx.rollback().await;
                return ApiResponse::failed("Failed to synchronize plugin capabilities");
            }

            if let Err(e) = tx.commit().await {
                error!("Failed to commit transaction: {}", e);
                return ApiResponse::failed("Database commit error");
            }

            ApiResponse::ok(f, "Edge function updated")
        }
        Err(e) => {
            error!("Failed to update edge function: {}", e);
            let _ = tx.rollback().await;
            ApiResponse::failed("Database error")
        }
    }
}

pub async fn handle_delete_edge_function(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(slug): Path<String>,
) -> ApiResponse<()> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).unwrap_or_default();

    let mut tx = match state.pool.begin().await {
        Ok(t) => t,
        Err(e) => return ApiResponse::failed(&format!("Failed to start transaction: {}", e)),
    };

    // Check ownership unless admin
    #[derive(FromRow)]
    struct IdUserRow {
        id: uuid::Uuid,
        user_id: Option<uuid::Uuid>,
    }

    let existing =
        sqlx::query_as::<_, IdUserRow>("SELECT id, user_id FROM edge_functions WHERE slug = $1")
            .bind(&slug)
            .fetch_optional(&mut *tx)
            .await;

    match existing {
        Ok(Some(record)) => {
            if claims.role != "admin" && record.user_id != Some(user_id) {
                return ApiResponse::failed("Unauthorized: You do not own this plugin.");
            }

            let result = sqlx::query("DELETE FROM edge_functions WHERE id = $1")
                .bind(record.id)
                .execute(&mut *tx)
                .await;

            match result {
                Ok(_) => {
                    if let Err(e) = tx.commit().await {
                        error!("Failed to commit delete transaction: {}", e);
                        return ApiResponse::failed("Database commit error");
                    }
                    ApiResponse::ok((), "Edge function deleted")
                }
                Err(e) => {
                    error!("Failed to delete edge function: {}", e);
                    let _ = tx.rollback().await;
                    ApiResponse::failed("Database error")
                }
            }
        }
        Ok(None) => ApiResponse::failed("Edge function not found"),
        Err(e) => {
            error!("Database lookup error: {}", e);
            ApiResponse::failed("Database error")
        }
    }
}

// Playground Execution Engine
#[derive(Deserialize)]
pub struct ExecuteEdgeFunctionRequest {
    pub script_code: String,
    pub args: Value,
}

pub async fn handle_execute_edge_function(
    State(_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    axum::Json(payload): axum::Json<ExecuteEdgeFunctionRequest>,
) -> ApiResponse<serde_json::Value> {
    let user_id = claims.sub.clone();

    let executor = crate::common::tools::edge_runner::BunEdgeExecutor {
        slug: "playground".to_string(),
        script_code: payload.script_code,
    };

    let bridge_token = "TEMP_PLAYGROUND_TOKEN_12345";
    let api_base_url = "http://localhost:8000";

    let incoming = serde_json::json!({
        "is_group": false,
        "is_private": true,
        "is_mentioned": true,
        "sender_id": user_id,
        "conversation_id": uuid::Uuid::nil(),
        "message_id": "playground_msg_1",
        "text": "Playground message",
        "channel": "web"
    });

    let workspace = serde_json::json!({
        "id": uuid::Uuid::nil(),
        "title": "Playground Workspace"
    });

    let env: HashMap<String, String> = HashMap::new();
    match executor
        .run(
            api_base_url,
            bridge_token,
            payload.args,
            incoming,
            workspace,
            env,
        )
        .await
    {
        Ok(exec_result) => {
            // Return both result and logs to the frontend
            ApiResponse::ok(
                serde_json::json!({
                    "result": exec_result.result,
                    "logs": exec_result.logs
                }),
                "Execution successful",
            )
        }
        Err(e) => {
            error!("Edge function execution failed: {}", e);
            ApiResponse::failed(&format!("{}", e))
        }
    }
}

#[derive(Deserialize)]
pub struct RetrieveKnowledgeRequest {
    pub query: String,
    pub limit: Option<i32>,
}

pub async fn handle_internal_retrieve_knowledge(
    State(_state): State<AppState>,
    axum::Json(payload): axum::Json<RetrieveKnowledgeRequest>,
) -> axum::Json<serde_json::Value> {
    info!(
        "Internal RPC: Retrieve Knowledge for query: {}",
        payload.query
    );
    axum::Json(serde_json::json!({
        "results": []
    }))
}

pub async fn handle_internal_incoming_history(
    State(_state): State<AppState>,
) -> axum::Json<serde_json::Value> {
    info!("Internal RPC: Incoming History");
    axum::Json(serde_json::json!({
        "history": []
    }))
}
