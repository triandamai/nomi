use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use crate::AppState;
use crate::common::api_response::ApiResponse;

#[derive(Deserialize)]
pub struct WaitlistRequest {
    pub email: String,
}

pub async fn handle_waitlist(
    State(state): State<AppState>,
    Json(payload): Json<WaitlistRequest>,
) -> ApiResponse<String> {
    let email = payload.email.trim().to_lowercase();
    
    if email.is_empty() {
        return ApiResponse::bad_request("Email is required");
    }

    // Basic email validation
    if !email.contains('@') {
        return ApiResponse::bad_request("Invalid email format");
    }

    match sqlx::query!(
        "INSERT INTO waitlist (email) VALUES ($1) ON CONFLICT (email) DO NOTHING",
        email
    )
    .execute(&state.pool)
    .await
    {
        Ok(_) => ApiResponse::ok("".to_string(),"Successfully joined the waitlist"),
        Err(e) => {
            tracing::error!("Failed to join waitlist: {:?}", e);
            ApiResponse::failed("Failed to join waitlist")
        }
    }
}
