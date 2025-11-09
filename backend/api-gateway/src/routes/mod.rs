pub mod v1;
pub mod v2;

use axum::Router;
use std::sync::Arc;

use crate::AppState;

/// Create the main router with all API versions
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/v1", v1::create_routes(state.clone()))
        .nest("/api/v2", v2::create_routes(state.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        // Test that router can be created
        // Note: This requires a mock AppState which should be implemented
        // For now, this is a placeholder test structure
    }
}
