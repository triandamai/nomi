pub mod common;
pub mod feature;
pub mod models;
pub mod rag;
pub mod routes;
pub mod utils;

use crate::common::sse::sse_builder::{SseBuilder, SseTarget};
use crate::common::sse::sse_emitter::SseBroadcaster;
use crate::feature::realtime::presence::PresenceManager;
use crate::feature::{OutboundMessage, PresenceMessage};
use axum::Router;
use dotenvy::dotenv;
use gemini_rust::{Gemini, Model};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env::var, sync::Arc};
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Clone)]
pub struct AppState {
    pub sse: Arc<SseBroadcaster>,
    pub pool: Pool<Postgres>,
    pub gemini: Arc<Gemini>,
    pub gemini_api_key: String,
    pub presence: Arc<PresenceManager>,
    pub redis: common::redis::RedisClient,
    pub storage: common::storage::StorageClient,
}

impl AppState {

    pub async fn send_sse_to_user(
        &self,
        user_id: &str,
        event_name: &str,
        sse_data: serde_json::Value,
    ) -> anyhow::Result<()> {
        info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::sent_to_user(user_id.to_string(), event_name.to_string()),
                sse_data,
            ))
            .await;
        Ok(())
    }
    pub async fn send_to_user(
        &self,
        user_id: &str,
        event_name: &str,
        sse_data: serde_json::Value,
        redis_data: &OutboundMessage,
    ) -> anyhow::Result<()> {
        info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::sent_to_user(user_id.to_string(), event_name.to_string()),
                sse_data,
            ))
            .await;
        let _ = self.publish_outbond(redis_data);
        Ok(())
    }

    pub async fn broadcast_sse(
        &self,
        event_name: &str,
        sse_data: serde_json::Value
    ) -> anyhow::Result<()> {
        info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::broadcast(event_name.to_string()),
                sse_data,
            ))
            .await;

        Ok(())
    }
    pub async fn broadcast(
        &self,
        event_name: &str,
        sse_data: serde_json::Value,
        redis_data: &OutboundMessage,
    ) -> anyhow::Result<()> {
        info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::broadcast(event_name.to_string()),
                sse_data,
            ))
            .await;

        let _ = self.publish_outbond(redis_data);
        Ok(())
    }


    pub async fn send_presence_to_user(
        &self,
        user_id: &str,
        sse_data: serde_json::Value,
        redis_data: &PresenceMessage,
    ) -> anyhow::Result<()> {
        info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::sent_to_user(user_id.to_string(), "presence".to_string()),
                sse_data,
            ))
            .await;
        let _ = self.publish_presence(redis_data);
        Ok(())
    }

    pub async fn broadcast_presence(
        &self,
        sse_data: serde_json::Value,
        redis_data: &PresenceMessage,
    ) -> anyhow::Result<()> {
        info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::broadcast("presence".to_string()),
                sse_data,
            ))
            .await;

        let _ = self.publish_presence(redis_data);
        Ok(())
    }

    pub async fn broadcast_presence_sse(
        &self,
        sse_data: serde_json::Value
    ) -> anyhow::Result<()> {
        info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::broadcast("presence".to_string()),
                sse_data,
            ))
            .await;
        Ok(())
    }
    pub async fn send_presence_sse_to_user(
        &self,
        user_id: &str,
        sse_data: serde_json::Value
    ) -> anyhow::Result<()> {
        info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::sent_to_user(user_id.to_string(), "presence".to_string()),
                sse_data,
            ))
            .await;
        Ok(())
    }

    pub async fn publish_outbond(&self, redis_data: &OutboundMessage) {
        info!("publish to redis");
        match self.redis.publish_event("nomi:outbound", redis_data).await {
            Ok(_) => {
                info!("outbound event sent");
            }
            Err(err) => {
                error!("outbound publishing failed: {}", err);
            }
        };
    }

    pub async fn publish_presence(&self, redis_data: &PresenceMessage) {
        info!("publish to redis");
        match self.redis.publish_event("nomi:presence", redis_data).await {
            Ok(_) => {
                info!("presence event sent");
            }
            Err(err) => {
                error!("presence publishing failed: {}", err);
            }
        };
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    info!("Starting OpenClaw Gateway...");

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let gemini_api_key = var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let redis_url = var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    debug!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| {
            error!("Failed to connect to database: {}", e);
            e
        })?;
    info!("Database connection established.");

    let redis = crate::common::redis::RedisClient::new(&redis_url)?;

    let storage_access_key = var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY must be set");
    let storage_secret_key = var("S3_SECRET_KEY").expect("S3_SECRET_KEY must be set");
    let storage_url = var("S3_URL").expect("S3_URL must be set");
    let storage = crate::common::storage::StorageClient::new(storage_access_key, storage_secret_key, storage_url);

    // Using Gemini25Flash which corresponds to gemini-1.5-flash
    let gemini = Gemini::with_model(&gemini_api_key, Model::Gemini25Flash)
        .expect("Failed to create Gemini client");
    let gemini = Arc::new(gemini);
    let sse = SseBroadcaster::create();

    // Create a temporary state to initialize PresenceManager
    let partial_state = AppState {
        sse: sse.clone(),
        pool: pool.clone(),
        gemini: gemini.clone(),
        gemini_api_key: gemini_api_key.clone(),
        presence: Arc::new(PresenceManager {
            debouncers: dashmap::DashMap::new(),
            channel_tx: tokio::sync::mpsc::channel(1).0, // Dummy
        }),
        redis: redis.clone(),
        storage: storage.clone(),
    };

    let presence = PresenceManager::new(partial_state);

    let state = AppState {
        sse,
        pool,
        gemini,
        gemini_api_key,
        presence,
        redis,
        storage,
    };

    // Start Redis Listener
    let redis_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::feature::redis::start_redis_listener(redis_state).await {
            error!("Redis listener failed: {}", e);
        }
    });

    // Start Reminder Worker
    let reminder_state = state.clone();
    tokio::spawn(async move {
        crate::feature::conversation::reminder::start_reminder_worker(reminder_state).await;
    });

    // Configure CORS
    let app_url = var("APP_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let localhost = "http://localhost:5173";

    let cors = CorsLayer::new()
        .allow_origin(localhost.parse::<axum::http::HeaderValue>().unwrap())
        .allow_origin(app_url.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::COOKIE,
        ])
        .allow_credentials(true);

    // Use the new routes module
    let app = Router::new()
        .nest("/api", routes::create_router(state))
        .layer(cors);

    let port = var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Gateway listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
