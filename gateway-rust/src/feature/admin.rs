use crate::AppState;
use axum::{
    extract::{Query, State, Request},
    http::StatusCode,
    response::IntoResponse,
    Json,
    middleware::Next,
    response::Response,
};
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;
use crate::common::api_response::ApiResponse;
use crate::feature::conversation::auth::Claims;

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

pub async fn admin_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = req.extensions().get::<Claims>().ok_or(StatusCode::UNAUTHORIZED)?;
    let admin_id = claims.sub.clone();
    let admin_id = Uuid::parse_str(&admin_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Check if user is admin
    let is_admin: Result<bool, sqlx::Error> = sqlx::query_scalar("SELECT role = 'admin' FROM users WHERE id = $1")
        .bind(admin_id)
        .fetch_one(&state.pool)
        .await;

    match is_admin {
        Ok(true) => {
            Ok(next.run(req).await)
        }
        Ok(false) => {
            Err(StatusCode::FORBIDDEN)
        }
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

    let result = state.storage.delete_file(file_path.to_string(), bucket_name.to_string()).await;
    match result {
        Ok(msg) => Json(ApiResponse::ok(msg, "Success")).into_response(),
        Err(e) => {
            error!("Error deleting file: {}", e);
            Json(ApiResponse::create(500, None::<()>, &e)).into_response()
        }
    }
}

use axum::extract::Multipart;

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
        return Json(ApiResponse::create(400, None::<()>, "Bucket name is required in prefix")).into_response();
    }

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("file").to_string();
        if name == "file" {
            let file_name = field.file_name().unwrap_or("unnamed").to_string();
            let data = match field.bytes().await {
                Ok(b) => b.to_vec(),
                Err(e) => return Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response(),
            };

            let full_path = if obj_prefix.is_empty() {
                file_name
            } else {
                format!("{}/{}", obj_prefix.trim_end_matches('/'), file_name)
            };

            match state.storage.upload_byte(bucket_name.to_string(), full_path, data).await {
                Ok(path) => return Json(ApiResponse::ok(path, "Success")).into_response(),
                Err(e) => return Json(ApiResponse::create(500, None::<()>, &e)).into_response(),
            }
        }
    }

    Json(ApiResponse::create(400, None::<()>, "No file found")).into_response()
}
