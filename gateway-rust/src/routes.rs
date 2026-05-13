use crate::AppState;
use crate::common::api_response::ApiResponse;
use crate::feature::conversation::{auth::{handle_get_profile, handle_logout, handle_request_otp, handle_verify_otp}, handle_chat_stream, handle_create_conversation, handle_create_pairing, handle_delete_conversation, handle_get_conversations, handle_get_file, handle_get_messages, handle_get_path_file, handle_get_soul_history, handle_get_user_channels, handle_restore_conversation_soul, handle_update_conversation, handle_upload_file};
use crate::common::identity::middleware::auth_middleware;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum::routing::{delete, get, put};
use axum::{routing::post, Json, Router};
use crate::feature::graph::{handle_get_graph, handle_search_graph};
use crate::feature::realtime::register_public_sse;
use axum::extract::DefaultBodyLimit;
use crate::common::reminder::handle_get_reminders;

use crate::feature::waitlist::handle_waitlist;

pub fn create_router(state: AppState) -> Router {
    let admin_routes = Router::new()
        .route("/storage/explore", get(crate::feature::admin::handle_explore_storage))
        .route("/storage/delete", delete(crate::feature::admin::handle_delete_storage))
        .route("/storage/upload", post(crate::feature::admin::handle_upload_to_storage))
        .route("/money/history", get(crate::feature::admin::handle_get_money_history))
        .route("/money/history/{id}", axum::routing::patch(crate::feature::admin::handle_update_money_history))
        .route("/money/history/{id}", delete(crate::feature::admin::handle_delete_money_history))
        .layer(middleware::from_fn_with_state(state.clone(), crate::feature::admin::admin_middleware))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let protected_routes = Router::new()
        .route("/user/profile", get(handle_get_profile))
        .route("/auth/logout", post(handle_logout))
        .route("/chat/stream", post(handle_chat_stream))
        .route("/conversations", get(handle_get_conversations))
        .route("/user/channels", get(handle_get_user_channels))
        .route("/reminders", get(handle_get_reminders))
        .route("/conversations", post(handle_create_conversation))
        .route("/conversations/{id}", put(handle_update_conversation))
        .route("/conversations/{id}", delete(handle_delete_conversation))
        .route("/conversations/{id}/messages", get(handle_get_messages))
        .route("/conversations/{id}/soul-history", get(handle_get_soul_history))
        .route(
            "/conversations/{id}/restore-soul",
            post(handle_restore_conversation_soul),
        )
        .route("/conversations/{id}/pairing", post(handle_create_pairing))
        .route("/graph", get(handle_get_graph))
        .route("/graph/search", get(handle_search_graph))
        .route("/upload", post(handle_upload_file))
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .route("/auth/request-otp", post(handle_request_otp))
        .route("/auth/verify-otp", post(handle_verify_otp))
        .route("/waitlist", post(handle_waitlist))
        .route("/realtime", get(register_public_sse))
        .route("/files/{filename}", get(handle_get_file))
        .route("/files/{path}/{filename}", get(handle_get_path_file))
        .nest("/v1/admin", admin_routes)
        .merge(protected_routes)
        .layer(axum::middleware::from_fn(method_not_allowed))
        .fallback(handle_fallback)
        .with_state(state)
}

async fn handle_fallback() -> ApiResponse<String> {
    ApiResponse::not_found("Not found.")
}

//https://github.com/tokio-rs/axum/discussions/932
pub async fn method_not_allowed(req: Request, next: Next) -> impl IntoResponse {
    let resp = next.run(req).await;
    let status = resp.status();

    match status {
        StatusCode::METHOD_NOT_ALLOWED => Err((
            StatusCode::METHOD_NOT_ALLOWED,
            Json(ApiResponse::create(
                405,
                None::<String>,
                "Method Not Allowed",
            )),
        )
            .into_response()),
        _ => Ok(resp),
    }
}
