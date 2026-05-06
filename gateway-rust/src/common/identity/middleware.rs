use crate::feature::conversation::auth::Claims;
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use tracing::error;

pub async fn auth_middleware(
    State(_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = if let Some(auth_header) = auth_header {
        if auth_header.starts_with("Bearer ") {
            auth_header[7..].to_string()
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    
    match decode::<Claims>(
        token.as_str(),
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims);
            Ok(next.run(req).await)
        }
        Err(e) => {
            error!("JWT validation error: {}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
