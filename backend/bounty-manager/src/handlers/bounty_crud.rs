use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use shared::types::ApiResponse;
use crate::services::reputation::ReputationService;

// Common types
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

// Bounty-related types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounty {
    pub id: Uuid,
    pub creator: String, // Wallet address
    pub title: String,
    pub description: String,
    pub artifact_type: ArtifactType,
    pub artifact_data: ArtifactData,
    pub reward_amount: u64, // Amount in wei
    pub currency: String,   // Token contract address
    pub min_stake: u64,     // Minimum stake required to participate
    pub max_participants: Option<u32>,
    pub deadline: DateTime<Utc>,
    pub status: BountyStatus,
    pub consensus_threshold: f32, // Percentage needed for consensus
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub submissions: Vec<SubmissionSummary>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    File,
    Url,
    Hash,
    IpAddress,
    Domain,
    Email,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactData {
    pub hash: Option<String>,    // File hash
    pub url: Option<String>,     // URL to analyze
    pub file_name: Option<String>,
    pub file_size: Option<u64>,
    pub mime_type: Option<String>,
    pub upload_path: Option<String>, // Internal storage path
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BountyStatus {
    Active,
    InProgress,
    Completed,
    Expired,
    Cancelled,
    UnderReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionSummary {
    pub id: Uuid,
    pub engine_id: String,
    pub verdict: ThreatVerdict,
    pub confidence: f32,
    pub stake_amount: u64,
    pub submitted_at: DateTime<Utc>,
    pub reputation_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatVerdict {
    Malicious,
    Benign,
    Suspicious,
    Unknown,
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct CreateBountyRequest {
    pub title: String,
    pub description: String,
    pub artifact_type: ArtifactType,
    pub artifact_data: ArtifactData,
    pub reward_amount: u64,
    pub currency: String,
    pub min_stake: u64,
    pub max_participants: Option<u32>,
    pub deadline_hours: u32, // Hours from now
    pub consensus_threshold: Option<f32>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBountyRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub deadline: Option<DateTime<Utc>>,
    pub status: Option<BountyStatus>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct BountyFilters {
    pub status: Option<BountyStatus>,
    pub artifact_type: Option<ArtifactType>,
    pub creator: Option<String>,
    pub min_reward: Option<u64>,
    pub max_reward: Option<u64>,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BountyListResponse {
    pub bounties: Vec<Bounty>,
    pub total_count: usize,
    pub page: u32,
    pub per_page: u32,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct BountyStatsResponse {
    pub total_bounties: u64,
    pub active_bounties: u64,
    pub completed_bounties: u64,
    pub total_rewards_paid: u64,
    pub avg_resolution_time_hours: f32,
    pub top_currencies: Vec<CurrencyStats>,
}

#[derive(Debug, Serialize)]
pub struct CurrencyStats {
    pub currency: String,
    pub total_amount: u64,
    pub bounty_count: u32,
}

// Application state (would typically come from dependency injection)
#[derive(Clone)]
pub struct BountyManagerState {
    // Database connection pool, blockchain client, etc.
    pub reputation_service: Arc<ReputationService>,
}

// Handler implementations
pub async fn create_bounty(
    State(state): State<BountyManagerState>,
    Extension(user_address): Extension<String>, // From auth middleware
    Json(req): Json<CreateBountyRequest>,
) -> Result<Json<ApiResponse<Bounty>>, StatusCode> {
    // Validate request
    if req.title.is_empty() || req.description.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if req.reward_amount == 0 || req.min_stake == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create bounty
    let bounty_id = Uuid::new_v4();
    let now = Utc::now();
    let deadline = now + chrono::Duration::hours(req.deadline_hours as i64);

    let bounty = Bounty {
        id: bounty_id,
        creator: user_address,
        title: req.title,
        description: req.description,
        artifact_type: req.artifact_type,
        artifact_data: req.artifact_data,
        reward_amount: req.reward_amount,
        currency: req.currency,
        min_stake: req.min_stake,
        max_participants: req.max_participants,
        deadline,
        status: BountyStatus::Active,
        consensus_threshold: req.consensus_threshold.unwrap_or(0.75),
        created_at: now,
        updated_at: now,
        submissions: Vec::new(),
        metadata: req.metadata.unwrap_or_default(),
    };

    // TODO: Save to database
    // TODO: Create blockchain transaction for bounty creation
    // TODO: Emit event for real-time updates

    Ok(Json(ApiResponse::success(bounty)))
}

pub async fn get_bounty(
    State(_state): State<BountyManagerState>,
    Path(bounty_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Bounty>>, StatusCode> {
    // TODO: Fetch from database
    // For now, return a mock bounty
    let mock_bounty = create_mock_bounty(bounty_id);

    Ok(Json(ApiResponse::success(mock_bounty)))
}

pub async fn list_bounties(
    State(_state): State<BountyManagerState>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<BountyFilters>,
) -> Result<Json<ApiResponse<BountyListResponse>>, StatusCode> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20).min(100);

    // TODO: Implement database query with filters and pagination
    let bounties = create_mock_bounty_list();
    let total_count = bounties.len();

    let response_data = BountyListResponse {
        bounties,
        total_count,
        page,
        per_page,
        has_more: false, // TODO: Calculate based on actual data
    };

    Ok(Json(ApiResponse::success(response_data)))
}

pub async fn update_bounty(
    State(_state): State<BountyManagerState>,
    Extension(user_address): Extension<String>,
    Path(bounty_id): Path<Uuid>,
    Json(req): Json<UpdateBountyRequest>,
) -> Result<Json<ApiResponse<Bounty>>, StatusCode> {
    // TODO: Fetch existing bounty from database
    // TODO: Check if user is the creator
    // TODO: Validate that bounty can be updated (not completed, etc.)
    
    let mut bounty = create_mock_bounty(bounty_id);
    
    // Verify ownership
    if bounty.creator != user_address {
        return Err(StatusCode::FORBIDDEN);
    }

    // Apply updates
    if let Some(title) = req.title {
        bounty.title = title;
    }
    if let Some(description) = req.description {
        bounty.description = description;
    }
    if let Some(deadline) = req.deadline {
        bounty.deadline = deadline;
    }
    if let Some(status) = req.status {
        bounty.status = status;
    }
    if let Some(metadata) = req.metadata {
        bounty.metadata = metadata;
    }

    bounty.updated_at = Utc::now();

    // TODO: Save to database
    // TODO: Emit update event

    Ok(Json(ApiResponse::success(bounty)))
}

pub async fn cancel_bounty(
    State(_state): State<BountyManagerState>,
    Extension(user_address): Extension<String>,
    Path(bounty_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // TODO: Fetch bounty and verify ownership
    // TODO: Check if bounty can be cancelled
    // TODO: Handle refunds and blockchain transactions

    Ok(Json(ApiResponse::success(())))
}

pub async fn get_bounty_stats(
    State(_state): State<BountyManagerState>,
) -> Result<Json<ApiResponse<BountyStatsResponse>>, StatusCode> {
    // TODO: Implement real statistics from database
    let stats = BountyStatsResponse {
        total_bounties: 156,
        active_bounties: 23,
        completed_bounties: 128,
        total_rewards_paid: 2500000, // wei
        avg_resolution_time_hours: 4.2,
        top_currencies: vec![
            CurrencyStats {
                currency: "0x...".to_string(), // ETH address
                total_amount: 1500000,
                bounty_count: 89,
            },
            CurrencyStats {
                currency: "0x...".to_string(), // USDC address
                total_amount: 1000000,
                bounty_count: 67,
            },
        ],
    };

    Ok(Json(ApiResponse::success(stats)))
}

pub async fn submit_to_bounty(
    State(state): State<BountyManagerState>,
    Extension(engine_id): Extension<String>, // From auth middleware
    Path(bounty_id): Path<Uuid>,
    Json(submission): Json<SubmissionRequest>,
) -> Result<Json<ApiResponse<SubmissionResponse>>, StatusCode> {
    // TODO: Validate bounty exists and is active
    // TODO: Check if engine can participate (reputation, stake requirements)
    // TODO: Process stake transaction
    // TODO: Store submission
    
    let submission_id = Uuid::new_v4();
    
    let response_data = SubmissionResponse {
        submission_id,
        bounty_id,
        engine_id: engine_id.clone(),
        status: "submitted".to_string(),
        stake_transaction_hash: "0x...".to_string(), // Mock transaction hash
    };

    Ok(Json(ApiResponse::success(response_data)))
}

// Helper types for submission
#[derive(Debug, Deserialize)]
pub struct SubmissionRequest {
    pub verdict: ThreatVerdict,
    pub confidence: f32,
    pub stake_amount: u64,
    pub analysis_data: Option<AnalysisData>,
}

#[derive(Debug, Deserialize)]
pub struct AnalysisData {
    pub detected_families: Vec<String>,
    pub behavioral_indicators: Vec<String>,
    pub static_features: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    pub submission_id: Uuid,
    pub bounty_id: Uuid,
    pub engine_id: String,
    pub status: String,
    pub stake_transaction_hash: String,
}

// Mock data functions (to be replaced with real database queries)
fn create_mock_bounty(id: Uuid) -> Bounty {
    Bounty {
        id,
        creator: "0x742d35Cc6634C0532925a3b8D404C8f89f6562b6".to_string(),
        title: "Analyze suspicious executable".to_string(),
        description: "Please analyze this PE file for malware indicators".to_string(),
        artifact_type: ArtifactType::File,
        artifact_data: ArtifactData {
            hash: Some("sha256:abc123...".to_string()),
            url: None,
            file_name: Some("suspicious.exe".to_string()),
            file_size: Some(1024000),
            mime_type: Some("application/x-msdownload".to_string()),
            upload_path: Some("/uploads/abc123...".to_string()),
        },
        reward_amount: 100000, // wei
        currency: "0x...".to_string(),
        min_stake: 10000,
        max_participants: Some(10),
        deadline: Utc::now() + chrono::Duration::hours(24),
        status: BountyStatus::Active,
        consensus_threshold: 0.75,
        created_at: Utc::now() - chrono::Duration::hours(2),
        updated_at: Utc::now() - chrono::Duration::hours(2),
        submissions: vec![],
        metadata: HashMap::new(),
    }
}

fn create_mock_bounty_list() -> Vec<Bounty> {
    vec![
        create_mock_bounty(Uuid::new_v4()),
        create_mock_bounty(Uuid::new_v4()),
    ]
}