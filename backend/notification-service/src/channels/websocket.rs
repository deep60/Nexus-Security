use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::models::{NotificationChannel, NotificationError, NotificationResult};
use shared::messaging::event_types::{NexusEvent, NotificationPayload};

/// WebSocket notification channel implementation
/// Maintains active WebSocket connections and broadcasts notifications to connected clients
pub struct WebSocketChannel {
    /// Active WebSocket connections mapped by user ID
    connections: Arc<RwLock<HashMap<Uuid, Vec<WebSocketConnection>>>>,
}

impl WebSocketChannel {
    /// Create a new WebSocket channel
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new WebSocket connection for a user
    pub async fn register_connection(
        &self,
        user_id: Uuid,
        connection: WebSocketConnection,
    ) {
        let mut connections = self.connections.write().await;
        connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(connection);

        info!("WebSocket connection registered for user {}", user_id);
    }

    /// Unregister a WebSocket connection
    pub async fn unregister_connection(&self, user_id: Uuid, connection_id: Uuid) {
        let mut connections = self.connections.write().await;

        if let Some(user_connections) = connections.get_mut(&user_id) {
            user_connections.retain(|conn| conn.id != connection_id);

            if user_connections.is_empty() {
                connections.remove(&user_id);
            }

            info!(
                "WebSocket connection {} unregistered for user {}",
                connection_id, user_id
            );
        }
    }

    /// Get the number of active connections for a user
    pub async fn get_connection_count(&self, user_id: Uuid) -> usize {
        let connections = self.connections.read().await;
        connections.get(&user_id).map(|v| v.len()).unwrap_or(0)
    }

    /// Get total number of active connections
    pub async fn get_total_connections(&self) -> usize {
        let connections = self.connections.read().await;
        connections.values().map(|v| v.len()).sum()
    }

    /// Build WebSocket message from notification payload
    fn build_websocket_message(payload: &NotificationPayload) -> WebSocketMessage {
        WebSocketMessage {
            message_id: Uuid::new_v4(),
            notification_id: payload.notification_id,
            event_type: Self::get_event_type(&payload.event),
            title: payload.event.get_title(),
            body: payload.event.get_description(),
            event: payload.event.clone(),
            timestamp: payload.created_at,
            priority: payload.priority.clone(),
        }
    }

    /// Get event type string
    fn get_event_type(event: &NexusEvent) -> String {
        match event {
            NexusEvent::BountyCreated(_) => "bounty.created",
            NexusEvent::BountyUpdated(_) => "bounty.updated",
            NexusEvent::BountyCompleted(_) => "bounty.completed",
            NexusEvent::BountyExpired(_) => "bounty.expired",
            NexusEvent::BountyCancelled(_) => "bounty.cancelled",
            NexusEvent::SubmissionReceived(_) => "submission.received",
            NexusEvent::SubmissionValidated(_) => "submission.validated",
            NexusEvent::SubmissionRejected(_) => "submission.rejected",
            NexusEvent::AnalysisStarted(_) => "analysis.started",
            NexusEvent::AnalysisCompleted(_) => "analysis.completed",
            NexusEvent::AnalysisFailed(_) => "analysis.failed",
            NexusEvent::ReputationUpdated(_) => "reputation.updated",
            NexusEvent::PaymentProcessed(_) => "payment.processed",
            NexusEvent::PaymentFailed(_) => "payment.failed",
            NexusEvent::StakeSlashed(_) => "stake.slashed",
            NexusEvent::UserRegistered(_) => "user.registered",
            NexusEvent::UserVerified(_) => "user.verified",
            NexusEvent::EngineRegistered(_) => "engine.registered",
            NexusEvent::DisputeCreated(_) => "dispute.created",
            NexusEvent::DisputeResolved(_) => "dispute.resolved",
            NexusEvent::SystemAlert(_) => "system.alert",
        }
        .to_string()
    }

    /// Broadcast message to all connections for a user
    async fn broadcast_to_user(
        &self,
        user_id: Uuid,
        message: &WebSocketMessage,
    ) -> NotificationResult<usize> {
        let connections = self.connections.read().await;

        let user_connections = connections.get(&user_id);
        if user_connections.is_none() {
            return Err(NotificationError::SendError(
                "No active WebSocket connections for user".to_string(),
            ));
        }

        let user_connections = user_connections.unwrap();
        let message_json = serde_json::to_string(message)
            .map_err(|e| NotificationError::SerializationError(e.to_string()))?;

        let mut successful_sends = 0;
        let mut failed_connections = Vec::new();

        for conn in user_connections {
            match conn.sender.send(message_json.clone()).await {
                Ok(_) => {
                    successful_sends += 1;
                }
                Err(e) => {
                    error!(
                        "Failed to send WebSocket message to connection {}: {}",
                        conn.id, e
                    );
                    failed_connections.push(conn.id);
                }
            }
        }

        // Clean up failed connections
        if !failed_connections.is_empty() {
            drop(connections);
            for conn_id in failed_connections {
                self.unregister_connection(user_id, conn_id).await;
            }
        }

        if successful_sends > 0 {
            Ok(successful_sends)
        } else {
            Err(NotificationError::SendError(
                "Failed to send to any WebSocket connections".to_string(),
            ))
        }
    }

    /// Cleanup stale connections periodically
    pub async fn cleanup_stale_connections(&self) {
        let mut connections = self.connections.write().await;
        let mut users_to_remove = Vec::new();

        for (user_id, user_connections) in connections.iter_mut() {
            user_connections.retain(|conn| !conn.sender.is_closed());

            if user_connections.is_empty() {
                users_to_remove.push(*user_id);
            }
        }

        for user_id in users_to_remove {
            connections.remove(&user_id);
        }
    }
}

impl Default for WebSocketChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationChannel for WebSocketChannel {
    async fn send(
        &self,
        payload: &NotificationPayload,
        recipient: &str,
    ) -> NotificationResult<()> {
        // Parse user_id from recipient
        let user_id = Uuid::parse_str(recipient)
            .map_err(|e| NotificationError::ValidationError(format!("Invalid user ID: {}", e)))?;

        info!(
            "Sending WebSocket notification to user {} for event: {}",
            user_id,
            payload.event.get_title()
        );

        let message = Self::build_websocket_message(payload);

        let sent_count = self.broadcast_to_user(user_id, &message).await?;

        info!(
            "WebSocket notification sent to {} connections for user {}",
            sent_count, user_id
        );

        Ok(())
    }

    fn channel_type(&self) -> &'static str {
        "websocket"
    }

    async fn validate_recipient(&self, recipient: &str) -> NotificationResult<bool> {
        Uuid::parse_str(recipient)
            .map(|_| true)
            .map_err(|e| NotificationError::ValidationError(format!("Invalid user ID: {}", e)))
    }
}

/// WebSocket connection information
#[derive(Clone)]
pub struct WebSocketConnection {
    /// Unique connection ID
    pub id: Uuid,
    /// User ID
    pub user_id: Uuid,
    /// Channel to send messages to this connection
    pub sender: mpsc::UnboundedSender<String>,
    /// Connection metadata
    pub metadata: ConnectionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetadata {
    /// IP address of the client
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Connection timestamp
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// Device type (web, mobile, etc.)
    pub device_type: Option<String>,
}

impl Default for ConnectionMetadata {
    fn default() -> Self {
        Self {
            ip_address: None,
            user_agent: None,
            connected_at: chrono::Utc::now(),
            device_type: None,
        }
    }
}

/// WebSocket message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// Unique message ID
    pub message_id: Uuid,
    /// Original notification ID
    pub notification_id: Uuid,
    /// Event type
    pub event_type: String,
    /// Message title
    pub title: String,
    /// Message body
    pub body: String,
    /// Full event data
    pub event: NexusEvent,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Priority
    pub priority: shared::messaging::event_types::NotificationPriority,
}

/// WebSocket control messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketControlMessage {
    /// Ping message to keep connection alive
    Ping { timestamp: chrono::DateTime<chrono::Utc> },
    /// Pong response
    Pong { timestamp: chrono::DateTime<chrono::Utc> },
    /// Subscribe to specific event types
    Subscribe { events: Vec<String> },
    /// Unsubscribe from event types
    Unsubscribe { events: Vec<String> },
    /// Acknowledgment of received notification
    Ack { notification_id: Uuid },
    /// Error message
    Error { code: String, message: String },
}

/// WebSocket subscription preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketSubscription {
    pub user_id: Uuid,
    pub connection_id: Uuid,
    pub subscribed_events: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use shared::messaging::event_types::{BountyCreatedEvent, NotificationPriority};

    #[tokio::test]
    async fn test_register_and_unregister_connection() {
        let channel = WebSocketChannel::new();
        let user_id = Uuid::new_v4();
        let connection_id = Uuid::new_v4();

        let (tx, _rx) = mpsc::unbounded_channel();
        let connection = WebSocketConnection {
            id: connection_id,
            user_id,
            sender: tx,
            metadata: ConnectionMetadata::default(),
        };

        // Register connection
        channel.register_connection(user_id, connection).await;
        assert_eq!(channel.get_connection_count(user_id).await, 1);

        // Unregister connection
        channel.unregister_connection(user_id, connection_id).await;
        assert_eq!(channel.get_connection_count(user_id).await, 0);
    }

    #[tokio::test]
    async fn test_validate_recipient() {
        let channel = WebSocketChannel::new();

        let valid_uuid = Uuid::new_v4();
        assert!(channel
            .validate_recipient(&valid_uuid.to_string())
            .await
            .is_ok());

        assert!(channel.validate_recipient("not-a-uuid").await.is_err());
    }

    #[test]
    fn test_get_event_type() {
        let event = NexusEvent::BountyCreated(BountyCreatedEvent {
            bounty_id: Uuid::new_v4(),
            creator_id: Uuid::new_v4(),
            title: "Test".to_string(),
            description: "Test bounty".to_string(),
            reward_amount: 1000,
            stake_requirement: 100,
            expires_at: Utc::now(),
            target_type: "file".to_string(),
            tags: vec![],
            created_at: Utc::now(),
        });

        assert_eq!(WebSocketChannel::get_event_type(&event), "bounty.created");
    }

    #[tokio::test]
    async fn test_get_total_connections() {
        let channel = WebSocketChannel::new();

        assert_eq!(channel.get_total_connections().await, 0);

        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        let (tx1, _rx1) = mpsc::unbounded_channel();
        let (tx2, _rx2) = mpsc::unbounded_channel();

        channel
            .register_connection(
                user1,
                WebSocketConnection {
                    id: Uuid::new_v4(),
                    user_id: user1,
                    sender: tx1,
                    metadata: ConnectionMetadata::default(),
                },
            )
            .await;

        channel
            .register_connection(
                user2,
                WebSocketConnection {
                    id: Uuid::new_v4(),
                    user_id: user2,
                    sender: tx2,
                    metadata: ConnectionMetadata::default(),
                },
            )
            .await;

        assert_eq!(channel.get_total_connections().await, 2);
    }
}
