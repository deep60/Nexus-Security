use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;

use crate::models::{Submission, CreateSubmissionRequest, SubmissionStatus};

/// Create a new submission record in the database
pub async fn create_submission(
    pool: &PgPool,
    request: CreateSubmissionRequest,
) -> Result<Submission, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let submission = sqlx::query_as::<_, Submission>(
        r#"
        INSERT INTO submissions (
            id,
            submitter_id,
            file_hash,
            original_filename,
            file_size,
            mime_type,
            file_path,
            submission_type,
            analysis_status,
            metadata,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(request.submitter_id)
    .bind(Some(&request.file_hash))
    .bind(Some(&request.original_filename))
    .bind(Some(request.file_size))
    .bind(request.mime_type)
    .bind(Some(&request.file_path))
    .bind(&request.submission_type)
    .bind(SubmissionStatus::Pending.as_str())
    .bind(request.metadata)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    tracing::info!("Created submission record: id={}", submission.id);
    Ok(submission)
}

/// Get a submission by ID
pub async fn get_submission_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Submission>, sqlx::Error> {
    let submission = sqlx::query_as::<_, Submission>(
        r#"
        SELECT * FROM submissions WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(submission)
}

/// Update the analysis status of a submission
pub async fn update_analysis_status(
    pool: &PgPool,
    id: Uuid,
    status: SubmissionStatus,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE submissions
        SET analysis_status = $1, updated_at = $2
        WHERE id = $3
        "#,
    )
    .bind(status.as_str())
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    tracing::info!("Updated submission {} status to {}", id, status.as_str());
    Ok(())
}

/// Update submission with analysis results
pub async fn update_submission_results(
    pool: &PgPool,
    id: Uuid,
    is_malicious: bool,
    confidence_score: f64,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE submissions
        SET
            is_malicious = $1,
            confidence_score = $2,
            analysis_status = $3,
            updated_at = $4
        WHERE id = $5
        "#,
    )
    .bind(is_malicious)
    .bind(confidence_score)
    .bind(SubmissionStatus::Completed.as_str())
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    tracing::info!("Updated submission {} results: malicious={}, confidence={}",
        id, is_malicious, confidence_score);
    Ok(())
}

/// Get all submissions with pagination
pub async fn list_submissions(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Submission>, sqlx::Error> {
    let submissions = sqlx::query_as::<_, Submission>(
        r#"
        SELECT * FROM submissions
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(submissions)
}

/// Get submissions by submitter
pub async fn get_submissions_by_submitter(
    pool: &PgPool,
    submitter_id: Uuid,
) -> Result<Vec<Submission>, sqlx::Error> {
    let submissions = sqlx::query_as::<_, Submission>(
        r#"
        SELECT * FROM submissions
        WHERE submitter_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(submitter_id)
    .fetch_all(pool)
    .await?;

    Ok(submissions)
}

/// Check if a file hash already exists
pub async fn file_hash_exists(
    pool: &PgPool,
    file_hash: &str,
) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM submissions WHERE file_hash = $1
        "#,
    )
    .bind(file_hash)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}
