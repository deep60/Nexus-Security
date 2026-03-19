pub mod v1;

use axum::Router;

use crate::AppState;

/// Create the main router with API v1
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", v1::create_routes(state))
}
