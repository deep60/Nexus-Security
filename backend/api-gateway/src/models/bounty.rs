use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Bounty status — matches CHECK constraint in 002_bounty_system.sql
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum BountyStatus {
    Active,
    Completed,
    Expired,
    Cancelled,
}

impl std::fmt::Display for BountyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Completed => write!(f, "completed"),
            Self::Expired => write!(f, "expired"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

// Main bounty record — matches `bounties` table after migrations 001+002+003+005
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Bounty {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub submission_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub reward_amount: String,            // DECIMAL(20,8) → String for precision
    pub min_stake_amount: String,         // DECIMAL(20,8)
    pub max_participants: Option<i32>,
    pub deadline: Option<DateTime<Utc>>,
    pub bounty_status: String,            // 'active','completed','expired','cancelled'
    pub requires_verification: bool,
    pub priority_level: i32,              // 1–5
    pub blockchain_tx_hash: Option<String>,
    pub smart_contract_address: Option<String>,
    pub total_staked: String,             // DECIMAL(20,8)
    pub participant_count: i32,
    pub consensus_threshold: String,      // DECIMAL(3,2)
    pub on_chain_id: Option<i64>,         // from migration 003
    pub token_address: Option<String>,    // from migration 005
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// Request/Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBountyRequest {
    pub submission_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub reward_amount: String,
    pub min_stake_amount: Option<String>,
    pub max_participants: Option<i32>,
    pub deadline_hours: Option<i32>,
    pub requires_verification: Option<bool>,
    pub priority_level: Option<i32>,
    pub consensus_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBountyRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority_level: Option<i32>,
    pub reward_amount: Option<String>,
    pub deadline: Option<DateTime<Utc>>,
    pub max_participants: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyListResponse {
    pub bounties: Vec<Bounty>,
    pub total_count: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

// Bounty submission from analysis engines
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BountySubmission {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub engine_id: Uuid,
    pub engine_name: String,
    pub engine_address: String,
    pub verdict: String,
    pub confidence: f64,
    pub stake_amount: String,
    pub details: serde_json::Value,
    pub submitted_at: DateTime<Utc>,
    pub is_verified: bool,
}

// Extended submission data for detailed views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedSubmission {
    pub submission: BountySubmission,
    pub engine_name: String,
    pub engine_version: String,
    pub threat_types: Vec<String>,
    pub risk_score: u8,
    pub analysis_summary: String,
    pub signatures: Vec<String>,
    pub status: crate::handlers::submission::SubmissionStatus,
    pub processing_metrics: Option<ProcessingMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetrics {
    pub processing_time_ms: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

impl Bounty {
    pub fn is_active(&self) -> bool {
        self.bounty_status == "active"
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
            self.participant_count < max
        } else {
            true
        }
    }

    pub fn time_remaining(&self) -> Option<chrono::Duration> {
        self.deadline.map(|deadline| deadline - Utc::now())
    }
}

// Bounty validation helpers
impl CreateBountyRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        if self.reward_amount.parse::<f64>().unwrap_or(0.0) <= 0.0 {
            return Err("Reward amount must be greater than 0".to_string());
        }

        if let Some(ref stake) = self.min_stake_amount {
            if stake.parse::<f64>().unwrap_or(0.0) < 0.0 {
                return Err("Minimum stake cannot be negative".to_string());
            }
        }

        if let Some(consensus) = self.consensus_threshold {
            if !(0.0..=1.0).contains(&consensus) {
                return Err("Consensus threshold must be between 0.00 and 1.00".to_string());
            }
        }

        Ok(())
    }
}
