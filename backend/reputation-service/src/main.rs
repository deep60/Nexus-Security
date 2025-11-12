mod config;
mod handlers;
mod models;
mod scoring;
mod services;
mod workers;
mod analytics;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, warn};

use crate::config::Config;
use crate::services::reputation_service::ReputationService;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .json()
        .init();

    info!("Starting Reputation Service...");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Initialize database connection pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;
    info!("Database connection pool established");

    // Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    info!("Database migrations completed");

    // Initialize Redis client
    let redis_client = redis::Client::open(config.redis.url.clone())?;
    let redis_conn = redis_client.get_connection_manager().await?;
    info!("Redis connection established");

    // Initialize reputation service
    let reputation_service = Arc::new(
        ReputationService::new(
            config.clone(),
            db_pool.clone(),
            redis_conn.clone(),
        )
        .await?,
    );
    info!("Reputation service initialized");

    // Start background workers
    let service_clone = reputation_service.clone();
    tokio::spawn(async move {
        if let Err(e) = workers::reputation_calculator::start(service_clone).await {
            warn!("Reputation calculator error: {}", e);
        }
    });

    let service_clone = reputation_service.clone();
    tokio::spawn(async move {
        if let Err(e) = workers::decay_processor::start(service_clone).await {
            warn!("Decay processor error: {}", e);
        }
    });

    let service_clone = reputation_service.clone();
    tokio::spawn(async move {
        if let Err(e) = workers::leaderboard_updater::start(service_clone).await {
            warn!("Leaderboard updater error: {}", e);
        }
    });

    info!("Background workers started");

    // Build application state
    let app_state = Arc::new(AppState {
        config: config.clone(),
        db_pool,
        redis_conn,
        reputation_service,
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health::health_check))
        // Reputation endpoints
        .route("/api/v1/reputation/user/:user_id", get(handlers::reputation::get_user_reputation))
        .route("/api/v1/reputation/user/:user_id/history", get(handlers::reputation::get_reputation_history))
        .route("/api/v1/reputation/user/:user_id/update", post(handlers::reputation::update_reputation))
        .route("/api/v1/reputation/engine/:engine_id", get(handlers::reputation::get_engine_reputation))
        .route("/api/v1/reputation/leaderboard", get(handlers::reputation::get_leaderboard))
        .route("/api/v1/reputation/badges/:user_id", get(handlers::reputation::get_user_badges))
        // Analytics endpoints
        .route("/api/v1/analytics/reputation/trends", get(handlers::analytics::get_reputation_trends))
        .route("/api/v1/analytics/reputation/distribution", get(handlers::analytics::get_score_distribution))
        .route("/api/v1/analytics/accuracy/stats", get(handlers::analytics::get_accuracy_stats))
        // Admin endpoints
        .route("/api/v1/admin/reputation/recalculate/:user_id", post(handlers::admin::recalculate_reputation))
        .route("/api/v1/admin/reputation/reset/:user_id", post(handlers::admin::reset_reputation))
        .route("/api/v1/admin/badges/award", post(handlers::admin::award_badge))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Reputation Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: sqlx::PgPool,
    pub redis_conn: redis::aio::ConnectionManager,
    pub reputation_service: Arc<ReputationService>,
}
