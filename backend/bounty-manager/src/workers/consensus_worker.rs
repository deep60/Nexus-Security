// backend/bounty-manager/src/workers/consensus_worker.rs

use crate::models::bounty::BountyModel;
use crate::models::submission::SubmissionModel;
use crate::services::consensus::{ConsensusService, SubmissionData};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use uuid::Uuid;

pub struct ConsensusWorker {
    db: PgPool,
    consensus_service: Arc<ConsensusService>,
    check_interval_seconds: u64,
}

impl ConsensusWorker {
    pub fn new(db: PgPool, consensus_service: Arc<ConsensusService>) -> Self {
        Self {
            db,
            consensus_service,
            check_interval_seconds: 60, // Check every minute
        }
    }

    /// Start the consensus worker
    pub async fn run(&self) {
        info!("Starting consensus worker...");
        let mut ticker = interval(Duration::from_secs(self.check_interval_seconds));

        loop {
            ticker.tick().await;

            if let Err(e) = self.process_pending_bounties().await {
                error!("Error processing bounties for consensus: {}", e);
            }
        }
    }

    /// Process all pending bounties to check for consensus
    async fn process_pending_bounties(&self) -> Result<(), WorkerError> {
        // Get all active bounties
        let bounties = BountyModel::find_active(&self.db)
            .await
            .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

        info!("Checking {} active bounties for consensus", bounties.len());

        for bounty in bounties {
            if let Err(e) = self.check_bounty_consensus(bounty.id).await {
                error!("Error checking consensus for bounty {}: {}", bounty.id, e);
            }
        }

        Ok(())
    }

    /// Check if a specific bounty has reached consensus
    async fn check_bounty_consensus(&self, bounty_id: Uuid) -> Result<(), WorkerError> {
        // Get all submissions for this bounty
        let submissions = SubmissionModel::find_by_bounty(&self.db, bounty_id)
            .await
            .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

        if submissions.is_empty() {
            return Ok(());
        }

        // Convert to submission data
        let mut submission_data: Vec<SubmissionData> = Vec::with_capacity(submissions.len());
        for s in submissions.iter() {
            let reputation_score =
                crate::models::ReputationModel::find_by_id(&self.db, &s.engine_id)
                    .await
                    .ok()
                    .flatten()
                    .map(|r| r.reputation_score)
                    .unwrap_or(1.0);

            submission_data.push(SubmissionData {
                submission_id: s.id,
                verdict: s.verdict.clone(),
                confidence: s.confidence,
                stake_amount: s.stake_amount as u64,
                reputation_score,
            });
        }

        // Calculate consensus
        let consensus_result = self
            .consensus_service
            .calculate_consensus(bounty_id, submission_data);

        if consensus_result.consensus_reached {
            info!(
                "Consensus reached for bounty {}: {} (confidence: {})",
                bounty_id, consensus_result.final_verdict, consensus_result.confidence
            );

            // Update bounty status
            BountyModel::update_status(&self.db, bounty_id, "Completed")
                .await
                .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

            // Update submission statuses based on accuracy
            for submission in &submissions {
                let accuracy = self.consensus_service.calculate_accuracy_score(
                    &submission.verdict,
                    &consensus_result.final_verdict,
                    submission.confidence,
                );

                SubmissionModel::update_accuracy_score(&self.db, submission.id, accuracy)
                    .await
                    .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

                // Update status to Correct or Incorrect
                let new_status = if submission.verdict == consensus_result.final_verdict {
                    "Correct"
                } else {
                    "Incorrect"
                };

                SubmissionModel::update_status(&self.db, submission.id, new_status)
                    .await
                    .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;
            }

            // Trigger payout processing
            info!("Triggering payout processing for bounty {}", bounty_id);
            // In a production deployment, this would enqueue a message for the payout worker.
            // For now the payout is triggered via the process_bounty_completion API endpoint.

            // Send notifications
            info!(
                bounty_id = %bounty_id,
                verdict = %consensus_result.final_verdict,
                "Consensus reached — notifying participants"
            );
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Consensus error: {0}")]
    ConsensusError(String),
}
