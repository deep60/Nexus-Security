use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade}},
    response::Response,
};
use std::sync::Arc;
use crate::AppState;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(_state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(_socket: WebSocket) {
    // WebSocket connection handling
}
