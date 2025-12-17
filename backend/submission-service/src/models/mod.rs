use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Submission {
    pub id: Uuid,
    pub submitter_id: Option<Uuid>,
    pub file_hash: Option<String>,
    pub url: Option<String>,
    pub original_filename: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub file_path: Option<String>,
    pub submission_type: String,
    pub is_malicious: Option<bool>,
    pub confidence_score: Option<f64>,
    pub analysis_status: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionType {
    File,
    Url,
}

impl SubmissionType {
    pub fn as_str(&self) -> &str {
        match self {
            SubmissionType::File => "file",
            SubmissionType::Url => "url",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionStatus {
    Pending,
    Analyzing,
    Completed,
    Failed,
}

impl SubmissionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SubmissionStatus::Pending => "pending",
            SubmissionStatus::Analyzing => "analyzing",
            SubmissionStatus::Completed => "completed",
            SubmissionStatus::Failed => "failed",
        }
    }
}

/// Request payload for creating a new submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubmissionRequest {
    pub submitter_id: Option<Uuid>,
    pub file_hash: String,
    pub original_filename: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub file_path: String,
    pub submission_type: String,
    pub metadata: Option<serde_json::Value>,
}
