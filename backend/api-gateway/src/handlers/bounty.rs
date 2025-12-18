use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{
    bounty::{Bounty, BountyStatus},
    user::User,
};
use crate::AppState;
// Import CreateBountyRequest from models if available, otherwise define here matching the service
// Re-using existing structs if they match, or updating them.

// Request/Response DTOs
#[derive(Deserialize)]
pub struct CreateBountyRequest {
    pub title: String,
    pub description: String,
    pub target_url: Option<String>,
    pub target_hash: Option<String>,
    pub target_type: String, // "url", "file", "binary"
    pub reward_amount: i64, // Amount in Wei (using i64 to match Diesel/SQLx usually, but u64 is better for amounts)
    pub deadline: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct SubmitAnalysisRequest {
    pub engine_id: String,
    pub verdict: String, // "malicious", "benign", "suspicious"
    pub confidence: f32, // 0.0-1.0
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
// handler Implementation
pub async fn create_bounty(
    State(state): State<AppState>,
    claims: crate::middleware::auth::Claims,
    Json(request): Json<CreateBountyRequest>,
) -> Result<Json<Bounty>, StatusCode> {
    // Map handler DTO to model DTO
    let mut metadata = serde_json::Map::new();
    if let Some(url) = &request.target_url {
        metadata.insert(
            "target_url".to_string(),
            serde_json::Value::String(url.clone()),
        );
    }
    if let Some(hash) = &request.target_hash {
        metadata.insert(
            "target_hash".to_string(),
            serde_json::Value::String(hash.clone()),
        );
    }
    metadata.insert(
        "target_type".to_string(),
        serde_json::Value::String(request.target_type.clone()),
    );

    let model_request = crate::models::bounty::CreateBountyRequest {
        title: request.title,
        description: request.description,
        bounty_type: crate::models::bounty::BountyType::Custom, // TODO: Map from request.target_type
        priority: crate::models::bounty::BountyPriority::Medium,
        total_reward: request.reward_amount.to_string(),
        minimum_stake: "0".to_string(),
        distribution_method: crate::models::bounty::DistributionMethod::ProportionalStake,
        max_participants: None,
        required_consensus: None,
        minimum_reputation: None,
        deadline_hours: Some(24), // derived from deadline difference ideally
        auto_finalize: Some(true),
        requires_human_analysis: Some(false),
        file_types_allowed: None,
        max_file_size: None,
        tags: None,
        template_id: None,
        metadata: Some(serde_json::Value::Object(metadata)),
    };

    let bounty = state
        .db
        .create_bounty(model_request, claims.sub)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create bounty: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(bounty))
}

// TODO: Rewrite to match actual Bounty model
pub async fn list_bounties(
    State(state): State<AppState>,
    Query(filters): Query<BountyFilters>,
) -> Result<Json<Vec<Bounty>>, StatusCode> {
    let limit = filters.limit.unwrap_or(20) as i64;
    let offset = ((filters.page.unwrap_or(1) - 1) * filters.limit.unwrap_or(20)) as i64;

    let bounties = state
        .db
        .get_active_bounties(limit, offset)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch bounties: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(bounties))
}

// TODO: Rewrite to match actual Bounty model
pub async fn get_bounty(
    State(state): State<AppState>,
    Path(bounty_id): Path<Uuid>,
) -> Result<Json<Bounty>, StatusCode> {
    let bounty = state
        .db
        .get_bounty_by_id(bounty_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch bounty: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(bounty))
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
pub fn create_bounty_router() -> Router<AppState> {
    Router::new()
        .route("/bounties", post(create_bounty))
        .route("/bounties", get(list_bounties))
        .route("/bounties/:id", get(get_bounty))
    // .route("/bounties/:id/submit", post(submit_analysis)) // TODO: Implement submit_analysis
    // .route("/bounties/:id/finalize", put(finalize_bounty)) // TODO: Implement finalize_bounty
}
