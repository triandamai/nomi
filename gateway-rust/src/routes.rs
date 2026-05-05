use crate::AppState;
use crate::common::api_response::ApiResponse;
use crate::feature::conversation::{
    handle_chat_stream, handle_create_conversation, handle_delete_conversation,
    handle_get_conversations, handle_get_messages, handle_get_soul_history,
    handle_internal_inbound, handle_restore_conversation_soul, handle_update_conversation,
};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{delete, get, put};
use axum::{Json, Router, routing::post};
use crate::feature::graph::{handle_get_graph, handle_search_graph};
use crate::feature::realtime::register_public_sse;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/chat/stream", post(handle_chat_stream))
        .route("/conversations", get(handle_get_conversations))
        .route("/conversations", post(handle_create_conversation))
        .route("/conversations/{id}", put(handle_update_conversation))
        .route("/conversations/{id}", delete(handle_delete_conversation))
        .route("/conversations/{id}/messages", get(handle_get_messages))
        .route("/conversations/{id}/soul-history", get(handle_get_soul_history))
        .route(
            "/conversations/{id}/restore-soul",
            post(handle_restore_conversation_soul),
        )
        .route("/realtime", get(register_public_sse))
        .route("/graph", get(handle_get_graph))
        .route("/graph/search", get(handle_search_graph))
        .route("/internal/inbound", post(handle_internal_inbound))
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
