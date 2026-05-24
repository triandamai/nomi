use crate::AppState;
use crate::common::api_response::ApiResponse;
use crate::common::identity::auth_model::{AuthResponse, OtpRequest, OtpVerify, UserProfile};
use crate::feature::OutboundMessage;
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use jsonwebtoken::{EncodingKey, Header, encode};
use rand::{RngExt, rng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use validator::Validate;
use crate::services::event_dispatcher::AppEvent;

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
    info!("[Auth] Requesting OTP for identity: {} via channel: {}", payload.identity, payload.channel);
    
    if let Err(e) = payload.validate() {
        error!("[Auth] Validation failed: {}", e);
        return ApiResponse::<()>::failed(&e.to_string());
    }

    // 1. Generate 6-digit code
    let otp_code: u32 = rng().random_range(100000..999999);
    let otp_str = otp_code.to_string();
    info!("[Auth] Generated OTP (masked)");

    // 2. Lookup user and channel info
    let transform_ids: Vec<String> = vec![
        format!("{}@s.whatsapp.net", payload.identity),
        format!("{}@lid", payload.identity),
        payload.identity.clone(),
    ];
    info!("[Auth] Searching for identity match with variants: {:?}", transform_ids);

    let user_channel = sqlx::query(
        "SELECT u.id, u.display_name, ch.external_id as channel_external_id, ch.channel_type
         FROM users u
         LEFT JOIN channels ch ON u.id = ch.user_id
         WHERE (ch.channel_type = $1 AND (ch.external_id = ANY($2::text[]) OR ch.external_chat_id = ANY($2::text[])))
            OR u.email = $3
         ORDER BY ch.created_at DESC NULLS LAST LIMIT 1"
    )
    .bind(&payload.channel)
    .bind(&transform_ids[..])
    .bind(&payload.identity)
    .fetch_optional(&state.pool)
    .await;

    use sqlx::Row;
    let (user_id, target_id, display_name) = match user_channel {
        Ok(Some(row)) => {
            let uid = row.get::<uuid::Uuid, _>("id");
            let tid = row.try_get::<String, _>("channel_external_id").unwrap_or_else(|_| payload.identity.clone());
            let dname = row.get::<Option<String>, _>("display_name").unwrap_or_else(|| "User".to_string());
            info!("[Auth] Found existing user: {} (ID: {}) linked to target: {}", dname, uid, tid);
            (Some(uid), tid, dname)
        },
        Ok(None) => {
            info!("[Auth] No existing user found for identity: {}", payload.identity);
            (None, payload.identity.clone(), "User".to_string())
        },
        Err(e) => {
            error!("[Auth] Database error during lookup: {}", e);
            (None, payload.identity.clone(), "User".to_string())
        }
    };

    // 3. Store in Redis with 5-minute expiry
    let redis_key = format!("otp:{}", payload.identity);
    info!("[Auth] Storing OTP in Redis with key: {}", redis_key);
    if let Err(e) = state.redis.set_ex(&redis_key, &otp_str, 300).await {
        error!("[Auth] Failed to store OTP in Redis: {}", e);
        return ApiResponse::<()>::failed("Failed to generate OTP");
    }

    // 4. Dispatch OTP via AppEvent (Event-Driven)
    let message_text = format!("Halo {}, Kode verifikasi Nomi Anda adalah: **{}**. Berlaku selama 5 menit. 🛡️", display_name, otp_str);
    
    let outbound = OutboundMessage {
        is_group: false,
        sender_id: "nomi_auth".to_string(),
        conversation_id: target_id.clone(),
        text: message_text.clone(),
        channel: payload.channel.clone(),
        video_url: None,
        image_url: None,
        audio_url: None,
        doc_url: None,
        sticker_url: None,
        metadata: None,
    };

    info!("[Auth] Dispatching outbound message to: {} via {}", target_id, payload.channel);
    let event = if let Some(uid) = user_id {
        AppEvent::user(&uid.to_string(), "otp_request", json!({ "channel": payload.channel }))
            .with_redis_outbound(outbound)
    } else {
        AppEvent::broadcast("otp_request", json!({ "identity": payload.identity, "channel": payload.channel }))
            .with_redis_outbound(outbound)
    };

    if let Err(e) = crate::services::event_dispatcher::dispatch(&state, event).await {
        error!("[Auth] Failed to dispatch OTP event: {}", e);
    }

    info!("[Auth] OTP process complete for: {}", payload.identity);
    ApiResponse::ok((), "OTP sent successfully")
}

pub async fn handle_verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<OtpVerify>,
) -> impl IntoResponse {
    let redis_key = format!("otp:{}", payload.identity);
    info!("[Auth] Verifying OTP for identity: {} with code: {}", payload.identity, payload.code);

    let stored_otp = match state.redis.get(&redis_key).await {
        Ok(Some(otp)) => otp,
        Ok(None) => {
            error!("[Auth] OTP expired or not found for key: {}", redis_key);
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<AuthResponse>::failed(
                    "OTP expired or not found",
                )),
            );
        }
        Err(e) => {
            error!("[Auth] Redis error during verification: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AuthResponse>::failed("Server error")),
            );
        }
    };

    if stored_otp != payload.code {
        error!("[Auth] Invalid OTP provided for identity: {}", payload.identity);
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<AuthResponse>::failed("Invalid OTP")),
        );
    }

    info!("[Auth] OTP verified successfully. Cleaning up Redis...");
    let _ = state.redis.del(&redis_key).await;

    // 4. Resolve User and Generate JWT
    let transform_ids: Vec<String> = vec![
        format!("{}@s.whatsapp.net", payload.identity),
        format!("{}@lid", payload.identity),
        payload.identity.clone(),
    ];
    info!("[Auth] Resolving user for identity Variants: {:?}", transform_ids);

    let user_row = sqlx::query(
        "SELECT u.id, u.role
         FROM users u
         LEFT JOIN channels ch ON u.id = ch.user_id
         WHERE ch.external_id = ANY($1::text[]) 
            OR ch.external_chat_id = ANY($1::text[]) 
            OR u.email = $2
         ORDER BY u.created_at DESC LIMIT 1"
    )
    .bind(&transform_ids[..])
    .bind(&payload.identity)
    .fetch_optional(&state.pool)
    .await;

    use sqlx::Row;
    let (user_id, user_role) = match user_row {
        Ok(Some(row)) => {
            let uid = row.get::<uuid::Uuid, _>("id");
            let role = row.get::<Option<String>, _>("role");
            info!("[Auth] Resolved existing user: {} with role: {:?}", uid, role);
            (uid, role)
        },
        Ok(None) => {
            info!("[Auth] No user found. Attempting automatic account creation for identity: {}", payload.identity);
            let (email, display_name) = if payload.identity.contains('@') {
                (Some(payload.identity.clone()), "User".to_string())
            } else {
                (None, payload.identity.clone())
            };

            match sqlx::query(
                "INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id, role"
            )
            .bind(email)
            .bind(display_name)
            .fetch_one(&state.pool)
            .await
            {
                Ok(row) => {
                    let uid = row.get::<uuid::Uuid, _>("id");
                    let role = row.get::<Option<String>, _>("role");
                    info!("[Auth] Successfully created new user: {} with role: {:?}", uid, role);
                    (uid, role)
                },
                Err(e) => {
                    error!("[Auth] Database error during user automatic creation: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<AuthResponse>::failed("Database error")),
                    );
                }
            }
        }
        Err(e) => {
            error!("[Auth] Database error during final user resolution: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AuthResponse>::failed("Database error")),
            );
        }
    };

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        role: user_role.unwrap_or_else(|| "user".to_string()),
        exp: expiration,
    };

    info!("[Auth] Generating JWT for user: {}", user_id);
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(e) => {
            error!("[Auth] JWT encoding error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AuthResponse>::failed(
                    "Token generation error",
                )),
            );
        }
    };

    info!("[Auth] Authentication successful for user: {}. Token generated.", user_id);
    (
        StatusCode::OK,
        Json(ApiResponse::ok(
            AuthResponse {
                access_token: token,
                user_id: user_id.to_string(),
                profile: None,
                channels: None,
                conversations: None,
            },
            "Login successful",
        )),
    )
}

pub async fn handle_get_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<UserProfile>::failed("Invalid user ID")),
            );
        }
    };

    let user = sqlx::query_as!(
        UserProfile,
        r#"
        SELECT id::text as "id!", COALESCE(display_name, name, 'User') as "display_name!", NULL as "avatar_url", role as "role"
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await;

    match user {
        Ok(Some(profile)) => (
            StatusCode::OK,
            Json(ApiResponse::ok(profile, "Profile retrieved")),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<UserProfile>::failed("User not found")),
        ),
        Err(e) => {
            error!("Database error during profile lookup: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UserProfile>::failed("Database error")),
            )
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: String,
}

pub async fn handle_update_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpdateProfileRequest>,
) -> impl IntoResponse {
    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::failed("Invalid user ID")),
            );
        }
    };

    if payload.display_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::failed("Display name cannot be empty")),
        );
    }

    let result = sqlx::query!(
        "UPDATE users SET display_name = $1 WHERE id = $2",
        payload.display_name,
        user_id
    )
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::ok((), "Profile updated successfully")),
        ),
        Err(e) => {
            error!("Database error during profile update: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::failed("Database error")),
            )
        }
    }
}

pub async fn handle_logout() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::ok((), "Logged out successfully")),
    )
}
