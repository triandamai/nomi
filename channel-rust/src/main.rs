use axum::{routing::{get, post}, Router, Json, extract::State};
use std::net::SocketAddr;
use tracing::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

mod feature;

#[derive(Clone)]
struct AppState {
    qr_code: Arc<Mutex<Option<String>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let state = AppState {
        qr_code: Arc::new(Mutex::new(None))
    };

    let app = Router::new()
        .route("/api/whatsapp/qr", get(get_whatsapp_qr))
        .route("/api/outbound", post(handle_outbound))
        .route("/api/presence/typing", post(handle_typing))
        .with_state(state);

    // Start Redis Listener
    tokio::spawn(async move {
        if let Err(e) = crate::feature::redis::start_redis_listener().await {
            tracing::error!("Redis listener failed: {}", e);
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8001));
    info!("channel-rust listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_whatsapp_qr(State(state): State<AppState>) -> Json<serde_json::Value> {
    let qr = state.qr_code.lock().await;
    Json(serde_json::json!({ "qr": *qr }))
}

#[derive(Debug, Deserialize, Serialize)]
struct OutboundMessage {
    sender_id: String,
    chat_id: String,
    text: String,
    channel: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TypingRequest {
    chat_id: String,
    channel: String,
    is_typing: bool,
}

async fn handle_typing(Json(payload): Json<TypingRequest>) -> Json<serde_json::Value> {
    info!("Presence (typing): {:?}", payload);
    // TODO: Implement WA/Telegram typing indicator logic
    Json(serde_json::json!({ "status": "ok" }))
}

async fn handle_outbound(Json(payload): Json<OutboundMessage>) -> Json<serde_json::Value> {
    info!("Outbound message: {:?}", payload);
    // TODO: Implement WA/Telegram sending logic
    Json(serde_json::json!({ "status": "sent" }))
}
