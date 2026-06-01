use axum::extract::State;
use axum::{Extension, Json};
use axum::response::IntoResponse;
use uuid::Uuid;
use serde::Deserialize;
use crate::common::api_response::ApiResponse;
use crate::common::app_state::AppState;
use crate::feature::conversation::auth::Claims;
use crate::common::repository::friendship_repo;

#[derive(Debug, Deserialize)]
pub struct FriendRequestPayload {
    pub receiver_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct RespondRequestPayload {
    pub sender_id: Uuid,
    pub accept: bool,
}

#[derive(Debug, Deserialize)]
pub struct BlockPayload {
    pub blocked_user_id: Uuid,
}

pub async fn handle_send_friend_request(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<FriendRequestPayload>,
) -> impl IntoResponse {
    let sender_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Json(ApiResponse::create(401, None::<()>, "Invalid user ID in token")).into_response(),
    };

    match friendship_repo::send_friend_request(&state.pool, sender_id, payload.receiver_id).await {
        Ok(_) => Json(ApiResponse::ok((), "Friend request sent successfully")).into_response(),
        Err(e) => Json(ApiResponse::create(400, None::<()>, &e.to_string())).into_response(),
    }
}

pub async fn handle_respond_friend_request(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<RespondRequestPayload>,
) -> impl IntoResponse {
    let receiver_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Json(ApiResponse::create(401, None::<()>, "Invalid user ID in token")).into_response(),
    };

    match friendship_repo::respond_friend_request(&state.pool, receiver_id, payload.sender_id, payload.accept).await {
        Ok(conv_id) => {
            let msg = if payload.accept {
                "Friend request accepted"
            } else {
                "Friend request declined"
            };
            Json(ApiResponse::ok(conv_id, msg)).into_response()
        }
        Err(e) => Json(ApiResponse::create(400, None::<()>, &e.to_string())).into_response(),
    }
}

pub async fn handle_get_friends(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Json(ApiResponse::create(401, None::<()>, "Invalid user ID in token")).into_response(),
    };

    match friendship_repo::get_friends(&state.pool, user_id).await {
        Ok(friends) => Json(ApiResponse::ok(friends, "Friends fetched successfully")).into_response(),
        Err(e) => Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response(),
    }
}

pub async fn handle_get_pending_requests(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Json(ApiResponse::create(401, None::<()>, "Invalid user ID in token")).into_response(),
    };

    match friendship_repo::get_pending_requests(&state.pool, user_id).await {
        Ok((incoming, outgoing)) => {
            let result = serde_json::json!({
                "incoming": incoming,
                "outgoing": outgoing
            });
            Json(ApiResponse::ok(result, "Pending requests fetched successfully")).into_response()
        }
        Err(e) => Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response(),
    }
}

pub async fn handle_block_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<BlockPayload>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Json(ApiResponse::create(401, None::<()>, "Invalid user ID in token")).into_response(),
    };

    match friendship_repo::block_user(&state.pool, user_id, payload.blocked_user_id).await {
        Ok(_) => Json(ApiResponse::ok((), "User blocked successfully")).into_response(),
        Err(e) => Json(ApiResponse::create(400, None::<()>, &e.to_string())).into_response(),
    }
}
