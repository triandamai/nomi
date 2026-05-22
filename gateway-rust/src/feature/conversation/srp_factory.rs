use axum::{extract::{State, Path}, Json};
use serde_json::{json, Value};
use crate::AppState;
use crate::common::api_response::ApiResponse;

// 1. GET /api/srp/proposals
pub async fn get_proposals(State(state): State<AppState>) -> ApiResponse<Vec<Value>> {
    let res = sqlx::query(
        "SELECT slug, name, description, schema_json, how_it_works, compiled_code, status FROM plugin_creation_suggestions ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool).await;

    match res {
        Ok(rows) => {
            use sqlx::Row;
            let list: Vec<Value> = rows.into_iter().map(|r| {
                json!({
                    "slug": r.get::<String, _>("slug"),
                    "name": r.get::<String, _>("name"),
                    "description": r.get::<String, _>("description"),
                    "schema_json": r.get::<serde_json::Value, _>("schema_json"),
                    "how_it_works": r.get::<String, _>("how_it_works"),
                    "compiled_code": r.get::<String, _>("compiled_code"),
                    "status": r.get::<String, _>("status")
                })
            }).collect();
            ApiResponse::ok(list, "Proposals retrieved successfully")
        },
        Err(e) => ApiResponse::failed(&e.to_string())
    }
}

// 2. PUT /api/srp/proposals/:slug
pub async fn update_proposal(
    State(state): State<AppState>, 
    Path(slug): Path<String>, 
    Json(payload): Json<serde_json::Value>
) -> ApiResponse<String> {
    let desc = payload["description"].as_str().unwrap_or("");
    let schema = &payload["schema_json"];
    
    let res = sqlx::query(
        "UPDATE plugin_creation_suggestions SET description = $1, schema_json = $2, updated_at = NOW() WHERE slug = $3"
    )
    .bind(desc)
    .bind(schema)
    .bind(&slug)
    .execute(&state.pool).await;

    match res {
        Ok(_) => ApiResponse::ok("".to_string(), "Proposal updated successfully"),
        Err(e) => ApiResponse::failed(&e.to_string())
    }
}

// 3. DELETE /api/srp/proposals/:slug
pub async fn delete_proposal(State(state): State<AppState>, Path(slug): Path<String>) -> ApiResponse<String> {
    let res = sqlx::query("DELETE FROM plugin_creation_suggestions WHERE slug = $1")
        .bind(&slug)
        .execute(&state.pool).await;
    match res {
        Ok(_) => ApiResponse::ok("".to_string(), "Proposal deleted successfully"),
        Err(e) => ApiResponse::failed(&e.to_string())
    }
}

// 4. POST /api/srp/proposals/:slug/approve
pub async fn approve_proposal(State(state): State<AppState>, Path(slug): Path<String>) -> ApiResponse<Value> {
    let active_res = sqlx::query(
        "SELECT COUNT(*) as count FROM plugin_creation_suggestions WHERE status = 'processing'"
    )
    .fetch_one(&state.pool).await;

    let active_count: i64 = match active_res {
        Ok(row) => {
            use sqlx::Row;
            row.get::<i64, _>("count")
        },
        Err(_) => 0,
    };

    let target_status = if active_count >= 2 { "approved" } else { "processing" };

    let res = sqlx::query(
        "UPDATE plugin_creation_suggestions SET status = $1, updated_at = NOW() WHERE slug = $2 RETURNING status"
    )
    .bind(target_status)
    .bind(&slug)
    .fetch_one(&state.pool).await;

    match res {
        Ok(row) => {
            use sqlx::Row;
            let status: String = row.get("status");
            if status == "processing" {
                let pool = state.pool.clone();
                let state_clone = state.clone();
                tokio::spawn(async move {
                    let _ = crate::services::swe_agent::process_factory_build(pool, state_clone, slug).await;
                });
            }
            ApiResponse::ok(json!({"status": status}), "Proposal approved")
        },
        Err(e) => ApiResponse::failed(&e.to_string())
    }
}

// 5. POST /api/srp/proposals/:slug/deploy
pub async fn deploy_proposal(State(state): State<AppState>, Path(slug): Path<String>) -> ApiResponse<String> {
    let record = sqlx::query(
        "SELECT name, description, schema_json, compiled_code, status FROM plugin_creation_suggestions WHERE slug = $1 LIMIT 1"
    )
    .bind(&slug)
    .fetch_optional(&state.pool).await;

    match record {
        Ok(Some(row)) => {
            use sqlx::Row;
            let status: String = row.get("status");
            if status != "ready" {
                return ApiResponse::failed("Only plugins in 'ready' state can be deployed.");
            }

            let name: String = row.get("name");
            let description: String = row.get("description");
            let schema_json: serde_json::Value = row.get("schema_json");
            let compiled_code: String = row.get("compiled_code");

            let embedding = match crate::rag::get_embedding(&state.gemini_api_key, &description).await {
                Ok(emb) => emb.embedding.values,
                Err(e) => return ApiResponse::failed(&format!("Embedding generation failed: {}", e))
            };

            let mut tx = match state.pool.begin().await {
                Ok(t) => t,
                Err(e) => return ApiResponse::failed(&e.to_string()),
            };
            
            let res = sqlx::query(
                "INSERT INTO edge_functions (slug, name, description, schema_json, rules_text, script_code, embedding) \
                 VALUES ($1, $2, $3, $4, '', $5, $6) \
                 ON CONFLICT (slug) DO UPDATE SET script_code = EXCLUDED.script_code, description = EXCLUDED.description, embedding = EXCLUDED.embedding, version = edge_functions.version + 1"
            )
            .bind(&slug)
            .bind(&name)
            .bind(&description)
            .bind(&schema_json)
            .bind(&compiled_code)
            .bind(embedding)
            .execute(&mut *tx).await;

            if let Err(e) = res {
                 let _ = tx.rollback().await;
                 return ApiResponse::failed(&format!("Deployment failed: {}", e));
            }

            let _ = sqlx::query("UPDATE plugin_creation_suggestions SET status = 'deployed', updated_at = NOW() WHERE slug = $1")
                .bind(&slug)
                .execute(&mut *tx).await;

            let _ = tx.commit().await;
            ApiResponse::ok("".to_string(), "Plugin hot-patched into production successfully.")
        },
        Ok(None) => ApiResponse::not_found("Proposal not found"),
        Err(e) => ApiResponse::failed(&e.to_string()),
    }
}
