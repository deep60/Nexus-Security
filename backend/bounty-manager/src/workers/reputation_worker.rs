// backend/bounty-manager/src/workers/reputation_worker.rs

use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use crate::services::reputation::ReputationService;
use crate::models::reputation::ReputationModel;
use crate::models::submission::SubmissionModel;

pub struct ReputationWorker {
    db: PgPool,
    reputation_service: Arc<ReputationService>,
    check_interval_seconds: u64,
}

impl ReputationWorker {
    pub fn new(db: PgPool, reputation_service: Arc<ReputationService>) -> Self {
        Self {
            db,
            reputation_service,
            check_interval_seconds: 120, // Check every 2 minutes
        }
    }

    /// Start the reputation worker
    pub async fn run(&self) {
        info!("Starting reputation worker...");
        let mut ticker = interval(Duration::from_secs(self.check_interval_seconds));

        loop {
            ticker.tick().await;
            
            if let Err(e) = self.update_reputations().await {
                error!("Error updating reputations: {}", e);
            }
        }
    }

    /// Update reputations for all engines with recent activity
    async fn update_reputations(&self) -> Result<(), WorkerError> {
        // TODO: Implement reputation updates
        // This would:
        // 1. Find engines with processed submissions
        // 2. Calculate new reputation scores
        // 3. Update reputation records
        
        info!("Checking for reputation updates...");
        
        Ok(())
    }

    /// Update reputation for a specific engine
    async fn update_engine_reputation(&self, engine_id: &str) -> Result<(), WorkerError> {
        // Get current reputation or create new one
        let mut reputation = match ReputationModel::find_by_id(&self.db, engine_id)
            .await
            .map_err(|e| WorkerError::DatabaseError(e.to_string()))?
        {
            Some(rep) => rep,
            None => {
                // Create new reputation record
                ReputationModel {
                    engine_id: engine_id.to_string(),
                    reputation_score: 1.0,
                    total_submissions: 0,
                    correct_submissions: 0,
                    accuracy_rate: 0.0,
                    average_confidence: 0.0,
                    total_stake: 0,
                    rewards_earned: 0,
                    penalties_incurred: 0,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            }
        };

        // Get all processed submissions for this engine
        let submissions = SubmissionModel::find_by_engine(&self.db, engine_id)
            .await
            .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

        if submissions.is_empty() {
            return Ok(());
        }

        // Calculate new metrics
        let total_submissions = submissions.len() as i32;
        let correct_submissions = submissions.iter()
            .filter(|s| s.status == "Correct")
            .count() as i32;
        
        let accuracy_rate = if total_submissions > 0 {
            correct_submissions as f32 / total_submissions as f32
        } else {
            0.0
        };

        let average_confidence: f32 = submissions.iter()
            .map(|s| s.confidence)
            .sum::<f32>() / total_submissions as f32;

        let total_stake: i64 = submissions.iter()
            .map(|s| s.stake_amount)
            .sum();

        // Calculate reputation score (weighted formula)
        let reputation_score = self.calculate_reputation_score(
            accuracy_rate,
            total_submissions,
            average_confidence,
        );

        // Update reputation
        reputation.reputation_score = reputation_score;
        reputation.total_submissions = total_submissions;
        reputation.correct_submissions = correct_submissions;
        reputation.accuracy_rate = accuracy_rate;
        reputation.average_confidence = average_confidence;
        reputation.total_stake = total_stake;

        ReputationModel::update(&self.db, &reputation)
            .await
            .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

        info!(
            "Updated reputation for {}: score={:.2}, accuracy={:.2}%",
            engine_id, reputation_score, accuracy_rate * 100.0
        );

        Ok(())
    }

    /// Calculate reputation score from metrics
    fn calculate_reputation_score(&self, accuracy: f32, total: i32, avg_confidence: f32) -> f32 {
        // Base score from accuracy (0-50 points)
        let accuracy_score = accuracy * 50.0;

        // Experience bonus (0-30 points, logarithmic)
        let experience_score = (total as f32).ln() * 5.0;
        let experience_score = experience_score.min(30.0);

        // Confidence bonus (0-20 points)
        let confidence_score = avg_confidence * 20.0;

        // Total score (0-100)
        let total_score = accuracy_score + experience_score + confidence_score;
        total_score.min(100.0).max(0.0) / 100.0 // Normalize to 0-1
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),
}
