use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;
use sqlx::PgPool;

mod handlers;
mod models;
mod storage;
mod db;
mod queue;

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

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    tracing::info!("Connecting to database...");
    let db_pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    tracing::info!("Database connection established");

    // Initialize Redis client
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    tracing::info!("Connecting to Redis at {}...", redis_url);
    let redis_client = redis::Client::open(redis_url)
        .expect("Failed to create Redis client");

    // Test Redis connection
    let mut redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    tracing::info!("Redis connection established");
    drop(redis_conn);

    // Initialize S3 client
    let s3_client = S3Client::new().await?;
    tracing::info!("S3 client initialized successfully");

    // Create app state
    let state = AppState {
        s3_client: Arc::new(s3_client),
        db_pool,
        redis_client,
    };

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

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
