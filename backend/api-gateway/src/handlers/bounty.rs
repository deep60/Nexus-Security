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

// handler Implementation
// TODO: Rewrite to match actual Bounty model structure from models/bounty.rs
pub async fn create_bounty(
    State(_state): State<crate::AppState>,
    Json(_request): Json<CreateBountyRequest>,
) -> Result<Json<BountyResponse>, StatusCode> {
    // Stub implementation - needs rewrite to match actual Bounty model
    Err(StatusCode::NOT_IMPLEMENTED)
}

// TODO: Rewrite to match actual Bounty model
pub async fn get_bounties(
    State(_state): State<crate::AppState>,
    Query(_filters): Query<BountyFilters>,
) -> Result<Json<BountListResponse>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

// TODO: Rewrite to match actual Bounty model
pub async fn get_bounties_details(
    State(_state): State<crate::AppState>,
    Path(_bounty_id): Path<Uuid>,
) -> Result<Json<BountyDetailsResponse>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

// TODO: Rewrite to match actual Bounty model
pub async fn submit_analysis(
    State(_state): State<crate::AppState>,
    Path(_bounty_id): Path<Uuid>,
    Json(_request): Json<SubmitAnalysisRequest>,
) -> Result<Json<SubmissionResponse>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

// TODO: Rewrite to match actual Bounty model
pub async fn finalize_bounty(
    State(_state): State<crate::AppState>,
    Path(_bounty_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

// Router setup
pub fn create_bounty_router() -> Router<crate::AppState> {
    Router::new()
        .route("/bounties", post(create_bounty))
        .route("/bounties", get(get_bounties))
        .route("/bounties/:id", get(get_bounties_details))
        .route("/bounties/:id/submit", post(submit_analysis))
        .route("/bounties/:id/finalize", put(finalize_bounty))
}
