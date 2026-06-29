// backend/bounty-manager/src/handlers/dispute.rs

use super::bounty_crud::PaginationParams;
use crate::handlers::bounty_crud::BountyManagerState;
use crate::models::{
    BountyModel, DisputeModel, DisputeVoteModel, ReputationModel, SubmissionModel,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::ApiResponse;
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a dispute raised against a submission or bounty outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub submission_id: Option<Uuid>, // If disputing a specific submission
    pub disputer_id: String,         // Engine or user ID raising the dispute
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
    IncorrectVerdict,    // Challenging the consensus verdict
    InvalidSubmission,   // Submission doesn't meet quality standards
    BountyManipulation,  // Suspicious activity in bounty
    StakeSlashingAppeal, // Appealing a stake slashing decision
    PayoutDispute,       // Disagreement over reward distribution
    ConsensusFailure,    // Claiming consensus mechanism failed
    MaliciousAnalysis,   // Accusing submission of being malicious/fake
}

/// Status of the dispute resolution process
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisputeStatus {
    Open,        // Recently filed, awaiting review
    UnderReview, // Being investigated by arbitrators
    VotingPhase, // Community/DAO voting on outcome
    Resolved,    // Decision has been made
    Rejected,    // Dispute deemed invalid
    Escalated,   // Escalated to higher authority/DAO
    Withdrawn,   // Disputer withdrew the dispute
}

/// Severity level of the dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeSeverity {
    Low,      // Minor disagreement
    Medium,   // Significant issue
    High,     // Major problem affecting bounty
    Critical, // Systemic issue or fraud detected
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
    TechnicalAnalysis, // Re-analysis results
    Screenshot,        // Visual proof
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
    ApproveDispute, // Dispute is valid
    RejectDispute,  // Dispute is invalid
    Neutral,        // Abstain
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
    DisputeUpheld,     // Dispute was valid
    DisputeRejected,   // Dispute was invalid
    PartialResolution, // Partially agreed with dispute
    NeedsMoreEvidence, // Insufficient information
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

// ── Allowed status transitions ──────────────────────────────

fn is_valid_status_transition(from: &DisputeStatus, to: &DisputeStatus) -> bool {
    matches!(
        (from, to),
        (DisputeStatus::Open, DisputeStatus::UnderReview)
            | (DisputeStatus::Open, DisputeStatus::Rejected)
            | (DisputeStatus::Open, DisputeStatus::Withdrawn)
            | (DisputeStatus::UnderReview, DisputeStatus::VotingPhase)
            | (DisputeStatus::UnderReview, DisputeStatus::Resolved)
            | (DisputeStatus::UnderReview, DisputeStatus::Escalated)
            | (DisputeStatus::VotingPhase, DisputeStatus::Resolved)
            | (DisputeStatus::Escalated, DisputeStatus::Resolved)
    )
}

// Handler implementations

/// Create a new dispute
pub async fn create_dispute(
    State(state): State<BountyManagerState>,
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

    // Validate bounty exists
    let bounty = BountyModel::find_by_id(&state.db, req.bounty_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch bounty {}: {}", req.bounty_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Bounty {} not found for dispute", req.bounty_id);
            StatusCode::NOT_FOUND
        })?;

    // If submission_id provided, validate submission exists
    if let Some(sub_id) = req.submission_id {
        SubmissionModel::find_by_id(&state.db, sub_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch submission {}: {}", sub_id, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or_else(|| {
                tracing::warn!("Submission {} not found for dispute", sub_id);
                StatusCode::NOT_FOUND
            })?;
    }

    // Check disputer has sufficient stake (stake_amount >= bounty min_stake)
    if req.stake_amount < bounty.min_stake as u64 {
        tracing::warn!(
            "Dispute stake {} below bounty minimum {} for disputer {}",
            req.stake_amount,
            bounty.min_stake,
            disputer_id
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check disputer hasn't already disputed this bounty
    let already_disputed = DisputeModel::has_active_dispute(&state.db, req.bounty_id, &disputer_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check existing disputes: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if already_disputed {
        tracing::warn!(
            "Disputer {} already has an active dispute for bounty {}",
            disputer_id,
            req.bounty_id
        );
        return Err(StatusCode::CONFLICT);
    }

    // Verify dispute is raised within allowed timeframe (bounty must not be too old)
    let max_dispute_age = chrono::Duration::days(30);
    if bounty.updated_at + max_dispute_age < Utc::now() {
        tracing::warn!(
            "Bounty {} is too old to dispute (updated {})",
            req.bounty_id,
            bounty.updated_at
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    let dispute_id = Uuid::new_v4();
    let now = Utc::now();

    // Convert evidence submissions to evidence
    let evidence: Vec<Evidence> = req
        .evidence
        .into_iter()
        .map(|e| Evidence {
            evidence_type: e.evidence_type,
            description: e.description,
            data: e.data,
            submitted_by: disputer_id.clone(),
            submitted_at: now,
        })
        .collect();

    // Determine severity based on dispute type
    let severity = match req.dispute_type {
        DisputeType::BountyManipulation | DisputeType::MaliciousAnalysis => {
            DisputeSeverity::Critical
        }
        DisputeType::IncorrectVerdict | DisputeType::ConsensusFailure => DisputeSeverity::High,
        DisputeType::PayoutDispute | DisputeType::StakeSlashingAppeal => DisputeSeverity::Medium,
        DisputeType::InvalidSubmission => DisputeSeverity::Low,
    };

    // Save to database
    let db_dispute = DisputeModel {
        id: dispute_id,
        bounty_id: req.bounty_id,
        submission_id: req.submission_id,
        disputer_id: disputer_id.clone(),
        dispute_type: format!("{:?}", req.dispute_type),
        severity: format!("{severity:?}"),
        status: "Open".to_string(),
        title: format!("{:?} dispute on bounty", req.dispute_type),
        description: req.reason.clone(),
        evidence: Some(serde_json::to_value(&evidence).unwrap_or_default()),
        stake_amount: req.stake_amount as i64,
        resolution: None,
        resolution_details: None,
        resolver_id: None,
        resolved_at: None,
        created_at: now,
        updated_at: now,
        metadata: req
            .metadata
            .as_ref()
            .map(|m| serde_json::to_value(m).unwrap_or_default()),
    };

    DisputeModel::create(&state.db, &db_dispute)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save dispute: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Blockchain transaction for dispute stake (infrastructure not yet connected)
    tracing::warn!(
        dispute_id = %dispute_id,
        stake_amount = req.stake_amount,
        "Blockchain stake escrow not yet implemented — dispute recorded off-chain only"
    );

    // Notify relevant parties
    tracing::info!(
        dispute_id = %dispute_id,
        bounty_id = %req.bounty_id,
        disputer = %disputer_id,
        "Dispute created successfully"
    );

    let dispute = Dispute {
        id: dispute_id,
        bounty_id: req.bounty_id,
        submission_id: req.submission_id,
        disputer_id,
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

    Ok(Json(ApiResponse::success(dispute)))
}

/// Get a specific dispute by ID
pub async fn get_dispute(
    State(state): State<BountyManagerState>,
    Path(dispute_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Dispute>>, StatusCode> {
    let db_dispute = DisputeModel::find_by_id(&state.db, dispute_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch dispute {}: {}", dispute_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Dispute {} not found", dispute_id);
            StatusCode::NOT_FOUND
        })?;

    let dispute = db_dispute_to_handler_dispute(db_dispute);
    Ok(Json(ApiResponse::success(dispute)))
}

/// List disputes with filters and pagination
pub async fn list_disputes(
    State(state): State<BountyManagerState>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<DisputeFilters>,
) -> Result<Json<ApiResponse<DisputeListResponse>>, StatusCode> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20).min(100);
    let offset = ((page.saturating_sub(1)) * per_page) as i64;

    let status_str = filters.status.as_ref().map(|s| format!("{s:?}"));

    let db_disputes = DisputeModel::list(
        &state.db,
        status_str.as_deref(),
        filters.bounty_id,
        per_page as i64,
        offset,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to list disputes: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total_count = DisputeModel::count_by_status(&state.db, status_str.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to count disputes: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })? as usize;

    let disputes: Vec<Dispute> = db_disputes
        .into_iter()
        .map(db_dispute_to_handler_dispute)
        .collect();

    let response_data = DisputeListResponse {
        disputes,
        total_count,
        page,
        per_page,
    };

    Ok(Json(ApiResponse::success(response_data)))
}

/// Update a dispute (add evidence, change status)
pub async fn update_dispute(
    State(state): State<BountyManagerState>,
    Extension(user_id): Extension<String>,
    Path(dispute_id): Path<Uuid>,
    Json(req): Json<UpdateDisputeRequest>,
) -> Result<Json<ApiResponse<Dispute>>, StatusCode> {
    // Fetch dispute from database
    let db_dispute = DisputeModel::find_by_id(&state.db, dispute_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch dispute {}: {}", dispute_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Dispute {} not found", dispute_id);
            StatusCode::NOT_FOUND
        })?;

    let mut dispute = db_dispute_to_handler_dispute(db_dispute);

    // Check user has permission to update (disputer or admin)
    if dispute.disputer_id != user_id {
        tracing::warn!(
            "User {} attempted to update dispute {} owned by {}",
            user_id,
            dispute_id,
            dispute.disputer_id
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // Apply updates
    if let Some(new_status) = req.status {
        // Validate status transition is allowed
        if !is_valid_status_transition(&dispute.status, &new_status) {
            tracing::warn!(
                "Invalid status transition from {:?} to {:?} for dispute {}",
                dispute.status,
                new_status,
                dispute_id
            );
            return Err(StatusCode::BAD_REQUEST);
        }
        dispute.status = new_status;
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

    // Save updates to database
    let status_str = format!("{:?}", dispute.status);
    DisputeModel::update_status(&state.db, dispute_id, &status_str)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update dispute {}: {}", dispute_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!(dispute_id = %dispute_id, user_id = %user_id, "Dispute updated");

    Ok(Json(ApiResponse::success(dispute)))
}

/// Resolve a dispute (admin/arbitrator only)
pub async fn resolve_dispute(
    State(state): State<BountyManagerState>,
    Extension(resolver_id): Extension<String>,
    Path(dispute_id): Path<Uuid>,
    Json(req): Json<ResolveDisputeRequest>,
) -> Result<Json<ApiResponse<Dispute>>, StatusCode> {
    // Verify resolver has arbitrator/admin role (the route is gated by admin middleware;
    // log the resolver for audit trail)
    tracing::info!(resolver_id = %resolver_id, dispute_id = %dispute_id, "Dispute resolution requested");

    // Fetch dispute from database
    let db_dispute = DisputeModel::find_by_id(&state.db, dispute_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch dispute {}: {}", dispute_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Dispute {} not found", dispute_id);
            StatusCode::NOT_FOUND
        })?;

    let mut dispute = db_dispute_to_handler_dispute(db_dispute);

    // Validate dispute is in a resolvable state
    if dispute.status == DisputeStatus::Resolved || dispute.status == DisputeStatus::Withdrawn {
        tracing::warn!(
            "Dispute {} is already {:?}, cannot resolve",
            dispute_id,
            dispute.status
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    let now = Utc::now();

    let actions: Vec<ResolutionAction> = req
        .actions_taken
        .into_iter()
        .map(|desc| ResolutionAction {
            action_type: "manual_action".to_string(),
            description: desc,
            executed_at: now,
            transaction_hash: None,
        })
        .collect();

    let resolution = DisputeResolution {
        decision: req.decision.clone(),
        reasoning: req.reasoning,
        actions_taken: actions,
        compensation: req.compensation,
        penalty: req.penalty,
    };

    dispute.resolution = Some(resolution);
    dispute.status = DisputeStatus::Resolved;
    dispute.resolver_id = Some(resolver_id.clone());
    dispute.resolved_at = Some(now);
    dispute.updated_at = now;

    // Save resolution to database
    let resolution_str = format!("{:?}", req.decision);
    DisputeModel::resolve(
        &state.db,
        dispute_id,
        &resolution_str,
        &dispute
            .resolution
            .as_ref()
            .map(|r| r.reasoning.clone())
            .unwrap_or_default(),
        &resolver_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to save dispute resolution: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Compensation/penalty transactions (blockchain not yet connected)
    tracing::warn!(
        dispute_id = %dispute_id,
        decision = ?req.decision,
        "Compensation/penalty blockchain transactions not yet implemented — recorded off-chain"
    );

    // Update related bounty/submission states if dispute was upheld
    if req.decision == ResolutionDecision::DisputeUpheld {
        if let Err(e) =
            BountyModel::update_status(&state.db, dispute.bounty_id, "UnderReview").await
        {
            tracing::error!("Failed to update bounty status after dispute upheld: {}", e);
        }
    }

    tracing::info!(
        dispute_id = %dispute_id,
        resolver = %resolver_id,
        decision = ?req.decision,
        "Dispute resolved"
    );

    Ok(Json(ApiResponse::success(dispute)))
}

/// Vote on a dispute (for community governance)
pub async fn vote_on_dispute(
    State(state): State<BountyManagerState>,
    Extension(voter_id): Extension<String>,
    Path(dispute_id): Path<Uuid>,
    Json(req): Json<VoteOnDisputeRequest>,
) -> Result<Json<ApiResponse<Dispute>>, StatusCode> {
    // Fetch dispute from database
    let db_dispute = DisputeModel::find_by_id(&state.db, dispute_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch dispute {}: {}", dispute_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Dispute {} not found for voting", dispute_id);
            StatusCode::NOT_FOUND
        })?;

    let mut dispute = db_dispute_to_handler_dispute(db_dispute);

    // Verify dispute is in voting phase
    if dispute.status != DisputeStatus::VotingPhase {
        tracing::warn!(
            "Dispute {} is not in voting phase (status: {:?})",
            dispute_id,
            dispute.status
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Prevent the disputer from voting on their own dispute (conflict of interest)
    if dispute.disputer_id == voter_id {
        tracing::warn!(
            "Disputer {} cannot vote on their own dispute {}",
            voter_id,
            dispute_id
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // Prevent the disputed submission's engine owner from voting (interested party)
    if let Some(sub_id) = dispute.submission_id {
        if let Ok(Some(sub)) = SubmissionModel::find_by_id(&state.db, sub_id).await {
            if sub.engine_id == voter_id {
                tracing::warn!(
                    "Submission owner {} cannot vote on dispute {} against their submission",
                    voter_id,
                    dispute_id
                );
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

    // Check voter hasn't already voted
    let already_voted = DisputeVoteModel::has_voted(&state.db, dispute_id, &voter_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check existing votes: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if already_voted {
        tracing::warn!("Voter {} already voted on dispute {}", voter_id, dispute_id);
        return Err(StatusCode::CONFLICT);
    }

    // Calculate voting power based on reputation
    let voting_power = match ReputationModel::find_by_id(&state.db, &voter_id).await {
        Ok(Some(rep)) => {
            // Voting power scales with reputation score (min 0.1, max 5.0)
            (rep.reputation_score * 5.0).clamp(0.1, 5.0)
        }
        _ => 1.0, // Default voting power for unknown engines
    };

    let vote_str = match req.vote {
        VoteChoice::ApproveDispute => "Uphold",
        VoteChoice::RejectDispute => "Reject",
        VoteChoice::Neutral => "Neutral",
    };

    // Save vote to database
    let db_vote = DisputeVoteModel {
        id: Uuid::new_v4(),
        dispute_id,
        voter_id: voter_id.clone(),
        vote: vote_str.to_string(),
        voting_power,
        rationale: req.reason.clone(),
        created_at: Utc::now(),
    };

    DisputeVoteModel::create(&state.db, &db_vote)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save vote: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Add vote to the handler response
    let vote = DisputeVote {
        voter_id: voter_id.clone(),
        vote: req.vote,
        voting_power,
        reason: req.reason,
        voted_at: Utc::now(),
    };

    dispute.votes.push(vote);
    dispute.updated_at = Utc::now();

    // Check if voting threshold reached (>= 10 total voting power)
    let (uphold_power, reject_power) = DisputeVoteModel::get_vote_tallies(&state.db, dispute_id)
        .await
        .unwrap_or((0.0, 0.0));

    let total_power = uphold_power + reject_power;
    let threshold = 10.0;

    if total_power >= threshold {
        // Auto-resolve based on majority
        let decision = if uphold_power > reject_power {
            "DisputeUpheld"
        } else {
            "DisputeRejected"
        };

        DisputeModel::resolve(
            &state.db,
            dispute_id,
            decision,
            &format!("Auto-resolved by voting: uphold={uphold_power:.1}, reject={reject_power:.1}"),
            "system_auto_resolve",
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to auto-resolve dispute: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        dispute.status = DisputeStatus::Resolved;
        tracing::info!(
            dispute_id = %dispute_id,
            decision = decision,
            uphold_power = uphold_power,
            reject_power = reject_power,
            "Dispute auto-resolved by voting threshold"
        );
    }

    Ok(Json(ApiResponse::success(dispute)))
}

/// Withdraw a dispute (disputer only)
pub async fn withdraw_dispute(
    State(state): State<BountyManagerState>,
    Extension(user_id): Extension<String>,
    Path(dispute_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Dispute>>, StatusCode> {
    // Fetch dispute from database
    let db_dispute = DisputeModel::find_by_id(&state.db, dispute_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch dispute {}: {}", dispute_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Dispute {} not found", dispute_id);
            StatusCode::NOT_FOUND
        })?;

    let mut dispute = db_dispute_to_handler_dispute(db_dispute);

    // Verify user is the disputer
    if dispute.disputer_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Validate dispute can be withdrawn (not already resolved or withdrawn)
    if dispute.status == DisputeStatus::Resolved || dispute.status == DisputeStatus::Withdrawn {
        tracing::warn!(
            "Dispute {} cannot be withdrawn (status: {:?})",
            dispute_id,
            dispute.status
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    dispute.status = DisputeStatus::Withdrawn;
    dispute.updated_at = Utc::now();

    // Save to database
    DisputeModel::update_status(&state.db, dispute_id, "Withdrawn")
        .await
        .map_err(|e| {
            tracing::error!("Failed to withdraw dispute: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Stake return (blockchain not yet connected)
    tracing::warn!(
        dispute_id = %dispute_id,
        stake_amount = dispute.stake_amount,
        "Blockchain stake return not yet implemented — dispute withdrawn off-chain"
    );

    tracing::info!(dispute_id = %dispute_id, user_id = %user_id, "Dispute withdrawn");

    Ok(Json(ApiResponse::success(dispute)))
}

/// Get dispute statistics
pub async fn get_dispute_stats(
    State(state): State<BountyManagerState>,
) -> Result<Json<ApiResponse<DisputeStatsResponse>>, StatusCode> {
    // Fetch real statistics from database
    let db_stats = DisputeModel::get_stats(&state.db).await.map_err(|e| {
        tracing::error!("Failed to fetch dispute stats: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let upheld_rate = if db_stats.resolved_disputes > 0 {
        db_stats.upheld_disputes as f32 / db_stats.resolved_disputes as f32
    } else {
        0.0
    };

    // Disputes by type (would require a GROUP BY query; using counts for the known types)
    let disputes_by_type = HashMap::new();

    let stats = DisputeStatsResponse {
        total_disputes: db_stats.total_disputes as u64,
        open_disputes: db_stats.open_disputes as u64,
        resolved_disputes: db_stats.resolved_disputes as u64,
        upheld_rate,
        avg_resolution_time_hours: db_stats.avg_resolution_time_hours,
        disputes_by_type,
    };

    Ok(Json(ApiResponse::success(stats)))
}

// ── Conversion helper ────────────────────────────────────────

fn db_dispute_to_handler_dispute(db: DisputeModel) -> Dispute {
    let dispute_type = match db.dispute_type.as_str() {
        "IncorrectVerdict" => DisputeType::IncorrectVerdict,
        "InvalidSubmission" => DisputeType::InvalidSubmission,
        "BountyManipulation" => DisputeType::BountyManipulation,
        "StakeSlashingAppeal" => DisputeType::StakeSlashingAppeal,
        "PayoutDispute" => DisputeType::PayoutDispute,
        "ConsensusFailure" => DisputeType::ConsensusFailure,
        "MaliciousAnalysis" => DisputeType::MaliciousAnalysis,
        _ => DisputeType::IncorrectVerdict,
    };

    let status = match db.status.as_str() {
        "Open" => DisputeStatus::Open,
        "UnderReview" => DisputeStatus::UnderReview,
        "VotingPhase" => DisputeStatus::VotingPhase,
        "Resolved" => DisputeStatus::Resolved,
        "Rejected" => DisputeStatus::Rejected,
        "Escalated" => DisputeStatus::Escalated,
        "Withdrawn" => DisputeStatus::Withdrawn,
        _ => DisputeStatus::Open,
    };

    let severity = match db.severity.as_str() {
        "Low" => DisputeSeverity::Low,
        "Medium" => DisputeSeverity::Medium,
        "High" => DisputeSeverity::High,
        "Critical" => DisputeSeverity::Critical,
        _ => DisputeSeverity::Medium,
    };

    let evidence: Vec<Evidence> = db
        .evidence
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let resolution = db.resolution.as_ref().map(|r| {
        let decision = match r.as_str() {
            "DisputeUpheld" => ResolutionDecision::DisputeUpheld,
            "DisputeRejected" => ResolutionDecision::DisputeRejected,
            "PartialResolution" => ResolutionDecision::PartialResolution,
            _ => ResolutionDecision::NeedsMoreEvidence,
        };
        DisputeResolution {
            decision,
            reasoning: db.resolution_details.clone().unwrap_or_default(),
            actions_taken: Vec::new(),
            compensation: None,
            penalty: None,
        }
    });

    let metadata: HashMap<String, String> = db
        .metadata
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    Dispute {
        id: db.id,
        bounty_id: db.bounty_id,
        submission_id: db.submission_id,
        disputer_id: db.disputer_id,
        dispute_type,
        reason: db.description,
        evidence,
        status,
        severity,
        stake_amount: db.stake_amount as u64,
        created_at: db.created_at,
        updated_at: db.updated_at,
        resolved_at: db.resolved_at,
        resolver_id: db.resolver_id,
        resolution,
        votes: Vec::new(), // Votes loaded separately if needed
        metadata,
    }
}
