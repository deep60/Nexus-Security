use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::channels::{ConnectionMetadata, WebSocketConnection};
use crate::AppState;

/// Upgrade an HTTP connection to a WebSocket. The client identifies itself via
/// `?user_id=<uuid>` (the gateway validates the JWT before proxying here).
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let user_id = params
        .get("user_id")
        .and_then(|v| Uuid::parse_str(v).ok());

    ws.on_upgrade(move |socket| handle_socket(socket, state, user_id))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>, user_id: Option<Uuid>) {
    let Some(user_id) = user_id else {
        warn!("WebSocket connection rejected: missing/invalid user_id");
        return;
    };

    let connection_id = Uuid::new_v4();
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Channel the notification system pushes serialized messages into.
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let channel = state.notification_manager.get_websocket_channel();
    channel
        .register_connection(
            user_id,
            WebSocketConnection {
                id: connection_id,
                user_id,
                sender: tx,
                metadata: ConnectionMetadata::default(),
            },
        )
        .await;

    info!("WebSocket open: user={} conn={}", user_id, connection_id);

    // Forward queued notifications to the socket.
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle inbound frames (ping/close); we don't process app messages yet.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Close(_) => break,
                Message::Ping(_) | Message::Pong(_) => { /* axum auto-pongs */ }
                _ => { /* ignore client text/binary for now */ }
            }
        }
    });

    // When either side ends, tear down the other and unregister.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    channel.unregister_connection(user_id, connection_id).await;
    info!("WebSocket closed: user={} conn={}", user_id, connection_id);
}
