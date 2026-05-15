pub mod common;
pub mod feature;
pub mod models;
pub mod prompts;
pub mod rag;
pub mod routes;
pub mod utils;

use crate::common::app_state::AppState;
use crate::common::sse::sse_emitter::SseBroadcaster;
use axum::Router;
use dotenvy::dotenv;
use gemini_rust::{Gemini, Model};
use sqlx::postgres::PgPoolOptions;
use std::{env::var, sync::Arc};
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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
    let storage = crate::common::storage::StorageClient::new(
        storage_access_key,
        storage_secret_key,
        storage_url,
    );

    // Using Gemini25Flash which corresponds to gemini-1.5-flash
    let gemini = Gemini::with_model(&gemini_api_key, Model::Gemini25Flash)
        .expect("Failed to create Gemini client");
    let gemini = Arc::new(gemini);
    let sse = SseBroadcaster::create();

    let state = AppState {
        sse,
        pool,
        gemini,
        gemini_api_key,
        // presence,
        redis,
        storage,
        model_info: common::agent::agent_model::ModelInfo {
            agent_model: "gemini-2.0-flash".to_string(),
            rag_embedding: "gemini-embedding-2".to_string(),
            media_classification: "gemini-2.0-flash".to_string(),
            media_analyze: "gemini-2.0-flash".to_string(),
        },
    };

    // Start Redis Listener
    let redis_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::feature::redis::start_redis_listener(redis_state).await {
            error!("Redis listener failed: {}", e);
        }
    });

    // Start Reminder Worker
    let schedule_task_state = state.clone();
    tokio::spawn(async move {
        common::reminder::start_schedule_worker(schedule_task_state).await;
    });

    // Start Stock Worker
    // let stock_state = state.clone();
    // tokio::spawn(async move {
    //     stock::start_stock_worker(stock_state).await;
    // });

    // Start Cleanup Worker for pending_media
    let cleanup_state = state.clone();
    tokio::spawn(async move {
        loop {
            info!("Running pending_media cleanup...");
            if let Err(e) = common::repository::pending_media_repo::cleanup_old_pending_media(
                &cleanup_state.pool,
            )
            .await
            {
                error!("Failed to cleanup pending_media: {}", e);
            }
            tokio::time::sleep(std::time::Duration::from_secs(6 * 3600)).await;
        }
    });

    // Configure CORS
    let app_url = var("APP_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    
    let origins = [
        "http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap(),
        "http://127.0.0.1:5173".parse::<axum::http::HeaderValue>().unwrap(),
        app_url.parse::<axum::http::HeaderValue>().unwrap(),
    ];

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
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
