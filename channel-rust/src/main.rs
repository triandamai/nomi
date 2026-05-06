use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
use dotenvy::dotenv;
use teloxide::prelude::*;
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod feature;
mod common;

#[derive(Clone)]
struct AppState {
    qr_code: Arc<Mutex<Option<String>>>,
    bot: Bot,
    redis: crate::common::redis::RedisClient,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    info!("Starting channel-rust initialization...");

    info!("Initializing Telegram Bot...");
    let tel_token = std::env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    let bot = Bot::new(tel_token);
    info!("Telegram Bot initialized.");

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis = crate::common::redis::RedisClient::new(&redis_url)?;

    let state = AppState {
        qr_code: Arc::new(Mutex::new(None)),
        bot: bot.clone(),
        redis: redis.clone(),
    };
    info!("AppState created.");

    info!("Configuring Router...");
    let app = Router::new()
        .route("/api/whatsapp/qr", get(get_whatsapp_qr))
        .route("/api/outbound", post(handle_outbound))
        .route("/api/presence/typing", post(handle_typing))
        .with_state(state.clone());
    info!("Router configured.");

    // Start Telegram Worker
    let bot_worker = bot.clone();
    let redis_worker = redis.clone();
    info!("Spawning Telegram Worker...");
    tokio::spawn(async move {
        info!("Telegram Worker task started.");
        crate::feature::telegram::start_telegram_worker(bot_worker, redis_worker).await;
        info!("Telegram Worker task finished (unexpectedly).");
    });

    // Start Redis Listener
    let redis_listener_state = state.clone();
    info!("Spawning Redis Listener...");
    tokio::spawn(async move {
        info!("Redis Listener task started.");
        if let Err(e) = crate::feature::redis::start_redis_listener(redis_listener_state).await {
            error!("Redis listener failed: {}", e);
        }
        info!("Redis Listener task finished.");
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8001));
    info!("Attempting to bind TcpListener to {}...", addr);
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => {
            info!("TcpListener bound successfully.");
            l
        },
        Err(e) => {
            error!("Failed to bind TcpListener: {}", e);
            return Err(e.into());
        }
    };

    info!("Starting Axum server on {}...", addr);
    axum::serve(listener, app).await?;

    info!("channel-rust shutdown.");
    Ok(())
}

async fn get_whatsapp_qr(State(state): State<AppState>) -> Json<serde_json::Value> {
    let qr = state.qr_code.lock().await;
    Json(serde_json::json!({ "qr": *qr }))
}

async fn handle_typing(
    State(state): State<AppState>,
    Json(payload): Json<crate::feature::TypingRequest>,
) -> Json<serde_json::Value> {
    info!("Presence (typing): {:?}", payload);

    if payload.channel == "telegram" {
        if let Err(e) =
            crate::feature::telegram::send_telegram_typing(state.bot.clone(), payload.chat_id).await
        {
            error!("Failed to send Telegram typing: {}", e);
        }
    }

    Json(serde_json::json!({ "status": "ok" }))
}

async fn handle_outbound(
    State(state): State<AppState>,
    Json(payload): Json<crate::feature::OutboundMessage>,
) -> Json<serde_json::Value> {
    info!("Outbound message: {:?}", payload);

    if payload.channel == "telegram" {
        if let Err(e) = crate::feature::telegram::send_telegram_message(
            state.bot.clone(),
            payload.chat_id,
            payload.text,
        )
        .await
        {
            error!("Failed to send Telegram message: {}", e);
            return Json(serde_json::json!({ "status": "error", "message": e.to_string() }));
        }
    }

    Json(serde_json::json!({ "status": "sent" }))
}
