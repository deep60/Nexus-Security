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
mod services;
mod utils;

use config::AppConfig;
use handlers::{auth, bounty, health, reputation, submission, user};
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

// File upload query parameters
#[derive(Deserialize)]
pub struct UploadQuery {
    pub bounty_amount: Option<f64>,
    pub priority: Option<String>,
    pub engines: Option<String>, // Comma-separated list
}

// File analysis endpoint
// TODO: Rewrite to match actual AnalysisRequest model structure
async fn analyze_file(
    State(_state): State<AppState>,
    Query(_params): Query<UploadQuery>,
    mut _multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    // Stub implementation - needs rewrite to match actual AnalysisRequest model
    Err::<axum::Json<serde_json::Value>, StatusCode>(StatusCode::NOT_IMPLEMENTED)
}

// Get analysis results
async fn get_analysis_result(
    State(state): State<AppState>,
    Path(analysis_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match state.db.get_analysis_result(analysis_id).await {
        Ok(result) => Ok(Json(ApiResponse::success(result))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

// Trigger analysis pipeline
// async fn trigger_analysis(
//     state: &AppState,
//     request: &models::analysis::AnalysisRequest,
// ) -> Result<()> {
//     // Queue analysis request in Redis
//     let analysis_queue_key = "analysis:queue";
//     let request_json = serde_json::to_string(request)?;

//     state
//         .redis
//         .push_to_queue(analysis_queue_key, &request_json)
//         .await?;

//     // If blockchain bounty is specified, create smart contract interaction
//     if request.bounty_amount > 0.0 {
//         state.blockchain.create_analysis_bounty(request).await?;
//     }

//     info!("Analysis request {} queued successfully", request.id);
//     Ok(())
// }

// Build application router
fn create_router(state: AppState) -> Router {
    let metrics = state.metrics.clone();

    Router::new()
        // Health and monitoring
        .route("/api/v1/health", get(health::health_check))
        .route("/api/v1/metrics", get(health::metrics))
        // Authentication routes
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/v1/auth/logout", post(auth::logout))
        .route("/api/v1/auth/refresh", post(auth::refresh_token))
        .route("/api/v1/auth/verify", post(auth::verify_token))
        // Analysis routes
        .route("/api/v1/analyze/file", post(analyze_file)) // Keep stub for now
        .route("/api/v1/analyze/url", post(submission::create_submission))
        .route("/api/v1/analysis/:id", get(get_analysis_result)) // Keep local stub or move to handler
        .route(
            "/api/v1/analysis/:id/report",
            get(submission::get_submission_details),
        )
        // Bounty management routes
        .route("/api/v1/bounties", get(bounty::list_bounties)) // Fixed name
        .route("/api/v1/bounties/:id", get(bounty::get_bounty)) // Fixed name
        // User and wallet routes
        .route("/api/v1/profile", get(user::get_current_user)) // Use user handler
        .route("/api/v1/wallet/connect", post(auth::collect_wallet)) // Assuming auth handles wallet for now
        .route("/api/v1/wallet/disconnect", post(auth::disconnect_wallet))
        // Reputation routes
        .route(
            "/api/v1/reputation/leaderboard",
            get(reputation::get_leaderboard),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum_middleware::from_fn(logging_middleware))
                .layer(axum_middleware::from_fn(move |req, next| {
                    metrics_middleware(metrics.clone(), None, req, next)
                })) // Add metrics middleware
                .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB limit
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        )
        .with_state(state)
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
    let app = create_router(state);

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
