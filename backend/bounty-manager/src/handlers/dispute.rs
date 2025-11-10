// backend/bounty-manager/src/handlers/dispute.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use shared::types::common::{ApiResponse, PaginationParams};
use crate::handlers::bounty_crud::{BountyManagerState, ThreatVerdict};

/// Represents a dispute raised against a submission or bounty outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub submission_id: Option<Uuid>, // If disputing a specific submission
    pub disputer_id: String, // Engine or user ID raising the dispute
    pub dispute_type: DisputeType,
    pub reason: String,
    pub evidence: Vec<Evidence>,
    pub status: DisputeStatus,
    pub severity: DisputeSeverity,
    pub stake_amount: u64, // Stake required to raise dispute
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolver_id: Option<String>, // ID of arbitrator/admin who resolved
    pub resolution: Option<DisputeResolution>,
    pub votes: Vec<DisputeVote>,
    pub metadata: HashMap<String, String>,
}

/// Types of disputes that can be raised
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisputeType {
    IncorrectVerdict,      // Challenging the consensus verdict
    InvalidSubmission,      // Submission doesn't meet quality standards
    BountyManipulation,    // Suspicious activity in bounty
    StakeSlashingAppeal,   // Appealing a stake slashing decision
    PayoutDispute,         // Disagreement over reward distribution
    ConsensusFailure,      // Claiming consensus mechanism failed
    MaliciousAnalysis,     // Accusing submission of being malicious/fake
}

/// Status of the dispute resolution process
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisputeStatus {
    Open,              // Recently filed, awaiting review
    UnderReview,       // Being investigated by arbitrators
    VotingPhase,       // Community/DAO voting on outcome
    Resolved,          // Decision has been made
    Rejected,          // Dispute deemed invalid
    Escalated,         // Escalated to higher authority/DAO
    Withdrawn,         // Disputer withdrew the dispute
}

/// Severity level of the dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeSeverity {
    Low,       // Minor disagreement
    Medium,    // Significant issue
    High,      // Major problem affecting bounty
    Critical,  // Systemic issue or fraud detected
}

/// Evidence supporting the dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_type: EvidenceType,
    pub description: String,
    pub data: EvidenceData,
    pub submitted_by: String,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    TechnicalAnalysis,  // Re-analysis results
    Screenshot,         // Visual proof
    Log,               // System/analysis logs
    ExpertOpinion,     // Third-party expert review
    BlockchainData,    // On-chain evidence
    CommunityReport,   // Reports from other users
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceData {
    pub file_hash: Option<String>,
    pub storage_path: Option<String>,
    pub external_url: Option<String>,
    pub inline_data: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Vote on a dispute (for community-governed resolution)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeVote {
    pub voter_id: String,
    pub vote: VoteChoice,
    pub voting_power: f32, // Based on reputation/stake
    pub reason: Option<String>,
    pub voted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteChoice {
    ApproveDispute,    // Dispute is valid
    RejectDispute,     // Dispute is invalid
    Neutral,           // Abstain
}

/// Resolution of the dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolution {
    pub decision: ResolutionDecision,
    pub reasoning: String,
    pub actions_taken: Vec<ResolutionAction>,
    pub compensation: Option<DisputeCompensation>,
    pub penalty: Option<DisputePenalty>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResolutionDecision {
    DisputeUpheld,      // Dispute was valid
    DisputeRejected,    // Dispute was invalid
    PartialResolution,  // Partially agreed with dispute
    NeedsMoreEvidence,  // Insufficient information
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionAction {
    pub action_type: String,
    pub description: String,
    pub executed_at: DateTime<Utc>,
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeCompensation {
    pub recipient: String,
    pub amount: u64,
    pub currency: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputePenalty {
    pub penalized_party: String,
    pub penalty_type: PenaltyType,
    pub amount: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PenaltyType {
    StakeSlash,
    ReputationDeduction,
    TemporaryBan,
    PermanentBan,
}

// Request/Response DTOs

#[derive(Debug, Deserialize)]
pub struct CreateDisputeRequest {
    pub bounty_id: Uuid,
    pub submission_id: Option<Uuid>,
    pub dispute_type: DisputeType,
    pub reason: String,
    pub evidence: Vec<EvidenceSubmission>,
    pub stake_amount: u64,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct EvidenceSubmission {
    pub evidence_type: EvidenceType,
    pub description: String,
    pub data: EvidenceData,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDisputeRequest {
    pub status: Option<DisputeStatus>,
    pub add_evidence: Option<Vec<EvidenceSubmission>>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveDisputeRequest {
    pub decision: ResolutionDecision,
    pub reasoning: String,
    pub actions_taken: Vec<String>,
    pub compensation: Option<DisputeCompensation>,
    pub penalty: Option<DisputePenalty>,
}

#[derive(Debug, Deserialize)]
pub struct VoteOnDisputeRequest {
    pub vote: VoteChoice,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DisputeFilters {
    pub bounty_id: Option<Uuid>,
    pub dispute_type: Option<DisputeType>,
    pub status: Option<DisputeStatus>,
    pub severity: Option<DisputeSeverity>,
    pub disputer_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DisputeListResponse {
    pub disputes: Vec<Dispute>,
    pub total_count: usize,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize)]
pub struct DisputeStatsResponse {
    pub total_disputes: u64,
    pub open_disputes: u64,
    pub resolved_disputes: u64,
    pub upheld_rate: f32, // Percentage of disputes that were upheld
    pub avg_resolution_time_hours: f32,
    pub disputes_by_type: HashMap<String, u64>,
}

// Handler implementations

/// Create a new dispute
pub async fn create_dispute(
    State(_state): State<BountyManagerState>,
    Extension(disputer_id): Extension<String>, // From auth middleware
    Json(req): Json<CreateDisputeRequest>,
) -> Result<Json<ApiResponse<Dispute>>, StatusCode> {
    // Validate request
    if req.reason.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if req.stake_amount == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Validate bounty exists
    // TODO: If submission_id provided, validate submission exists
    // TODO: Check disputer has sufficient stake
    // TODO: Check disputer hasn't already disputed this
    // TODO: Verify dispute is raised within allowed timeframe

    let dispute_id = Uuid::new_v4();
    let now = Utc::now();

    // Convert evidence submissions to evidence
    let evidence: Vec<Evidence> = req.evidence.into_iter().map(|e| Evidence {
        evidence_type: e.evidence_type,
        description: e.description,
        data: e.data,
        submitted_by: disputer_id.clone(),
        submitted_at: now,
    }).collect();

    // Determine severity based on dispute type
    let severity = match req.dispute_type {
        DisputeType::BountyManipulation | DisputeType::MaliciousAnalysis => DisputeSeverity::Critical,
        DisputeType::IncorrectVerdict | DisputeType::ConsensusFailure => DisputeSeverity::High,
        DisputeType::PayoutDispute | DisputeType::StakeSlashingAppeal => DisputeSeverity::Medium,
        DisputeType::InvalidSubmission => DisputeSeverity::Low,
    };

    let dispute = Dispute {
        id: dispute_id,
        bounty_id: req.bounty_id,
        submission_id: req.submission_id,
        disputer_id: disputer_id.clone(),
        dispute_type: req.dispute_type,
        reason: req.reason,
        evidence,
        status: DisputeStatus::Open,
        severity,
        stake_amount: req.stake_amount,
        created_at: now,
        updated_at: now,
        resolved_at: None,
        resolver_id: None,
        resolution: None,
        votes: Vec::new(),
        metadata: req.metadata.unwrap_or_default(),
    };

    // TODO: Save to database
    // TODO: Create blockchain transaction for dispute stake
    // TODO: Emit dispute created event
    // TODO: Notify relevant parties

    let response = ApiResponse {
        success: true,
        data: Some(dispute),
        message: Some("Dispute created successfully".to_string()),
        errors: None,
    };

    Ok(Json(response))
}

/// Get a specific dispute by ID
pub async fn get_dispute(
    State(_state): State<BountyManagerState>,
    Path(dispute_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Dispute>>, StatusCode> {
    // TODO: Fetch from database
    let mock_dispute = create_mock_dispute(dispute_id);

    let response = ApiResponse {
        success: true,
        data: Some(mock_dispute),
        message: None,
        errors: None,
    };

    Ok(Json(response))
}

/// List disputes with filters and pagination
pub async fn list_disputes(
    State(_state): State<BountyManagerState>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<DisputeFilters>,
) -> Result<Json<ApiResponse<DisputeListResponse>>, StatusCode> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20).min(100);

    // TODO: Implement database query with filters
    let disputes = vec![
        create_mock_dispute(Uuid::new_v4()),
        create_mock_dispute(Uuid::new_v4()),
    ];

    let response_data = DisputeListResponse {
        disputes: disputes.clone(),
        total_count: disputes.len(),
        page,
        per_page,
    };

    let response = ApiResponse {
        success: true,
        data: Some(response_data),
        message: None,
        errors: None,
    };

    Ok(Json(response))
}

/// Update a dispute (add evidence, change status)
pub async fn update_dispute(
    State(_state): State<BountyManagerState>,
    Extension(user_id): Extension<String>,
    Path(dispute_id): Path<Uuid>,
    Json(req): Json<UpdateDisputeRequest>,
) -> Result<Json<ApiResponse<Dispute>>, StatusCode> {
    // TODO: Fetch dispute from database
    let mut dispute = create_mock_dispute(dispute_id);

    // TODO: Check user has permission to update (disputer or admin)
    if dispute.disputer_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Apply updates
    if let Some(status) = req.status {
        // TODO: Validate status transition is allowed
        dispute.status = status;
    }

    if let Some(new_evidence) = req.add_evidence {
        let now = Utc::now();
        for e in new_evidence {
            dispute.evidence.push(Evidence {
                evidence_type: e.evidence_type,
                description: e.description,
                data: e.data,
                submitted_by: user_id.clone(),
                submitted_at: now,
            });
        }
    }

    dispute.updated_at = Utc::now();

    // TODO: Save to database
    // TODO: Emit update event

    let response = ApiResponse {
        success: true,
        data: Some(dispute),
        message: Some("Dispute updated successfully".to_string()),
        errors: None,
    };

    Ok(Json(response))
}

/// Resolve a dispute (admin/arbitrator only)
pub async fn resolve_dispute(
    State(_state): State<BountyManagerState>,
    Extension(resolver_id): Extension<String>,
    Path(dispute_id): Path<Uuid>,
    Json(req): Json<ResolveDisputeRequest>,
) -> Result<Json<ApiResponse<Dispute>>, StatusCode> {
    // TODO: Verify resolver has arbitrator/admin role
    // TODO: Fetch dispute from database
    let mut dispute = create_mock_dispute(dispute_id);

    // TODO: Validate dispute is in a resolvable state
    if dispute.status == DisputeStatus::Resolved {
        return Err(StatusCode::BAD_REQUEST);
    }

    let now = Utc::now();

    let actions: Vec<ResolutionAction> = req.actions_taken.into_iter().map(|desc| {
        ResolutionAction {
            action_type: "manual_action".to_string(),
            description: desc,
            executed_at: now,
            transaction_hash: None,
        }
    }).collect();

    let resolution = DisputeResolution {
        decision: req.decision,
        reasoning: req.reasoning,
        actions_taken: actions,
        compensation: req.compensation,
        penalty: req.penalty,
    };

    dispute.resolution = Some(resolution);
    dispute.status = DisputeStatus::Resolved;
    dispute.resolver_id = Some(resolver_id);
    dispute.resolved_at = Some(now);
    dispute.updated_at = now;

    // TODO: Save to database
    // TODO: Execute compensation/penalty transactions
    // TODO: Update related bounty/submission states
    // TODO: Emit resolution event
    // TODO: Notify all parties

    let response = ApiResponse {
        success: true,
        data: Some(dispute),
        message: Some("Dispute resolved successfully".to_string()),
        errors: None,
    };

    Ok(Json(response))
}

/// Vote on a dispute (for community governance)
pub async fn vote_on_dispute(
    State(_state): State<BountyManagerState>,
    Extension(voter_id): Extension<String>,
    Path(dispute_id): Path<Uuid>,
    Json(req): Json<VoteOnDisputeRequest>,
) -> Result<Json<ApiResponse<Dispute>>, StatusCode> {
    // TODO: Fetch dispute from database
    let mut dispute = create_mock_dispute(dispute_id);

    // TODO: Verify dispute is in voting phase
    if dispute.status != DisputeStatus::VotingPhase {
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Check voter hasn't already voted
    if dispute.votes.iter().any(|v| v.voter_id == voter_id) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Calculate voting power based on reputation/stake
    let voting_power = 1.0; // Placeholder

    let vote = DisputeVote {
        voter_id: voter_id.clone(),
        vote: req.vote,
        voting_power,
        reason: req.reason,
        voted_at: Utc::now(),
    };

    dispute.votes.push(vote);
    dispute.updated_at = Utc::now();

    // TODO: Save to database
    // TODO: Check if voting threshold reached
    // TODO: If threshold reached, automatically resolve

    let response = ApiResponse {
        success: true,
        data: Some(dispute),
        message: Some("Vote recorded successfully".to_string()),
        errors: None,
    };

    Ok(Json(response))
}

/// Withdraw a dispute (disputer only)
pub async fn withdraw_dispute(
    State(_state): State<BountyManagerState>,
    Extension(user_id): Extension<String>,
    Path(dispute_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Dispute>>, StatusCode> {
    // TODO: Fetch dispute from database
    let mut dispute = create_mock_dispute(dispute_id);

    // Verify user is the disputer
    if dispute.disputer_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // TODO: Validate dispute can be withdrawn (not already resolved)
    if dispute.status == DisputeStatus::Resolved {
        return Err(StatusCode::BAD_REQUEST);
    }

    dispute.status = DisputeStatus::Withdrawn;
    dispute.updated_at = Utc::now();

    // TODO: Save to database
    // TODO: Return stake to disputer
    // TODO: Emit withdrawal event

    let response = ApiResponse {
        success: true,
        data: Some(dispute),
        message: Some("Dispute withdrawn successfully".to_string()),
        errors: None,
    };

    Ok(Json(response))
}

/// Get dispute statistics
pub async fn get_dispute_stats(
    State(_state): State<BountyManagerState>,
) -> Result<Json<ApiResponse<DisputeStatsResponse>>, StatusCode> {
    // TODO: Implement real statistics from database
    let mut disputes_by_type = HashMap::new();
    disputes_by_type.insert("IncorrectVerdict".to_string(), 45);
    disputes_by_type.insert("InvalidSubmission".to_string(), 23);
    disputes_by_type.insert("StakeSlashingAppeal".to_string(), 12);

    let stats = DisputeStatsResponse {
        total_disputes: 80,
        open_disputes: 8,
        resolved_disputes: 67,
        upheld_rate: 0.62, // 62% of disputes were upheld
        avg_resolution_time_hours: 18.5,
        disputes_by_type,
    };

    let response = ApiResponse {
        success: true,
        data: Some(stats),
        message: None,
        errors: None,
    };

    Ok(Json(response))
}

// Helper function for mock data
fn create_mock_dispute(id: Uuid) -> Dispute {
    let now = Utc::now();

    Dispute {
        id,
        bounty_id: Uuid::new_v4(),
        submission_id: Some(Uuid::new_v4()),
        disputer_id: "engine_456".to_string(),
        dispute_type: DisputeType::IncorrectVerdict,
        reason: "The consensus verdict appears to be incorrect based on additional analysis. I have re-analyzed the artifact and found evidence suggesting a different classification.".to_string(),
        evidence: vec![
            Evidence {
                evidence_type: EvidenceType::TechnicalAnalysis,
                description: "Re-analysis with updated signatures shows benign behavior".to_string(),
                data: EvidenceData {
                    file_hash: Some("sha256:def789...".to_string()),
                    storage_path: Some("/evidence/analysis_report.json".to_string()),
                    external_url: None,
                    inline_data: None,
                    metadata: HashMap::new(),
                },
                submitted_by: "engine_456".to_string(),
                submitted_at: now,
            }
        ],
        status: DisputeStatus::UnderReview,
        severity: DisputeSeverity::High,
        stake_amount: 25000,
        created_at: now - chrono::Duration::hours(6),
        updated_at: now - chrono::Duration::hours(2),
        resolved_at: None,
        resolver_id: None,
        resolution: None,
        votes: vec![
            DisputeVote {
                voter_id: "community_member_1".to_string(),
                vote: VoteChoice::ApproveDispute,
                voting_power: 1.5,
                reason: Some("Additional evidence is compelling".to_string()),
                voted_at: now - chrono::Duration::hours(1),
            }
        ],
        metadata: HashMap::new(),
    }
}
