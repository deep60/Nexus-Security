use axum::{
    middleware,
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::{
    handlers::{
        analysis, auth, bounty, health, reputation, submission, user, wallet, webhook,
    },
    middleware::{auth as auth_middleware, cors, logging, metrics, rate_limiter},
    AppState,
};

/// Create all routes for API v1
pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check routes (public, no auth required)
        .nest("/health", health_routes())

        // Authentication routes (public)
        .nest("/auth", auth_routes())

        // User routes (requires authentication)
        .nest("/users", user_routes())

        // Bounty routes (requires authentication)
        .nest("/bounties", bounty_routes())

        // Analysis routes (requires authentication)
        .nest("/analysis", analysis_routes())

        // Submission routes (requires authentication)
        .nest("/submissions", submission_routes())

        // Wallet routes (requires authentication)
        .nest("/wallet", wallet_routes())

        // Reputation routes (public read, authenticated write)
        .nest("/reputation", reputation_routes())

        // Webhook routes (requires authentication)
        .nest("/webhooks", webhook_routes())

        // Apply global middleware
        .layer(middleware::from_fn(metrics::metrics_middleware))
        .layer(middleware::from_fn(logging::logging_middleware))
        .layer(cors::create_cors_layer())
        .with_state(state)
}

/// Health check routes
fn health_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .route("/live", get(health::liveness_check))
        .route("/metrics", get(health::metrics))
}

/// Authentication routes
fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/refresh", post(auth::refresh_token))
        .route("/verify-email", post(auth::verify_email))
        .route("/forgot-password", post(auth::forgot_password))
        .route("/reset-password", post(auth::reset_password))
        .route("/api-key", post(auth::generate_api_key))
        // Apply rate limiting to auth endpoints
        .layer(middleware::from_fn(rate_limiter::auth_rate_limiter))
}

/// User routes
fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/me", get(user::get_current_user))
        .route("/me", put(user::update_profile))
        .route("/me/stats", get(user::get_user_stats))
        .route("/:user_id", get(user::get_user_by_id))
        .route("/:user_id/stats", get(user::get_user_stats_by_id))
        .route("/me/api-keys", get(user::list_api_keys))
        .route("/me/api-keys/:key_id", delete(user::revoke_api_key))
        // Require authentication for all user routes
        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::auth_middleware,
        ))
}

/// Bounty routes
fn bounty_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Public routes (read-only)
        .route("/", get(bounty::list_bounties))
        .route("/:bounty_id", get(bounty::get_bounty))
        .route("/:bounty_id/stats", get(bounty::get_bounty_stats))
        .route("/active", get(bounty::list_active_bounties))
        .route("/completed", get(bounty::list_completed_bounties))

        // Protected routes (require authentication)
        .route("/", post(bounty::create_bounty))
        .route("/:bounty_id", put(bounty::update_bounty))
        .route("/:bounty_id/cancel", post(bounty::cancel_bounty))
        .route("/:bounty_id/extend", post(bounty::extend_bounty))
        .route("/:bounty_id/claim", post(bounty::claim_reward))

        // Apply authentication to write operations
        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::optional_auth_middleware,
        ))
}

/// Analysis routes
fn analysis_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(analysis::list_analyses))
        .route("/:analysis_id", get(analysis::get_analysis))
        .route("/:analysis_id/details", get(analysis::get_analysis_details))
        .route("/stats", get(analysis::get_analysis_stats))
        .route("/by-bounty/:bounty_id", get(analysis::get_analyses_by_bounty))
        .route("/by-hash/:file_hash", get(analysis::get_analyses_by_hash))

        // Submit analysis (requires authentication)
        .route("/submit", post(analysis::submit_analysis))
        .route("/:analysis_id/dispute", post(analysis::dispute_analysis))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::optional_auth_middleware,
        ))
}

/// Submission routes
fn submission_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(submission::list_submissions))
        .route("/:submission_id", get(submission::get_submission))
        .route("/", post(submission::create_submission))
        .route("/:submission_id/vote", post(submission::vote_on_submission))
        .route("/:submission_id/verify", post(submission::verify_submission))
        .route("/my-submissions", get(submission::get_my_submissions))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::auth_middleware,
        ))
}

/// Wallet routes
fn wallet_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/connect", post(wallet::connect_wallet))
        .route("/disconnect", post(wallet::disconnect_wallet))
        .route("/balance", get(wallet::get_balance))
        .route("/stake", post(wallet::stake_tokens))
        .route("/unstake", post(wallet::unstake_tokens))
        .route("/transactions", get(wallet::get_transactions))
        .route("/transactions/:tx_hash", get(wallet::get_transaction))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::auth_middleware,
        ))
}

/// Reputation routes
fn reputation_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Public routes
        .route("/leaderboard", get(reputation::get_leaderboard))
        .route("/leaderboard/top", get(reputation::get_top_analysts))
        .route("/user/:user_id", get(reputation::get_user_reputation))
        .route("/badges", get(reputation::list_available_badges))

        // Protected routes
        .route("/claim-badge", post(reputation::claim_badge))
        .route("/history", get(reputation::get_reputation_history))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::optional_auth_middleware,
        ))
}

/// Webhook routes
fn webhook_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(webhook::list_webhooks))
        .route("/", post(webhook::create_webhook))
        .route("/:webhook_id", get(webhook::get_webhook))
        .route("/:webhook_id", put(webhook::update_webhook))
        .route("/:webhook_id", delete(webhook::delete_webhook))
        .route("/:webhook_id/test", post(webhook::test_webhook))
        .route("/:webhook_id/deliveries", get(webhook::get_webhook_deliveries))
        .route("/events", get(webhook::list_available_events))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::auth_middleware,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_routes() {
        let router = health_routes();
        // Test that health routes can be created
        // Actual route testing requires integration tests
    }

    #[test]
    fn test_auth_routes() {
        let router = auth_routes();
        // Test that auth routes can be created
    }
}
