mod channels;
mod config;
mod handlers;
mod models;
mod notification_manager;
mod templates;

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
use crate::notification_manager::NotificationManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .json()
        .init();

    info!("Starting Notification Service...");

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
    let redis_conn = redis_client
        .get_connection_manager()
        .await?;
    info!("Redis connection established");

    // Initialize notification manager
    let notification_manager = Arc::new(
        NotificationManager::new(
            config.clone(),
            db_pool.clone(),
            redis_conn.clone(),
        )
        .await?
    );
    info!("Notification manager initialized");

    // Start background workers
    let manager_clone = notification_manager.clone();
    tokio::spawn(async move {
        if let Err(e) = manager_clone.start_event_listener().await {
            warn!("Event listener error: {}", e);
        }
    });

    let manager_clone = notification_manager.clone();
    tokio::spawn(async move {
        if let Err(e) = manager_clone.start_retry_worker().await {
            warn!("Retry worker error: {}", e);
        }
    });

    info!("Background workers started");

    // Build application state
    let app_state = Arc::new(AppState {
        config: config.clone(),
        db_pool,
        redis_conn,
        notification_manager,
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/api/v1/notifications/send", post(handlers::notification::send_notification))
        .route("/api/v1/notifications/preferences", get(handlers::preferences::get_preferences))
        .route("/api/v1/notifications/preferences", post(handlers::preferences::update_preferences))
        .route("/api/v1/notifications/history", get(handlers::notification::get_notification_history))
        .route("/api/v1/notifications/:id/retry", post(handlers::notification::retry_notification))
        .route("/api/v1/webhooks/register", post(handlers::webhook::register_webhook))
        .route("/api/v1/webhooks/unregister", post(handlers::webhook::unregister_webhook))
        .route("/ws", get(handlers::websocket::websocket_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Notification Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: sqlx::PgPool,
    pub redis_conn: redis::aio::ConnectionManager,
    pub notification_manager: Arc<NotificationManager>,
}
