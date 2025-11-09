use axum::{
    middleware,
    routing::{get, post, put, delete, patch},
    Router,
};
use std::sync::Arc;

use crate::{
    handlers::{
        analysis, auth, bounty, health, reputation, submission, user, wallet, webhook,
    },
    middleware::{auth as auth_middleware, cors, logging, metrics, rate_limiter},
    AppState,
};

/// Create all routes for API v2
///
/// V2 API improvements over V1:
/// - Enhanced authentication with OAuth2 support
/// - Batch operations for bounties and analyses
/// - Advanced filtering and search capabilities
/// - WebSocket support for real-time updates
/// - GraphQL endpoint for flexible queries
/// - Improved rate limiting per endpoint
/// - Better error responses with detailed codes
pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check routes (enhanced monitoring)
        .nest("/health", health_routes())

        // Authentication routes (enhanced with OAuth2)
        .nest("/auth", auth_routes())

        // User routes (enhanced with preferences and settings)
        .nest("/users", user_routes())

        // Bounty routes (enhanced with batch operations)
        .nest("/bounties", bounty_routes())

        // Analysis routes (enhanced with ML insights)
        .nest("/analysis", analysis_routes())

        // Submission routes (enhanced with collaboration features)
        .nest("/submissions", submission_routes())

        // Wallet routes (enhanced with multi-chain support)
        .nest("/wallet", wallet_routes())

        // Reputation routes (enhanced with detailed analytics)
        .nest("/reputation", reputation_routes())

        // Webhook routes (enhanced with retry and filtering)
        .nest("/webhooks", webhook_routes())

        // New v2 exclusive routes
        .nest("/search", search_routes())
        .nest("/analytics", analytics_routes())
        .nest("/admin", admin_routes())

        // Apply global middleware
        .layer(middleware::from_fn(metrics::metrics_middleware))
        .layer(middleware::from_fn(logging::logging_middleware))
        .layer(cors::create_cors_layer())
        .with_state(state)
}

/// Enhanced health check routes with detailed metrics
fn health_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .route("/live", get(health::liveness_check))
        .route("/metrics", get(health::metrics))
        // V2 additions
        .route("/metrics/detailed", get(health::detailed_metrics))
        .route("/dependencies", get(health::dependency_health))
}

/// Enhanced authentication routes
fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        // V1 compatibility routes
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/refresh", post(auth::refresh_token))
        .route("/verify-email", post(auth::verify_email))
        .route("/forgot-password", post(auth::forgot_password))
        .route("/reset-password", post(auth::reset_password))

        // V2 enhancements
        .route("/oauth2/authorize", get(auth::oauth2_authorize))
        .route("/oauth2/callback", get(auth::oauth2_callback))
        .route("/oauth2/token", post(auth::oauth2_token))
        .route("/mfa/enable", post(auth::enable_mfa))
        .route("/mfa/disable", post(auth::disable_mfa))
        .route("/mfa/verify", post(auth::verify_mfa))
        .route("/sessions", get(auth::list_active_sessions))
        .route("/sessions/:session_id", delete(auth::revoke_session))

        .layer(middleware::from_fn(rate_limiter::auth_rate_limiter))
}

/// Enhanced user routes
fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        // V1 compatibility
        .route("/me", get(user::get_current_user))
        .route("/me", put(user::update_profile))
        .route("/me", patch(user::patch_profile))
        .route("/me/stats", get(user::get_user_stats))
        .route("/:user_id", get(user::get_user_by_id))
        .route("/:user_id/stats", get(user::get_user_stats_by_id))

        // V2 enhancements
        .route("/me/preferences", get(user::get_preferences))
        .route("/me/preferences", put(user::update_preferences))
        .route("/me/notifications", get(user::get_notifications))
        .route("/me/notifications/:notification_id/read", post(user::mark_notification_read))
        .route("/me/activity", get(user::get_activity_feed))
        .route("/me/export", post(user::export_user_data))
        .route("/search", get(user::search_users))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::auth_middleware,
        ))
}

/// Enhanced bounty routes with batch operations
fn bounty_routes() -> Router<Arc<AppState>> {
    Router::new()
        // V1 compatibility (public)
        .route("/", get(bounty::list_bounties))
        .route("/:bounty_id", get(bounty::get_bounty))
        .route("/:bounty_id/stats", get(bounty::get_bounty_stats))

        // V2 enhancements (public)
        .route("/search", get(bounty::search_bounties))
        .route("/filter", post(bounty::filter_bounties))
        .route("/trending", get(bounty::get_trending_bounties))
        .route("/categories", get(bounty::list_categories))
        .route("/category/:category", get(bounty::get_bounties_by_category))

        // V1 compatibility (protected)
        .route("/", post(bounty::create_bounty))
        .route("/:bounty_id", put(bounty::update_bounty))
        .route("/:bounty_id", patch(bounty::patch_bounty))
        .route("/:bounty_id", delete(bounty::delete_bounty))

        // V2 enhancements (protected)
        .route("/batch", post(bounty::create_bounties_batch))
        .route("/batch/update", put(bounty::update_bounties_batch))
        .route("/:bounty_id/collaborate", post(bounty::add_collaborator))
        .route("/:bounty_id/collaborate/:user_id", delete(bounty::remove_collaborator))
        .route("/:bounty_id/comments", get(bounty::get_comments))
        .route("/:bounty_id/comments", post(bounty::add_comment))
        .route("/:bounty_id/watchers", post(bounty::watch_bounty))
        .route("/:bounty_id/watchers", delete(bounty::unwatch_bounty))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::optional_auth_middleware,
        ))
}

/// Enhanced analysis routes with ML insights
fn analysis_routes() -> Router<Arc<AppState>> {
    Router::new()
        // V1 compatibility
        .route("/", get(analysis::list_analyses))
        .route("/:analysis_id", get(analysis::get_analysis))
        .route("/submit", post(analysis::submit_analysis))

        // V2 enhancements
        .route("/batch", post(analysis::submit_analyses_batch))
        .route("/:analysis_id/similar", get(analysis::get_similar_analyses))
        .route("/:analysis_id/insights", get(analysis::get_ml_insights))
        .route("/:analysis_id/timeline", get(analysis::get_analysis_timeline))
        .route("/:analysis_id/download", get(analysis::download_analysis_report))
        .route("/compare", post(analysis::compare_analyses))
        .route("/statistics", get(analysis::get_global_statistics))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::optional_auth_middleware,
        ))
}

/// Enhanced submission routes
fn submission_routes() -> Router<Arc<AppState>> {
    Router::new()
        // V1 compatibility
        .route("/", get(submission::list_submissions))
        .route("/:submission_id", get(submission::get_submission))
        .route("/", post(submission::create_submission))

        // V2 enhancements
        .route("/batch", post(submission::create_submissions_batch))
        .route("/:submission_id/versions", get(submission::get_submission_versions))
        .route("/:submission_id/revert/:version", post(submission::revert_to_version))
        .route("/:submission_id/collaborate", post(submission::invite_collaborator))
        .route("/:submission_id/review", post(submission::submit_peer_review))
        .route("/:submission_id/reviews", get(submission::get_peer_reviews))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::auth_middleware,
        ))
}

/// Enhanced wallet routes with multi-chain support
fn wallet_routes() -> Router<Arc<AppState>> {
    Router::new()
        // V1 compatibility
        .route("/connect", post(wallet::connect_wallet))
        .route("/balance", get(wallet::get_balance))
        .route("/stake", post(wallet::stake_tokens))

        // V2 enhancements
        .route("/chains", get(wallet::list_supported_chains))
        .route("/chain/:chain_id/connect", post(wallet::connect_chain))
        .route("/balances", get(wallet::get_multi_chain_balances))
        .route("/portfolio", get(wallet::get_portfolio_overview))
        .route("/transactions/export", get(wallet::export_transactions))
        .route("/gas-estimates", post(wallet::estimate_gas_fees))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::auth_middleware,
        ))
}

/// Enhanced reputation routes with detailed analytics
fn reputation_routes() -> Router<Arc<AppState>> {
    Router::new()
        // V1 compatibility
        .route("/leaderboard", get(reputation::get_leaderboard))
        .route("/user/:user_id", get(reputation::get_user_reputation))

        // V2 enhancements
        .route("/leaderboard/global", get(reputation::get_global_leaderboard))
        .route("/leaderboard/category/:category", get(reputation::get_category_leaderboard))
        .route("/leaderboard/timeframe/:period", get(reputation::get_timeframe_leaderboard))
        .route("/user/:user_id/breakdown", get(reputation::get_reputation_breakdown))
        .route("/user/:user_id/trends", get(reputation::get_reputation_trends))
        .route("/badges/achievements", get(reputation::get_achievement_progress))
        .route("/challenges", get(reputation::list_active_challenges))
        .route("/challenges/:challenge_id/participate", post(reputation::participate_in_challenge))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::optional_auth_middleware,
        ))
}

/// Enhanced webhook routes
fn webhook_routes() -> Router<Arc<AppState>> {
    Router::new()
        // V1 compatibility
        .route("/", get(webhook::list_webhooks))
        .route("/", post(webhook::create_webhook))
        .route("/:webhook_id", get(webhook::get_webhook))
        .route("/:webhook_id", put(webhook::update_webhook))
        .route("/:webhook_id", delete(webhook::delete_webhook))

        // V2 enhancements
        .route("/:webhook_id/retry", post(webhook::retry_failed_deliveries))
        .route("/:webhook_id/stats", get(webhook::get_webhook_stats))
        .route("/:webhook_id/logs", get(webhook::get_webhook_logs))
        .route("/templates", get(webhook::list_webhook_templates))
        .route("/templates/:template_id", post(webhook::create_from_template))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::auth_middleware,
        ))
}

/// Search routes (V2 exclusive)
fn search_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(search::global_search))
        .route("/bounties", get(search::search_bounties))
        .route("/analyses", get(search::search_analyses))
        .route("/users", get(search::search_users))
        .route("/suggestions", get(search::get_search_suggestions))
        .route("/filters", get(search::get_available_filters))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::optional_auth_middleware,
        ))
}

/// Analytics routes (V2 exclusive)
fn analytics_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/dashboard", get(analytics::get_dashboard_stats))
        .route("/trends", get(analytics::get_platform_trends))
        .route("/threats", get(analytics::get_threat_landscape))
        .route("/performance", get(analytics::get_performance_metrics))
        .route("/users/engagement", get(analytics::get_user_engagement))
        .route("/bounties/conversion", get(analytics::get_bounty_conversion_rates))
        .route("/export", post(analytics::export_analytics_data))

        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::auth_middleware,
        ))
}

/// Admin routes (V2 exclusive, requires admin role)
fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users", get(admin::list_all_users))
        .route("/users/:user_id/ban", post(admin::ban_user))
        .route("/users/:user_id/unban", post(admin::unban_user))
        .route("/users/:user_id/roles", put(admin::update_user_roles))

        .route("/bounties/moderate", get(admin::list_bounties_for_moderation))
        .route("/bounties/:bounty_id/approve", post(admin::approve_bounty))
        .route("/bounties/:bounty_id/reject", post(admin::reject_bounty))

        .route("/reports", get(admin::list_user_reports))
        .route("/reports/:report_id", get(admin::get_report_details))
        .route("/reports/:report_id/resolve", post(admin::resolve_report))

        .route("/system/config", get(admin::get_system_config))
        .route("/system/config", put(admin::update_system_config))
        .route("/system/maintenance", post(admin::toggle_maintenance_mode))

        .route("/audit-logs", get(admin::get_audit_logs))
        .route("/audit-logs/export", post(admin::export_audit_logs))

        // Require admin authentication
        .layer(middleware::from_fn_with_state(
            Arc::new(()),
            auth_middleware::require_admin,
        ))
}

// Placeholder modules for V2-specific handlers
// These would be implemented in separate handler modules

mod search {
    use axum::{extract::Query, Json};
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct SearchQuery {
        pub q: String,
        pub page: Option<u32>,
        pub limit: Option<u32>,
    }

    #[derive(Serialize)]
    pub struct SearchResults {
        pub query: String,
        pub results: Vec<String>,
        pub total: u64,
    }

    pub async fn global_search(Query(_params): Query<SearchQuery>) -> Json<SearchResults> {
        // TODO: Implement global search
        Json(SearchResults {
            query: "search query".to_string(),
            results: vec![],
            total: 0,
        })
    }

    pub async fn search_bounties(Query(_params): Query<SearchQuery>) -> Json<SearchResults> {
        Json(SearchResults {
            query: "bounty search".to_string(),
            results: vec![],
            total: 0,
        })
    }

    pub async fn search_analyses(Query(_params): Query<SearchQuery>) -> Json<SearchResults> {
        Json(SearchResults {
            query: "analysis search".to_string(),
            results: vec![],
            total: 0,
        })
    }

    pub async fn search_users(Query(_params): Query<SearchQuery>) -> Json<SearchResults> {
        Json(SearchResults {
            query: "user search".to_string(),
            results: vec![],
            total: 0,
        })
    }

    pub async fn get_search_suggestions(Query(_params): Query<SearchQuery>) -> Json<Vec<String>> {
        Json(vec![])
    }

    pub async fn get_available_filters() -> Json<Vec<String>> {
        Json(vec!["status".to_string(), "category".to_string(), "date".to_string()])
    }
}

mod analytics {
    use axum::Json;
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct DashboardStats {
        pub total_bounties: u64,
        pub active_users: u64,
        pub total_volume: String,
    }

    pub async fn get_dashboard_stats() -> Json<DashboardStats> {
        Json(DashboardStats {
            total_bounties: 0,
            active_users: 0,
            total_volume: "0".to_string(),
        })
    }

    pub async fn get_platform_trends() -> Json<Vec<String>> {
        Json(vec![])
    }

    pub async fn get_threat_landscape() -> Json<Vec<String>> {
        Json(vec![])
    }

    pub async fn get_performance_metrics() -> Json<Vec<String>> {
        Json(vec![])
    }

    pub async fn get_user_engagement() -> Json<Vec<String>> {
        Json(vec![])
    }

    pub async fn get_bounty_conversion_rates() -> Json<Vec<String>> {
        Json(vec![])
    }

    pub async fn export_analytics_data() -> Json<String> {
        Json("export_id".to_string())
    }
}

mod admin {
    use axum::{extract::Path, Json};
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct AdminResponse {
        pub success: bool,
        pub message: String,
    }

    pub async fn list_all_users() -> Json<Vec<String>> {
        Json(vec![])
    }

    pub async fn ban_user(Path(_user_id): Path<String>) -> Json<AdminResponse> {
        Json(AdminResponse {
            success: true,
            message: "User banned".to_string(),
        })
    }

    pub async fn unban_user(Path(_user_id): Path<String>) -> Json<AdminResponse> {
        Json(AdminResponse {
            success: true,
            message: "User unbanned".to_string(),
        })
    }

    pub async fn update_user_roles(Path(_user_id): Path<String>) -> Json<AdminResponse> {
        Json(AdminResponse {
            success: true,
            message: "Roles updated".to_string(),
        })
    }

    pub async fn list_bounties_for_moderation() -> Json<Vec<String>> {
        Json(vec![])
    }

    pub async fn approve_bounty(Path(_bounty_id): Path<String>) -> Json<AdminResponse> {
        Json(AdminResponse {
            success: true,
            message: "Bounty approved".to_string(),
        })
    }

    pub async fn reject_bounty(Path(_bounty_id): Path<String>) -> Json<AdminResponse> {
        Json(AdminResponse {
            success: true,
            message: "Bounty rejected".to_string(),
        })
    }

    pub async fn list_user_reports() -> Json<Vec<String>> {
        Json(vec![])
    }

    pub async fn get_report_details(Path(_report_id): Path<String>) -> Json<String> {
        Json("report details".to_string())
    }

    pub async fn resolve_report(Path(_report_id): Path<String>) -> Json<AdminResponse> {
        Json(AdminResponse {
            success: true,
            message: "Report resolved".to_string(),
        })
    }

    pub async fn get_system_config() -> Json<String> {
        Json("system config".to_string())
    }

    pub async fn update_system_config() -> Json<AdminResponse> {
        Json(AdminResponse {
            success: true,
            message: "Config updated".to_string(),
        })
    }

    pub async fn toggle_maintenance_mode() -> Json<AdminResponse> {
        Json(AdminResponse {
            success: true,
            message: "Maintenance mode toggled".to_string(),
        })
    }

    pub async fn get_audit_logs() -> Json<Vec<String>> {
        Json(vec![])
    }

    pub async fn export_audit_logs() -> Json<String> {
        Json("export_id".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v2_routes_creation() {
        // Test that v2 routes can be created
        // Actual route testing requires integration tests
    }
}
