pub mod models;
pub mod rag;
pub mod common;
pub mod routes;
pub mod feature;

use crate::common::sse::sse_emitter::SseBroadcaster;
use axum::Router;
use dotenvy::dotenv;
use gemini_rust::{Gemini, Model};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env::var, sync::Arc};
use std::fmt::format;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use crate::common::agent::parse_llm_output;

#[derive(Clone)]
pub struct AppState {
    pub sse: Arc<SseBroadcaster>,
    pub pool: Pool<Postgres>,
    pub gemini: Arc<Gemini>,
    pub gemini_api_key: String,
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

    // Using Gemini25Flash which corresponds to gemini-1.5-flash
    let gemini = Gemini::with_model(&gemini_api_key, Model::Gemini25Flash).expect("Failed to create Gemini client");
    let gemini = Arc::new(gemini);
    let sse = SseBroadcaster::create();
    let state = AppState {
        sse,
        pool,
        gemini,
        gemini_api_key,
    };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Use the new routes module
    let app = Router::new()
        .nest("/api", routes::create_router(state))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    info!("Gateway listening on 0.0.0.0:8000");
    axum::serve(listener, app).await?;

    Ok(())
}
