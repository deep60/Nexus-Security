mod aggregation;
mod config;
mod handlers;
mod models;
mod services;
mod validators;
mod workers;

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
use crate::services::consensus_service::ConsensusService;

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

    info!("Starting Consensus Service...");

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

    // Initialize consensus service
    let consensus_service = Arc::new(
        ConsensusService::new(
            config.clone(),
            db_pool.clone(),
            redis_conn.clone(),
        )
        .await?,
    );
    info!("Consensus service initialized");

    // Start background workers
    let service_clone = consensus_service.clone();
    tokio::spawn(async move {
        if let Err(e) = workers::consensus_processor::start(service_clone).await {
            warn!("Consensus processor error: {}", e);
        }
    });

    let service_clone = consensus_service.clone();
    tokio::spawn(async move {
        if let Err(e) = workers::dispute_resolver::start(service_clone).await {
            warn!("Dispute resolver error: {}", e);
        }
    });

    info!("Background workers started");

    // Build application state
    let app_state = Arc::new(AppState {
        config: config.clone(),
        db_pool,
        redis_conn,
        consensus_service,
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health::health_check))
        // Consensus endpoints
        .route("/api/v1/consensus/bounty/:bounty_id", get(handlers::consensus::get_bounty_consensus))
        .route("/api/v1/consensus/bounty/:bounty_id/calculate", post(handlers::consensus::calculate_consensus))
        .route("/api/v1/consensus/submission/:submission_id", get(handlers::consensus::get_submission_consensus))
        .route("/api/v1/consensus/stats/:bounty_id", get(handlers::consensus::get_consensus_stats))
        // Dispute endpoints
        .route("/api/v1/disputes/create", post(handlers::dispute::create_dispute))
        .route("/api/v1/disputes/:dispute_id", get(handlers::dispute::get_dispute))
        .route("/api/v1/disputes/:dispute_id/resolve", post(handlers::dispute::resolve_dispute))
        .route("/api/v1/disputes/bounty/:bounty_id", get(handlers::dispute::get_bounty_disputes))
        // Validation endpoints
        .route("/api/v1/validation/submission/:submission_id", post(handlers::validation::validate_submission))
        .route("/api/v1/validation/batch", post(handlers::validation::batch_validate))
        // Admin endpoints
        .route("/api/v1/admin/consensus/recalculate/:bounty_id", post(handlers::admin::recalculate_consensus))
        .route("/api/v1/admin/consensus/override/:bounty_id", post(handlers::admin::override_consensus))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Consensus Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: sqlx::PgPool,
    pub redis_conn: redis::aio::ConnectionManager,
    pub consensus_service: Arc<ConsensusService>,
}
