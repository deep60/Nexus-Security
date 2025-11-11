// backend/bounty-manager/src/workers/validation_worker.rs

use sqlx::PgPool;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use crate::models::submission::SubmissionModel;
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
        // TODO: Implement actual validation logic
        // This is a placeholder that would:
        // 1. Get all pending submissions
        // 2. Run validation checks
        // 3. Update submission status
        
        info!("Checking for submissions to validate...");
        
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
    async fn perform_validation_checks(&self, submission: &SubmissionModel) -> Result<bool, WorkerError> {
        // Check 1: Confidence is within valid range
        if submission.confidence < 0.0 || submission.confidence > 1.0 {
            return Ok(false);
        }

        // Check 2: Stake amount meets minimum requirement
        if submission.stake_amount < 1000 {
            return Ok(false);
        }

        // Check 3: Analysis details are present
        // TODO: Parse and validate analysis_details JSON

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
