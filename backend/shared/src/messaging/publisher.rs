use anyhow::{Result, anyhow};
use redis::AsyncCommands;
use serde::Serialize;
use tracing::{info, error};

use super::event_types::NexusEvent;

/// Redis Pub/Sub channel prefix for events
const EVENT_CHANNEL_PREFIX: &str = "events:";

/// Event publisher for Redis Pub/Sub
pub struct EventPublisher {
    redis_client: redis::Client,
}

impl EventPublisher {
    /// Create a new event publisher
    pub fn new(redis_client: redis::Client) -> Self {
        Self { redis_client }
    }

    /// Create from Redis URL
    pub fn from_url(redis_url: &str) -> Result<Self> {
        let redis_client = redis::Client::open(redis_url)
            .map_err(|e| anyhow!("Failed to create Redis client: {}", e))?;
        Ok(Self { redis_client })
    }

    /// Publish an event to the appropriate Redis Pub/Sub channel
    pub async fn publish(&self, event: &NexusEvent) -> Result<()> {
        let channel = self.get_channel_for_event(event);
        let payload = serde_json::to_string(event)
            .map_err(|e| anyhow!("Failed to serialize event: {}", e))?;

        let mut conn = self.redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow!("Failed to connect to Redis: {}", e))?;

        conn.publish::<_, _, ()>(&channel, payload)
            .await
            .map_err(|e| anyhow!("Failed to publish event to {}: {}", channel, e))?;

        info!("Published event to channel: {}", channel);
        Ok(())
    }

    /// Publish multiple events in batch
    pub async fn publish_batch(&self, events: Vec<NexusEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let event_count = events.len();

        let mut conn = self.redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow!("Failed to connect to Redis: {}", e))?;

        for event in events {
            let channel = self.get_channel_for_event(&event);
            let payload = serde_json::to_string(&event)
                .map_err(|e| anyhow!("Failed to serialize event: {}", e))?;

            if let Err(e) = conn.publish::<_, _, ()>(&channel, payload).await {
                error!("Failed to publish event to {}: {}", channel, e);
            }
        }

        info!("Published {} events in batch", event_count);
        Ok(())
    }

    /// Get the Redis channel name for a given event
    fn get_channel_for_event(&self, event: &NexusEvent) -> String {
        let event_name = match event {
            NexusEvent::BountyCreated(_) => "bounty_created",
            NexusEvent::BountyUpdated(_) => "bounty_updated",
            NexusEvent::BountyCompleted(_) => "bounty_completed",
            NexusEvent::BountyExpired(_) => "bounty_expired",
            NexusEvent::BountyCancelled(_) => "bounty_cancelled",

            NexusEvent::SubmissionReceived(_) => "submission_received",
            NexusEvent::SubmissionValidated(_) => "submission_validated",
            NexusEvent::SubmissionRejected(_) => "submission_rejected",

            NexusEvent::AnalysisStarted(_) => "analysis_started",
            NexusEvent::AnalysisCompleted(_) => "analysis_completed",
            NexusEvent::AnalysisFailed(_) => "analysis_failed",

            NexusEvent::ReputationUpdated(_) => "reputation_updated",

            NexusEvent::PaymentProcessed(_) => "payment_processed",
            NexusEvent::PaymentFailed(_) => "payment_failed",
            NexusEvent::StakeSlashed(_) => "stake_slashed",

            NexusEvent::UserRegistered(_) => "user_registered",
            NexusEvent::UserVerified(_) => "user_verified",
            NexusEvent::EngineRegistered(_) => "engine_registered",

            NexusEvent::DisputeCreated(_) => "dispute_created",
            NexusEvent::DisputeResolved(_) => "dispute_resolved",

            NexusEvent::SystemAlert(_) => "system_alert",
        };

        format!("{}{}", EVENT_CHANNEL_PREFIX, event_name)
    }
}

/// Publish a single event (convenience function)
pub async fn publish_event(redis_client: &redis::Client, event: &NexusEvent) -> Result<()> {
    let publisher = EventPublisher::new(redis_client.clone());
    publisher.publish(event).await
}

/// Publish a JSON payload to a specific channel (generic version)
pub async fn publish_to_channel<T: Serialize>(
    redis_client: &redis::Client,
    channel: &str,
    payload: &T,
) -> Result<()> {
    let message = serde_json::to_string(payload)
        .map_err(|e| anyhow!("Failed to serialize payload: {}", e))?;

    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| anyhow!("Failed to connect to Redis: {}", e))?;

    conn.publish::<_, _, ()>(channel, message)
        .await
        .map_err(|e| anyhow!("Failed to publish to channel {}: {}", channel, e))?;

    info!("Published message to channel: {}", channel);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::event_types::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_channel_mapping() {
        let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
        let publisher = EventPublisher::new(redis_client);

        let event = NexusEvent::UserRegistered(UserRegisteredEvent {
            user_id: Uuid::new_v4(),
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
            ethereum_address: "0x0000000000000000000000000000000000000000".to_string(),
            registered_at: Utc::now(),
        });

        let channel = publisher.get_channel_for_event(&event);
        assert_eq!(channel, "events:user_registered");
    }

    #[test]
    fn test_payment_event_channel() {
        let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
        let publisher = EventPublisher::new(redis_client);

        let event = NexusEvent::PaymentProcessed(PaymentProcessedEvent {
            bounty_id: Uuid::new_v4(),
            recipient_id: Uuid::new_v4(),
            amount: 1000.0,
            tx_hash: "0x1234567890abcdef".to_string(),
            payment_type: PaymentType::BountyReward,
            processed_at: Utc::now(),
        });

        let channel = publisher.get_channel_for_event(&event);
        assert_eq!(channel, "events:payment_processed");
    }
}