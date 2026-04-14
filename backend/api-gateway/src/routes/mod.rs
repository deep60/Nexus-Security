pub mod v1;

use axum::{routing::get, Router};

use crate::AppState;

/// Create the main router with API v1 and the root-level /ws proxy.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // WebSocket proxy — lives at the root so the frontend can connect to /ws
        .route("/ws", get(crate::handlers::websocket::ws_proxy))
        .with_state(state.clone())
        .nest("/api/v1", v1::create_routes(state.clone()))
        .nest("/api", v1::create_routes(state))
}
