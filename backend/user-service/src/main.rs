mod auth;
mod config;
mod handlers;
mod models;
mod services;
mod middleware;

use anyhow::Result;
use axum::{
    middleware as axum_middleware,
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

use crate::config::Config;
use crate::middleware::{auth_middleware, admin_middleware};
use crate::services::user_service::UserService;

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

    info!("Starting User Service...");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Initialize database connection pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;
    info!("Database connection pool established");

    // Run migrations (if migrations directory exists and has migrations)
    // Note: Database schema is managed centrally in /database folder
    if std::path::Path::new("./migrations").exists() {
        match sqlx::migrate!("./migrations").run(&db_pool).await {
            Ok(_) => info!("Database migrations completed"),
            Err(e) => info!("No migrations to run or already applied: {}", e),
        }
    } else {
        info!("Migrations directory not found - using centralized database schema");
    }

    // Initialize Redis client for sessions
    let redis_client = redis::Client::open(config.redis.url.clone())?;
    let redis_conn = redis_client.get_connection_manager().await?;
    info!("Redis connection established");

    // Initialize user service
    let user_service = Arc::new(
        UserService::new(
            config.clone(),
            db_pool.clone(),
            redis_conn.clone(),
        )
        .await?,
    );
    info!("User service initialized");

    // Build application state
    let app_state = Arc::new(AppState {
        config: config.clone(),
        db_pool,
        redis_conn,
        user_service,
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/api/v1/auth/register", post(handlers::auth::register))
        .route("/api/v1/auth/login", post(handlers::auth::login))
        .route("/api/v1/auth/refresh", post(handlers::auth::refresh_token))
        .route("/api/v1/auth/forgot-password", post(handlers::auth::forgot_password))
        .route("/api/v1/auth/reset-password", post(handlers::auth::reset_password));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/api/v1/auth/logout", post(handlers::auth::logout))
        .route("/api/v1/auth/verify-email", post(handlers::auth::verify_email))
        .route("/api/v1/auth/wallet/verify", post(handlers::auth::verify_wallet))

        // Profile endpoints
        .route("/api/v1/profile", get(handlers::profile::get_profile))
        .route("/api/v1/profile", put(handlers::profile::update_profile))
        .route("/api/v1/profile/avatar", post(handlers::profile::upload_avatar))
        .route("/api/v1/profile/:user_id", get(handlers::profile::get_user_profile))

        // Settings endpoints
        .route("/api/v1/settings", get(handlers::settings::get_settings))
        .route("/api/v1/settings", put(handlers::settings::update_settings))
        .route("/api/v1/settings/password", put(handlers::settings::change_password))
        .route("/api/v1/settings/2fa/enable", post(handlers::settings::enable_2fa))
        .route("/api/v1/settings/2fa/disable", post(handlers::settings::disable_2fa))
        .route("/api/v1/settings/2fa/verify", post(handlers::settings::verify_2fa))

        // KYC endpoints
        .route("/api/v1/kyc/submit", post(handlers::kyc::submit_kyc))
        .route("/api/v1/kyc/status", get(handlers::kyc::get_kyc_status))
        .route("/api/v1/kyc/documents", post(handlers::kyc::upload_documents))

        // Wallet endpoints
        .route("/api/v1/wallet/link", post(handlers::wallet::link_wallet))
        .route("/api/v1/wallet/unlink", delete(handlers::wallet::unlink_wallet))
        .route("/api/v1/wallet/list", get(handlers::wallet::list_wallets))
        .layer(axum_middleware::from_fn_with_state(app_state.clone(), auth_middleware));

    // Admin routes (admin role required)
    let admin_routes = Router::new()
        .route("/api/v1/admin/users", get(handlers::admin::list_users))
        .route("/api/v1/admin/users/:user_id", get(handlers::admin::get_user))
        .route("/api/v1/admin/users/:user_id/suspend", post(handlers::admin::suspend_user))
        .route("/api/v1/admin/users/:user_id/activate", post(handlers::admin::activate_user))
        .route("/api/v1/admin/kyc/:user_id/approve", post(handlers::admin::approve_kyc))
        .route("/api/v1/admin/kyc/:user_id/reject", post(handlers::admin::reject_kyc))
        .layer(axum_middleware::from_fn_with_state(app_state.clone(), admin_middleware));

    // Combine all routes
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("User Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: sqlx::PgPool,
    pub redis_conn: redis::aio::ConnectionManager,
    pub user_service: Arc<UserService>,
}
