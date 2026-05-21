use crate::AppState;
use crate::common::api_response::ApiResponse;
use axum::{
    Json,
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
    pub rag_id: Option<uuid::Uuid>,
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
        "SELECT id, slug, name, description, schema_json, rules_text, script_code, intents, rag_id, version, created_at FROM edge_functions ORDER BY created_at DESC"
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
    Json(payload): Json<CreateEdgeFunctionRequest>,
) -> ApiResponse<EdgeFunction> {
    // Sync capabilities to knowledge base
    let save_result =
        IntentClassifierService::sync_dynamic_plugin_intents_to_knowledge(&state, payload.clone())
            .await;

    let rag_id = match save_result {
        Ok(ids) => ids.first().cloned(),
        Err(e) => {
            error!("Failed to save intent classifier: {}", e);
            return ApiResponse::failed("Failed to synchronize plugin capabilities");
        }
    };

    let function = sqlx::query_as::<_, EdgeFunction>(
            "INSERT INTO edge_functions (slug, name, description, schema_json, rules_text, script_code, intents, rag_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, slug, name, description, schema_json, rules_text, script_code, intents, rag_id, version, created_at"
    )
            .bind(&payload.slug)
            .bind(&payload.name)
            .bind(&payload.description)
            .bind(&payload.schema_json)
            .bind(&payload.rules_text)
            .bind(&payload.script_code)
            .bind(&payload.intents)
            .bind(rag_id)
            .fetch_one(&state.pool)
            .await;

    match function {
        Ok(f) => ApiResponse::ok(f, "Edge function created"),
        Err(e) => {
            error!("Failed to create edge function: {}", e);
            ApiResponse::failed("Database error")
        }
    }
}

pub async fn handle_update_edge_function(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<CreateEdgeFunctionRequest>,
) -> ApiResponse<EdgeFunction> {
    // Sync capabilities to knowledge base
    let save_result =
        IntentClassifierService::sync_dynamic_plugin_intents_to_knowledge(&state, payload.clone())
            .await;

    let rag_id = match save_result {
        Ok(ids) => ids.first().cloned(),
        Err(e) => {
            error!("Failed to sync plugin intents during update: {}", e);
            None
        }
    };

    let function = sqlx::query_as::<_, EdgeFunction>(
        "UPDATE edge_functions
         SET name = $1, description = $2, schema_json = $3, rules_text = $4, script_code = $5, intents = $6, rag_id = $7, version = version + 1
         WHERE slug = $8
         RETURNING id, slug, name, description, schema_json, rules_text, script_code, intents, rag_id, version, created_at"
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.schema_json)
    .bind(&payload.rules_text)
    .bind(&payload.script_code)
    .bind(&payload.intents)
    .bind(rag_id)
    .bind(&slug)
    .fetch_one(&state.pool)
    .await;

    match function {
        Ok(f) => ApiResponse::ok(f, "Edge function updated"),
        Err(e) => {
            error!("Failed to update edge function: {}", e);
            ApiResponse::failed("Database error")
        }
    }
}

pub async fn handle_delete_edge_function(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResponse<()> {
    #[derive(FromRow)]
    struct RagIdRow {
        rag_id: Option<uuid::Uuid>,
    }

    // Delete edge function and return rag_id
    let result = sqlx::query_as::<_, RagIdRow>(
        "DELETE FROM edge_functions WHERE slug = $1 RETURNING rag_id",
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(record)) => {
            // Delete associated knowledge base entry if it exists
            if let Some(rag_id) = record.rag_id {
                let _ = sqlx::query("DELETE FROM knowledge_base WHERE id = $1")
                    .bind(rag_id)
                    .execute(&state.pool)
                    .await;
            }
            ApiResponse::ok((), "Edge function deleted")
        }
        Ok(None) => ApiResponse::failed("Edge function not found"),
        Err(e) => {
            error!("Failed to delete edge function: {}", e);
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
    Json(payload): Json<ExecuteEdgeFunctionRequest>,
) -> ApiResponse<String> {
    let executor = crate::common::tools::edge_runner::BunEdgeExecutor {
        slug: "playground".to_string(),
        script_code: payload.script_code,
    };

    let bridge_token = "TEMP_PLAYGROUND_TOKEN_12345";
    let api_base_url = "http://localhost:8000";

    // Dummy context for playground
    let incoming = serde_json::json!({
        "is_group": false,
        "is_private": true,
        "is_mentioned": true,
        "sender_id": "playground_user",
        "conversation_id": "playground_convo",
        "message_id": "playground_msg",
        "text": "Hello Nomi!",
        "channel": "web"
    });

    let workspace = serde_json::json!({
        "id": uuid::Uuid::nil(),
        "title": "Playground Workspace"
    });

    match executor
        .run(
            payload.args,
            incoming,
            workspace,
            bridge_token,
            api_base_url,
        )
        .await
    {
        Ok(output) => ApiResponse::ok(output, "Execution successful"),
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
    Json(payload): Json<RetrieveKnowledgeRequest>,
) -> Json<serde_json::Value> {
    info!(
        "Internal RPC: Retrieve Knowledge for query: {}",
        payload.query
    );
    // TODO: Connect to actual RAG pipeline
    Json(serde_json::json!({
        "results": []
    }))
}

pub async fn handle_internal_incoming_history(
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    info!("Internal RPC: Incoming History");
    // TODO: Connect to Postgres history
    Json(serde_json::json!({
        "history": []
    }))
}
