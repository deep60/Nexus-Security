use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

use crate::types::common::{
    UserId, BountyId, SubmissionId, EngineId, ThreatVerdict,
    TokenAmount, TransactionHash
};

/// Core event types for the Nexus-Security platform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum NexusEvent {
    // Bounty events
    BountyCreated(BountyCreatedEvent),
    BountyUpdated(BountyUpdatedEvent),
    BountyCompleted(BountyCompletedEvent),
    BountyExpired(BountyExpiredEvent),
    BountyCancelled(BountyCancelledEvent),

    // Submission events
    SubmissionReceived(SubmissionReceivedEvent),
    SubmissionValidated(SubmissionValidatedEvent),
    SubmissionRejected(SubmissionRejectedEvent),

    // Analysis events
    AnalysisStarted(AnalysisStartedEvent),
    AnalysisCompleted(AnalysisCompletedEvent),
    AnalysisFailed(AnalysisFailedEvent),

    // Reputation events
    ReputationUpdated(ReputationUpdatedEvent),

    // Payment events
    PaymentProcessed(PaymentProcessedEvent),
    PaymentFailed(PaymentFailedEvent),
    StakeSlashed(StakeSlashedEvent),

    // User events
    UserRegistered(UserRegisteredEvent),
    UserVerified(UserVerifiedEvent),
    EngineRegistered(EngineRegisteredEvent),

    // Dispute events
    DisputeCreated(DisputeCreatedEvent),
    DisputeResolved(DisputeResolvedEvent),

    // System events
    SystemAlert(SystemAlertEvent),
}

// Bounty Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyCreatedEvent {
    pub bounty_id: BountyId,
    pub creator_id: UserId,
    pub title: String,
    pub description: String,
    pub reward_amount: TokenAmount,
    pub stake_requirement: TokenAmount,
    pub expires_at: DateTime<Utc>,
    pub target_type: String, // "file", "url", "hash"
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyUpdatedEvent {
    pub bounty_id: BountyId,
    pub updated_fields: HashMap<String, serde_json::Value>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyCompletedEvent {
    pub bounty_id: BountyId,
    pub creator_id: UserId,
    pub final_verdict: ThreatVerdict,
    pub total_submissions: u32,
    pub winning_submission_id: Option<SubmissionId>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyExpiredEvent {
    pub bounty_id: BountyId,
    pub creator_id: UserId,
    pub expired_at: DateTime<Utc>,
    pub total_submissions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyCancelledEvent {
    pub bounty_id: BountyId,
    pub creator_id: UserId,
    pub reason: String,
    pub cancelled_at: DateTime<Utc>,
}

// Submission Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionReceivedEvent {
    pub submission_id: SubmissionId,
    pub bounty_id: BountyId,
    pub engine_id: EngineId,
    pub submitter_id: UserId,
    pub verdict: ThreatVerdict,
    pub confidence_score: f32,
    pub stake_amount: TokenAmount,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionValidatedEvent {
    pub submission_id: SubmissionId,
    pub bounty_id: BountyId,
    pub submitter_id: UserId,
    pub is_correct: bool,
    pub reputation_change: i32,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionRejectedEvent {
    pub submission_id: SubmissionId,
    pub bounty_id: BountyId,
    pub submitter_id: UserId,
    pub reason: String,
    pub rejected_at: DateTime<Utc>,
}

// Analysis Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStartedEvent {
    pub bounty_id: BountyId,
    pub engine_id: EngineId,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCompletedEvent {
    pub bounty_id: BountyId,
    pub engine_id: EngineId,
    pub submission_id: SubmissionId,
    pub verdict: ThreatVerdict,
    pub confidence: f32,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFailedEvent {
    pub bounty_id: BountyId,
    pub engine_id: EngineId,
    pub error_message: String,
    pub failed_at: DateTime<Utc>,
}

// Reputation Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationUpdatedEvent {
    pub user_id: UserId,
    pub old_score: i32,
    pub new_score: i32,
    pub change_reason: String,
    pub related_submission_id: Option<SubmissionId>,
    pub updated_at: DateTime<Utc>,
}

// Payment Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProcessedEvent {
    pub bounty_id: BountyId,
    pub recipient_id: UserId,
    pub amount: TokenAmount,
    pub tx_hash: TransactionHash,
    pub payment_type: PaymentType,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentFailedEvent {
    pub bounty_id: BountyId,
    pub recipient_id: UserId,
    pub amount: TokenAmount,
    pub error_message: String,
    pub failed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeSlashedEvent {
    pub submission_id: SubmissionId,
    pub bounty_id: BountyId,
    pub user_id: UserId,
    pub slashed_amount: TokenAmount,
    pub reason: String,
    pub slashed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentType {
    BountyReward,
    StakeReturn,
    DisputeResolution,
}

// User Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    pub user_id: UserId,
    pub username: String,
    pub email: String,
    pub ethereum_address: String,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserVerifiedEvent {
    pub user_id: UserId,
    pub verification_type: String,
    pub verified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRegisteredEvent {
    pub engine_id: EngineId,
    pub user_id: UserId,
    pub engine_name: String,
    pub engine_type: String,
    pub registered_at: DateTime<Utc>,
}

// Dispute Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeCreatedEvent {
    pub dispute_id: Uuid,
    pub submission_id: SubmissionId,
    pub bounty_id: BountyId,
    pub initiator_id: UserId,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolvedEvent {
    pub dispute_id: Uuid,
    pub submission_id: SubmissionId,
    pub bounty_id: BountyId,
    pub resolution: String,
    pub resolved_by: UserId,
    pub resolved_at: DateTime<Utc>,
}

// System Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAlertEvent {
    pub alert_id: Uuid,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub affected_services: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

// Notification specific types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub notification_id: Uuid,
    pub user_id: UserId,
    pub channels: Vec<NotificationChannel>,
    pub event: NexusEvent,
    pub priority: NotificationPriority,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationChannel {
    Email,
    Push,
    Webhook,
    WebSocket,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl NexusEvent {
    /// Get a human-readable title for the event
    pub fn get_title(&self) -> String {
        match self {
            NexusEvent::BountyCreated(e) => format!("New Bounty Created: {}", e.title),
            NexusEvent::BountyUpdated(_) => "Bounty Updated".to_string(),
            NexusEvent::BountyCompleted(_) => "Bounty Completed".to_string(),
            NexusEvent::BountyExpired(_) => "Bounty Expired".to_string(),
            NexusEvent::BountyCancelled(_) => "Bounty Cancelled".to_string(),
            NexusEvent::SubmissionReceived(_) => "New Submission Received".to_string(),
            NexusEvent::SubmissionValidated(_) => "Submission Validated".to_string(),
            NexusEvent::SubmissionRejected(_) => "Submission Rejected".to_string(),
            NexusEvent::AnalysisStarted(_) => "Analysis Started".to_string(),
            NexusEvent::AnalysisCompleted(_) => "Analysis Completed".to_string(),
            NexusEvent::AnalysisFailed(_) => "Analysis Failed".to_string(),
            NexusEvent::ReputationUpdated(_) => "Reputation Updated".to_string(),
            NexusEvent::PaymentProcessed(_) => "Payment Processed".to_string(),
            NexusEvent::PaymentFailed(_) => "Payment Failed".to_string(),
            NexusEvent::StakeSlashed(_) => "Stake Slashed".to_string(),
            NexusEvent::UserRegistered(_) => "Welcome to Nexus Security!".to_string(),
            NexusEvent::UserVerified(_) => "Account Verified".to_string(),
            NexusEvent::EngineRegistered(_) => "Engine Registered".to_string(),
            NexusEvent::DisputeCreated(_) => "Dispute Created".to_string(),
            NexusEvent::DisputeResolved(_) => "Dispute Resolved".to_string(),
            NexusEvent::SystemAlert(e) => format!("System Alert: {}", e.title),
        }
    }

    /// Get a human-readable description for the event
    pub fn get_description(&self) -> String {
        match self {
            NexusEvent::BountyCreated(e) => format!(
                "A new bounty has been created with a reward of {} tokens. {}",
                e.reward_amount, e.description
            ),
            NexusEvent::SubmissionReceived(e) => format!(
                "Your submission for bounty {} has been received with verdict: {:?}",
                e.bounty_id, e.verdict
            ),
            NexusEvent::PaymentProcessed(e) => format!(
                "Payment of {} tokens has been processed. Transaction: {}",
                e.amount, e.tx_hash
            ),
            NexusEvent::ReputationUpdated(e) => format!(
                "Your reputation has changed from {} to {}. Reason: {}",
                e.old_score, e.new_score, e.change_reason
            ),
            _ => "Event occurred".to_string(),
        }
    }
}
