pub mod common;
pub mod feature;
pub mod prompts;
pub mod rag;
pub mod routes;
pub mod services;
pub mod utils;

use crate::common::app_state::AppState;
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

    let mqtt_client_id = var("MQTT_CLIENT_ID").expect("MQTT_CLIENT_ID must be set");
    let mqtt_host = var("MQTT_HOST").expect("MQTT_HOST must be set");
    let mqtt_user = var("MQTT_USER").expect("MQTT_USER must be set");
    let mqtt_password = var("MQTT_PASSWORD").expect("MQTT_PASSWORD must be set");

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let gemini_api_key = var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let redis_url = var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let storage_access_key = var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY must be set");
    let storage_secret_key = var("S3_SECRET_KEY").expect("S3_SECRET_KEY must be set");

    let app_url = var("APP_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

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


    // Bootstraps our independent background worker loop without touching existing engines
    let mqtt_manager = services::mqtt_service::MqttManager::init(
        mqtt_client_id.as_str(),
        mqtt_host.as_str(),
        8883,
        Some(mqtt_user.as_str()),
        Some(mqtt_password.as_str())
    );
    let mqtt = Arc::new(mqtt_manager);

    let redis = crate::common::redis::RedisClient::new(&redis_url)?;


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

    let state = AppState {
        pool,
        gemini,
        gemini_api_key,
        // presence,
        redis,
        storage,
        mqtt,
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

    // Boot-time Intent Synchronization
    let sync_state = state.clone();
    tokio::spawn(async move {
        let boot_dispatcher = common::tools::ToolDispatcher::new(
            sync_state.pool.clone(),
            std::path::PathBuf::from("."),
            None,
            None,
            sync_state.gemini.clone(),
            sync_state.gemini_api_key.clone(),
            sync_state.storage.clone(),
            sync_state.clone(),
        );

        if let Err(e) = services::intent_classifier::IntentClassifierService::sync_plugin_intents_to_knowledge(&boot_dispatcher).await {
            error!("Failed to sync plugin intents: {}", e);
        }

        let guardrail_service = services::guardrail::GuardrailService::new(
            sync_state.pool.clone(),
            sync_state.gemini_api_key.clone(),
        );
        if let Err(e) = guardrail_service.sync_injection_patterns().await {
            error!("Failed to sync guardrail patterns: {}", e);
        }
    });

    // Start Reminder Worker
    let schedule_task_state = state.clone();
    tokio::spawn(async move {
        common::reminder::start_schedule_worker(schedule_task_state).await;
    });

    // Configure CORS


    
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
