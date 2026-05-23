use crate::AppState;
use crate::common::api_response::ApiResponse;
use crate::feature::conversation::auth;
use crate::feature::edge_functions::{
    CreateEdgeFunctionRequest, handle_create_edge_function, handle_update_edge_function,
};
use axum::extract::Extension;
use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::{Value, json};

// 1. GET /api/srp/proposals
pub async fn get_proposals(State(state): State<AppState>) -> ApiResponse<Vec<Value>> {
    let res = sqlx::query(
        "SELECT slug, name, description, schema_json, how_it_works, compiled_code, status, intents, error_logs FROM plugin_creation_suggestions ORDER BY created_at DESC"
    )
        .fetch_all(&state.pool).await;

    match res {
        Ok(rows) => {
            use sqlx::Row;
            let list: Vec<Value> = rows
                .into_iter()
                .map(|r| {
                    json!({
                        "slug": r.get::<String, _>("slug"),
                        "name": r.get::<String, _>("name"),
                        "description": r.get::<String, _>("description"),
                        "schema_json": r.get::<serde_json::Value, _>("schema_json"),
                        "how_it_works": r.get::<String, _>("how_it_works"),
                        "compiled_code": r.get::<String, _>("compiled_code"),
                        "status": r.get::<String, _>("status"),
                        "intents": r.get::<Vec<String>, _>("intents"),
                        "error_logs": r.get::<Option<String>, _>("error_logs")
                    })
                })
                .collect();
            ApiResponse::ok(list, "Proposals retrieved successfully")
        }
        Err(e) => ApiResponse::failed(&e.to_string()),
    }
}

// 1.5 GET /api/srp/proposals/{slug}
pub async fn get_proposal(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResponse<Value> {
    let res = sqlx::query("SELECT id,slug, name, description, schema_json, how_it_works, compiled_code, status, intents, error_logs FROM plugin_creation_suggestions WHERE slug = $1")
        .bind(&slug)
        .fetch_optional(&state.pool).await;

    match res {
        Ok(Some(r)) => {
            use sqlx::Row;
            let val = json!({
                "id": r.get::<String, _>("id"),
                "slug": r.get::<String, _>("slug"),
                "name": r.get::<String, _>("name"),
                "description": r.get::<String, _>("description"),
                "schema_json": r.get::<serde_json::Value, _>("schema_json"),
                "how_it_works": r.get::<String, _>("how_it_works"),
                "compiled_code": r.get::<String, _>("compiled_code"),
                "status": r.get::<String, _>("status"),
                "intents": r.get::<Vec<String>, _>("intents"),
                "error_logs": r.get::<Option<String>, _>("error_logs")
            });
            ApiResponse::ok(val, "Proposal retrieved successfully")
        }
        Ok(None) => ApiResponse::not_found("Proposal not found"),
        Err(e) => ApiResponse::failed(&e.to_string()),
    }
}

pub async fn get_proposal_logs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResponse<Vec<Value>> {
    let res = sqlx::query!(
        "SELECT id,log_type,target_slug,event_step,message,created_at,metadata
         FROM system_logs
        WHERE metadata->>'ref_id' = $1",
        id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or(Vec::new())
    .iter()
    .map(|r| json!({
            "id":r.id,
            "log_type":r.log_type,
            "target_slug":r.target_slug,
            "event_step":r.event_step,
            "message":r.message,
            "created_at":r.created_at,
            "metadata":r.metadata
        })).collect();
    ApiResponse::ok(res, "Proposal logs retrieved successfully")
}

// 2. PUT /api/srp/proposals/:slug
pub async fn update_proposal(
    State(state): State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<auth::Claims>,
    Path(slug): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> ApiResponse<String> {
    if claims.role != "admin" {
        return ApiResponse::access_denied("Only administrators can update skill proposals.");
    }
    let desc = payload["description"].as_str().unwrap_or("");
    let schema = &payload["schema_json"];

    // Extract intents if provided
    let intents: Option<Vec<String>> = payload["intents"].as_array().map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    });

    let res = if let Some(intents_list) = intents {
        sqlx::query(
            "UPDATE plugin_creation_suggestions SET description = $1, schema_json = $2, intents = $3, updated_at = NOW() WHERE slug = $4"
        )
        .bind(desc)
        .bind(schema)
        .bind(&intents_list)
        .bind(&slug)
        .execute(&state.pool).await
    } else {
        sqlx::query(
            "UPDATE plugin_creation_suggestions SET description = $1, schema_json = $2, updated_at = NOW() WHERE slug = $3"
        )
        .bind(desc)
        .bind(schema)
        .bind(&slug)
        .execute(&state.pool).await
    };

    match res {
        Ok(_) => ApiResponse::ok("".to_string(), "Proposal updated successfully"),
        Err(e) => ApiResponse::failed(&e.to_string()),
    }
}

// 3. DELETE /api/srp/proposals/:slug
pub async fn delete_proposal(
    State(state): State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<auth::Claims>,
    Path(slug): Path<String>,
) -> ApiResponse<String> {
    if claims.role != "admin" {
        return ApiResponse::access_denied("Only administrators can delete skill proposals.");
    }
    let res = sqlx::query("DELETE FROM plugin_creation_suggestions WHERE slug = $1")
        .bind(&slug)
        .execute(&state.pool)
        .await;
    match res {
        Ok(_) => ApiResponse::ok("".to_string(), "Proposal deleted successfully"),
        Err(e) => ApiResponse::failed(&e.to_string()),
    }
}

// 4. POST /api/srp/proposals/:slug/approve
pub async fn approve_proposal(
    State(state): State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<auth::Claims>,
    Path(slug): Path<String>,
) -> ApiResponse<Value> {
    if claims.role != "admin" {
        return ApiResponse::access_denied("Only administrators can approve and initiate builds.");
    }
    let active_res = sqlx::query(
        "SELECT COUNT(*) as count FROM plugin_creation_suggestions WHERE status = 'processing'",
    )
    .fetch_one(&state.pool)
    .await;

    let active_count: i64 = match active_res {
        Ok(row) => {
            use sqlx::Row;
            row.get::<i64, _>("count")
        }
        Err(_) => 0,
    };

    let target_status = if active_count >= 2 {
        "approved"
    } else {
        "processing"
    };

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
                    let _ =
                        crate::services::swe_agent::process_factory_build(pool, state_clone, slug)
                            .await;
                });
            }
            ApiResponse::ok(json!({"status": status}), "Proposal approved")
        }
        Err(e) => ApiResponse::failed(&e.to_string()),
    }
}

// 5. POST /api/srp/proposals/:slug/deploy
pub async fn deploy_proposal(
    State(state): State<AppState>,
    Extension(claims): Extension<auth::Claims>,
    Path(slug): Path<String>,
) -> ApiResponse<String> {
    if claims.role != "admin" {
        return ApiResponse::access_denied("Only administrators can deploy skills to production.");
    }

    // 1. Fetch the candidate blueprint
    let record = sqlx::query(
        "SELECT name, description, schema_json, compiled_code, status, intents FROM plugin_creation_suggestions WHERE slug = $1 LIMIT 1"
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
            let mut intents: Vec<String> = row.get("intents");

            // 🌟 SRP EVOLUTION: Standardizing the intents with the ones from staging
            if intents.is_empty() {
                intents = vec!["GENERAL".to_string(), slug.clone()];
            } else if !intents.contains(&"GENERAL".to_string()) {
                intents.push("GENERAL".to_string());
            }

            // 2. Prepare the payload for handle_create_edge_function
            let payload = CreateEdgeFunctionRequest {
                slug: slug.clone(),
                name,
                description,
                schema_json,
                rules_text: "".to_string(),
                script_code: compiled_code,
                intents,
            };

            // 3. 🌟 DEPLOYMENT VIA UNIFIED CORE PATH 🌟
            // We check if it exists first to decide between CREATE or UPDATE
            let exists = sqlx::query("SELECT 1 FROM edge_functions WHERE slug = $1")
                .bind(&slug)
                .fetch_optional(&state.pool)
                .await
                .unwrap_or(None)
                .is_some();

            let deployment_res = if exists {
                handle_update_edge_function(
                    State(state.clone()),
                    Extension(claims),
                    Path(slug.clone()),
                    Json(payload),
                )
                .await
            } else {
                handle_create_edge_function(State(state.clone()), Extension(claims), Json(payload))
                    .await
            };

            if deployment_res.meta.code != 200 {
                return ApiResponse::create(
                    deployment_res.meta.code,
                    "".to_string(),
                    &deployment_res.meta.message,
                );
            }

            // 4. Mark staging record as deployed
            let _ = sqlx::query("UPDATE plugin_creation_suggestions SET status = 'deployed', updated_at = NOW() WHERE slug = $1")
                .bind(&slug)
                .execute(&state.pool).await;

            ApiResponse::ok(
                "".to_string(),
                "Plugin hot-patched into production successfully via standard deployment pipeline.",
            )
        }
        Ok(None) => ApiResponse::not_found("Proposal not found"),
        Err(e) => ApiResponse::failed(&e.to_string()),
    }
}
