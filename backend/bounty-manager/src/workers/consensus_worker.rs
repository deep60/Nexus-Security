// backend/bounty-manager/src/workers/consensus_worker.rs

use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use crate::services::consensus::{ConsensusService, SubmissionData};
use crate::models::submission::SubmissionModel;
use crate::models::bounty::BountyModel;
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
        let submission_data: Vec<SubmissionData> = submissions.iter().map(|s| {
            SubmissionData {
                submission_id: s.id,
                verdict: s.verdict.clone(),
                confidence: s.confidence,
                stake_amount: s.stake_amount as u64,
                reputation_score: 1.0, // TODO: Get actual reputation score
            }
        }).collect();

        // Calculate consensus
        let consensus_result = self.consensus_service.calculate_consensus(submission_data);

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

            // TODO: Trigger payout worker
            // TODO: Send notifications
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
