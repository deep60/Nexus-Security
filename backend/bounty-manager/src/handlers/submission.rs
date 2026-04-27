// backend/bounty-manager/src/handlers/submission_handler.rs

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
use shared::types::ApiResponse;
use super::bounty_crud::PaginationParams;
use crate::handlers::bounty_crud::{BountyManagerState, ThreatVerdict};
use crate::models::SubmissionModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub engine_id: String,
    pub engine_type: EngineType,
    pub verdict: ThreatVerdict,
    pub confidence: f32, // 0.0 to 1.0
    pub stake_amount: u64,
    pub analysis_details: AnalysisDetails,
    pub status: SubmissionStatus,
    pub transaction_hash: Option<String>, // Blockchain transaction for stake
    pub submitted_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub accuracy_score: Option<f32>, // Calculated after consensus
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineType {
    Human,      // Security expert
    Automated,  // AI/ML engine
    Hybrid,     // Combination
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubmissionStatus {
    Pending,     // Awaiting blockchain confirmation
    Active,      // Stake confirmed, participating in consensus
    Correct,     // Matched final consensus
    Incorrect,   // Did not match consensus
    Invalid,     // Failed validation checks
    Slashed,     // Stake was slashed for incorrect analysis
    Rewarded,    // Received reward for correct analysis
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisDetails {
    pub malware_families: Vec<String>,
    pub threat_indicators: Vec<ThreatIndicator>,
    pub behavioral_analysis: Option<BehavioralAnalysis>,
    pub static_analysis: Option<StaticAnalysis>,
    pub network_analysis: Option<NetworkAnalysis>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    pub indicator_type: String, // "hash", "ip", "domain", "registry_key", etc.
    pub value: String,
    pub severity: ThreatSeverity,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralAnalysis {
    pub network_connections: Vec<String>,
    pub file_operations: Vec<String>,
    pub registry_modifications: Vec<String>,
    pub process_creation: Vec<String>,
    pub api_calls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysis {
    pub file_entropy: Option<f32>,
    pub pe_sections: Vec<PeSection>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub strings: Vec<String>,
    pub yara_matches: Vec<YaraMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeSection {
    pub name: String,
    pub virtual_size: u64,
    pub raw_size: u64,
    pub entropy: f32,
    pub suspicious_characteristics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraMatch {
    pub rule_name: String,
    pub rule_family: String,
    pub tags: Vec<String>,
    pub matches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAnalysis {
    pub dns_requests: Vec<String>,
    pub http_requests: Vec<HttpRequest>,
    pub tcp_connections: Vec<TcpConnection>,
    pub suspicious_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConnection {
    pub destination_ip: String,
    pub destination_port: u16,
    pub protocol: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct SubmitAnalysisRequest {
    pub verdict: ThreatVerdict,
    pub confidence: f32,
    pub stake_amount: u64,
    pub analysis_details: AnalysisDetails,
    pub engine_type: EngineType,
}

#[derive(Debug, Deserialize)]
pub struct SubmissionFilters {
    pub engine_id: Option<String>,
    pub verdict: Option<ThreatVerdict>,
    pub status: Option<SubmissionStatus>,
    pub min_confidence: Option<f32>,
    pub engine_type: Option<EngineType>,
}

#[derive(Debug, Serialize)]
pub struct SubmissionListResponse {
    pub submissions: Vec<Submission>,
    pub consensus_data: Option<ConsensusData>,
    pub total_count: usize,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize)]
pub struct ConsensusData {
    pub current_consensus: Option<ThreatVerdict>,
    pub confidence_level: f32,
    pub total_submissions: u32,
    pub verdict_breakdown: HashMap<String, u32>, // verdict -> count
    pub weighted_score: f32, // Weighted by reputation and stake
}

// Handler implementations
pub async fn submit_analysis(
    State(state): State<BountyManagerState>,
    Extension(engine_id): Extension<String>,
    Path(bounty_id): Path<Uuid>,
    Json(req): Json<SubmitAnalysisRequest>,
) -> Result<Json<ApiResponse<Submission>>, StatusCode> {
    // Validate request
    if req.confidence < 0.0 || req.confidence > 1.0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    if req.stake_amount == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate bounty exists and is active
    let bounty = crate::models::BountyModel::find_by_id(&state.db, bounty_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch bounty {}: {}", bounty_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Bounty {} not found", bounty_id);
            StatusCode::NOT_FOUND
        })?;

    if bounty.status != "Active" {
        tracing::warn!("Bounty {} is not active (status: {})", bounty_id, bounty.status);
        return Err(StatusCode::CONFLICT);
    }

    if bounty.deadline < Utc::now() {
        tracing::warn!("Bounty {} deadline has passed", bounty_id);
        return Err(StatusCode::CONFLICT);
    }

    // Check if engine already submitted to this bounty
    let existing_submissions = SubmissionModel::find_by_bounty(&state.db, bounty_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check existing submissions: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if existing_submissions.iter().any(|s| s.engine_id == engine_id) {
        tracing::warn!("Engine {} already submitted to bounty {}", engine_id, bounty_id);
        return Err(StatusCode::CONFLICT);
    }

    // Verify minimum stake requirements
    if req.stake_amount < bounty.min_stake as u64 {
        tracing::warn!(
            "Stake amount {} below minimum {} for bounty {}",
            req.stake_amount, bounty.min_stake, bounty_id
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check engine reputation requirements (engines with very low reputation are rejected)
    let min_reputation = state.reputation_service
        .calculate_minimum_stake(&engine_id)
        .await;
    if let Ok(required_stake) = min_reputation {
        if req.stake_amount < required_stake {
            tracing::warn!(
                "Engine {} reputation requires minimum stake of {}, got {}",
                engine_id, required_stake, req.stake_amount
            );
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let submission_id = Uuid::new_v4();
    let now = Utc::now();

    let analysis_json = serde_json::to_value(&req.analysis_details).unwrap_or_default();

    let db_sub = SubmissionModel {
        id: submission_id,
        bounty_id,
        engine_id: engine_id.clone(),
        engine_type: format!("{:?}", req.engine_type),
        verdict: format!("{:?}", req.verdict),
        confidence: req.confidence,
        stake_amount: req.stake_amount as i64,
        analysis_details: analysis_json,
        status: "Pending".to_string(),
        transaction_hash: None,
        submitted_at: now,
        processed_at: None,
        accuracy_score: None,
    };

    let saved = SubmissionModel::create(&state.db, &db_sub)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create submission: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let submission = db_submission_to_handler_submission(saved, req.analysis_details);

    Ok(Json(ApiResponse::success(submission)))
}

pub async fn get_submission(
    State(state): State<BountyManagerState>,
    Path(submission_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Submission>>, StatusCode> {
    let db_sub = SubmissionModel::find_by_id(&state.db, submission_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch submission: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let details: AnalysisDetails = serde_json::from_value(db_sub.analysis_details.clone())
        .unwrap_or_else(|_| default_analysis_details());
    let submission = db_submission_to_handler_submission(db_sub, details);

    Ok(Json(ApiResponse::success(submission)))
}

pub async fn list_submissions_for_bounty(
    State(state): State<BountyManagerState>,
    Path(bounty_id): Path<Uuid>,
    Query(pagination): Query<PaginationParams>,
    Query(_filters): Query<SubmissionFilters>,
) -> Result<Json<ApiResponse<SubmissionListResponse>>, StatusCode> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20).min(100);

    let db_submissions = SubmissionModel::find_by_bounty(&state.db, bounty_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list submissions: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let total_count = db_submissions.len();

    // Build real consensus data from submissions
    let mut verdict_breakdown: HashMap<String, u32> = HashMap::new();
    let mut total_confidence: f32 = 0.0;
    let mut total_stake: i64 = 0;

    for s in &db_submissions {
        *verdict_breakdown.entry(s.verdict.clone()).or_insert(0) += 1;
        total_confidence += s.confidence;
        total_stake += s.stake_amount;
    }

    let avg_confidence = if total_count > 0 {
        total_confidence / total_count as f32
    } else {
        0.0
    };

    // Determine current consensus verdict (most common)
    let current_consensus = verdict_breakdown
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(verdict, _)| match verdict.as_str() {
            "Malicious" => ThreatVerdict::Malicious,
            "Benign" => ThreatVerdict::Benign,
            "Suspicious" => ThreatVerdict::Suspicious,
            _ => ThreatVerdict::Unknown,
        });

    let consensus_data = ConsensusData {
        current_consensus,
        confidence_level: avg_confidence,
        total_submissions: total_count as u32,
        verdict_breakdown,
        weighted_score: avg_confidence, // Simplified; could weight by stake/reputation
    };

    let submissions: Vec<Submission> = db_submissions
        .into_iter()
        .map(|s| {
            let details: AnalysisDetails = serde_json::from_value(s.analysis_details.clone())
                .unwrap_or_else(|_| default_analysis_details());
            db_submission_to_handler_submission(s, details)
        })
        .collect();

    let response_data = SubmissionListResponse {
        submissions,
        consensus_data: Some(consensus_data),
        total_count,
        page,
        per_page,
    };

    Ok(Json(ApiResponse::success(response_data)))
}

pub async fn update_submission_status(
    State(state): State<BountyManagerState>,
    Extension(caller_id): Extension<String>, // From auth middleware
    Path(submission_id): Path<Uuid>,
    Json(status): Json<SubmissionStatus>,
) -> Result<Json<ApiResponse<Submission>>, StatusCode> {
    // Fetch the submission first to verify ownership
    let db_sub = SubmissionModel::find_by_id(&state.db, submission_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch submission: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Only the submission's engine owner or the bounty creator may update status
    let bounty = crate::models::BountyModel::find_by_id(&state.db, db_sub.bounty_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch bounty {}: {}", db_sub.bounty_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if db_sub.engine_id != caller_id && bounty.creator != caller_id {
        tracing::warn!(
            "Caller {} denied status update on submission {} (owner: {}, bounty creator: {})",
            caller_id, submission_id, db_sub.engine_id, bounty.creator
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // Validate status transition
    let current_status = match db_sub.status.as_str() {
        "Active" => SubmissionStatus::Active,
        "Correct" => SubmissionStatus::Correct,
        "Incorrect" => SubmissionStatus::Incorrect,
        "Invalid" => SubmissionStatus::Invalid,
        "Slashed" => SubmissionStatus::Slashed,
        "Rewarded" => SubmissionStatus::Rewarded,
        _ => SubmissionStatus::Pending,
    };

    if !is_valid_submission_transition(&current_status, &status) {
        tracing::warn!(
            "Invalid submission status transition from {:?} to {:?} for {}",
            current_status, status, submission_id
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    let status_str = format!("{:?}", status);
    SubmissionModel::update_status(&state.db, submission_id, &status_str)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update submission status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Re-fetch updated submission
    let updated = SubmissionModel::find_by_id(&state.db, submission_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to re-fetch submission: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let details: AnalysisDetails = serde_json::from_value(updated.analysis_details.clone())
        .unwrap_or_else(|_| default_analysis_details());
    let submission = db_submission_to_handler_submission(updated, details);

    Ok(Json(ApiResponse::success(submission)))
}

/// Allowed submission status transitions
fn is_valid_submission_transition(from: &SubmissionStatus, to: &SubmissionStatus) -> bool {
    matches!(
        (from, to),
        (SubmissionStatus::Pending, SubmissionStatus::Active)
        | (SubmissionStatus::Pending, SubmissionStatus::Invalid)
        | (SubmissionStatus::Active, SubmissionStatus::Correct)
        | (SubmissionStatus::Active, SubmissionStatus::Incorrect)
        | (SubmissionStatus::Active, SubmissionStatus::Invalid)
        | (SubmissionStatus::Incorrect, SubmissionStatus::Slashed)
        | (SubmissionStatus::Correct, SubmissionStatus::Rewarded)
    )
}

// Conversion helper: SubmissionModel (DB) -> Submission (handler DTO)
fn db_submission_to_handler_submission(db: SubmissionModel, details: AnalysisDetails) -> Submission {
    let engine_type = match db.engine_type.as_str() {
        "Human" => EngineType::Human,
        "Hybrid" => EngineType::Hybrid,
        _ => EngineType::Automated,
    };

    let verdict = match db.verdict.as_str() {
        "Malicious" => ThreatVerdict::Malicious,
        "Benign" => ThreatVerdict::Benign,
        "Suspicious" => ThreatVerdict::Suspicious,
        _ => ThreatVerdict::Unknown,
    };

    let status = match db.status.as_str() {
        "Active" => SubmissionStatus::Active,
        "Correct" => SubmissionStatus::Correct,
        "Incorrect" => SubmissionStatus::Incorrect,
        "Slashed" => SubmissionStatus::Slashed,
        "Rewarded" => SubmissionStatus::Rewarded,
        _ => SubmissionStatus::Pending,
    };

    Submission {
        id: db.id,
        bounty_id: db.bounty_id,
        engine_id: db.engine_id,
        engine_type,
        verdict,
        confidence: db.confidence,
        stake_amount: db.stake_amount as u64,
        analysis_details: details,
        status,
        transaction_hash: db.transaction_hash,
        submitted_at: db.submitted_at,
        processed_at: db.processed_at,
        accuracy_score: db.accuracy_score,
    }
}

fn default_analysis_details() -> AnalysisDetails {
    AnalysisDetails {
        malware_families: Vec::new(),
        threat_indicators: Vec::new(),
        behavioral_analysis: None,
        static_analysis: None,
        network_analysis: None,
        metadata: HashMap::new(),
    }
}