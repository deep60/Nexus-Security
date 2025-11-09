use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::redis::RedisService;

/// Event bus for application-wide event publishing and subscription
/// Supports both in-memory (broadcast channels) and distributed (Redis pub/sub) events
#[derive(Clone)]
pub struct EventBus {
    redis: Arc<RwLock<RedisService>>,
    local_channels: Arc<RwLock<HashMap<EventType, broadcast::Sender<Event>>>>,
    config: EventBusConfig,
    stats: Arc<RwLock<EventStats>>,
}

/// Event bus configuration
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    pub enable_redis_pubsub: bool,
    pub enable_local_broadcast: bool,
    pub channel_buffer_size: usize,
    pub enable_event_history: bool,
    pub history_retention_hours: u64,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            enable_redis_pubsub: true,
            enable_local_broadcast: true,
            channel_buffer_size: 1000,
            enable_event_history: true,
            history_retention_hours: 24,
        }
    }
}

/// Event statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventStats {
    pub total_published: u64,
    pub total_delivered: u64,
    pub total_failed: u64,
    pub events_by_type: HashMap<String, u64>,
}

/// Event types in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // User events
    UserRegistered,
    UserLoggedIn,
    UserLoggedOut,
    UserProfileUpdated,
    UserVerified,

    // Bounty events
    BountyCreated,
    BountyUpdated,
    BountyCompleted,
    BountyExpired,
    BountyCancelled,

    // Analysis events
    AnalysisSubmitted,
    AnalysisStarted,
    AnalysisCompleted,
    AnalysisFailed,
    ConsensusReached,

    // Blockchain events
    TransactionSubmitted,
    TransactionConfirmed,
    TransactionFailed,
    TokensStaked,
    RewardClaimed,

    // Reputation events
    ReputationUpdated,
    BadgeEarned,
    LeaderboardChanged,

    // System events
    SystemHealthCheck,
    ServiceStarted,
    ServiceStopped,
    ErrorOccurred,

    // Webhook events
    WebhookTriggered,
    WebhookDelivered,
    WebhookFailed,

    // Custom event
    Custom(String),
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::UserRegistered => "user_registered",
            Self::UserLoggedIn => "user_logged_in",
            Self::UserLoggedOut => "user_logged_out",
            Self::UserProfileUpdated => "user_profile_updated",
            Self::UserVerified => "user_verified",
            Self::BountyCreated => "bounty_created",
            Self::BountyUpdated => "bounty_updated",
            Self::BountyCompleted => "bounty_completed",
            Self::BountyExpired => "bounty_expired",
            Self::BountyCancelled => "bounty_cancelled",
            Self::AnalysisSubmitted => "analysis_submitted",
            Self::AnalysisStarted => "analysis_started",
            Self::AnalysisCompleted => "analysis_completed",
            Self::AnalysisFailed => "analysis_failed",
            Self::ConsensusReached => "consensus_reached",
            Self::TransactionSubmitted => "transaction_submitted",
            Self::TransactionConfirmed => "transaction_confirmed",
            Self::TransactionFailed => "transaction_failed",
            Self::TokensStaked => "tokens_staked",
            Self::RewardClaimed => "reward_claimed",
            Self::ReputationUpdated => "reputation_updated",
            Self::BadgeEarned => "badge_earned",
            Self::LeaderboardChanged => "leaderboard_changed",
            Self::SystemHealthCheck => "system_health_check",
            Self::ServiceStarted => "service_started",
            Self::ServiceStopped => "service_stopped",
            Self::ErrorOccurred => "error_occurred",
            Self::WebhookTriggered => "webhook_triggered",
            Self::WebhookDelivered => "webhook_delivered",
            Self::WebhookFailed => "webhook_failed",
            Self::Custom(name) => name,
        }
    }

    pub fn redis_channel(&self) -> String {
        format!("events:{}", self.as_str())
    }
}

/// Event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: EventType,
    pub payload: serde_json::Value,
    pub metadata: EventMetadata,
    pub timestamp: DateTime<Utc>,
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub source: String,
    pub version: String,
    pub correlation_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
}

impl Event {
    pub fn new(event_type: EventType, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            payload,
            metadata: EventMetadata {
                source: "api-gateway".to_string(),
                version: "1.0.0".to_string(),
                correlation_id: None,
                user_id: None,
                session_id: None,
            },
            timestamp: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.metadata.user_id = Some(user_id);
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.metadata.correlation_id = Some(correlation_id);
        self
    }
}

/// Event handler trait
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle_event(&self, event: &Event) -> Result<()>;
    fn event_type(&self) -> EventType;
}

/// Event subscription
pub struct EventSubscription {
    pub id: Uuid,
    pub event_type: EventType,
    pub receiver: broadcast::Receiver<Event>,
}

impl EventBus {
    /// Create a new event bus
    pub async fn new(redis: RedisService, config: EventBusConfig) -> Result<Self> {
        info!("Initializing event bus with config: {:?}", config);

        Ok(Self {
            redis: Arc::new(RwLock::new(redis)),
            local_channels: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(EventStats::default())),
        })
    }

    /// Publish an event
    pub async fn publish(&self, event: Event) -> Result<()> {
        let event_type_str = event.event_type.as_str().to_string();
        debug!("Publishing event: {} (ID: {})", event_type_str, event.id);

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_published += 1;
            *stats.events_by_type.entry(event_type_str.clone()).or_insert(0) += 1;
        }

        // Publish to local broadcast channel
        if self.config.enable_local_broadcast {
            self.publish_local(&event).await?;
        }

        // Publish to Redis pub/sub
        if self.config.enable_redis_pubsub {
            self.publish_redis(&event).await?;
        }

        // Store event history
        if self.config.enable_event_history {
            self.store_event_history(&event).await?;
        }

        info!("Published event: {} (ID: {})", event_type_str, event.id);
        Ok(())
    }

    /// Publish event to local broadcast channel
    async fn publish_local(&self, event: &Event) -> Result<()> {
        let mut channels = self.local_channels.write().await;

        // Get or create channel for this event type
        let sender = channels
            .entry(event.event_type.clone())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(self.config.channel_buffer_size);
                tx
            });

        // Send event
        match sender.send(event.clone()) {
            Ok(count) => {
                debug!("Event delivered to {} local subscribers", count);

                let mut stats = self.stats.write().await;
                stats.total_delivered += count as u64;

                Ok(())
            }
            Err(e) => {
                warn!("No local subscribers for event type: {}", event.event_type.as_str());
                Ok(()) // Not an error if no subscribers
            }
        }
    }

    /// Publish event to Redis pub/sub
    async fn publish_redis(&self, event: &Event) -> Result<()> {
        let channel = event.event_type.redis_channel();
        let serialized = serde_json::to_string(event)
            .context("Failed to serialize event")?;

        let mut redis = self.redis.write().await;
        let _: () = redis
            .connection_pool
            .publish(&channel, serialized)
            .await
            .context("Failed to publish event to Redis")?;

        debug!("Event published to Redis channel: {}", channel);
        Ok(())
    }

    /// Store event in history
    async fn store_event_history(&self, event: &Event) -> Result<()> {
        let key = format!("event_history:{}:{}", event.event_type.as_str(), event.id);
        let ttl = self.config.history_retention_hours * 3600;

        let serialized = serde_json::to_string(event)
            .context("Failed to serialize event for history")?;

        let mut redis = self.redis.write().await;
        let _: () = redis
            .connection_pool
            .setex(&key, ttl, serialized)
            .await
            .context("Failed to store event history")?;

        // Also add to sorted set for querying
        let history_index = format!("event_history_index:{}", event.event_type.as_str());
        let score = event.timestamp.timestamp() as f64;
        let _: () = redis
            .connection_pool
            .zadd(&history_index, event.id.to_string(), score)
            .await
            .context("Failed to add to event history index")?;

        Ok(())
    }

    /// Subscribe to events of a specific type
    pub async fn subscribe(&self, event_type: EventType) -> Result<EventSubscription> {
        debug!("Creating subscription for event type: {}", event_type.as_str());

        let mut channels = self.local_channels.write().await;

        // Get or create channel for this event type
        let sender = channels
            .entry(event_type.clone())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(self.config.channel_buffer_size);
                tx
            });

        let receiver = sender.subscribe();

        Ok(EventSubscription {
            id: Uuid::new_v4(),
            event_type,
            receiver,
        })
    }

    /// Get event history for a specific event type
    pub async fn get_event_history(
        &self,
        event_type: EventType,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Event>> {
        let history_index = format!("event_history_index:{}", event_type.as_str());

        let mut redis = self.redis.write().await;

        // Get event IDs from sorted set (newest first)
        let event_ids: Vec<String> = redis
            .connection_pool
            .zrevrange(&history_index, offset as isize, (offset + limit) as isize)
            .await
            .context("Failed to get event history index")?;

        let mut events = Vec::new();

        for event_id in event_ids {
            let key = format!("event_history:{}:{}", event_type.as_str(), event_id);

            if let Ok(Some(serialized)) = redis.connection_pool.get::<_, Option<String>>(&key).await {
                if let Ok(event) = serde_json::from_str::<Event>(&serialized) {
                    events.push(event);
                }
            }
        }

        Ok(events)
    }

    /// Get event by ID
    pub async fn get_event(&self, event_type: EventType, event_id: Uuid) -> Result<Option<Event>> {
        let key = format!("event_history:{}:{}", event_type.as_str(), event_id);

        let mut redis = self.redis.write().await;
        let serialized: Option<String> = redis
            .connection_pool
            .get(&key)
            .await
            .context("Failed to get event")?;

        match serialized {
            Some(data) => {
                let event = serde_json::from_str(&data)
                    .context("Failed to deserialize event")?;
                Ok(Some(event))
            }
            None => Ok(None),
        }
    }

    /// Get event statistics
    pub async fn get_stats(&self) -> EventStats {
        self.stats.read().await.clone()
    }

    /// Reset event statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = EventStats::default();
        info!("Event statistics reset");
    }

    /// Emit a simple event with minimal data
    pub async fn emit(
        &self,
        event_type: EventType,
        data: serde_json::Value,
    ) -> Result<Uuid> {
        let event = Event::new(event_type, data);
        let event_id = event.id;
        self.publish(event).await?;
        Ok(event_id)
    }

    /// Emit event with user context
    pub async fn emit_user_event(
        &self,
        event_type: EventType,
        user_id: Uuid,
        data: serde_json::Value,
    ) -> Result<Uuid> {
        let event = Event::new(event_type, data).with_user_id(user_id);
        let event_id = event.id;
        self.publish(event).await?;
        Ok(event_id)
    }

    /// Clean up old event history
    pub async fn cleanup_event_history(&self, event_type: EventType, older_than_hours: u64) -> Result<u64> {
        let history_index = format!("event_history_index:{}", event_type.as_str());
        let cutoff_timestamp = (Utc::now() - chrono::Duration::hours(older_than_hours as i64)).timestamp() as f64;

        let mut redis = self.redis.write().await;

        // Remove old entries from sorted set
        let removed: u64 = redis::cmd("ZREMRANGEBYSCORE")
            .arg(&history_index)
            .arg("-inf")
            .arg(cutoff_timestamp)
            .query_async(&mut redis.connection_pool)
            .await
            .context("Failed to cleanup event history")?;

        info!("Cleaned up {} old events for type: {}", removed, event_type.as_str());
        Ok(removed)
    }
}

/// Helper macro for publishing events
#[macro_export]
macro_rules! publish_event {
    ($event_bus:expr, $event_type:expr, $data:expr) => {
        $event_bus.emit($event_type, serde_json::json!($data)).await
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::UserRegistered.as_str(), "user_registered");
        assert_eq!(EventType::BountyCreated.as_str(), "bounty_created");
        assert_eq!(EventType::AnalysisCompleted.as_str(), "analysis_completed");
    }

    #[test]
    fn test_event_creation() {
        let event = Event::new(
            EventType::UserRegistered,
            serde_json::json!({"email": "test@example.com"}),
        );

        assert_eq!(event.event_type, EventType::UserRegistered);
        assert_eq!(event.metadata.source, "api-gateway");
    }

    #[test]
    fn test_event_with_metadata() {
        let user_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();

        let event = Event::new(
            EventType::UserLoggedIn,
            serde_json::json!({"success": true}),
        )
        .with_user_id(user_id)
        .with_correlation_id(correlation_id);

        assert_eq!(event.metadata.user_id, Some(user_id));
        assert_eq!(event.metadata.correlation_id, Some(correlation_id));
    }

    #[test]
    fn test_redis_channel_name() {
        let event_type = EventType::AnalysisCompleted;
        assert_eq!(event_type.redis_channel(), "events:analysis_completed");
    }
}
