use axum::{extract::{Multipart, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::db::repository;
use crate::models::{CreateSubmissionRequest, SubmissionType};
use crate::queue::publisher;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSubmissionResponse {
    pub submission_id: String,
    pub file_hash: String,
    pub file_key: String,
    pub file_size: usize,
    pub filename: String,
    pub content_type: Option<String>,
    pub status: String,
    pub message: String,
}

// Maximum file size: 100MB
const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;

// Allowed MIME types for analysis
const ALLOWED_MIME_TYPES: &[&str] = &[
    "application/x-msdownload",  // .exe
    "application/x-dosexec",     // .exe
    "application/vnd.microsoft.portable-executable", // .exe
    "application/x-executable",  // executable
    "application/x-elf",         // ELF
    "application/x-mach-binary", // Mach-O
    "application/pdf",           // PDF
    "application/zip",           // ZIP
    "application/x-zip-compressed", // ZIP
    "application/x-rar-compressed", // RAR
    "application/x-7z-compressed",  // 7z
    "application/java-archive",  // JAR
    "application/vnd.android.package-archive", // APK
    "application/x-sh",          // Shell scripts
    "application/x-python-code", // Python
    "text/x-python",             // Python
    "application/javascript",    // JavaScript
    "text/javascript",           // JavaScript
    "application/octet-stream",  // Generic binary
];

/// Handle file upload submission
pub async fn submit_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<FileSubmissionResponse>, (StatusCode, String)> {
    tracing::info!("Received file submission request");

    let mut filename = String::new();
    let mut file_data: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;

    // Extract file from multipart form
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid multipart: {}", e)))?
    {
        let field_name = field.name().unwrap_or("unknown").to_string();
        tracing::debug!("Processing field: {}", field_name);

        if field_name == "file" {
            // Get filename
            filename = field
                .file_name()
                .unwrap_or("unknown")
                .to_string();

            // Get content type
            content_type = field.content_type().map(|s| s.to_string());

            // Read file data
            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read file: {}", e)))?
                .to_vec();

            // Validate file size
            if data.len() > MAX_FILE_SIZE {
                return Err((
                    StatusCode::PAYLOAD_TOO_LARGE,
                    format!("File too large. Maximum size is {} MB", MAX_FILE_SIZE / (1024 * 1024)),
                ));
            }

            if data.is_empty() {
                return Err((StatusCode::BAD_REQUEST, "Empty file provided".to_string()));
            }

            file_data = Some(data);
            break;
        }
    }

    // Ensure file was provided
    let data = file_data.ok_or((StatusCode::BAD_REQUEST, "No file provided".to_string()))?;
    let file_size = data.len();

    // Validate content type if provided
    if let Some(ref ct) = content_type {
        if !ALLOWED_MIME_TYPES.iter().any(|&allowed| ct.contains(allowed)) {
            tracing::warn!("Potentially unsupported MIME type: {}", ct);
            // Don't reject, just warn - we'll analyze it anyway
        }
    }

    tracing::info!(
        "File received: filename={}, size={} bytes, content_type={:?}",
        filename,
        file_size,
        content_type
    );

    // Generate unique submission ID
    let submission_id = Uuid::new_v4().to_string();

    // Generate S3 key: submissions/{submission_id}/{filename}
    let s3_key = format!("submissions/{}/{}", submission_id, filename);

    // Upload file to S3/MinIO
    let file_hash = state
        .s3_client
        .upload_file(&s3_key, data, content_type.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to upload file to S3: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to store file: {}", e))
        })?;

    tracing::info!(
        "File uploaded successfully: submission_id={}, hash={}, key={}",
        submission_id,
        file_hash,
        s3_key
    );

    // Create submission record in database
    let create_request = CreateSubmissionRequest {
        submitter_id: None, // TODO: Get from authenticated user context
        file_hash: file_hash.clone(),
        original_filename: filename.clone(),
        file_size: file_size as i64,
        mime_type: content_type.clone(),
        file_path: s3_key.clone(),
        submission_type: SubmissionType::File.as_str().to_string(),
        metadata: None,
    };

    let submission = match repository::create_submission(&state.db_pool, create_request).await {
        Ok(sub) => sub,
        Err(e) => {
            tracing::error!("Failed to create submission record: {}", e);

            // Rollback: Delete file from S3
            if let Err(delete_err) = state.s3_client.delete_file(&s3_key).await {
                tracing::error!("Failed to rollback S3 upload: {}", delete_err);
            }

            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create submission record: {}", e),
            ));
        }
    };

    // Queue file for analysis
    if let Err(e) = publisher::publish_to_analysis_queue(&state.redis_client, submission.id).await {
        tracing::error!("Failed to queue submission for analysis: {}", e);
        // Don't fail the request - the submission is recorded, just not queued
        // It can be retried later
    }

    Ok(Json(FileSubmissionResponse {
        submission_id: submission.id.to_string(),
        file_hash,
        file_key: s3_key,
        file_size,
        filename,
        content_type,
        status: "pending".to_string(),
        message: "File submitted successfully and queued for analysis".to_string(),
    }))
}
