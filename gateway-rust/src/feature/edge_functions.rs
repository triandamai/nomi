use crate::AppState;
use crate::common::api_response::ApiResponse;
use axum::{
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info};

use crate::services::intent_classifier::IntentClassifierService;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct EdgeFunction {
    pub id: uuid::Uuid,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub schema_json: Value,
    pub rules_text: String,
    pub script_code: String,
    pub intents: Vec<String>,
    pub version: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
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
) -> ApiResponse<Vec<EdgeFunction>> {
    let functions = sqlx::query_as::<_, EdgeFunction>(
        "SELECT id, slug, name, description, schema_json, rules_text, script_code, intents, version, created_at FROM edge_functions ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await;

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
    axum::Json(payload): axum::Json<CreateEdgeFunctionRequest>,
) -> ApiResponse<EdgeFunction> {
    let mut tx = match state.pool.begin().await {
        Ok(t) => t,
        Err(e) => return ApiResponse::failed(&format!("Failed to start transaction: {}", e)),
    };

    let function_res = sqlx::query_as::<_, EdgeFunction>(
        "INSERT INTO edge_functions (slug, name, description, schema_json, rules_text, script_code, intents)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, slug, name, description, schema_json, rules_text, script_code, intents, version, created_at"
    )
    .bind(&payload.slug)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.schema_json)
    .bind(&payload.rules_text)
    .bind(&payload.script_code)
    .bind(&payload.intents)
    .fetch_one(&mut *tx)
    .await;

    match function_res {
        Ok(f) => {
            if let Err(e) = IntentClassifierService::sync_dynamic_plugin_intents_to_knowledge(
                &mut tx,
                &state.gemini_api_key,
                &payload,
                f.id,
            ).await {
                error!("Failed to sync plugin intents: {}", e);
                let _ = tx.rollback().await;
                return ApiResponse::failed("Failed to synchronize plugin capabilities");
            }

            if let Err(e) = tx.commit().await {
                error!("Failed to commit transaction: {}", e);
                return ApiResponse::failed("Database commit error");
            }

            ApiResponse::ok(f, "Edge function created")
        },
        Err(e) => {
            error!("Failed to create edge function: {}", e);
            let _ = tx.rollback().await;
            ApiResponse::failed("Database error")
        }
    }
}

pub async fn handle_update_edge_function(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    axum::Json(payload): axum::Json<CreateEdgeFunctionRequest>,
) -> ApiResponse<EdgeFunction> {
    let mut tx = match state.pool.begin().await {
        Ok(t) => t,
        Err(e) => return ApiResponse::failed(&format!("Failed to start transaction: {}", e)),
    };

    let function_res = sqlx::query_as::<_, EdgeFunction>(
        "UPDATE edge_functions
         SET name = $1, description = $2, schema_json = $3, rules_text = $4, script_code = $5, intents = $6, version = version + 1
         WHERE slug = $7
         RETURNING id, slug, name, description, schema_json, rules_text, script_code, intents, version, created_at"
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.schema_json)
    .bind(&payload.rules_text)
    .bind(&payload.script_code)
    .bind(&payload.intents)
    .bind(&slug)
    .fetch_one(&mut *tx)
    .await;

    match function_res {
        Ok(f) => {
            if let Err(e) = IntentClassifierService::sync_dynamic_plugin_intents_to_knowledge(
                &mut tx,
                &state.gemini_api_key,
                &payload,
                f.id,
            ).await {
                error!("Failed to sync plugin intents during update: {}", e);
                let _ = tx.rollback().await;
                return ApiResponse::failed("Failed to synchronize plugin capabilities");
            }

            if let Err(e) = tx.commit().await {
                error!("Failed to commit transaction: {}", e);
                return ApiResponse::failed("Database commit error");
            }

            ApiResponse::ok(f, "Edge function updated")
        },
        Err(e) => {
            error!("Failed to update edge function: {}", e);
            let _ = tx.rollback().await;
            ApiResponse::failed("Database error")
        }
    }
}

pub async fn handle_delete_edge_function(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResponse<()> {
    let mut tx = match state.pool.begin().await {
        Ok(t) => t,
        Err(e) => return ApiResponse::failed(&format!("Failed to start transaction: {}", e)),
    };

    #[derive(FromRow)]
    struct IdRow { id: uuid::Uuid }

    let row = sqlx::query_as::<_, IdRow>("SELECT id FROM edge_functions WHERE slug = $1")
        .bind(&slug)
        .fetch_optional(&mut *tx)
        .await;

    match row {
        Ok(Some(record)) => {
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
                },
                Err(e) => {
                    error!("Failed to delete edge function: {}", e);
                    let _ = tx.rollback().await;
                    ApiResponse::failed("Database error")
                }
            }
        },
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
    axum::Json(payload): axum::Json<ExecuteEdgeFunctionRequest>,
) -> ApiResponse<serde_json::Value> {
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
        "sender_id": "playground_user",
        "conversation_id": uuid::Uuid::nil(),
        "message_id": "playground_msg_1",
        "text": "Playground message",
        "channel": "web"
    });

    let workspace = serde_json::json!({
        "id": uuid::Uuid::nil(),
        "title": "Playground Workspace"
    });

    match executor.run(payload.args, incoming, workspace, bridge_token, api_base_url).await {
        Ok(exec_result) => {
            // Return both result and logs to the frontend
            ApiResponse::ok(serde_json::json!({
                "result": exec_result.result,
                "logs": exec_result.logs
            }), "Execution successful")
        },
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
    info!("Internal RPC: Retrieve Knowledge for query: {}", payload.query);
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
