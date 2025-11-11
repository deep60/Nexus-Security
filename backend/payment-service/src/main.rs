mod blockchain;
mod config;
mod handlers;
mod models;
mod services;
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
use crate::services::payment_service::PaymentService;

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

    info!("Starting Payment Service...");

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

    // Initialize blockchain provider
    let provider = blockchain::provider::create_provider(&config.blockchain).await?;
    info!("Blockchain provider initialized");

    // Initialize payment service
    let payment_service = Arc::new(
        PaymentService::new(
            config.clone(),
            db_pool.clone(),
            redis_conn.clone(),
            provider.clone(),
        )
        .await?,
    );
    info!("Payment service initialized");

    // Start background workers
    let service_clone = payment_service.clone();
    tokio::spawn(async move {
        if let Err(e) = workers::transaction_monitor::start(service_clone).await {
            warn!("Transaction monitor error: {}", e);
        }
    });

    let service_clone = payment_service.clone();
    tokio::spawn(async move {
        if let Err(e) = workers::pending_payment_processor::start(service_clone).await {
            warn!("Pending payment processor error: {}", e);
        }
    });

    let service_clone = payment_service.clone();
    tokio::spawn(async move {
        if let Err(e) = workers::balance_reconciliation::start(service_clone).await {
            warn!("Balance reconciliation worker error: {}", e);
        }
    });

    info!("Background workers started");

    // Build application state
    let app_state = Arc::new(AppState {
        config: config.clone(),
        db_pool,
        redis_conn,
        payment_service,
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health::health_check))
        // Payment endpoints
        .route("/api/v1/payments/bounty/deposit", post(handlers::payment::deposit_bounty_reward))
        .route("/api/v1/payments/bounty/distribute", post(handlers::payment::distribute_bounty_reward))
        .route("/api/v1/payments/stake/lock", post(handlers::payment::lock_stake))
        .route("/api/v1/payments/stake/unlock", post(handlers::payment::unlock_stake))
        .route("/api/v1/payments/stake/slash", post(handlers::payment::slash_stake))
        .route("/api/v1/payments/withdraw", post(handlers::payment::withdraw_funds))
        .route("/api/v1/payments/balance/:address", get(handlers::payment::get_balance))
        .route("/api/v1/payments/transactions/:address", get(handlers::payment::get_transactions))
        .route("/api/v1/payments/transaction/:tx_hash", get(handlers::payment::get_transaction_status))
        // Gas estimation
        .route("/api/v1/payments/gas/estimate", post(handlers::payment::estimate_gas))
        // Admin endpoints
        .route("/api/v1/admin/payments/pending", get(handlers::admin::get_pending_payments))
        .route("/api/v1/admin/payments/failed", get(handlers::admin::get_failed_payments))
        .route("/api/v1/admin/payments/:id/retry", post(handlers::admin::retry_payment))
        .route("/api/v1/admin/treasury/balance", get(handlers::admin::get_treasury_balance))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Payment Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: sqlx::PgPool,
    pub redis_conn: redis::aio::ConnectionManager,
    pub payment_service: Arc<PaymentService>,
}
