use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::{
    handlers::{
        analysis, auth, bounty, health, reputation, submission, user, wallet, webhook,
    },
    AppState,
};

/// Create all routes for API v1
/// Note: Global middleware (CORS, metrics, logging, auth) is applied in main.rs
pub fn create_routes(state: AppState) -> Router {
    Router::new()
        // Health check routes (public, no auth required)
        .nest("/health", health_routes())

        // Authentication routes (public)
        .nest("/auth", auth_routes())

        // User routes (requires authentication)
        .nest("/users", user_routes())

        // Bounty routes (mixed auth)
        .nest("/bounties", bounty_routes())

        // Analysis routes (mixed auth)
        .nest("/analysis", analysis_routes())

        // Submission routes (requires authentication)
        .nest("/submissions", submission_routes())

        // Wallet routes (requires authentication)
        .nest("/wallet", wallet_routes())

        // Reputation routes (public read, authenticated write)
        .nest("/reputation", reputation_routes())

        // Webhook routes (requires authentication)
        .nest("/webhooks", webhook_routes())

        .with_state(state)
}

/// Health check routes
fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .route("/live", get(health::liveness_check))
        .route("/metrics", get(health::metrics))
}

/// Authentication routes
fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/refresh", post(auth::refresh_token))
        .route("/verify", post(auth::verify_token))
        .route("/verify-email", post(auth::verify_email))
        .route("/forgot-password", post(auth::forgot_password))
        .route("/reset-password", post(auth::reset_password))
        .route("/api-key", post(auth::generate_api_key))
        .route("/wallet/connect", post(auth::collect_wallet))
        .route("/wallet/disconnect", post(auth::disconnect_wallet))
}

/// User routes
fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(user::get_current_user))
        .route("/me", put(user::update_profile))
        .route("/me/stats", get(user::get_user_stats))
        .route("/:user_id", get(user::get_user_by_id))
        .route("/:user_id/stats", get(user::get_user_stats_by_id))
        .route("/me/api-keys", get(user::list_api_keys))
        .route("/me/api-keys/:key_id", delete(user::revoke_api_key))
}

/// Bounty routes
fn bounty_routes() -> Router<AppState> {
    Router::new()
        // Public routes (read-only)
        .route("/", get(bounty::list_bounties))
        .route("/:bounty_id", get(bounty::get_bounty))
        .route("/:bounty_id/stats", get(bounty::get_bounty_stats))
        .route("/active", get(bounty::list_active_bounties))
        .route("/completed", get(bounty::list_completed_bounties))
        // Protected routes
        .route("/", post(bounty::create_bounty))
        .route("/:bounty_id", put(bounty::update_bounty))
        .route("/:bounty_id/cancel", post(bounty::cancel_bounty))
        .route("/:bounty_id/extend", post(bounty::extend_bounty))
        .route("/:bounty_id/claim", post(bounty::claim_reward))
        .route("/:bounty_id/submit", post(bounty::submit_analysis))
        .route("/:bounty_id/finalize", put(bounty::finalize_bounty))
}

/// Analysis routes
fn analysis_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(analysis::list_analyses))
        .route("/:analysis_id", get(analysis::get_analysis))
        .route("/:analysis_id/details", get(analysis::get_analysis_details))
        .route("/stats", get(analysis::get_analysis_stats))
        .route("/by-bounty/:bounty_id", get(analysis::get_analyses_by_bounty))
        .route("/by-hash/:file_hash", get(analysis::get_analyses_by_hash))
        .route("/submit", post(analysis::submit_analysis))
        .route("/:analysis_id/dispute", post(analysis::dispute_analysis))
}

/// Submission routes
fn submission_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(submission::list_submissions))
        .route("/:submission_id", get(submission::get_submission))
        .route("/", post(submission::create_submission))
        .route("/:submission_id/vote", post(submission::vote_on_submission))
        .route("/:submission_id/verify", post(submission::verify_submission))
        .route("/my-submissions", get(submission::get_my_submissions))
}

/// Wallet routes
fn wallet_routes() -> Router<AppState> {
    Router::new()
        .route("/connect", post(wallet::connect_wallet))
        .route("/disconnect", post(wallet::disconnect_wallet))
        .route("/balance", get(wallet::get_balance))
        .route("/balance/:address", get(wallet::get_balance_by_address))
        .route("/stake", post(wallet::stake_tokens))
        .route("/unstake/:bounty_id", post(wallet::unstake_tokens))
        .route("/transactions", get(wallet::get_transactions))
        .route("/claim-rewards", post(wallet::claim_rewards))
}

/// Reputation routes
fn reputation_routes() -> Router<AppState> {
    Router::new()
        // Public routes
        .route("/leaderboard", get(reputation::get_leaderboard))
        .route("/leaderboard/top", get(reputation::get_top_analysts))
        .route("/user/:user_id", get(reputation::get_user_reputation))
        .route("/badges", get(reputation::list_available_badges))
        .route("/history/:user_id", get(reputation::get_reputation_history))
        // Protected routes
        .route("/claim-badge", post(reputation::claim_badge))
}

/// Webhook routes
fn webhook_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(webhook::list_webhooks))
        .route("/", post(webhook::create_webhook))
        .route("/:webhook_id", get(webhook::get_webhook))
        .route("/:webhook_id", put(webhook::update_webhook))
        .route("/:webhook_id", delete(webhook::delete_webhook))
        .route("/:webhook_id/test", post(webhook::test_webhook))
        .route("/:webhook_id/deliveries", get(webhook::get_webhook_deliveries))
        .route("/events", get(webhook::list_available_events))
}
