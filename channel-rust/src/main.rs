use crate::common::storage::StorageClient;
use axum::{
    extract::State, routing::{get, post},
    Json,
    Router,
};
use dotenvy::{dotenv, var};
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

mod common;
mod feature;

#[derive(Clone)]
struct AppState {
    qr_code: Arc<Mutex<Option<String>>>,
    bot: Bot,
    redis: common::redis::RedisClient,
    storage: StorageClient,
    wa_tx: tokio::sync::mpsc::UnboundedSender<feature::OutboundMessage>,
    wa_cmd_tx: tokio::sync::mpsc::UnboundedSender<feature::WhatsAppCommand>,
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

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis = common::redis::RedisClient::new(&redis_url)?;
    let storage_s3 = StorageClient::new(
        var("S3_ACCESS_KEY").unwrap_or("this-is-access-key".to_string()),
        var("S3_SECRET_KEY").unwrap_or("this-is-secret-and-important".to_string()),
        var("S3_URL").unwrap_or("http://localhost:4000".to_string()),
    );

    let qr_code = Arc::new(Mutex::new(None));
    let (wa_tx, mut wa_rx) =
        tokio::sync::mpsc::unbounded_channel::<feature::OutboundMessage>();
    let (wa_cmd_tx, mut wa_cmd_rx) =
        tokio::sync::mpsc::unbounded_channel::<feature::WhatsAppCommand>();

    let state = AppState {
        qr_code: qr_code.clone(),
        bot: bot.clone(),
        redis: redis.clone(),
        storage:storage_s3.clone(),
        wa_tx,
        wa_cmd_tx,
    };
    info!("AppState created.");

    // WhatsApp Worker
    let wa_redis = redis.clone();
    let wa_qr = qr_code.clone();
    let storage = storage_s3.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        let s3 = storage.clone();
        let db_path =
            std::env::var("WHATSAPP_DB_PATH").unwrap_or_else(|_| "whatsapp.db".to_string());
        match feature::whatsapp::WhatsAppWorker::new(&db_path, wa_redis,storage, wa_qr).await {
            Ok((worker, mut bot)) => {
                info!("Starting WhatsApp bot...");
                let client = bot.client();
                // Run the bot event loop and task processor
                match bot.run().await {
                    Ok(_handle) => {
                        // Connect the client to WhatsApp
                        tokio::spawn(async move {
                            if let Err(e) = client.connect().await {
                                error!("WhatsApp client connection failed: {}", e);
                            }
                        });
                    }
                    Err(e) => error!("Failed to run WhatsApp bot: {}", e),
                }

                // Listen for outbound messages and commands for WhatsApp
                loop {
                    tokio::select! {
                        Some(msg) = wa_rx.recv() => {
                            if let Err(e) = worker.send_message(msg,&s3).await {
                                error!("Failed to send WhatsApp message: {}", e);
                            }
                        }
                        Some(cmd) = wa_cmd_rx.recv() => {
                            match cmd {
                                crate::feature::WhatsAppCommand::Logout => {
                                    if let Err(e) = worker.logout(&state_clone).await {
                                        error!("Failed to logout from WhatsApp: {}", e);
                                    }
                                }
                            }
                        }
                        else => break,
                    }
                }
            }
            Err(e) => error!("Failed to initialize WhatsApp worker: {}", e),
        }
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    info!("Configuring Router...");
    let app = Router::new()
        .route("/api/whatsapp/qr", get(get_whatsapp_qr))
        .route("/api/whatsapp/logout", post(logout_whatsapp))
        .route("/api/outbound", post(handle_outbound))
        .route("/api/presence/typing", post(handle_typing))
        .with_state(state.clone())
        .layer(cors);
    info!("Router configured.");

    // Start Telegram Worker
    let bot_worker = bot.clone();
    let redis_worker = redis.clone();
    let storage = storage_s3.clone();
    info!("Spawning Telegram Worker...");
    tokio::spawn(async move {
        info!("Telegram Worker task started.");
        feature::telegram::start_telegram_worker(bot_worker, redis_worker,storage).await;
        info!("Telegram Worker task finished (unexpectedly).");
    });

    // Start Redis Listener
    let redis_listener_state = state.clone();
    info!("Spawning Redis Listener...");
    tokio::spawn(async move {
        info!("Redis Listener task started.");
        if let Err(e) = feature::redis::start_redis_listener(redis_listener_state).await {
            error!("Redis listener failed: {}", e);
        }
        info!("Redis Listener task finished.");
    });

    let port = std::env::var("PORT").unwrap_or_else(|_| "8001".to_string());
    let addr = format!("0.0.0.0:{}", port);
    info!("Attempting to bind TcpListener to {}...", addr);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => {
            info!("TcpListener bound successfully.");
            l
        }
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

async fn logout_whatsapp(State(state): State<AppState>) -> Json<serde_json::Value> {
    let _ = state.wa_cmd_tx.send(crate::feature::WhatsAppCommand::Logout);
    Json(serde_json::json!({ "status": "logout_initiated" }))
}

async fn handle_typing(
    State(state): State<AppState>,
    Json(payload): Json<feature::TypingRequest>,
) -> Json<serde_json::Value> {
    info!("Presence (typing): {:?}", payload);

    if payload.channel == "telegram" {
        if let Err(e) =
            feature::telegram::send_telegram_typing(state.bot.clone(), payload.chat_id).await
        {
            error!("Failed to send Telegram typing: {}", e);
        }
    }

    Json(serde_json::json!({ "status": "ok" }))
}

async fn handle_outbound(
    State(state): State<AppState>,
    Json(payload): Json<feature::OutboundMessage>,
) -> Json<serde_json::Value> {
    info!("Outbound message: {:?}", payload);

    match payload.channel.as_str() {
        "telegram" => {
            if let Err(e) = feature::telegram::send_telegram_message(
                state.bot.clone(),
                payload,
                &state.storage
            )
            .await
            {
                error!("Failed to send Telegram message: {}", e);
                return Json(serde_json::json!({ "status": "error", "message": e.to_string() }));
            }
        }
        "whatsapp" => {
            let _ = state.wa_tx.send(payload);
        }
        _ => {
            error!("Unknown channel: {}", payload.channel);
        }
    }

    Json(serde_json::json!({ "status": "sent" }))
}
