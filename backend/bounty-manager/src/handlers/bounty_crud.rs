use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use shared::types::ApiResponse;
use crate::models::{BountyModel, SubmissionModel};
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

// Application state
#[derive(Clone)]
pub struct BountyManagerState {
    pub db: PgPool,
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

    let bounty_id = Uuid::new_v4();
    let now = Utc::now();
    let deadline = now + chrono::Duration::hours(req.deadline_hours as i64);
    let consensus = req.consensus_threshold.unwrap_or(0.75);
    let metadata_clone = req.metadata.clone();

    // Build DB model and persist
    let db_bounty = BountyModel {
        id: bounty_id,
        creator: user_address.clone(),
        title: req.title.clone(),
        description: req.description.clone(),
        artifact_type: format!("{:?}", req.artifact_type),
        artifact_hash: req.artifact_data.hash.clone(),
        artifact_url: req.artifact_data.url.clone(),
        file_name: req.artifact_data.file_name.clone(),
        file_size: req.artifact_data.file_size.map(|s| s as i64),
        mime_type: req.artifact_data.mime_type.clone(),
        upload_path: req.artifact_data.upload_path.clone(),
        reward_amount: req.reward_amount as i64,
        currency: req.currency.clone(),
        min_stake: req.min_stake as i64,
        max_participants: req.max_participants.map(|m| m as i32),
        deadline,
        status: "Active".to_string(),
        consensus_threshold: consensus,
        created_at: now,
        updated_at: now,
        metadata: metadata_clone.map(|m| serde_json::to_value(m).unwrap_or_default()),
    };

    BountyModel::create(&state.db, &db_bounty)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create bounty: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

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
        consensus_threshold: consensus,
        created_at: now,
        updated_at: now,
        submissions: Vec::new(),
        metadata: req.metadata.unwrap_or_default(),
    };

    Ok(Json(ApiResponse::success(bounty)))
}

pub async fn get_bounty(
    State(state): State<BountyManagerState>,
    Path(bounty_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Bounty>>, StatusCode> {
    let db_bounty = BountyModel::find_by_id(&state.db, bounty_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch bounty: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let bounty = db_bounty_to_handler_bounty(db_bounty);
    Ok(Json(ApiResponse::success(bounty)))
}

pub async fn list_bounties(
    State(state): State<BountyManagerState>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<BountyFilters>,
) -> Result<Json<ApiResponse<BountyListResponse>>, StatusCode> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20).min(100);
    let offset = ((page.saturating_sub(1)) * per_page) as i64;

    let status_str = filters.status.as_ref().map(|s| format!("{:?}", s));
    let creator_str = filters.creator.as_deref();

    let db_bounties = BountyModel::list(
        &state.db,
        status_str.as_deref(),
        creator_str,
        per_page as i64,
        offset,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to list bounties: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total_count = BountyModel::count(&state.db, status_str.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to count bounties: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })? as usize;

    let bounties: Vec<Bounty> = db_bounties.into_iter().map(db_bounty_to_handler_bounty).collect();
    let has_more = (page as usize * per_page as usize) < total_count;

    let response_data = BountyListResponse {
        bounties,
        total_count,
        page,
        per_page,
        has_more,
    };

    Ok(Json(ApiResponse::success(response_data)))
}

pub async fn update_bounty(
    State(state): State<BountyManagerState>,
    Extension(user_address): Extension<String>,
    Path(bounty_id): Path<Uuid>,
    Json(req): Json<UpdateBountyRequest>,
) -> Result<Json<ApiResponse<Bounty>>, StatusCode> {
    // Fetch existing bounty
    let db_bounty = BountyModel::find_by_id(&state.db, bounty_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch bounty: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Verify ownership
    if db_bounty.creator != user_address {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check bounty can be updated (not completed/cancelled)
    if db_bounty.status == "Completed" || db_bounty.status == "Cancelled" {
        return Err(StatusCode::CONFLICT);
    }

    // Build a single transactional UPDATE with only the provided fields
    let mut tx = state.db.begin().await.map_err(|e| {
        tracing::error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let now = Utc::now();
    let title = req.title.as_deref().unwrap_or(&db_bounty.title);
    let description = req.description.as_deref().unwrap_or(&db_bounty.description);
    let deadline = req.deadline.unwrap_or(db_bounty.deadline);
    let status = req.status.as_ref()
        .map(|s| format!("{:?}", s))
        .unwrap_or(db_bounty.status.clone());
    let metadata = req.metadata.as_ref()
        .map(|m| serde_json::to_value(m).unwrap_or_default())
        .or(db_bounty.metadata.clone());

    sqlx::query(
        r#"
        UPDATE bounties
        SET title = $1, description = $2, deadline = $3,
            status = $4, metadata = $5, updated_at = $6
        WHERE id = $7
        "#,
    )
    .bind(title)
    .bind(description)
    .bind(deadline)
    .bind(&status)
    .bind(&metadata)
    .bind(now)
    .bind(bounty_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update bounty: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Re-fetch to return updated state
    let updated = BountyModel::find_by_id(&state.db, bounty_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to re-fetch bounty: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ApiResponse::success(db_bounty_to_handler_bounty(updated))))
}

pub async fn cancel_bounty(
    State(state): State<BountyManagerState>,
    Extension(user_address): Extension<String>,
    Path(bounty_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // Fetch bounty and verify ownership
    let db_bounty = BountyModel::find_by_id(&state.db, bounty_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch bounty: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if db_bounty.creator != user_address {
        return Err(StatusCode::FORBIDDEN);
    }

    if db_bounty.status == "Completed" || db_bounty.status == "Cancelled" {
        return Err(StatusCode::CONFLICT);
    }

    BountyModel::update_status(&state.db, bounty_id, "Cancelled")
        .await
        .map_err(|e| {
            tracing::error!("Failed to cancel bounty: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse::success(())))
}

pub async fn get_bounty_stats(
    State(state): State<BountyManagerState>,
) -> Result<Json<ApiResponse<BountyStatsResponse>>, StatusCode> {
    let total_bounties = BountyModel::count(&state.db, None).await.unwrap_or(0) as u64;
    let active_bounties = BountyModel::count(&state.db, Some("Active")).await.unwrap_or(0) as u64;
    let completed_bounties = BountyModel::count(&state.db, Some("Completed")).await.unwrap_or(0) as u64;

    // Aggregate reward stats
    let reward_row: (Option<i64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(reward_amount), 0) FROM bounties WHERE status = 'Completed'"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or((Some(0),));

    let total_rewards_paid = reward_row.0.unwrap_or(0) as u64;

    // Average resolution time
    let avg_row: (Option<f64>,) = sqlx::query_as(
        "SELECT AVG(EXTRACT(EPOCH FROM (updated_at - created_at)) / 3600.0) FROM bounties WHERE status = 'Completed'"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or((Some(0.0),));

    let avg_resolution_time_hours = avg_row.0.unwrap_or(0.0) as f32;

    // Top currencies
    let currency_rows: Vec<(String, Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT currency, SUM(reward_amount) as total, COUNT(*) as cnt FROM bounties GROUP BY currency ORDER BY total DESC LIMIT 5"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let top_currencies: Vec<CurrencyStats> = currency_rows
        .into_iter()
        .map(|(currency, total, cnt)| CurrencyStats {
            currency,
            total_amount: total.unwrap_or(0) as u64,
            bounty_count: cnt.unwrap_or(0) as u32,
        })
        .collect();

    let stats = BountyStatsResponse {
        total_bounties,
        active_bounties,
        completed_bounties,
        total_rewards_paid,
        avg_resolution_time_hours,
        top_currencies,
    };

    Ok(Json(ApiResponse::success(stats)))
}

pub async fn submit_to_bounty(
    State(state): State<BountyManagerState>,
    Extension(engine_id): Extension<String>, // From auth middleware
    Path(bounty_id): Path<Uuid>,
    Json(submission): Json<SubmissionRequest>,
) -> Result<Json<ApiResponse<SubmissionResponse>>, StatusCode> {
    // Validate bounty exists and is active
    let db_bounty = BountyModel::find_by_id(&state.db, bounty_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch bounty: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if db_bounty.status != "Active" {
        return Err(StatusCode::CONFLICT);
    }

    // Check max participants
    if let Some(max) = db_bounty.max_participants {
        let current = SubmissionModel::count_by_bounty(&state.db, bounty_id)
            .await
            .unwrap_or(0);
        if current >= max as i64 {
            return Err(StatusCode::CONFLICT);
        }
    }

    let submission_id = Uuid::new_v4();
    let now = Utc::now();

    let analysis_json = submission.analysis_data
        .as_ref()
        .map(|a| serde_json::to_value(a).unwrap_or_default())
        .unwrap_or(serde_json::json!({}));

    let db_sub = SubmissionModel {
        id: submission_id,
        bounty_id,
        engine_id: engine_id.clone(),
        engine_type: "Automated".to_string(),
        verdict: format!("{:?}", submission.verdict),
        confidence: submission.confidence,
        stake_amount: submission.stake_amount as i64,
        analysis_details: analysis_json,
        status: "Pending".to_string(),
        transaction_hash: None,
        submitted_at: now,
        processed_at: None,
        accuracy_score: None,
    };

    SubmissionModel::create(&state.db, &db_sub)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create submission: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response_data = SubmissionResponse {
        submission_id,
        bounty_id,
        engine_id,
        status: "submitted".to_string(),
        stake_transaction_hash: String::new(),
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

#[derive(Debug, Deserialize, Serialize)]
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

// Conversion helper: BountyModel (DB) -> Bounty (handler DTO)
fn db_bounty_to_handler_bounty(db: BountyModel) -> Bounty {
    let artifact_type = match db.artifact_type.as_str() {
        "Url" => ArtifactType::Url,
        "Hash" => ArtifactType::Hash,
        "IpAddress" => ArtifactType::IpAddress,
        "Domain" => ArtifactType::Domain,
        "Email" => ArtifactType::Email,
        _ => ArtifactType::File,
    };

    let status = match db.status.as_str() {
        "InProgress" => BountyStatus::InProgress,
        "Completed" => BountyStatus::Completed,
        "Expired" => BountyStatus::Expired,
        "Cancelled" => BountyStatus::Cancelled,
        "UnderReview" => BountyStatus::UnderReview,
        _ => BountyStatus::Active,
    };

    let metadata: HashMap<String, String> = db.metadata
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    Bounty {
        id: db.id,
        creator: db.creator,
        title: db.title,
        description: db.description,
        artifact_type,
        artifact_data: ArtifactData {
            hash: db.artifact_hash,
            url: db.artifact_url,
            file_name: db.file_name,
            file_size: db.file_size.map(|s| s as u64),
            mime_type: db.mime_type,
            upload_path: db.upload_path,
        },
        reward_amount: db.reward_amount as u64,
        currency: db.currency,
        min_stake: db.min_stake as u64,
        max_participants: db.max_participants.map(|m| m as u32),
        deadline: db.deadline,
        status,
        consensus_threshold: db.consensus_threshold,
        created_at: db.created_at,
        updated_at: db.updated_at,
        submissions: Vec::new(), // Loaded separately if needed
        metadata,
    }
}