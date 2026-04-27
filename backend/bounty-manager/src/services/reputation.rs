use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use tracing::{info, warn};

use crate::models::ReputationModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineReputation {
    pub engine_id: String,
    pub total_score: f64,
    pub accuracy_rate: f64,
    pub total_submissions: u64,
    pub correct_predictions: u64,
    pub total_stake: i64,
    pub rewards_earned: i64,
    pub penalties_incurred: i64,
    pub last_updated: DateTime<Utc>,
    pub tier: ReputationTier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreatVerdict {
    Malicious,
    Benign,
    Suspicious,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReputationTier {
    Novice,     // 0-100 score
    Skilled,    // 101-500 score
    Expert,     // 501-1000 score
    Master,     // 1001-2500 score
    Legendary,  // 2500+ score
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationUpdate {
    pub engine_id: String,
    pub submission_id: Uuid,
    pub stake_amount: u64,
    pub prediction: ThreatVerdict,
    pub actual_result: ThreatVerdict,
    pub threat_type: String,
    pub consensus_confidence: f64,
}

/// Database-backed reputation service.
///
/// All state is persisted in the `reputations` table via `ReputationModel`.
/// Pure scoring helpers remain stateless.
pub struct ReputationService {
    db: PgPool,
}

impl ReputationService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Register a new engine in the reputation system (persists to DB).
    pub async fn register_engine(&self, engine_id: String) -> Result<(), ReputationError> {
        // Check if already registered
        if let Some(_) = ReputationModel::find_by_id(&self.db, &engine_id).await
            .map_err(|e| ReputationError::DatabaseError(e.to_string()))?
        {
            return Err(ReputationError::EngineAlreadyExists);
        }

        let now = Utc::now();
        let model = ReputationModel {
            engine_id,
            reputation_score: 0.0,
            total_submissions: 0,
            correct_submissions: 0,
            accuracy_rate: 0.0,
            average_confidence: 0.0,
            total_stake: 0,
            rewards_earned: 0,
            penalties_incurred: 0,
            created_at: now,
            updated_at: now,
        };

        ReputationModel::create(&self.db, &model)
            .await
            .map_err(|e| ReputationError::DatabaseError(e.to_string()))?;

        info!(engine_id = %model.engine_id, "Registered new engine in reputation system");
        Ok(())
    }

    /// Record a stake submission — increments total_submissions and adds to total_stake in DB.
    pub async fn record_stake(
        &self,
        engine_id: &str,
        _submission_id: Uuid,
        stake_amount: u64,
        _prediction: ThreatVerdict,
    ) -> Result<(), ReputationError> {
        let mut rep = ReputationModel::find_by_id(&self.db, engine_id)
            .await
            .map_err(|e| ReputationError::DatabaseError(e.to_string()))?
            .ok_or(ReputationError::EngineNotFound)?;

        rep.total_submissions += 1;
        rep.total_stake += stake_amount as i64;
        rep.updated_at = Utc::now();

        ReputationModel::update(&self.db, &rep)
            .await
            .map_err(|e| ReputationError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Update reputation based on analysis results — reads from DB, computes, writes back.
    pub async fn update_reputation(
        &self,
        update: ReputationUpdate,
    ) -> Result<ReputationStats, ReputationError> {
        let mut rep = ReputationModel::find_by_id(&self.db, &update.engine_id)
            .await
            .map_err(|e| ReputationError::DatabaseError(e.to_string()))?
            .ok_or(ReputationError::EngineNotFound)?;

        let is_correct = update.prediction == update.actual_result;
        let base_reward = update.stake_amount as f64;

        // Calculate reward/penalty
        let tier = Self::calculate_tier(rep.reputation_score as f64);
        let reward = if is_correct {
            Self::calculate_reward(base_reward, update.consensus_confidence, &tier)
        } else {
            -Self::calculate_penalty(base_reward, update.consensus_confidence)
        };

        // Update DB model
        if is_correct {
            rep.correct_submissions += 1;
        }
        rep.accuracy_rate = if rep.total_submissions > 0 {
            rep.correct_submissions as f32 / rep.total_submissions as f32
        } else {
            0.0
        };
        rep.reputation_score += reward as f32;
        if reward > 0.0 {
            rep.rewards_earned += reward as i64;
        } else {
            rep.penalties_incurred += (-reward) as i64;
        }
        rep.average_confidence = (rep.average_confidence * (rep.total_submissions - 1) as f32
            + update.consensus_confidence as f32)
            / rep.total_submissions as f32;
        rep.updated_at = Utc::now();

        ReputationModel::update(&self.db, &rep)
            .await
            .map_err(|e| ReputationError::DatabaseError(e.to_string()))?;

        let new_tier = Self::calculate_tier(rep.reputation_score as f64);

        Ok(ReputationStats {
            engine_id: update.engine_id,
            total_score: rep.reputation_score as f64,
            accuracy_rate: rep.accuracy_rate as f64,
            tier: new_tier,
            reward_earned: reward,
        })
    }

    /// Get reputation for a specific engine from DB.
    pub async fn get_reputation(&self, engine_id: &str) -> Option<EngineReputation> {
        match ReputationModel::find_by_id(&self.db, engine_id).await {
            Ok(Some(rep)) => Some(Self::model_to_engine_rep(rep)),
            Ok(None) => None,
            Err(e) => {
                warn!("Failed to fetch reputation for {}: {}", engine_id, e);
                None
            }
        }
    }

    /// Get top engines by reputation score from DB.
    pub async fn get_top_engines(&self, limit: usize) -> Vec<EngineReputation> {
        match ReputationModel::get_leaderboard(&self.db, limit as i64).await {
            Ok(records) => records.into_iter().map(Self::model_to_engine_rep).collect(),
            Err(e) => {
                warn!("Failed to fetch leaderboard: {}", e);
                Vec::new()
            }
        }
    }

    /// Calculate minimum stake required for an engine based on DB reputation.
    pub async fn calculate_minimum_stake(&self, engine_id: &str) -> Result<u64, ReputationError> {
        let rep = ReputationModel::find_by_id(&self.db, engine_id)
            .await
            .map_err(|e| ReputationError::DatabaseError(e.to_string()))?
            .ok_or(ReputationError::EngineNotFound)?;

        let tier = Self::calculate_tier(rep.reputation_score as f64);

        // Base stake requirements by tier
        let base_stake: u64 = match tier {
            ReputationTier::Novice => 100,
            ReputationTier::Skilled => 50,
            ReputationTier::Expert => 25,
            ReputationTier::Master => 10,
            ReputationTier::Legendary => 5,
        };

        // Adjust based on accuracy
        let multiplier = if rep.accuracy_rate < 0.5 {
            2.0 // Double the stake for poor performance
        } else if rep.accuracy_rate > 0.8 {
            0.5 // Halve the stake for excellent performance
        } else {
            1.0
        };

        Ok((base_stake as f64 * multiplier) as u64)
    }

    /// Decay reputation scores for all engines inactive for >30 days — bulk SQL update.
    pub async fn apply_reputation_decay(&self) {
        let cutoff = Utc::now() - Duration::days(30);

        match sqlx::query(
            r#"
            UPDATE reputations
            SET reputation_score = reputation_score * 0.95,
                updated_at = NOW()
            WHERE updated_at < $1
            "#,
        )
        .bind(cutoff)
        .execute(&self.db)
        .await
        {
            Ok(result) => {
                info!(
                    rows_affected = result.rows_affected(),
                    "Applied reputation decay for inactive engines"
                );
            }
            Err(e) => {
                warn!("Failed to apply reputation decay: {}", e);
            }
        }
    }

    // ── Pure helpers (no DB) ──────────────────────────────────

    fn calculate_reward(base_reward: f64, consensus_confidence: f64, tier: &ReputationTier) -> f64 {
        let tier_multiplier = match tier {
            ReputationTier::Novice => 1.0,
            ReputationTier::Skilled => 1.1,
            ReputationTier::Expert => 1.2,
            ReputationTier::Master => 1.3,
            ReputationTier::Legendary => 1.5,
        };

        let confidence_multiplier = 0.5 + (consensus_confidence * 1.5);
        base_reward * tier_multiplier * confidence_multiplier
    }

    fn calculate_penalty(base_penalty: f64, consensus_confidence: f64) -> f64 {
        let confidence_multiplier = 0.5 + (consensus_confidence * 1.5);
        base_penalty * confidence_multiplier
    }

    pub fn calculate_tier(total_score: f64) -> ReputationTier {
        match total_score as i32 {
            0..=100 => ReputationTier::Novice,
            101..=500 => ReputationTier::Skilled,
            501..=1000 => ReputationTier::Expert,
            1001..=2500 => ReputationTier::Master,
            _ => ReputationTier::Legendary,
        }
    }

    fn model_to_engine_rep(rep: ReputationModel) -> EngineReputation {
        let tier = Self::calculate_tier(rep.reputation_score as f64);
        EngineReputation {
            engine_id: rep.engine_id,
            total_score: rep.reputation_score as f64,
            accuracy_rate: rep.accuracy_rate as f64,
            total_submissions: rep.total_submissions as u64,
            correct_predictions: rep.correct_submissions as u64,
            total_stake: rep.total_stake,
            rewards_earned: rep.rewards_earned,
            penalties_incurred: rep.penalties_incurred,
            last_updated: rep.updated_at,
            tier,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationStats {
    pub engine_id: String,
    pub total_score: f64,
    pub accuracy_rate: f64,
    pub tier: ReputationTier,
    pub reward_earned: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum ReputationError {
    #[error("Engine already exists")]
    EngineAlreadyExists,
    #[error("Engine not found")]
    EngineNotFound,
    #[error("Stake event not found")]
    StakeEventNotFound,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_calculation() {
        assert_eq!(ReputationService::calculate_tier(50.0), ReputationTier::Novice);
        assert_eq!(ReputationService::calculate_tier(200.0), ReputationTier::Skilled);
        assert_eq!(ReputationService::calculate_tier(750.0), ReputationTier::Expert);
        assert_eq!(ReputationService::calculate_tier(2000.0), ReputationTier::Master);
        assert_eq!(ReputationService::calculate_tier(3000.0), ReputationTier::Legendary);
    }

    #[test]
    fn test_reward_calculation() {
        let reward = ReputationService::calculate_reward(100.0, 0.9, &ReputationTier::Expert);
        assert!(reward > 0.0);
        // Expert multiplier 1.2 * (0.5 + 0.9*1.5) = 1.2 * 1.85 = 222.0
        assert!((reward - 222.0).abs() < 0.01);
    }

    #[test]
    fn test_penalty_calculation() {
        let penalty = ReputationService::calculate_penalty(100.0, 0.9);
        assert!(penalty > 0.0);
        // (0.5 + 0.9*1.5) = 1.85 => 185.0
        assert!((penalty - 185.0).abs() < 0.01);
    }
}