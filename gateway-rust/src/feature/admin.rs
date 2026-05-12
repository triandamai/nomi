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
