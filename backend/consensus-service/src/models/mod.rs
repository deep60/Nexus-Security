use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub type ConsensusResult<T> = Result<T, ConsensusError>;

#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Insufficient submissions: need {required}, got {actual}")]
    InsufficientSubmissions { required: usize, actual: usize },
    
    #[error("Consensus failed: {0}")]
    ConsensusFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Malicious,
    Benign,
    Suspicious,
    Unknown,
}

impl ToString for Verdict {
    fn to_string(&self) -> String {
        match self {
            Verdict::Malicious => "malicious".to_string(),
            Verdict::Benign => "benign".to_string(),
            Verdict::Suspicious => "suspicious".to_string(),
            Verdict::Unknown => "unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BountyConsensus {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub final_verdict: String,
    pub confidence_score: Decimal,
    pub total_submissions: i32,
    pub agreement_score: Decimal,
    pub participating_engines: Vec<String>,
    pub weighted_votes: serde_json::Value,
    pub verdict_distribution: serde_json::Value,
    pub is_disputed: bool,
    pub finalized_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionVote {
    pub submission_id: Uuid,
    pub user_id: Uuid,
    pub engine_id: String,
    pub verdict: Verdict,
    pub confidence: Decimal,
    pub reputation_score: i32,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictDistribution {
    pub malicious: VoteStats,
    pub benign: VoteStats,
    pub suspicious: VoteStats,
    pub unknown: VoteStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteStats {
    pub count: usize,
    pub weighted_count: Decimal,
    pub percentage: Decimal,
    pub avg_confidence: Decimal,
    pub voters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Dispute {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub submission_id: Option<Uuid>,
    pub initiator_id: Uuid,
    pub disputed_verdict: String,
    pub claimed_verdict: String,
    pub reason: String,
    pub evidence: Option<serde_json::Value>,
    pub status: String,
    pub resolution: Option<String>,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisputeStatus {
    Open,
    UnderReview,
    Resolved,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusCalculationRequest {
    pub bounty_id: Uuid,
    pub force_recalculate: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusResponse {
    pub bounty_id: Uuid,
    pub final_verdict: Verdict,
    pub confidence_score: Decimal,
    pub agreement_score: Decimal,
    pub verdict_distribution: VerdictDistribution,
    pub total_submissions: usize,
    pub is_finalized: bool,
    pub can_be_disputed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDisputeRequest {
    pub bounty_id: Uuid,
    pub submission_id: Option<Uuid>,
    pub disputed_verdict: Verdict,
    pub claimed_verdict: Verdict,
    pub reason: String,
    pub evidence: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveDisputeRequest {
    pub resolution: String,
    pub final_verdict: Verdict,
    pub compensation: Option<Decimal>,
}

impl Default for VerdictDistribution {
    fn default() -> Self {
        Self {
            malicious: VoteStats::default(),
            benign: VoteStats::default(),
            suspicious: VoteStats::default(),
            unknown: VoteStats::default(),
        }
    }
}

impl Default for VoteStats {
    fn default() -> Self {
        Self {
            count: 0,
            weighted_count: Decimal::new(0, 0),
            percentage: Decimal::new(0, 0),
            avg_confidence: Decimal::new(0, 0),
            voters: Vec::new(),
        }
    }
}
