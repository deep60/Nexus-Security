use axum::{
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{get, post, put, delete},
    Router,
};
use axum_extra::extract::Multipart;
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
    trace::TraceLayer,
    limit::RequestBodyLimitLayer,
};
use uuid::Uuid;
use anyhow::{Context, Result};
use tracing::{info, warn, error, debug};

mod handlers;
mod models;
mod services;
mod utils;

use handlers::{auth, bounty, submission};
use models::{bounty::Bounty, user::User, analysis::AnalysisResult};
use services::{blockchain::BlockchainService, database::DatabaseService, redis::RedisService};
use utils::{crypto::JwtClaims, validation::ValidationError};

use crate::utils::helpers::current_timestamp;

// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseService>,
    pub redis: Arc<RedisService>,
    pub blockchain: Arc<BlockchainService>,
    pub config: Arc<AppConfig>,
    pub active_sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
}

// Configuration strucutre
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub blockchain_rpc_url: String,
    pub jwt_secret: String,
    pub max_file_size: usize,
    pub analysis_timeout: u64,
    pub cors_origins: Vec<String>,
    pub rate_limit_per_minute: u32,
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

// Standard API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub timestamp: u64,
}

// Error response structure
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
    pub details: Option<serde_json::Value>,
}

// Health check response
#[derive(Serialize)]
pub struct HealthCheck {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub services: HashMap<String, bool>,
}

// File upload query parameters
#[derive(Deserialize)]
pub struct UploadQuery {
    pub bounty_amount: Option<f64>,
    pub priority: Option<String>,
    pub engines: Option<String>, // Comma-separated list
}

impl Default for AppConfig {
    fn default() -> Self {
        Self { 
            server_host: "0.0.0.0".to_string(), 
            server_port: 8080, 
            database_url: "postgresql://localhost/nexus_security".to_string(), 
            redis_url: "redis://localhost:6379".to_string(), 
            blockchain_rpc_url: "http://localhost:8545".to_string(), 
            jwt_secret: "your-super-secret-jwt-key".to_string(), 
            max_file_size: 50 * 1024 * 1024, 
            analysis_timeout: 300, 
            cors_origins: vec!["http://localhost:3000".to_string()], 
            rate_limit_per_minute: 60,
        }
    }
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self { 
            success: true, 
            data: Some(data), 
            message: None, 
            timestamp: current_timestamp(), 
        }
    }

    pub fn error(message: String) -> ApiResponse<()> {
        ApiResponse { 
            success: false, 
            data: None, 
            message: Some(message), 
            timestamp: current_timestamp(), 
        }
    }
}

// Middleware for authentication
async fn auth_middleware(State(state): State<AppState>, mut request: axum::extract::Request, next: Next) -> Result<Response, StatusCode> {
    // Extract authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // Validate JWT token
            match utils::crypto::validate_jwt(token, &state.config.jwt_secret) {
                Ok(claims) => {
                    // Check if session is still active
                    let session = state.active_sessions.read().await;
                    if let Some(session) = sessions.get(&claims.sub) {
                        // Add user info to request extensions
                        request.extensions_mut().insert(claims);
                        request.extensions_mut().insert(session.clone());
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
    if path.starts_with("/api/v1/health") || path.starts_with("/api/v1/auth/login") || path.starts_with("api/v1/auth/register") {
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

// health check endpoint
async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = current_timestamp() - state.config.server_port as u64;    // Placeholder
    let mut services = HashMap::new();

    // Check database connection
    services.insert("database".to_string(), state.db.health_check().await);

    // Check Redis connection
    services.insert("redis".to_string(), state.redis.health_check().await);

    // Check blockchain connection
    services.insert("blockchain".to_string(), state.blockchain.health_check().await);

    let all_healthy = services.values().all(|&healthy| healthy);
    let status = if all_healthy { "ok" } else { "degraded" };

    let health = HealthCheck {
        status: status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime,
        services,
    };

    let status_code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(ApiResponse::success(health)))
}

// File analysis endpoint
async fn analyze_file(
    State(state): State<AppState>,
    Query(params): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    let mut file_data = Vec::new();
    let mut filename = String::new();
    let mut content_type = String::new();

    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let field_name = field.name().unwrap_or("").to_string();
        
        match field_name.as_str() {
            "file" => {
                filename = field.file_name().unwrap_or("unknown").to_string();
                content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
                file_data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec();
            }
            _ => {
                // Handle other form fields if needed
                continue;
            }
        }
    }

    if file_data.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate file size
    if file_data.len() > state.config.max_file_size {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    // Generate unique submission ID
    let submission_id = Uuid::new_v4();
    
    // Create analysis request
    let analysis_request = models::analysis::AnalysisRequest {
        id: submission_id,
        filename,
        content_type,
        file_size: file_data.len(),
        file_hash: utils::crypto::calculate_sha256(&file_data),
        bounty_amount: params.bounty_amount.unwrap_or(0.0),
        priority: params.priority.unwrap_or_else(|| "normal".to_string()),
        engines: params.engines.map(|e| e.split(',').map(|s| s.trim().to_string()).collect()),
        created_at: current_timestamp(),
    };

    // Store file and metadata
    match state.db.store_analysis_request(&analysis_request, &file_data).await {
        Ok(_) => {
            // Trigger analysis pipeline
            if let Err(e) = trigger_analysis(&state, &analysis_request).await {
                error!("Failed to trigger analysis: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            Ok((StatusCode::ACCEPTED, Json(ApiResponse::success(analysis_request))))
        }
        Err(e) => {
            error!("Failed to store analysis request: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get analysis results
async fn get_analysis_result(
    State(state): State<AppState>,
    Path(analysis_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    match state.db.get_analysis_result(analysis_id).await {
        Ok(Some(result)) => Ok(Json(ApiResponse::success(result))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get analysis result: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Trigger analysis pipeline
async fn trigger_analysis(state: &AppState, request: &models::analysis::AnalysisRequest) -> Result<()> {
    // Queue analysis request in Redis
    let analysis_queue_key = "analysis:queue";
    let request_json = serde_json::to_string(request)?;
    
    state.redis.push_to_queue(analysis_queue_key, &request_json).await?;
    
    // If blockchain bounty is specified, create smart contract interaction
    if request.bounty_amount > 0.0 {
        state.blockchain.create_analysis_bounty(request).await?;
    }
    
    info!("Analysis request {} queued successfully", request.id);
    Ok(())
}

// Build application router
fn create_router(state: AppState) -> Router {
    Router::new()
        // Health and monitoring
        .route("/api/v1/health", get(health_check))
        
        // Authentication routes
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/v1/auth/logout", post(auth::logout))
        .route("/api/v1/auth/refresh", post(auth::refresh_token))
        
        // Analysis routes
        .route("/api/v1/analyze/file", post(analyze_file))
        .route("/api/v1/analyze/url", post(submission::analyze_url))
        .route("/api/v1/analysis/:id", get(get_analysis_result))
        .route("/api/v1/analysis/:id/report", get(submission::get_detailed_report))
        
        // Bounty management routes
        .route("/api/v1/bounties", get(bounty::list_bounties))
        .route("/api/v1/bounties", post(bounty::create_bounty))
        .route("/api/v1/bounties/:id", get(bounty::get_bounty))
        .route("/api/v1/bounties/:id", put(bounty::update_bounty))
        .route("/api/v1/bounties/:id/participate", post(bounty::participate_in_bounty))
        .route("/api/v1/bounties/:id/submit", post(bounty::submit_analysis))
        
        // User and reputation routes
        .route("/api/v1/profile", get(auth::get_profile))
        .route("/api/v1/profile", put(auth::update_profile))
        .route("/api/v1/reputation/leaderboard", get(auth::get_leaderboard))
        
        // WebSocket for real-time updates
        .route("/api/v1/ws", get(handlers::websocket::websocket_handler))
        
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
                .layer(middleware::from_fn(logging_middleware))
                .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB limit
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any)
                )
        )
}

// Initialize services
async fn initialize_services(config: &AppConfig) -> Result<(DatabaseService, RedisService, BlockchainService)> {
    info!("Initializing services...");
    
    // Initialize database
    let db = DatabaseService::new(&config.database_url).await
        .context("Failed to initialize database service")?;
    
    // Run database migrations
    db.run_migrations().await
        .context("Failed to run database migrations")?;
    
    // Initialize Redis
    let redis = RedisService::new(&config.redis_url).await
        .context("Failed to initialize Redis service")?;
    
    // Initialize blockchain service
    let blockchain = BlockchainService::new(&config.blockchain_rpc_url).await
        .context("Failed to initialize blockchain service")?;
    
    info!("All services initialized successfully");
    Ok((db, redis, blockchain))
}

// Load configuration from environment or config files
fn load_config() -> AppConfig {
    let mut config = AppConfig::default();
    
    // Override with environment variables if present
    if let Ok(host) = std::env::var("SERVER_HOST") {
        config.server_host = host;
    }
    
    if let Ok(port) = std::env::var("SERVER_PORT") {
        if let Ok(port_num) = port.parse::<u16>() {
            config.server_port = port_num;
        }
    }
    
    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        config.database_url = db_url;
    }
    
    if let Ok(redis_url) = std::env::var("REDIS_URL") {
        config.redis_url = redis_url;
    }
    
    if let Ok(blockchain_url) = std::env::var("BLOCKCHAIN_RPC_URL") {
        config.blockchain_rpc_url = blockchain_url;
    }
    
    if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
        config.jwt_secret = jwt_secret;
    }
    
    config
}


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

    info!("Starting Nexus-Security API Gateway v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = load_config();
    info!("Configuration loaded: {}:{}", config.server_host, config.server_port);

    // Initialize services
    let (db, redis, blockchain) = initialize_services(&config).await?;

    // Create application state
    let state = AppState {
        db: Arc::new(db),
        redis: Arc::new(redis),
        blockchain: Arc::new(blockchain),
        config: Arc::new(config.clone()),
        active_sessions: Arc::new(RwLock::new(HashMap::new())),
    };

    // Create router with all routes and middleware
    let app = create_router(state);

    // Create server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    
    // Create TCP listener
    let listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind to address")?;

    info!("🚀 Nexus-Security API Gateway running on http://{}", addr);
    info!("📚 API Documentation available at http://{}/api/v1/docs", addr);
    info!("🔍 Health check available at http://{}/api/v1/health", addr);

    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server error")?;

    info!("Nexus-Security API Gateway shut down gracefully");
    Ok(())
}