use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Bounty status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bounty_status", rename_all = "lowercase")]
pub enum BountyStatus {
    Draft,
    Active,
    InProgress,
    Completed,
    Expired,
    Cancelled,
    Disputed,
}

// Bounty priority levels
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bounty_priority", rename_all = "lowercase")]
pub enum BountyPriority {
    Low,
    Medium,
    High,
    Critical,
    Emergency,
}

// Bounty type categories
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bounty_type", rename_all = "lowercase")]
pub enum BountyType {
    FileAnalysis,
    UrlAnalysis,
    DomainAnalysis,
    HashAnalysis,
    SignatureDetection,
    BehaviorAnalysis,
    NetworkAnalysis,
    Custom,
}

// Payment distribution method
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "distribution_method", rename_all = "lowercase")]
pub enum DistributionMethod {
    WinnerTakeAll,
    ProportionalStake,
    FixedReward,
    TieredRewards,
}

// Main bounty record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Bounty {
    pub id: Uuid,
    pub creator_address: String,
    pub title: String,
    pub description: String,
    pub bounty_type: BountyType,
    pub priority: BountyPriority,
    pub status: BountyStatus,
    pub total_reward: String, // Use string for precise decimal handling
    pub minimum_stake: String,
    pub distribution_method: DistributionMethod,
    pub max_participants: Option<i32>,
    pub current_participants: i32,
    pub required_consensus: f64, // Percentage threshold for consensus
    pub minimum_reputation: f64,
    pub deadline: Option<DateTime<Utc>>,
    pub auto_finalize: bool,
    pub requires_human_analysis: bool,
    pub file_types_allowed: Vec<String>,
    pub max_file_size: Option<i64>,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub blockchain_tx_hash: Option<String>,
    pub escrow_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

// Bounty participation tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BountyParticipant {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub engine_id: Uuid,
    pub engine_address: String,
    pub stake_amount: String,
    pub joined_at: DateTime<Utc>,
    pub is_active: bool,
    pub submission_count: i32,
    pub last_submission_at: Option<DateTime<Utc>>,
}

// Bounty rewards and payouts
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BountyPayout {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub recipient_address: String,
    pub engine_id: Option<Uuid>,
    pub payout_type: PayoutType,
    pub amount: String,
    pub reason: String,
    pub blockchain_tx_hash: Option<String>,
    pub status: PayoutStatus,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payout_type", rename_all = "lowercase")]
pub enum PayoutType {
    WinnerReward,
    ParticipationReward,
    StakeReturn,
    BonusReward,
    Penalty,
    Refund,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payout_status", rename_all = "lowercase")]
pub enum PayoutStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

// Bounty requirements and constraints
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BountyRequirement {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub requirement_type: RequirementType,
    pub description: String,
    pub parameters: serde_json::Value,
    pub is_mandatory: bool,
    pub weight: f64, // For scoring importance
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "requirement_type", rename_all = "lowercase")]
pub enum RequirementType {
    MinimumEngines,
    RequiredEngineTypes,
    GeographicRestriction,
    TimeConstraint,
    QualityThreshold,
    ComplianceCheck,
    CustomRule,
}

// Bounty templates for common analysis types
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BountyTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub bounty_type: BountyType,
    pub default_reward: String,
    pub default_stake: String,
    pub default_deadline_hours: Option<i32>,
    pub requirements: Vec<BountyRequirement>,
    pub metadata_schema: serde_json::Value,
    pub is_public: bool,
    pub creator_address: String,
    pub usage_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Bounty statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BountyStats {
    pub bounty_id: Uuid,
    pub total_submissions: i32,
    pub unique_participants: i32,
    pub avg_response_time: Option<f64>,
    pub consensus_reached_at: Option<DateTime<Utc>>,
    pub quality_score: Option<f64>,
    pub total_stake_pool: String,
    pub rewards_distributed: String,
    pub penalties_applied: String,
    pub dispute_count: i32,
    pub completion_rate: f64,
    pub calculated_at: DateTime<Utc>,
}

// Bounty search and filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyFilter {
    pub status: Option<Vec<BountyStatus>>,
    pub bounty_type: Option<Vec<BountyType>>,
    pub priority: Option<Vec<BountyPriority>>,
    pub min_reward: Option<String>,
    pub max_reward: Option<String>,
    pub creator_address: Option<String>,
    pub tags: Option<Vec<String>>,
    pub requires_human: Option<bool>,
    pub deadline_before: Option<DateTime<Utc>>,
    pub deadline_after: Option<DateTime<Utc>>,
    pub min_reputation_required: Option<f64>,
    pub max_participants_available: Option<bool>,
}

// Request/Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBountyRequest {
    pub title: String,
    pub description: String,
    pub bounty_type: BountyType,
    pub priority: BountyPriority,
    pub total_reward: String,
    pub minimum_stake: String,
    pub distribution_method: DistributionMethod,
    pub max_participants: Option<i32>,
    pub required_consensus: Option<f64>,
    pub minimum_reputation: Option<f64>,
    pub deadline_hours: Option<i32>,
    pub auto_finalize: Option<bool>,
    pub requires_human_analysis: Option<bool>,
    pub file_types_allowed: Option<Vec<String>>,
    pub max_file_size: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub template_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBountyRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<BountyPriority>,
    pub total_reward: Option<String>,
    pub deadline: Option<DateTime<Utc>>,
    pub max_participants: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyResponse {
    pub bounty: Bounty,
    pub participants: Vec<BountyParticipant>,
    pub requirements: Vec<BountyRequirement>,
    pub stats: Option<BountyStats>,
    pub can_participate: bool,
    pub participation_requirements_met: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyListResponse {
    pub bounties: Vec<Bounty>,
    pub total_count: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipateRequest {
    pub stake_amount: String,
    pub engine_capabilities: Vec<String>,
    pub estimated_completion_time: Option<i32>, // hours
}

// Bounty lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BountyEvent {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub event_type: BountyEventType,
    pub actor_address: String,
    pub description: String,
    pub metadata: serde_json::Value,
    pub blockchain_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bounty_event_type", rename_all = "lowercase")]
pub enum BountyEventType {
    Created,
    Started,
    ParticipantJoined,
    ParticipantLeft,
    SubmissionReceived,
    ConsensusReached,
    Completed,
    Expired,
    Cancelled,
    Disputed,
    RewardDistributed,
    StakeSlashed,
}

// Escrow and payment management
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BountyEscrow {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub escrow_address: String,
    pub total_amount: String,
    pub locked_amount: String,
    pub available_amount: String,
    pub contract_address: String,
    pub deployment_tx_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub finalized_at: Option<DateTime<Utc>>,
}

impl Bounty {
    pub fn new(
        creator_address: String,
        title: String,
        description: String,
        bounty_type: BountyType,
        total_reward: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            creator_address,
            title,
            description,
            bounty_type,
            priority: BountyPriority::Medium,
            status: BountyStatus::Draft,
            total_reward,
            minimum_stake: "0".to_string(),
            distribution_method: DistributionMethod::ProportionalStake,
            max_participants: None,
            current_participants: 0,
            required_consensus: 70.0, // 70% consensus by default
            minimum_reputation: 0.0,
            deadline: None,
            auto_finalize: true,
            requires_human_analysis: false,
            file_types_allowed: Vec::new(),
            max_file_size: None,
            tags: Vec::new(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            blockchain_tx_hash: None,
            escrow_address: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, BountyStatus::Active | BountyStatus::InProgress)
    }

    pub fn is_expired(&self) -> bool {
        if let Some(deadline) = self.deadline {
            Utc::now() > deadline
        } else {
            false
        }
    }

    pub fn can_accept_participants(&self) -> bool {
        if let Some(max) = self.max_participants {
            self.current_participants < max
        } else {
            true
        }
    }

    pub fn time_remaining(&self) -> Option<chrono::Duration> {
        self.deadline.map(|deadline| deadline - Utc::now())
    }

    pub fn participation_rate(&self) -> f64 {
        if let Some(max) = self.max_participants {
            (self.current_participants as f64 / max as f64) * 100.0
        } else {
            0.0
        }
    }
}

impl BountyStats {
    pub fn avg_reward_per_participant(&self) -> Option<f64> {
        if self.unique_participants > 0 {
            self.rewards_distributed.parse::<f64>()
                .map(|total| total / self.unique_participants as f64)
                .ok()
        } else {
            None
        }
    }

    pub fn roi_percentage(&self) -> Option<f64> {
        let rewards = self.rewards_distributed.parse::<f64>().ok()?;
        let stakes = self.total_stake_pool.parse::<f64>().ok()?;
        
        if stakes > 0.0 {
            Some((rewards / stakes) * 100.0)
        } else {
            None
        }
    }
}

// Helper functions for bounty management
impl BountyFilter {
    pub fn new() -> Self {
        Self {
            status: None,
            bounty_type: None,
            priority: None,
            min_reward: None,
            max_reward: None,
            creator_address: None,
            tags: None,
            requires_human: None,
            deadline_before: None,
            deadline_after: None,
            min_reputation_required: None,
            max_participants_available: None,
        }
    }

    pub fn active_bounties() -> Self {
        Self {
            status: Some(vec![BountyStatus::Active, BountyStatus::InProgress]),
            ..Self::new()
        }
    }

    pub fn high_value_bounties(min_reward: String) -> Self {
        Self {
            min_reward: Some(min_reward),
            status: Some(vec![BountyStatus::Active]),
            ..Self::new()
        }
    }
}

// Bounty validation helpers
impl CreateBountyRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        if self.description.trim().is_empty() {
            return Err("Description cannot be empty".to_string());
        }

        if self.total_reward.parse::<f64>().unwrap_or(0.0) <= 0.0 {
            return Err("Total reward must be greater than 0".to_string());
        }

        if self.minimum_stake.parse::<f64>().unwrap_or(0.0) < 0.0 {
            return Err("Minimum stake cannot be negative".to_string());
        }

        if let Some(consensus) = self.required_consensus {
            if consensus < 0.0 || consensus > 100.0 {
                return Err("Required consensus must be between 0 and 100".to_string());
            }
        }

        Ok(())
    }
}