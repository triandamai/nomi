use crate::common::api_response::ApiResponse;
use crate::common::identity::auth_model::{AuthResponse, OtpRequest, OtpVerify, UserProfile};
use crate::feature::OutboundMessage;
use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Extension,
    Json,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::{rng, RngExt};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub async fn handle_request_otp(
    State(state): State<AppState>,
    Json(payload): Json<OtpRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::failed(&e.to_string())));
    }

    // 1. Generate 6-digit code
    let otp_code: u32 = rng().random_range(100000..999999);
    let otp_str = otp_code.to_string();

    // 2. Store in Redis with 5-minute expiry
    let redis_key = format!("otp:{}", payload.external_id);
    if let Err(e) = state.redis.set_ex(&redis_key, &otp_str, 300).await {
        error!("Failed to store OTP in Redis: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::failed("Failed to generate OTP")));
    }

    // 3. Send via channel
    if payload.channel == "email" {
        // Here you would integrate with SendGrid, SES, etc.
        info!("[Auth] Sending OTP {} to Email: {}", otp_str, payload.external_id);
        // For prototype, we'll just log it.
    } else {
        // For Telegram, WhatsApp, etc. we publish to nomi:outbound
        let outbound = OutboundMessage {
            sender_id: "nomi_auth".to_string(),
            chat_id: payload.external_id.clone(),
            text: format!("Your Open Agent verification code is: {}", otp_str),
            channel: payload.channel.clone(), metadata: None,
        };

        if let Err(e) = state.redis.publish_event("nomi:outbound", &outbound).await {
            error!("Failed to publish OTP to nomi:outbound: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::failed("Failed to deliver OTP")));
        }
    }

    info!("[Auth] OTP generated for User: {} via {}", payload.external_id, payload.channel);
    (StatusCode::OK, Json(ApiResponse::ok((), "OTP sent successfully")))
}

pub async fn handle_verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<OtpVerify>,
) -> impl IntoResponse {
    let redis_key = format!("otp:{}", payload.external_id);

    info!("Received OTP verification code: {}", redis_key);
    let stored_otp = match state.redis.get(&redis_key).await {
        Ok(Some(otp)) => otp,
        Ok(None) => return (StatusCode::UNAUTHORIZED, Json(ApiResponse::<AuthResponse>::failed("OTP expired or not found"))),
        Err(e) => {
            error!("Redis error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<AuthResponse>::failed("Server error")));
        }
    };

    if stored_otp != payload.code {
        return (StatusCode::UNAUTHORIZED, Json(ApiResponse::<AuthResponse>::failed("Invalid OTP")));
    }

    // OTP is valid, clean up
    let _ = state.redis.del(&redis_key).await;

    // 4. Resolve User and Generate JWT
    // Attempt to parse external_id as UUID first (for chat-triggered login)
    let user = if let Ok(u_id) = uuid::Uuid::parse_str(&payload.external_id) {
        sqlx::query("SELECT id, role FROM users WHERE id = $1")
            .bind(u_id)
            .fetch_optional(&state.pool)
            .await
    } else {
        sqlx::query("SELECT id, role FROM users WHERE external_id = $1")
            .bind(&payload.external_id)
            .fetch_optional(&state.pool)
            .await
    };

    let user_row = match user {
        Ok(Some(row)) => row,
        Ok(None) => {
            // If not found, and it wasn't a UUID, try upserting (legacy email flow)
            if uuid::Uuid::parse_str(&payload.external_id).is_err() {
                 match sqlx::query(
                    "INSERT INTO users (external_id, display_name) 
                     VALUES ($1, $1) 
                     ON CONFLICT (external_id) 
                     DO UPDATE SET external_id = EXCLUDED.external_id
                     RETURNING id, role"
                )
                .bind(&payload.external_id)
                .fetch_one(&state.pool)
                .await {
                    Ok(u) => u,
                    Err(e) => {
                        error!("Database error during user upsert: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<AuthResponse>::failed("Database error")));
                    }
                }
            } else {
                return (StatusCode::NOT_FOUND, Json(ApiResponse::<AuthResponse>::failed("User not found")));
            }
        },
        Err(e) => {
            error!("Database error during user lookup: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<AuthResponse>::failed("Database error")));
        }
    };

    use sqlx::Row;
    let user_id = user_row.get::<uuid::Uuid, _>("id");
    let user_role = user_row.get::<Option<String>, _>("role");

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        role: user_role.unwrap_or_else(|| "user".to_string()),
        exp: expiration,
    };

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref())) {
        Ok(t) => t,
        Err(e) => {
            error!("JWT encoding error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<AuthResponse>::failed("Token generation error")));
        }
    };

    let cookie = format!(
        "auth_token={}; Path=/; HttpOnly; SameSite=None; Secure; Max-Age={}",
        token,
        7 * 24 * 60 * 60 // 7 days
    );

    info!("[Auth] User verified: {}", user_id);
    (
        StatusCode::OK,
        Json(ApiResponse::ok(AuthResponse {
            access_token: token,
            user_id: user_id.to_string(),
        }, "Login successful"))
    )
}

pub async fn handle_get_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<UserProfile>::failed("Invalid user ID"))),
    };

    let user = sqlx::query_as!(
        UserProfile,
        r#"
        SELECT id::text as "id!", external_id as "external_id!", COALESCE(display_name, external_id) as "display_name!", NULL as "avatar_url"
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await;

    match user {
        Ok(Some(profile)) => (StatusCode::OK, Json(ApiResponse::ok(profile, "Profile retrieved"))),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<UserProfile>::failed("User not found"))),
        Err(e) => {
            error!("Database error during profile lookup: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<UserProfile>::failed("Database error")))
        }
    }
}

pub async fn handle_logout() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::ok((), "Logged out successfully"))
    )
}
