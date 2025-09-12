use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use sha2::{Sha256, Digest};

use crate::models::{
    bounty::{BountySubmission, EngineVerdict},
    analysis::{AnalysisResult, ThreatIndicator, FileMetadata},
};

use crate::services::{
    database::DatabaseService,
    blockchain::BlockchainService,
    redis::RedisService,
};

use crate::utils::{
    validation::validate_file_type,
    crypto::calculate_file_hash,
};

// Request/Response DTOs
#[derive(Deserialize)]
pub struct CreateSubmissionRequest {
    pub bounty_id: Uuid,
    pub engine_name: String,
    pub engine_version: String,
    pub verdict: String, // "malicious", "benign", "suspicious"
    pub confidence: f32, // 0.0-1.0
    pub threat_types: Vec<String>,
    pub risk_score: u8, // 0-100
    pub analysis_summary: String,
    pub technical_details: serde_json::Value,
    pub stake_amount: u64,
    pub signatures: Vec<String>, // YARA rules, hashes, etc.
}

#[derive(Deserialize)]
pub struct UpdateSubmissionRequest {
    pub analysis_summary: Option<String>,
    pub technical_details: Option<serde_json::Value>,
    pub additional_signatures: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct SubmissionFilters {
    pub bounty_id: Option<Uuid>,
    pub engine_id: Option<String>,
    pub verdict: Option<String>,
    pub min_confidence: Option<f32>,
    pub max_confidence: Option<f32>,
    pub status: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct SubmissionResponse {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub engine_id: String,
    pub engine_name: String,
    pub engine_version: String,
    pub verdict: String,
    pub confidence: f32,
    pub threat_types: Vec<String>,
    pub risk_score: u8,
    pub analysis_summary: String,
    pub stake_amount: u64,
    pub submitted_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub status: SubmissionStatus,
    pub is_winner: Option<bool>,
    pub reward_earned: Option<u64>,
    pub reputation_change: Option<i32>,
}

#[derive(Serialize)]
pub struct DetailedSubmissionResponse {
    pub submission: SubmissionResponse,
    pub technical_details: serde_json::Value,
    pub signatures: Vec<String>,
    pub analysis_metrics: AnalysisMetrics,
    pub file_info: Option<FileInfo>,
}

#[derive(Serialize)]
pub struct SubmissionListResponse {
    pub submissions: Vec<SubmissionResponse>,
    pub total_count: u32,
    pub page: u32,
    pub limit: u32,
    pub filters_applied: SubmissionFilters,
}

#[derive(Serialize)]
pub struct FileUploadResponse {
    pub file_id: Uuid,
    pub file_hash: String,
    pub file_size: u64,
    pub file_type: String,
    pub upload_timestamp: DateTime<Utc>,
    pub analysis_status: String,
}

#[derive(Serialize)]
pub struct BulkSubmissionResponse {
    pub successful: Vec<SubmissionResponse>,
    pub failed: Vec<SubmissionError>,
    pub total_processed: u32,
}

#[derive(Serialize)]
pub struct SubmissionError {
    pub index: u32,
    pub error: String,
    pub request_data: serde_json::Value,
}

#[derive(Serialize)]
pub struct AnalysisMetrics {
    pub processing_time_ms: u64,
    pub signatures_matched: u32,
    pub false_positive_rate: Option<f32>,
    pub detection_accuracy: Option<f32>,
    pub resource_usage: ResourceUsage,
}

#[derive(Serialize)]
pub struct ResourceUsage {
    pub cpu_time_ms: u64,
    pub memory_usage_mb: u64,
    pub disk_io_mb: u64,
}

#[derive(Serialize)]
pub struct FileInfo {
    pub hash: String,
    pub size: u64,
    pub file_type: String,
    pub mime_type: String,
    pub upload_timestamp: DateTime<Utc>,
    pub scan_count: u32,
    pub last_analysis: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SubmissionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Disputed,
    Verified,
}

// Application state
pub struct AppState {
    pub db: DatabaseService,
    pub blockchain: BlockchainService,
    pub redis: RedisService,
    pub upload_path: String,
    pub max_file_size: u64,
}

// Handler Implementation
pub async fn upload_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<json<FileUploadResponse>, StatusCode> {
    // Check content length
    if let Some(content_length) = headers.get("content-length") {
        if let Ok(size) = content_length.to_str().unwrap_or("0").parse::<u64>() {
            if size > state.max_file_size {
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }
        }
    }

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let filename = field.file_name().map(|s| s.to_string());
            let content_type = field.content_type().map(|s| s.to_string());
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

            // validate file type
            if !validate_file_type(&data, content_type.as_deref()) {
                return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
            }

            // Calculate file hash
            let file_hash = calculate_file_hash(&data);
            let file_id = Uuid::new_v4();

            // save file to disk
            let file_path = format!("{}/{}", state.upload_path, file_hash);
            let mut file = fs::File::create(&file_path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            file.write_all(&data).await.map_err(|_| StatusCode::INERNAL_SERVER_ERROR)?;

            // STORE FILE METADATA IN DATABASE
            let file_metadata = FileMetadata {
                id: file_id,
                hash: file_hash.clone(),
                filename: filename.unwrap_or_else(|| "unknown".to_string()),
                size: data.len() as u64,
                content_type: content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
                upload_timestamp: Utc::now(),
                file_path: file_path.clone(),
                // scan_count: 0,
                // last_analysis: None,
            }; 
            match state.db.store_file_metadata(&file_metadata).await {
                Ok(_) => {
                    // Cache file info in Redis for quick access
                    let _ = state.redis.cache_file_info(&file_hash, &file_metadata).await;

                    // Trigger automatic analysis
                    trigger_automatic_analysis(&state, &file_hash).await;

                    return Ok(Json(FileUploadResponse {
                        file_id,
                        file_hash,
                        file_size: data.len() as u64,
                        file_type: detect_file_type(&data),
                        upload_timestamp: file_metadata.upload_timestamp,
                        analysis_status: "queued".to_string(),
                    }));
                }

                Err(_) => {
                    // Clean up file on database error
                    let _ = fs::remove_file(file_path).await;
                    return Err(StatusCode::INERNAL_SERVER_ERROR);
                }
            }
        }
    }

    Err(StatusCode::BAD_REQUEST)
}

pub async fn create_submission(
    State(state): State<AppState>,
    Json(request): Json<CreateSubmissionRequest>,
) -> Result<Json<SubmissionResponse>, StatusCode> {
    // Validate request
    if request.confidence < 0.0 || request.confidence > 1.0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    if request.risk_score > 100 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let verdict = match request.verdict.as_str() {
        "malicious" => EngineVerdict::Malicious,
        "benign" => EngineVerdict::Benign,
        "suspicious" => EngineVerdict::Suspicious,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    // TODO: Extract engine ID from JWT token
    let engine_id = format!("{}_{}", request.engine_name, Uuid::new_v4());

    // Check if bounty exists and is active
    match state.db.get_bounty_by_id(request.bounty_id).await {
        Ok(Some(bounty)) => {
            if bounty.deadline <= Utc::now() {
                return Err(StatusCode::CONFLICT);
            }

            // Check if engine already submitted
            if bounty.submissions.iter().any(|s| s.engine_id == engine_id) {
                return Err(StatusCode::CONFLICT);
            }
        }
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    // Create submission
    let submission = BountySubmission {
        id: Uuid::new_v4(),
        bounty_id: request.bounty_id,
        engine_id: engine_id.clone(),
        verdict,
        confidence: request.confidence,
        analysis_details: request.technical_details,
        stake_amount: request.stake_amount,
        submitted_at: Utc::now(),
        is_winner: None,
    };

    // Create extended submission data
    let extended_submission = ExtendedSubmission {
        submission: submission.clone(),
        engine_name: request.engine_name,
        engine_version: request.engine_version,
        threat_types: request.threat_types,
        risk_score: request.risk_score,
        analysis_summary: request.analysis_summary,
        signatures: request.signatures,
        status: SubmissionStatus::Completed,
        processing_metrics: None,
    };

    match state.db.create_extended_submission(&extended_submission).await {
        Ok(_) => {
            // Submit to blockchain
            match state.blockchain.submit_analysis(
                request.bounty_id,
                &engine_id,
                submission.verdict.clone(),
                request.stake_amount,
            ).await {
                Ok(tx_hash) => {
                    println!("Submission recorded on blockchain: {}", tx_hash);
                }
                Err(e) => {
                    eprintln!("Failed to record submission on blockchain: {}", e);
                }
            }

            // Cache submission for quick access
            let _ = state.redis.cache_submission(&submission.id, &extended_submission).await;

            let response = SubmissionResponse {
                id: submission.id,
                bounty_id: submission.bounty_id,
                engine_id: submission.engine_id,
                engine_name: extended_submission.engine_name,
                engine_version: extended_submission.engine_version,
                verdict: submission.verdict.to_string(),
                confidence: submission.confidence,
                threat_types: extended_submission.threat_types,
                risk_score: extended_submission.risk_score,
                analysis_summary: extended_submission.analysis_summary,
                stake_amount: submission.stake_amount,
                submitted_at: submission.submitted_at,
                updated_at: None,
                status: extended_submission.status,
                is_winner: submission.is_winner,
                reward_earned: None,
                reputation_change: None,
            };

            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_submissions(
    State(state): State<AppState>,
    Query(filters): Query<SubmissionFilters>,
) -> Result<Json<SubmissionListResponse>, StatusCode> {
    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(20).min(100);

    match state.db.get_submissions_with_filters(&filters, page, limit).await {
        Ok((submissions, total_count)) => {
            let submission_responses: Vec<SubmissionResponse> = submissions
                .into_iter()
                .map(|sub| SubmissionResponse {
                    id: sub.submission.id,
                    bounty_id: sub.submission.bounty_id,
                    engine_id: sub.submission.engine_id,
                    engine_name: sub.engine_name,
                    engine_version: sub.engine_version,
                    verdict: sub.submission.verdict.to_string(),
                    confidence: sub.submission.confidence,
                    threat_types: sub.threat_types,
                    risk_score: sub.risk_score,
                    analysis_summary: sub.analysis_summary,
                    stake_amount: sub.submission.stake_amount,
                    submitted_at: sub.submission.submitted_at,
                    updated_at: None,
                    status: sub.status,
                    is_winner: sub.submission.is_winner,
                    reward_earned: None,
                    reputation_change: None,
                })
                .collect();

            Ok(Json(SubmissionListResponse {
                submissions: submission_responses,
                total_count,
                page,
                limit,
                filters_applied: filters,
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_submission_details(
    State(state): State<AppState>,
    Path(submission_id): Path<Uuid>,
) -> Result<Json<DetailedSubmissionResponse>, StatusCode> {
    // Try cache first
    if let Ok(Some(cached)) = state.redis.get_cached_submission(submission_id).await {
        return Ok(Json(cached));
    }

    match state.db.get_extended_submission_by_id(submission_id).await {
        Ok(Some(extended_sub)) => {
            let analysis_metrics = AnalysisMetrics {
                processing_time_ms: extended_sub.processing_metrics
                    .as_ref()
                    .map(|m| m.processing_time_ms)
                    .unwrap_or(0),
                signatures_matched: extended_sub.signatures.len() as u32,
                false_positive_rate: None, // TODO: Calculate from historical data
                detection_accuracy: None,  // TODO: Calculate from historical data
                resource_usage: ResourceUsage {
                    cpu_time_ms: 0,    // TODO: Add to metrics
                    memory_usage_mb: 0, // TODO: Add to metrics
                    disk_io_mb: 0,     // TODO: Add to metrics
                },
            };

            // Get file info if available
            let file_info = if let Ok(Some(bounty)) = state.db.get_bounty_by_id(extended_sub.submission.bounty_id).await {
                if let Some(hash) = bounty.file_hash {
                    state.db.get_file_info(&hash).await.ok().flatten()
                } else {
                    None
                }
            } else {
                None
            };

            let submission_response = SubmissionResponse {
                id: extended_sub.submission.id,
                bounty_id: extended_sub.submission.bounty_id,
                engine_id: extended_sub.submission.engine_id,
                engine_name: extended_sub.engine_name,
                engine_version: extended_sub.engine_version,
                verdict: extended_sub.submission.verdict.to_string(),
                confidence: extended_sub.submission.confidence,
                threat_types: extended_sub.threat_types,
                risk_score: extended_sub.risk_score,
                analysis_summary: extended_sub.analysis_summary,
                stake_amount: extended_sub.submission.stake_amount,
                submitted_at: extended_sub.submission.submitted_at,
                updated_at: None,
                status: extended_sub.status,
                is_winner: extended_sub.submission.is_winner,
                reward_earned: None,
                reputation_change: None,
            };

            let response = DetailedSubmissionResponse {
                submission: submission_response,
                technical_details: extended_sub.submission.analysis_details,
                signatures: extended_sub.signatures,
                analysis_metrics,
                file_info,
            };

            // Cache the response
            let _ = state.redis.cache_detailed_submission(submission_id, &response).await;

            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_submission(
    State(state): State<AppState>,
    Path(submission_id): Path<Uuid>,
    Json(request): Json<UpdateSubmissionRequest>,
) -> Result<Json<SubmissionResponse>, StatusCode> {
    // TODO: Verify that the requesting engine owns this submission

    match state.db.update_submission(submission_id, &request).await {
        Ok(updated_submission) => {
            // Invalidate cache
            let _ = state.redis.invalidate_submission_cache(submission_id).await;

            let response = SubmissionResponse {
                id: updated_submission.submission.id,
                bounty_id: updated_submission.submission.bounty_id,
                engine_id: updated_submission.submission.engine_id,
                engine_name: updated_submission.engine_name,
                engine_version: updated_submission.engine_version,
                verdict: updated_submission.submission.verdict.to_string(),
                confidence: updated_submission.submission.confidence,
                threat_types: updated_submission.threat_types,
                risk_score: updated_submission.risk_score,
                analysis_summary: updated_submission.analysis_summary,
                stake_amount: updated_submission.submission.stake_amount,
                submitted_at: updated_submission.submission.submitted_at,
                updated_at: Some(Utc::now()),
                status: updated_submission.status,
                is_winner: updated_submission.submission.is_winner,
                reward_earned: None,
                reputation_change: None,
            };

            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_submission(
    State(state): State<AppState>,
    Path(submission_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Verify that the requesting engine owns this submission
    // TODO: Check if submission can be deleted (not finalized, etc.)

    match state.db.delete_submission(submission_id).await {
        Ok(_) => {
            // Invalidate cache
            let _ = state.redis.invalidate_submission_cache(submission_id).await;
            Ok(StatusCode::NO_CONTENT)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn bulk_create_submissions(
    State(state): State<AppState>,
    Json(requests): Json<Vec<CreateSubmissionRequest>>,
) -> Result<Json<BulkSubmissionResponse>, StatusCode> {
    let mut successful = Vec::new();
    let mut failed = Vec::new();
    let total_processed = requests.len() as u32;

    for (index, request) in requests.into_iter().enumerate() {
        match process_single_submission(&state, request.clone()).await {
            Ok(response) => successful.push(response),
            Err(error) => failed.push(SubmissionError {
                index: index as u32,
                error: format!("{:?}", error),
                request_data: serde_json::to_value(request).unwrap_or_default(),
            }),
        }
    }

    Ok(Json(BulkSubmissionResponse {
        successful,
        failed,
        total_processed,
    }))
}

pub async fn get_file_info(
    State(state): State<AppState>,
    Path(file_hash): Path<String>,
) -> Result<Json<FileInfo>, StatusCode> {
    // Try cache first
    if let Ok(Some(cached)) = state.redis.get_cached_file_info(&file_hash).await {
        return Ok(Json(cached));
    }

    match state.db.get_file_info(&file_hash).await {
        Ok(Some(file_info)) => {
            // Cache the result
            let _ = state.redis.cache_file_info(&file_hash, &file_info).await;
            Ok(Json(file_info))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Helper functions
async fn trigger_automatic_analysis(state: &AppState, file_hash: &str) {
    // Queue file for automatic analysis by available engines
    let _ = state.redis.queue_for_analysis(file_hash).await;
}

fn detect_file_type(data: &[u8]) -> String {
    // Simple file type detection based on magic bytes
    if data.len() < 4 {
        return "unknown".to_string();
    }

    match &data[0..4] {
        [0x4D, 0x5A, _, _] => "executable".to_string(), // PE
        [0x7F, 0x45, 0x4C, 0x46] => "elf".to_string(),  // ELF
        [0x50, 0x4B, 0x03, 0x04] => "zip".to_string(),  // ZIP/JAR
        [0x25, 0x50, 0x44, 0x46] => "pdf".to_string(),  // PDF
        _ => "unknown".to_string(),
    }
}

async fn process_single_submission(
    state: &AppState,
    request: CreateSubmissionRequest,
) -> Result<SubmissionResponse, StatusCode> {
    // This is a simplified version of create_submission for bulk processing
    // You might want to optimize this further for bulk operations
    let json_request = Json(request);
    match create_submission(State(state.clone()), json_request).await {
        Ok(Json(response)) => Ok(response),
        Err(status) => Err(status),
    }
}

// Additional structures for extended submission data
#[derive(Clone)]
pub struct ExtendedSubmission {
    pub submission: BountySubmission,
    pub engine_name: String,
    pub engine_version: String,
    pub threat_types: Vec<String>,
    pub risk_score: u8,
    pub analysis_summary: String,
    pub signatures: Vec<String>,
    pub status: SubmissionStatus,
    pub processing_metrics: Option<ProcessingMetrics>,
}

#[derive(Clone)]
pub struct ProcessingMetrics {
    pub processing_time_ms: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

// Router setup
pub fn create_submission_router() -> Router<AppState> {
    Router::new()
        .route("/upload", post(upload_file))
        .route("/submissions", post(create_submission))
        .route("/submissions", get(get_submissions))
        .route("/submissions/bulk", post(bulk_create_submissions))
        .route("/submissions/:id", get(get_submission_details))
        .route("/submissions/:id", put(update_submission))
        .route("/submissions/:id", delete(delete_submission))
        .route("/files/:hash", get(get_file_info))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB max file size
}