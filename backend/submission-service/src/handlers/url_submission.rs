use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::db::repository;
use crate::models::{CreateSubmissionRequest, SubmissionType};
use crate::queue::publisher;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct UrlSubmissionRequest {
    pub url: String,
    pub bounty_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct UrlSubmissionResponse {
    pub submission_id: String,
    pub url: String,
    pub url_hash: String,
    pub status: String,
    pub message: String,
}

/// Handle URL submission
pub async fn submit_url(
    State(state): State<AppState>,
    submitter_id: Option<crate::handlers::SubmitterId>,
    Json(payload): Json<UrlSubmissionRequest>,
) -> Result<Json<UrlSubmissionResponse>, (StatusCode, String)> {
    tracing::info!("Received URL submission: {}", payload.url);

    // 1. Validate URL format
    let parsed = url::Url::parse(&payload.url)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid URL: {}", e)))?;

    // Only allow http/https schemes
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Only HTTP/HTTPS URLs are accepted".to_string(),
        ));
    }

    // 2. Compute a hash of the URL for deduplication
    let url_hash = format!("{:x}", Sha256::digest(payload.url.as_bytes()));

    // 3. Check if URL was already submitted recently (last 24h)
    let existing = repository::find_recent_url_submission(&state.db_pool, &url_hash)
        .await
        .map_err(|e| {
            tracing::error!("DB lookup failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    if let Some(existing_sub) = existing {
        return Ok(Json(UrlSubmissionResponse {
            submission_id: existing_sub.id.to_string(),
            url: payload.url,
            url_hash,
            status: existing_sub.analysis_status,
            message: "URL was already submitted recently — returning existing submission"
                .to_string(),
        }));
    }

    // 4. Create submission record
    let create_request = CreateSubmissionRequest {
        submitter_id: submitter_id.map(|s| s.0),
        file_hash: url_hash.clone(),
        original_filename: parsed.host_str().unwrap_or("unknown").to_string(),
        file_size: 0,
        mime_type: Some("text/x-uri".to_string()),
        file_path: String::new(),
        url: Some(payload.url.clone()),
        submission_type: SubmissionType::Url.as_str().to_string(),
        metadata: payload.metadata,
    };

    let submission = repository::create_submission(&state.db_pool, create_request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create URL submission record: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create submission: {}", e),
            )
        })?;

    // 5. Queue for analysis
    if let Err(e) = publisher::publish_to_analysis_queue(&state.redis_client, submission.id).await {
        tracing::error!("Failed to queue URL submission for analysis: {}", e);
        // Don't fail — submission is persisted and can be retried
    }

    Ok(Json(UrlSubmissionResponse {
        submission_id: submission.id.to_string(),
        url: payload.url,
        url_hash,
        status: "pending".to_string(),
        message: "URL submitted successfully and queued for analysis".to_string(),
    }))
}
