use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UrlSubmissionRequest {
    pub url: String,
    pub bounty: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct UrlSubmissionResponse {
    pub submission_id: String,
    pub url: String,
    pub status: String,
    pub message: String,
}

/// Handle URL submission
pub async fn submit_url(
    Json(payload): Json<UrlSubmissionRequest>,
) -> Result<Json<UrlSubmissionResponse>, (StatusCode, String)> {
    tracing::info!("Received URL submission: {}", payload.url);

    // TODO: Implement URL submission logic
    // 1. Validate URL format
    // 2. Check if URL already analyzed recently
    // 3. Create submission record
    // 4. Queue for analysis
    // 5. Return submission ID

    Ok(Json(UrlSubmissionResponse {
        submission_id: uuid::Uuid::new_v4().to_string(),
        url: payload.url,
        status: "pending".to_string(),
        message: "URL submitted successfully (TODO: implement full logic)".to_string(),
    }))
}
