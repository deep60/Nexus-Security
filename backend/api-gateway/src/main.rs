use anyhow::{Context, Result};
use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, StatusCode},
    middleware::{self as axum_middleware, Next},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{net::TcpListener, sync::RwLock};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

mod config;
mod handlers;
mod middleware;
use middleware::metrics::{metrics_middleware, MetricsCollector};
mod models;
mod routes;
mod services;
mod utils;

use config::AppConfig;
use handlers::{auth, health, reputation, user};
use models::{analysis::AnalysisResult, bounty::Bounty, user::User};
use services::{blockchain::BlockchainService, database::DatabaseService, redis::RedisService};
use utils::{crypto::JwtClaims, validation::ValidationError};

use crate::models::response::ApiResponse;

use crate::utils::helpers::current_timestamp;

// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseService>,
    pub redis: Arc<RedisService>,
    pub blockchain: Arc<BlockchainService>,
    pub config: Arc<AppConfig>,
    pub active_sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    pub metrics: Arc<MetricsCollector>,
}

// Session information for active users
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub user_id: Uuid,
    pub wallet_address: Option<String>,
    pub reputation_score: i32,
    pub last_activity: u64,
    pub permissions: Vec<String>,
}

// ApiResponse moved to models::response

// Middleware for authentication
async fn auth_middleware(
    State(state): State<AppState>,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // Validate JWT token
            match utils::crypto::validate_jwt(token, &state.config.security.jwt_secret) {
                Ok(claims) => {
                    // Check if session is still active
                    let sessions = state.active_sessions.read().await;
                    if let Some(session_info) = sessions.get(&claims.sub) {
                        // Add user info to request extensions
                        request.extensions_mut().insert(claims);
                        request.extensions_mut().insert(session_info.clone());
                        return Ok(next.run(request).await);
                    }
                }
                Err(e) => {
                    warn!("JWT validation failed: {}", e);
                }
            }
        }
    }

    // For public endpoints, continue without authentication
    let path = request.uri().path();
    if path.starts_with("/api/v1/health")
        || path.starts_with("/api/v1/auth/login")
        || path.starts_with("api/v1/auth/register")
    {
        return Ok(next.run(request).await);
    }

    Err(StatusCode::UNAUTHORIZED)
}

// Middleware for request logging
async fn logging_middleware(request: axum::extract::Request, next: Next) -> Response {
    let start_time = SystemTime::now();
    let method = request.method().clone();
    let uri = request.uri().clone();

    debug!("Incoming request: {} {}", method, uri);

    let response = next.run(request).await;

    let elapsed = start_time.elapsed().unwrap_or_default();
    info!(
        "Request completed: {} {} - Status: {} - Duration: {:?}",
        method,
        uri,
        response.status(),
        elapsed
    );
    response
}




// Initialize services
async fn initialize_services(
    config: &AppConfig,
) -> Result<(DatabaseService, RedisService, BlockchainService)> {
    info!("Initializing services...");

    // Initialize database with connection pool
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .context("Failed to connect to database")?;

    let db = DatabaseService::new(pool);

    // Run database migrations
    sqlx::migrate!("./migrations")
        .run(db.pool())
        .await
        .context("Failed to run database migrations")?;

    // Initialize Redis
    let redis = RedisService::new(&config.redis.url)
        .await
        .context("Failed to initialize Redis service")?;

    // Initialize blockchain service
    let blockchain = BlockchainService::new(config.blockchain.clone())
        .await
        .context("Failed to initialize blockchain service")?;

    info!("All services initialized successfully");
    Ok((db, redis, blockchain))
}

// Load configuration from environment or config files
fn load_config() -> Result<AppConfig> {
    AppConfig::load().context("Failed to load configuration")
}

// // Utility function to get current timestamp
// fn current_timestamp() -> u64 {
//     SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .unwrap_or_default()
//         .as_secs()
// }

// Graceful shutdown handler
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    info!("Shutdown signal received, starting graceful shutdown...");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .compact()
        .init();

    info!(
        "Starting Nexus-Security API Gateway v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Load configuration
    let config = load_config()?;
    info!(config.server.host, config.server.port);

    // Initialize metrics collector
    let metrics_collector = Arc::new(MetricsCollector::new());

    // Initialize services
    let (db, redis, blockchain) = initialize_services(&config).await?;

    // Create application state
    let state = AppState {
        db: Arc::new(db),
        redis: Arc::new(redis),
        blockchain: Arc::new(blockchain),
        config: Arc::new(config.clone()),
        active_sessions: Arc::new(RwLock::new(HashMap::new())),
        metrics: metrics_collector.clone(),
    };

    // Create router with all routes and middleware
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(Any);

    let app = routes::create_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)); // 10MB

    // Create server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    // Create TCP listener
    let listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind to address")?;

    info!("🚀 Nexus-Security API Gateway running on http://{}", addr);
    info!(
        "📚 API Documentation available at http://{}/api/v1/docs",
        addr
    );
    info!("🔍 Health check available at http://{}/api/v1/health", addr);

    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server error")?;

    info!("Nexus-Security API Gateway shut down gracefully");
    Ok(())
}
