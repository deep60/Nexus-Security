//! WebSocket proxy handler.
//!
//! The notification-service owns the real WebSocket implementation.
//! This handler accepts the `/ws` upgrade on the API gateway and relays
//! frames bidirectionally to the notification-service so that the gateway
//! remains the single point of entry for all frontend traffic.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tracing::{error, info};

use crate::AppState;

/// Accept a WebSocket upgrade and proxy all frames to the notification-service.
pub async fn ws_proxy(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    let notification_ws_url = build_ws_url(&state.config.services.notification_service_url);
    ws.on_upgrade(move |socket| proxy_ws(socket, notification_ws_url))
}

/// Rewrite an HTTP URL into a WebSocket URL, appending `/ws`.
fn build_ws_url(http_url: &str) -> String {
    let base = http_url
        .replace("https://", "wss://")
        .replace("http://", "ws://");
    format!("{}/ws", base.trim_end_matches('/'))
}

/// Bidirectional frame relay between the client socket and the upstream
/// notification-service socket.
async fn proxy_ws(client_socket: WebSocket, upstream_url: String) {
    // Connect to the upstream (notification-service)
    let upstream = match connect_async(&upstream_url).await {
        Ok((stream, _)) => stream,
        Err(e) => {
            error!(
                "Failed to connect to notification-service at {}: {}",
                upstream_url, e
            );
            return;
        }
    };

    info!("WebSocket proxy established → {}", upstream_url);

    let (mut client_tx, mut client_rx) = client_socket.split();
    let (mut upstream_tx, mut upstream_rx) = upstream.split();

    // client → upstream
    let client_to_upstream = async {
        while let Some(Ok(msg)) = client_rx.next().await {
            let tung_msg = axum_to_tungstenite(msg);
            if upstream_tx.send(tung_msg).await.is_err() {
                break;
            }
        }
    };

    // upstream → client
    let upstream_to_client = async {
        while let Some(Ok(msg)) = upstream_rx.next().await {
            let axum_msg = tungstenite_to_axum(msg);
            if client_tx.send(axum_msg).await.is_err() {
                break;
            }
        }
    };

    tokio::select! {
        _ = client_to_upstream => {},
        _ = upstream_to_client => {},
    }

    info!("WebSocket proxy connection closed");
}

// ─── Frame conversion helpers ───────────────────────────────────────

fn axum_to_tungstenite(msg: Message) -> tokio_tungstenite::tungstenite::Message {
    use tokio_tungstenite::tungstenite::Message as TM;
    match msg {
        // axum 0.8 and tungstenite 0.24 each have their own Utf8Bytes type;
        // bridge via &str. Binary/Ping/Pong share bytes::Bytes so pass through.
        // axum 0.8 uses Bytes / its own Utf8Bytes; tungstenite 0.24 uses Vec<u8>
        // for Binary/Ping/Pong and its own Utf8Bytes for Text. Bridge explicitly.
        Message::Text(t) => TM::Text(t.to_string()),
        Message::Binary(b) => TM::Binary(b.to_vec()),
        Message::Ping(p) => TM::Ping(p.to_vec()),
        Message::Pong(p) => TM::Pong(p.to_vec()),
        Message::Close(c) => {
            TM::Close(
                c.map(|f| tokio_tungstenite::tungstenite::protocol::CloseFrame {
                    code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::from(
                        f.code,
                    ),
                    reason: f.reason.to_string().into(),
                }),
            )
        }
    }
}

fn tungstenite_to_axum(msg: tokio_tungstenite::tungstenite::Message) -> Message {
    use tokio_tungstenite::tungstenite::Message as TM;
    match msg {
        TM::Text(t) => Message::Text(t.to_string().into()),
        TM::Binary(b) => Message::Binary(b.into()),
        TM::Ping(p) => Message::Ping(p.into()),
        TM::Pong(p) => Message::Pong(p.into()),
        TM::Close(c) => Message::Close(c.map(|f| axum::extract::ws::CloseFrame {
            code: f.code.into(),
            reason: f.reason.to_string().into(),
        })),
        TM::Frame(_) => Message::Binary(axum::body::Bytes::new()),
    }
}
