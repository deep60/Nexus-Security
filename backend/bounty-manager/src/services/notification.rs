// backend/bounty-manager/src/services/notification.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NotificationService {
    // TODO: Add Redis or message queue connection
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub recipient: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    BountyCreated,
    BountyExpired,
    SubmissionReceived,
    ConsensusReached,
    PayoutProcessed,
    DisputeRaised,
    DisputeResolved,
    ReputationUpdated,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {}
    }

    /// Send a notification to a user
    pub async fn send_notification(&self, notification: Notification) -> Result<(), NotificationError> {
        // TODO: Implement actual notification sending (email, websocket, push, etc.)
        tracing::info!(
            "Sending notification to {}: {}",
            notification.recipient,
            notification.title
        );
        Ok(())
    }

    /// Send bounty created notification
    pub async fn notify_bounty_created(
        &self,
        bounty_id: Uuid,
        creator: &str,
        title: &str,
    ) -> Result<(), NotificationError> {
        let notification = Notification {
            id: Uuid::new_v4(),
            recipient: creator.to_string(),
            notification_type: NotificationType::BountyCreated,
            title: "Bounty Created".to_string(),
            message: format!("Your bounty '{}' has been created successfully", title),
            data: Some(serde_json::json!({ "bounty_id": bounty_id })),
            created_at: chrono::Utc::now(),
        };

        self.send_notification(notification).await
    }

    /// Send submission received notification
    pub async fn notify_submission_received(
        &self,
        bounty_id: Uuid,
        creator: &str,
        engine_id: &str,
    ) -> Result<(), NotificationError> {
        let notification = Notification {
            id: Uuid::new_v4(),
            recipient: creator.to_string(),
            notification_type: NotificationType::SubmissionReceived,
            title: "New Submission".to_string(),
            message: format!("A new submission from {} has been received", engine_id),
            data: Some(serde_json::json!({ "bounty_id": bounty_id, "engine_id": engine_id })),
            created_at: chrono::Utc::now(),
        };

        self.send_notification(notification).await
    }

    /// Send consensus reached notification
    pub async fn notify_consensus_reached(
        &self,
        bounty_id: Uuid,
        participants: Vec<String>,
        verdict: &str,
    ) -> Result<(), NotificationError> {
        for participant in participants {
            let notification = Notification {
                id: Uuid::new_v4(),
                recipient: participant,
                notification_type: NotificationType::ConsensusReached,
                title: "Consensus Reached".to_string(),
                message: format!("Consensus has been reached with verdict: {}", verdict),
                data: Some(serde_json::json!({ "bounty_id": bounty_id, "verdict": verdict })),
                created_at: chrono::Utc::now(),
            };

            self.send_notification(notification).await?;
        }

        Ok(())
    }

    /// Send payout processed notification
    pub async fn notify_payout_processed(
        &self,
        recipient: &str,
        amount: u64,
        transaction_hash: &str,
    ) -> Result<(), NotificationError> {
        let notification = Notification {
            id: Uuid::new_v4(),
            recipient: recipient.to_string(),
            notification_type: NotificationType::PayoutProcessed,
            title: "Payout Processed".to_string(),
            message: format!("Your payout of {} has been processed", amount),
            data: Some(serde_json::json!({ "amount": amount, "tx_hash": transaction_hash })),
            created_at: chrono::Utc::now(),
        };

        self.send_notification(notification).await
    }

    /// Send dispute raised notification
    pub async fn notify_dispute_raised(
        &self,
        bounty_id: Uuid,
        affected_parties: Vec<String>,
    ) -> Result<(), NotificationError> {
        for party in affected_parties {
            let notification = Notification {
                id: Uuid::new_v4(),
                recipient: party,
                notification_type: NotificationType::DisputeRaised,
                title: "Dispute Raised".to_string(),
                message: "A dispute has been raised on a bounty you're involved in".to_string(),
                data: Some(serde_json::json!({ "bounty_id": bounty_id })),
                created_at: chrono::Utc::now(),
            };

            self.send_notification(notification).await?;
        }

        Ok(())
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Failed to send notification: {0}")]
    SendError(String),

    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),
}
