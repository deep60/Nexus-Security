// backend/bounty-manager/src/workers/validation_worker.rs

use crate::models::submission::SubmissionModel;
use sqlx::PgPool;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use uuid::Uuid;

pub struct ValidationWorker {
    db: PgPool,
    check_interval_seconds: u64,
}

impl ValidationWorker {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            check_interval_seconds: 45, // Check every 45 seconds
        }
    }

    /// Start the validation worker
    pub async fn run(&self) {
        info!("Starting validation worker...");
        let mut ticker = interval(Duration::from_secs(self.check_interval_seconds));

        loop {
            ticker.tick().await;

            if let Err(e) = self.validate_pending_submissions().await {
                error!("Error validating submissions: {}", e);
            }
        }
    }

    /// Validate all pending submissions
    async fn validate_pending_submissions(&self) -> Result<(), WorkerError> {
        info!("Checking for submissions to validate...");

        // Get all pending submissions
        let pending: Vec<SubmissionModel> = sqlx::query_as(
            "SELECT * FROM submissions WHERE status = 'Pending' ORDER BY submitted_at ASC LIMIT 50",
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

        if pending.is_empty() {
            return Ok(());
        }

        info!("Validating {} pending submissions", pending.len());

        for submission in &pending {
            if let Err(e) = self.validate_submission(submission.id).await {
                error!("Failed to validate submission {}: {}", submission.id, e);
            }
        }

        Ok(())
    }

    /// Validate a single submission
    async fn validate_submission(&self, submission_id: Uuid) -> Result<bool, WorkerError> {
        let submission = SubmissionModel::find_by_id(&self.db, submission_id)
            .await
            .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

        if let Some(sub) = submission {
            // Perform validation checks
            let is_valid = self.perform_validation_checks(&sub).await?;

            if is_valid {
                // Update to Active status
                SubmissionModel::update_status(&self.db, submission_id, "Active")
                    .await
                    .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

                info!("Submission {} validated successfully", submission_id);
            } else {
                // Mark as invalid
                SubmissionModel::update_status(&self.db, submission_id, "Invalid")
                    .await
                    .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

                info!("Submission {} failed validation", submission_id);
            }

            Ok(is_valid)
        } else {
            Err(WorkerError::SubmissionNotFound(submission_id))
        }
    }

    /// Perform validation checks on a submission
    async fn perform_validation_checks(
        &self,
        submission: &SubmissionModel,
    ) -> Result<bool, WorkerError> {
        // Check 1: Confidence is within valid range
        if submission.confidence < 0.0 || submission.confidence > 1.0 {
            return Ok(false);
        }

        // Check 2: Stake amount meets minimum requirement
        if submission.stake_amount < 1000 {
            return Ok(false);
        }

        // Check 3: Analysis details are present and have valid structure
        let details = &submission.analysis_details;
        if details.is_null() {
            return Ok(false);
        }
        // Ensure the JSON has at least one meaningful field
        let has_content = details.get("threat_indicators").is_some()
            || details.get("malware_families").is_some()
            || details.get("behavioral_analysis").is_some()
            || details.get("static_analysis").is_some();
        if !has_content {
            return Ok(false);
        }

        // Check 4: Transaction hash is valid (if present)
        if let Some(tx_hash) = &submission.transaction_hash {
            if tx_hash.is_empty() {
                return Ok(false);
            }
        }

        // All checks passed
        Ok(true)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Submission not found: {0}")]
    SubmissionNotFound(Uuid),

    #[error("Validation error: {0}")]
    ValidationError(String),
}
