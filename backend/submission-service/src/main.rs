// Pre-production scaffolding: some items are intentionally unused while
// features are wired up. This crate-level allow keeps `clippy -D warnings`
// green without deleting code we are about to use. Remove before GA.
#![allow(dead_code)]

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

mod db;
mod handlers;
mod models;
mod queue;
mod storage;

use storage::s3_client::S3Client;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub s3_client: Arc<S3Client>,
    pub db_pool: PgPool,
    pub redis_client: redis::Client,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Submission Service...");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database connection with proper error handling
    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;

    tracing::info!("Connecting to database...");
    let db_pool = match PgPool::connect(&database_url).await {
        Ok(pool) => {
            tracing::info!("Database connection established");
            pool
        }
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            return Err(anyhow::anyhow!("Database connection failed: {e}").into());
        }
    };

    // Initialize Redis client with error handling
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    tracing::info!("Connecting to Redis at {}...", redis_url);
    let redis_client = match redis::Client::open(redis_url) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to create Redis client: {}", e);
            return Err(anyhow::anyhow!("Redis client creation failed: {e}").into());
        }
    };

    // Test Redis connection
    match redis_client.get_multiplexed_async_connection().await {
        Ok(_) => tracing::info!("Redis connection established"),
        Err(e) => {
            tracing::error!("Failed to connect to Redis: {}", e);
            return Err(anyhow::anyhow!("Redis connection failed: {e}").into());
        }
    }

    // Initialize S3 client
    let s3_client = S3Client::new().await?;
    tracing::info!("S3 client initialized successfully");

    // Create app state
    let state = AppState {
        s3_client: Arc::new(s3_client),
        db_pool,
        redis_client,
    };

    // Build CORS layer - allow specific origins from environment
    let allowed_origins: Vec<_> = std::env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string())
        .split(',')
        .filter_map(|origin| origin.trim().parse::<axum::http::HeaderValue>().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/submit/file", post(handlers::file_upload::submit_file))
        .route("/submit/url", post(handlers::url_submission::submit_url))
        .layer(cors)
        .with_state(state);

    // Get port from environment or use default
    let port = std::env::var("SUBMISSION_SERVICE_PORT")
        .unwrap_or_else(|_| "8085".to_string())
        .parse::<u16>()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Submission Service listening on {}", addr);

    // Run the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Submission Service is healthy"
}
