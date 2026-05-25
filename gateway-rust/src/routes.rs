use crate::AppState;
use crate::common::api_response::ApiResponse;
use crate::common::identity::middleware::auth_middleware;
use crate::feature::conversation::{
    auth::{handle_get_profile, handle_update_profile, handle_logout, handle_request_otp, handle_verify_otp},
    handle_chat_stream, handle_create_conversation, handle_create_pairing, handle_pairing_handshake,
    handle_delete_conversation,
    handle_get_conversations, handle_get_file, handle_get_messages, handle_get_model_info,
    handle_get_path_file, handle_get_soul_history, handle_get_user_channels,
    handle_restore_conversation_soul, handle_update_conversation, handle_upload_file,
    handle_get_available_tools, handle_get_guardrail_patterns, handle_insert_guardrail_pattern,
    handle_delete_guardrail_pattern, handle_get_skill_schemas, handle_execute_skill,
    handle_get_readme, handle_get_skills_readme, handle_get_public_skills,
    handle_get_srp_state, handle_test_srp, handle_learn_srp, handle_get_available_plugins,
    srp_factory::{get_proposals, update_proposal, delete_proposal, approve_proposal, deploy_proposal},
    };

use crate::feature::graph::{handle_get_graph, handle_search_graph};
use crate::feature::reminder::{handle_get_reminders, handle_get_reminder_detail};
use crate::feature::waitlist::handle_waitlist;
use crate::feature::edge_functions::{
    handle_get_edge_functions, handle_create_edge_function, handle_delete_edge_function,
    handle_internal_retrieve_knowledge, handle_internal_incoming_history, handle_update_edge_function,
    handle_execute_edge_function
};
use axum::extract::DefaultBodyLimit;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum::routing::{delete, get, put};
use axum::{Json, Router, routing::post};

pub fn create_router(state: AppState) -> Router {
    let admin_routes = Router::new()
        .route(
            "/storage/explore",
            get(crate::feature::admin::handle_explore_storage),
        )
        .route(
            "/storage/delete",
            delete(crate::feature::admin::handle_delete_storage),
        )
        .route(
            "/storage/upload",
            post(crate::feature::admin::handle_upload_to_storage),
        )
        .route(
            "/conversations",
            get(crate::feature::admin::handle_get_admin_conversations),
        )
        .route(
            "/conversations/{id}",
            axum::routing::patch(crate::feature::admin::handle_update_admin_conversation),
        )
        .route("/users", get(crate::feature::admin::handle_get_users))
        .route(
            "/users/{id}",
            get(crate::feature::admin::handle_get_user_detail),
        )
        .route(
            "/users/{id}",
            axum::routing::patch(crate::feature::admin::handle_update_user),
        )
        .route(
            "/users/{id}",
            delete(crate::feature::admin::handle_delete_user),
        )
        .route(
            "/redis/publish/inbound",
            post(crate::feature::admin::handle_inbound_redis),
        )
        .route(
            "/redis/publish/outbound",
            post(crate::feature::admin::handle_outbound_redis),
        )
        .route("/guardrails/patterns", get(handle_get_guardrail_patterns))
        .route("/guardrails/patterns", post(handle_insert_guardrail_pattern))
        .route("/guardrails/patterns/{id}", delete(handle_delete_guardrail_pattern))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::feature::admin::admin_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let protected_routes = Router::new()
        .route(
            "/health/sync",
            post(crate::feature::health_tracking::handle_health_sync),
        )
        .route("/plugins", get(handle_get_edge_functions))
        .route("/plugins/execute", post(handle_execute_edge_function))
        .route("/plugins", post(handle_create_edge_function))
        .route("/plugins/{slug}", put(handle_update_edge_function))
        .route("/plugins/{slug}", delete(handle_delete_edge_function))
        .route("/auth/profile", get(handle_get_profile))
        .route("/auth/profile", put(handle_update_profile))

        .route("/model/info", get(handle_get_model_info))
        .route("/auth/logout", post(handle_logout))
        .route("/chat/stream", post(handle_chat_stream))
        .route("/conversations", get(handle_get_conversations))
        .route("/user/channels", get(handle_get_user_channels))
        .route("/tools", get(handle_get_available_tools))
        .route("/skills/schemas", get(handle_get_skill_schemas))
        .route("/skills/execute", post(handle_execute_skill))
        .route("/skills/readme", get(handle_get_skills_readme))
        .route("/reminders", get(handle_get_reminders))
        .route("/reminders/{id}", get(handle_get_reminder_detail))
        .route("/readme", get(handle_get_readme))
        .route("/srp/available", get(handle_get_available_plugins))
        .route("/srp/test", post(handle_test_srp))
        .route("/srp/learn", post(handle_learn_srp))
        .route("/srp/proposals", get(get_proposals))
        .route("/srp/proposals/{slug}", get(crate::feature::conversation::srp_factory::get_proposal))
        .route("/srp/proposals/{slug}/logs", get(crate::feature::conversation::srp_factory::get_proposal_logs))
        .route("/srp/proposals/{slug}", put(update_proposal))
        .route("/srp/proposals/{slug}", delete(delete_proposal))
        .route("/srp/proposals/{slug}/approve", post(approve_proposal))
        .route("/srp/proposals/{slug}/deploy", post(deploy_proposal))
        .route("/srp/{slug}", get(handle_get_srp_state))
        .route(
            "/money/history",
            get(crate::feature::money_tracking::handle_get_money_history),
        )
        .route(
            "/money/history/{id}",
            get(crate::feature::money_tracking::handle_get_money_detail),
        )
        .route(
            "/money/history/{id}",
            axum::routing::patch(crate::feature::money_tracking::handle_update_money_history),
        )
        .route(
            "/money/history/{id}",
            delete(crate::feature::money_tracking::handle_delete_money_history),
        )
        .route(
            "/health/history",
            get(crate::feature::health_tracking::handle_get_health_history),
        )
        .route("/conversations", post(handle_create_conversation))
        .route("/conversations/{id}", put(handle_update_conversation))
        .route("/conversations/{id}", delete(handle_delete_conversation))
        .route("/conversations/{id}/messages", get(handle_get_messages))
        .route(
            "/conversations/{id}/soul-history",
            get(handle_get_soul_history),
        )
        .route(
            "/conversations/{id}/restore-soul",
            post(handle_restore_conversation_soul),
        )
        .route("/conversations/{id}/pairing", post(handle_create_pairing))
        .route("/graph", get(handle_get_graph))
        .route("/graph/search", get(handle_search_graph))
        .route("/upload", post(handle_upload_file))
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let money_routes = Router::new()
        .route(
            "/history",
            get(crate::feature::money_tracking::handle_get_money_history),
        )
        .route(
            "/history/{id}",
            get(crate::feature::money_tracking::handle_get_money_detail),
        )
        .route(
            "/history/{id}",
            axum::routing::patch(crate::feature::money_tracking::handle_update_money_history),
        )
        .route(
            "/history/{id}",
            delete(crate::feature::money_tracking::handle_delete_money_history),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .route("/auth/request-otp", post(handle_request_otp))
        .route("/auth/verify-otp", post(handle_verify_otp))
        .route("/health/diagnostics", get(crate::feature::diagnostics::handle_diagnostics))
        .route("/auth/pair", post(handle_pairing_handshake))
        .route("/skills", get(handle_get_public_skills))
        .route("/waitlist", post(handle_waitlist))
        .route("/files/{filename}", get(handle_get_file))
        .route("/files/{path}/{filename}", get(handle_get_path_file))
        .route("/internal/rpc/retrieve-knowledge", post(handle_internal_retrieve_knowledge))
        .route("/internal/rpc/incoming-history", post(handle_internal_incoming_history))
        .nest("/v1/admin", admin_routes)
        .nest("/v1/money", money_routes)
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
