use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::models::bounty::{Bounty, BountyStatus, BountySubmission, EngineVerdict};
use crate::services::{database::DatabaseService, blockchain::BlockchainService};

// Request/Response DTOs
#[derive(Deserialize)]
pub struct CreateBountyRequest {
    pub title: String,
    pub description: String,
    pub file_hash: Option<String>,
    pub url: Option<String>,
    pub reward_amount: u64,       // Amount in Wei
    pub deadline: DateTime<Utc>,
    pub required_consensus: u8,          // Minimum number of engines needed
    pub confidence_threshold: f32,       // Minimum confidence score (0.0-1.0)
}

#[derive(Deserialize)]
pub struct SubmitAnalysisRequest {
    pub engine_id: String,
    pub verdict: String,          // "malicious", "benign", "suspicious"
    pub confidence: f32,          // 0.0-1.0
    pub analysis_details: serde_json::Value,
    pub stake_amount: u64,
}
#[derive(Deserialize)]
pub struct BountyFilters {
    pub status: Option<String>,
    pub min_reward: Option<u64>,
    pub max_reward: Option<u64>,
    pub category: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}
#[derive(Serialize)]
pub struct BountyResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub creator: String,
    pub reward_amount: u64,
    pub current_pool: u64,
    pub status: BountyStatus,
    pub created_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub submission_count: u32,
    pub consensus_reached: bool,
    pub final_verdict: Option<String>,
    pub confidence_score: Option<f32>,
}
#[derive(Serialize)]
pub struct BountListResponse {
    pub bounties: Vec<BountyResponse>,
    pub total_count: u32,
    pub page: u32,
    pub limit: u32,
}

#[derive(Serialize)]
pub struct SubmissionResponse {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub engine_id: String,
    pub verdict: String,
    pub confidence: f32,
    pub stake_amount: u64,
    pub submitted_at: DateTime<Utc>,
    pub is_winner: Option<bool>,
}

#[derive(Serialize)]
pub struct BountyDetailsResponse {
    pub bounty: BountyResponse,
    pub submissions: Vec<SubmissionResponse>,
    pub file_info: Option<FileInfo>,
}

#[derive(Serialize)]
pub struct FileInfo {
    pub hash: String,
    pub size: u64,
    pub file_type: String,
    pub upload_timestamp: DateTime<Utc>,
}

// Application state
pub struct AppState {
    pub db: DatabaseService,
    pub blockchain: BlockchainService,
}

// handler Implementation
pub async fn create_bounty(
    State(state): State<std::sync::Arc<AppState>>,
    Json(request): Json<CreateBountyRequest>,
) -> Result<Json<BountyResponse>, StatusCode> {
    // Validate request
    if request.reward_amount == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    if request.deadline <= Utc::now() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if request.file_hash.is_none() && request.url.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Extract user ID from JWT token
    let creator_id = "user_placeholder".to_string();

    // Create bounty in database
    let bounty = Bounty {
        id: Uuid::new_v4(),
        title: request.title,
        description: request.description,
        creator: creator_id,
        file_hash: request.file_hash,
        url: request.url,
        reward_amount: request.reward_amount,
        current_pool: request.reward_amount,
        status: BountyStatus::Active,
        created_at: Utc::now(),
        deadline: request.deadline,
        required_consensus: request.required_consensus,
        confidence_threshold: request.confidence_threshold,
        submission: Vec::new(),
        final_verdict: None,
        confidence_score: None,
    };

    match state.db.create_bounty(&bounty).await {
        Ok(_) => {
            // Create smart contract bounty
            match state.blockchain.create_bounty(
                bounty.id,
                bounty.reward_amount,
                bounty.deadline.timestamp() as u64,
            ).await {
                Ok(tx_hash) => {
                    println!("bounty created on blockchain: {}", tx_hash);
                }

                Err(e) => {
                    eprintln!("Failed to create bounty on blockchain: {}", e);
                    // consider eolling back database transaction
                }
            }

            let response = BountyResponse {
                id: bounty.id,
                title: bounty.title,
                description: bounty.description,
                creator: bounty.creator,
                reward_amount: bounty.reward_amount,
                current_pool: bounty.reward_amount,
                status: bounty.status,
                created_at: bounty.created_at,
                deadline: bounty.deadline,
                submission_count: 0,
                consensus_reached: false,
                final_verdict: None,
                confidence_score: None, 
            };

            Ok(Json(response))
        }

        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_bounties(
    State(state): State<std::sync::Arc<AppState>>,
    Query(filters): Query<BountyFilters>,
) -> Result<Json<BountListResponse>, StatusCode> {
    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(20).min(100);     // Cap at 100

    match state.db.get_bounties(filters, page, limit).await {
        Ok((bounties, total_count)) => {
            let bounty_responses: Vec<BountyResponse> = bounties
                .into_iter()
                .map(|bounty| BountyResponse {
                    id: bounty.id,
                    title: bounty.title,
                    description: bounty.description,
                    creator: bounty.creator,
                    reward_amount: bounty.reward_amount,
                    current_pool: bounty.reward_amount,
                    status: bounty.status,
                    created_at: bounty.created_at,
                    deadline: bounty.deadline,
                    submission_count: bounty.submission.len() as u32,
                    consensus_reached: bounty.final_verdict.is_some(),
                    final_verdict: bounty.final_verdict,
                    confidence_score: bounty.confidence_score,  
                })
                .collect();

            Ok(Json(BountListResponse {
                bounties: bounty_responses,
                total_count,
                page,
                limit,
            }))
        }

        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_bounties_details(
    State(state): State<std::sync::Arc<AppState>>,
    Path(bounty_id): Path<Uuid>,
) -> Result<Json<BountyDetailsResponse>, StatusCode> {
    match state.db.get_bounty_by_id(bounty_id).await {
        Ok(Some(bounty)) => {
            let submission: Vec<SubmissionResponse> = bounty
                .submission
                .iter()
                .map(|sub| SubmissionResponse {
                    id: sub.id,
                    bounty_id: sub.bounty_id,
                    engine_id: sub.engine_id.clone(),
                    verdict: sub.verdict.to_string(),
                    confidence: sub.confidence,
                    stake_amount: sub.stake_amount,
                    submitted_at: sub.submitted_at,
                    is_winner: sub.is_winner,
                })
                .collect();

            let bounty_response = BountyResponse {
                id: bounty.id,
                title: bounty.title,
                description: bounty.description,
                creator: bounty.creator,
                reward_amount: bounty.reward_amount,
                current_pool: bounty.current_pool,
                status: bounty.status,
                created_at: bounty.created_at,
                deadline: bounty.deadline,
                submission_count: bounty.submission.len() as u32,
                consensus_reached: bounty.final_verdict.is_some(),
                final_verdict: bounty.final_verdict,
                confidence_score: bounty.confidence_score,
            };

            // Get file info if hash exists
            let file_info = if let Some(hash) = &bounty.file_hash {
                state.db.get_file_info(hash).await.ok().flatten()
            } else {
                None
            };

            Ok(Json(BountyDetailsResponse {
                bounty: bounty_response,
                submissions,
                file_info,
            }))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn submit_analysis(
    State(state): State<std::sync::Arc<AppState>>,
    Path(bounty_id): Path<Uuid>,
    Json(request): Json<SubmitAnalysisRequest>,
) -> Result<Json<SubmissionResponse>, StatusCode> {
    // validate verdict
    let verdict = match request.verdict.as_str() {
        "malicious" => EngineVerdict::Malicious,
        "benign" => EngineVerdict::Benign,
        "suspicious" => EngineVerdict::Suspicious,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    // validate confidence
    if request.confidence < 0.0 || request.confidence > 1.0 {
        return Err(StatusCode::BAD_REQUEST);

    }

    // Check if bounty exists and is active
    match state.db.get_bounty_by_id(bounty_id).await {
        Ok(Some(bounty)) => {
            if bounty.status != BountyStatus::Active {
                return Err(StatusCode::CONFLICT);
            }

            if bounty.deadline <= Utc::now() {
                return Err(StatusCode::CONFLICT);
            }

            // Check if engine already submitted
            if bounty.submission.iter().any(|s| s.engine_id == request.engine_id) {
                return Err(StatusCode::CONFLICT);
            }
        }
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    // Create submissions
    let submission = BountySubmission {
        id: Uuid::new_v4(),
        bounty_id,
        engine_id: request.engine_id,
        verdict,
        confidence: request.confidence,
        analysis_details: request.analysis_details,
        stake_amount: request.stake_amount,
        submitted_at: Utc::now(),
        is_winner: None,
    };

    match state.db.create_submission(&submission).await {
        Ok(_) => {
            // Submit to blockhain  
            match state.blockchain.submit_analysis(
                bounty_id,
                &submission.engine_id,
                submission.verdict.clone(),
                submission.stake_amount,
            ).await {
                Ok(tx_hash) => {
                    println!("Analysis submitted to blockchain: {}", tx_hash);
                }

                Err(e) => {
                    eprintln!("Failed to submit analysis to blockchain: {}", e);
                }
            }

            // Check if consensus is reached
            check_consensus(&state, bounty_id).await;

            let response = SubmissionResponse {
                id: submission.id,
                bounty_id: submission.bounty_id,
                engine_id: submission.engine_id,
                verdict: submission.verdict.to_string(),
                confidence: submission.confidence,
                stake_amount: submission.stake_amount,
                submitted_at: submission.submitted_at,
                is_winner: submission.is_winner,
            };

            Ok(Json(response))
        }

        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn finalize_bounty(
    State(state): State<std::sync::Arc<AppState>>,
    Path(bounty_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.db.get_bounty_by_id(bounty_id).await {
        Ok(Some(bounty)) => {
            if bounty.status != BountyStatus::Active {
                return Err(StatusCode::CONFLICT);
            }

            // Force finalization (deadline passed or manual trigger)
            finalize_bounty_internal(&state, bounty_id).await;
            Ok(StatusCode::OK)
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Internal helper functions
async fn check_consensus(state: &std::sync::Arc<AppState>, bounty_id: Uuid) {
    if let Ok(Some(bounty)) = state.db.get_bounty_by_id(bounty_id).await {
        if bounty.submission.len() >= bounty.required_consensus as usize {
            let consensus = calculate_consensus(&bounty.submission);
            
            if consensus.confidence >= bounty.confidence_threshold {
                finalize_bounty_internal(state, bounty_id).await;
            }
        }
    }
}

async fn finalize_bounty_internal(state: &std::sync::Arc<AppState>, bounty_id: Uuid) {
    if let Ok(Some(bounty)) = state.db.get_bounty_by_id(bounty_id).await {
        let consensus = calculate_consensus(&bounty.submission);

        // Update bounty status
        let _ = state.db.finalize_bounty(
            bounty_id,
            consensus.verdict.clone(),
            consensus.confidence,
        ).await;

        // Distribute rewards on blockchain
        let winners = determine_winners(&bounty.submission, &consensus.verdict);
        for winner in winners {
            let _ = state.blockchain.distribute_reward(
                bounty_id,
                &winner.engine_id,
                winner.reward_amount,
            ).await;
        }
    }
}

#[derive(Clone)]
struct Consensus {
    verdict: EngineVerdict,
    confidence: f32,
}

#[derive(Clone)]
struct Winner {
    engine_id: String,
    reward_amount: u64,
}

fn calculate_consensus(submissions: &[BountySubmission]) -> Consensus {
    if submissions.is_empty() {
        return Consensus {
            verdict: EngineVerdict::Benign,
            confidence: 0.0,
        };
    }

    // Weighted voting based on stake and confidence
    let mut malicious_weight = 0.0;
    let mut benign_weight = 0.0;
    let mut suspicious_weight = 0.0;

    for submission in submissions {
        let weight = (submission.stake_amount as f32) * submission.confidence;
        match submission.verdict {
            EngineVerdict::Malicious => malicious_weight += weight,
            EngineVerdict::Benign => benign_weight += weight,
            EngineVerdict::Suspicious => suspicious_weight += weight,
        }
    }

    let total_weight = malicious_weight + benign_weight + suspicious_weight;
    if total_weight == 0.0 {
        return Consensus {
            verdict: EngineVerdict::Benign,
            confidence: 0.0,
        };
    }

    let (verdict, max_weight) = if malicious_weight >= benign_weight && malicious_weight >= suspicious_weight {
        (EngineVerdict::Malicious, malicious_weight)
    } else if benign_weight >= suspicious_weight {
        (EngineVerdict::Benign, benign_weight)
    } else {
        (EngineVerdict::Suspicious, suspicious_weight)
    };

    Consensus {
        verdict,
        confidence: max_weight / total_weight,
    }
}

fn determine_winners(submissions: &[BountySubmission], final_verdict: &EngineVerdict) -> Vec<Winner> {
    let correct_submissions: Vec<_> = submissions
        .iter()
        .filter(|s| &s.verdict == final_verdict)
        .collect();

    if correct_submissions.is_empty() {
        return Vec::new();
    }

    // Calculate rewards based on stake and confidence
    let total_stake: u64 = correct_submissions.iter().map(|s| s.stake_amount).sum();
    
    correct_submissions
        .into_iter()
        .map(|submission| {
            let reward_ratio = submission.stake_amount as f64 / total_stake as f64;
            Winner {
                engine_id: submission.engine_id.clone(),
                reward_amount: (reward_ratio * 1000000.0) as u64, // Placeholder calculation
            }
        })
        .collect()
}

// Router setup
pub fn create_bounty_router() -> Router<std::sync::Arc<AppState>> {
    Router::new()
        .route("/bounties", post(create_bounty))
        .route("/bounties", get(get_bounties))
        .route("/bounties/:id", get(get_bounty_details))
        .route("/bounties/:id/submit", post(submit_analysis))
        .route("/bounties/:id/finalize", put(finalize_bounty))
}
