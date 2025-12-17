use anyhow::{Result, anyhow};
use redis::AsyncCommands;
use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

use crate::analyzers::{AnalysisEngine, FileAnalysisRequest, AnalysisOptions};
use crate::storage::S3Client;

/// Redis queue key for analysis tasks
const ANALYSIS_QUEUE_KEY: &str = "analysis_queue";

/// WebSocket event channel for real-time updates
const WS_CHANNEL_ANALYSIS_UPDATED: &str = "events:analysis_updated";
const WS_CHANNEL_ANALYSIS_COMPLETED: &str = "events:analysis_completed";

/// Submission record from database
#[derive(Debug, Clone, sqlx::FromRow)]
struct Submission {
    pub id: Uuid,
    pub submitter_id: Option<Uuid>,
    pub file_hash: Option<String>,
    pub original_filename: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub file_path: Option<String>,
    pub submission_type: String,
    pub analysis_status: String,
}

/// Start the analysis queue consumer worker
pub async fn start_analysis_worker(
    redis_client: redis::Client,
    db_pool: PgPool,
    s3_client: Arc<S3Client>,
    analysis_engine: Arc<Mutex<AnalysisEngine>>,
) -> Result<()> {
    info!("Starting analysis queue consumer worker");

    loop {
        // Step 1: Listen to Redis analysis queue (blocking pop)
        let submission_id = match pop_from_queue(&redis_client).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                // Timeout, retry
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                continue;
            }
            Err(e) => {
                error!("Failed to pop from queue: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        info!("Received submission for analysis: {}", submission_id);

        // Process the submission
        if let Err(e) = process_submission(
            submission_id,
            &redis_client,
            &db_pool,
            &s3_client,
            &analysis_engine,
        )
        .await
        {
            error!("Failed to process submission {}: {}", submission_id, e);

            // Update submission status to failed
            if let Err(update_err) =
                update_submission_status(&db_pool, submission_id, "failed").await
            {
                error!(
                    "Failed to update submission {} status to failed: {}",
                    submission_id, update_err
                );
            }
        }
    }
}

/// Pop a submission ID from the Redis queue
async fn pop_from_queue(redis_client: &redis::Client) -> Result<Option<Uuid>> {
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| anyhow!("Failed to connect to Redis: {}", e))?;

    // BRPOP with 5 second timeout
    let result: Option<(String, String)> = conn
        .brpop(ANALYSIS_QUEUE_KEY, 5.0)
        .await
        .map_err(|e| anyhow!("BRPOP failed: {}", e))?;

    match result {
        Some((_key, value)) => {
            let submission_id = Uuid::parse_str(&value)
                .map_err(|e| anyhow!("Invalid UUID in queue: {}", e))?;
            Ok(Some(submission_id))
        }
        None => Ok(None), // Timeout
    }
}

/// Process a single submission
async fn process_submission(
    submission_id: Uuid,
    redis_client: &redis::Client,
    db_pool: &PgPool,
    s3_client: &S3Client,
    analysis_engine: &Arc<Mutex<AnalysisEngine>>,
) -> Result<()> {
    // Step 2: Fetch submission from database
    let submission = fetch_submission_from_db(db_pool, submission_id).await?;

    info!(
        "Processing submission: id={}, filename={:?}, file_path={:?}",
        submission.id, submission.original_filename, submission.file_path
    );

    // Update status to analyzing
    update_submission_status(db_pool, submission_id, "analyzing").await?;

    // Publish WebSocket event: analysis started
    publish_ws_event(
        redis_client,
        WS_CHANNEL_ANALYSIS_UPDATED,
        &serde_json::json!({
            "submission_id": submission_id.to_string(),
            "status": "analyzing"
        }),
    )
    .await?;

    // Step 3: Download file from S3
    let file_path = submission
        .file_path
        .as_ref()
        .ok_or_else(|| anyhow!("Submission has no file_path"))?;

    let file_data = s3_client
        .download_file(file_path)
        .await
        .map_err(|e| anyhow!("Failed to download file from S3: {}", e))?;

    info!(
        "Downloaded file from S3: submission_id={}, size={} bytes",
        submission_id,
        file_data.len()
    );

    // Step 4: Run ClamAV + other analyzers
    let filename = submission
        .original_filename
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    let analysis_request = FileAnalysisRequest {
        filename: filename.clone(),
        file_data,
        file_hashes: None,
        analysis_options: AnalysisOptions::default(), // Enable all analyzers
    };

    let mut engine = analysis_engine.lock().await;
    let analysis_result = engine
        .analyze_file(analysis_request)
        .await
        .map_err(|e| anyhow!("Analysis failed: {}", e))?;

    drop(engine); // Release lock

    info!(
        "Analysis completed for submission {}: status={:?}, detections={}",
        submission_id,
        analysis_result.status,
        analysis_result.detections.len()
    );

    // Step 5: Store results in database
    let (is_malicious, confidence_score) = calculate_verdict(&analysis_result);

    store_analysis_results(
        db_pool,
        submission_id,
        is_malicious,
        confidence_score,
        &analysis_result,
    )
    .await?;

    // Update submission status to completed
    update_submission_status(db_pool, submission_id, "completed").await?;

    // Publish WebSocket event: analysis completed
    publish_ws_event(
        redis_client,
        WS_CHANNEL_ANALYSIS_COMPLETED,
        &serde_json::json!({
            "submission_id": submission_id.to_string(),
            "status": "completed",
            "is_malicious": is_malicious,
            "confidence_score": confidence_score,
            "detections_count": analysis_result.detections.len()
        }),
    )
    .await?;

    info!(
        "Submission {} processed successfully: is_malicious={}, confidence={}",
        submission_id, is_malicious, confidence_score
    );

    Ok(())
}

/// Fetch submission record from database
async fn fetch_submission_from_db(db_pool: &PgPool, submission_id: Uuid) -> Result<Submission> {
    let submission = sqlx::query_as::<_, Submission>(
        r#"
        SELECT id, submitter_id, file_hash, original_filename, file_size,
               mime_type, file_path, submission_type, analysis_status
        FROM submissions
        WHERE id = $1
        "#,
    )
    .bind(submission_id)
    .fetch_one(db_pool)
    .await
    .map_err(|e| anyhow!("Failed to fetch submission from database: {}", e))?;

    Ok(submission)
}

/// Update submission status in database
async fn update_submission_status(
    db_pool: &PgPool,
    submission_id: Uuid,
    status: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE submissions
        SET analysis_status = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(status)
    .bind(submission_id)
    .execute(db_pool)
    .await
    .map_err(|e| anyhow!("Failed to update submission status: {}", e))?;

    info!("Updated submission {} status to: {}", submission_id, status);
    Ok(())
}

/// Calculate final verdict from analysis result
fn calculate_verdict(
    analysis_result: &crate::models::analysis_result::AnalysisResult,
) -> (bool, f64) {
    use crate::models::analysis_result::ThreatVerdict;

    let mut malicious_count = 0;
    let mut benign_count = 0;
    let mut confidence_sum = 0.0;
    let total_detections = analysis_result.detections.len() as f64;

    if total_detections == 0.0 {
        return (false, 0.0);
    }

    for detection in &analysis_result.detections {
        confidence_sum += detection.confidence;

        match detection.verdict {
            ThreatVerdict::Malicious => malicious_count += 1,
            ThreatVerdict::Benign => benign_count += 1,
            _ => {}
        }
    }

    let avg_confidence = confidence_sum / total_detections;
    let is_malicious = malicious_count > benign_count;

    (is_malicious, avg_confidence)
}

/// Store analysis results in database
async fn store_analysis_results(
    db_pool: &PgPool,
    submission_id: Uuid,
    is_malicious: bool,
    confidence_score: f64,
    analysis_result: &crate::models::analysis_result::AnalysisResult,
) -> Result<()> {
    // Update submission with final results
    sqlx::query(
        r#"
        UPDATE submissions
        SET
            is_malicious = $1,
            confidence_score = $2,
            updated_at = NOW()
        WHERE id = $3
        "#,
    )
    .bind(is_malicious)
    .bind(confidence_score)
    .bind(submission_id)
    .execute(db_pool)
    .await
    .map_err(|e| anyhow!("Failed to store analysis results: {}", e))?;

    // TODO: Store individual detection results in analysis_results table
    // This would require the full schema for analysis_results table

    info!(
        "Stored analysis results for submission {}: malicious={}, confidence={}",
        submission_id, is_malicious, confidence_score
    );

    Ok(())
}

/// Publish WebSocket event to Redis Pub/Sub
async fn publish_ws_event(
    redis_client: &redis::Client,
    channel: &str,
    payload: &serde_json::Value,
) -> Result<()> {
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| anyhow!("Failed to connect to Redis: {}", e))?;

    let message = serde_json::to_string(payload)
        .map_err(|e| anyhow!("Failed to serialize WebSocket event: {}", e))?;

    conn.publish(channel, message)
        .await
        .map_err(|e| anyhow!("Failed to publish WebSocket event: {}", e))?;

    info!("Published WebSocket event to channel: {}", channel);
    Ok(())
}
