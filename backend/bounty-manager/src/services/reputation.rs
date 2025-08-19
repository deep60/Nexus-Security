use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineReputation {
    pub engine_id: String,
    pub total_score: f64,
    pub accuracy_rate: f64,
    pub total_submissions: u64,
    pub correct_predictions: u64,
    pub stake_history: Vec<StakeEvent>,
    pub expertise_areas: HashMap<String, f64>, // threat_type -> expertise_score
    pub last_updated: DateTime<Utc>,
    pub tier: ReputationTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeEvent {
    pub submission_id: Uuid,
    pub stake_amount: u64,
    pub prediction: ThreatVerdict,
    pub actual_result: Option<ThreatVerdict>,
    pub reward_earned: Option<i64>, // Can be negative for losses
    pub timestamp: DateTime<Utc>,
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

pub struct ReputationService {
    reputations: RwLock<HashMap<String, EngineReputation>>,
}

impl ReputationService {
    pub fn new() -> Self {
        Self {
            reputations: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new engine in the reputation system
    pub async fn register_engine(&self, engine_id: String) -> Result<(), ReputationError> {
        let mut reputations = self.reputations.write().await;
        
        if reputations.contains_key(&engine_id) {
            return Err(ReputationError::EngineAlreadyExists);
        }

        let reputation = EngineReputation {
            engine_id: engine_id.clone(),
            total_score: 0.0,
            accuracy_rate: 0.0,
            total_submissions: 0,
            correct_predictions: 0,
            stake_history: Vec::new(),
            expertise_areas: HashMap::new(),
            last_updated: Utc::now(),
            tier: ReputationTier::Novice,
        };

        reputations.insert(engine_id, reputation);
        Ok(())
    }

    /// Record a stake submission before analysis results are known
    pub async fn record_stake(
        &self,
        engine_id: &str,
        submission_id: Uuid,
        stake_amount: u64,
        prediction: ThreatVerdict,
    ) -> Result<(), ReputationError> {
        let mut reputations = self.reputations.write().await;
        
        let reputation = reputations
            .get_mut(engine_id)
            .ok_or(ReputationError::EngineNotFound)?;

        let stake_event = StakeEvent {
            submission_id,
            stake_amount,
            prediction,
            actual_result: None,
            reward_earned: None,
            timestamp: Utc::now(),
        };

        reputation.stake_history.push(stake_event);
        reputation.total_submissions += 1;
        reputation.last_updated = Utc::now();

        Ok(())
    }

    /// Update reputation based on analysis results
    pub async fn update_reputation(
        &self,
        update: ReputationUpdate,
    ) -> Result<ReputationStats, ReputationError> {
        let mut reputations = self.reputations.write().await;
        
        let reputation = reputations
            .get_mut(&update.engine_id)
            .ok_or(ReputationError::EngineNotFound)?;

        // Find the corresponding stake event
        let stake_event = reputation
            .stake_history
            .iter_mut()
            .find(|event| event.submission_id == update.submission_id)
            .ok_or(ReputationError::StakeEventNotFound)?;

        // Update the stake event with results
        stake_event.actual_result = Some(update.actual_result.clone());
        
        let is_correct = stake_event.prediction == update.actual_result;
        let base_reward = stake_event.stake_amount as f64;
        
        // Calculate reward/penalty based on accuracy and confidence
        let reward = if is_correct {
            self.calculate_reward(base_reward, update.consensus_confidence, reputation.tier.clone())
        } else {
            -self.calculate_penalty(base_reward, update.consensus_confidence)
        };

        stake_event.reward_earned = Some(reward as i64);

        // Update overall reputation metrics
        if is_correct {
            reputation.correct_predictions += 1;
        }

        reputation.accuracy_rate = reputation.correct_predictions as f64 / reputation.total_submissions as f64;
        reputation.total_score += reward;

        // Update expertise in specific threat type
        let expertise_score = reputation.expertise_areas
            .entry(update.threat_type)
            .or_insert(0.0);
        
        *expertise_score += if is_correct { 1.0 } else { -0.5 };
        *expertise_score = expertise_score.max(0.0); // Don't go below 0

        // Update tier based on total score
        reputation.tier = self.calculate_tier(reputation.total_score);
        reputation.last_updated = Utc::now();

        Ok(ReputationStats {
            engine_id: update.engine_id,
            total_score: reputation.total_score,
            accuracy_rate: reputation.accuracy_rate,
            tier: reputation.tier.clone(),
            reward_earned: reward,
        })
    }

    /// Get reputation for a specific engine
    pub async fn get_reputation(&self, engine_id: &str) -> Option<EngineReputation> {
        let reputations = self.reputations.read().await;
        reputations.get(engine_id).cloned()
    }

    /// Get top engines by reputation score
    pub async fn get_top_engines(&self, limit: usize) -> Vec<EngineReputation> {
        let reputations = self.reputations.read().await;
        let mut engines: Vec<_> = reputations.values().cloned().collect();
        
        engines.sort_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap());
        engines.truncate(limit);
        engines
    }

    /// Get engines by expertise in a specific threat type
    pub async fn get_experts_for_threat_type(
        &self,
        threat_type: &str,
        min_expertise: f64,
    ) -> Vec<EngineReputation> {
        let reputations = self.reputations.read().await;
        let mut experts: Vec<_> = reputations
            .values()
            .filter(|rep| {
                rep.expertise_areas
                    .get(threat_type)
                    .map(|&score| score >= min_expertise)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        experts.sort_by(|a, b| {
            let a_score = a.expertise_areas.get(threat_type).unwrap_or(&0.0);
            let b_score = b.expertise_areas.get(threat_type).unwrap_or(&0.0);
            b_score.partial_cmp(a_score).unwrap()
        });

        experts
    }

    /// Calculate minimum stake required for an engine based on reputation
    pub async fn calculate_minimum_stake(&self, engine_id: &str) -> Result<u64, ReputationError> {
        let reputations = self.reputations.read().await;
        let reputation = reputations
            .get(engine_id)
            .ok_or(ReputationError::EngineNotFound)?;

        // Base stake requirements by tier
        let base_stake = match reputation.tier {
            ReputationTier::Novice => 100,
            ReputationTier::Skilled => 50,
            ReputationTier::Expert => 25,
            ReputationTier::Master => 10,
            ReputationTier::Legendary => 5,
        };

        // Adjust based on recent accuracy
        let recent_accuracy = self.calculate_recent_accuracy(reputation);
        let multiplier = if recent_accuracy < 0.5 {
            2.0 // Double the stake for poor recent performance
        } else if recent_accuracy > 0.8 {
            0.5 // Halve the stake for excellent recent performance
        } else {
            1.0
        };

        Ok((base_stake as f64 * multiplier) as u64)
    }

    /// Decay reputation scores over time for inactive engines
    pub async fn apply_reputation_decay(&self) {
        let mut reputations = self.reputations.write().await;
        let cutoff_date = Utc::now() - Duration::days(30);

        for reputation in reputations.values_mut() {
            if reputation.last_updated < cutoff_date {
                let decay_factor = 0.95; // 5% decay per month of inactivity
                reputation.total_score *= decay_factor;
                reputation.tier = self.calculate_tier(reputation.total_score);
                
                // Decay expertise scores as well
                for expertise_score in reputation.expertise_areas.values_mut() {
                    *expertise_score *= decay_factor;
                }
            }
        }
    }

    // Private helper methods

    fn calculate_reward(&self, base_reward: f64, consensus_confidence: f64, tier: ReputationTier) -> f64 {
        let tier_multiplier = match tier {
            ReputationTier::Novice => 1.0,
            ReputationTier::Skilled => 1.1,
            ReputationTier::Expert => 1.2,
            ReputationTier::Master => 1.3,
            ReputationTier::Legendary => 1.5,
        };

        // Higher rewards for high-confidence correct predictions
        let confidence_multiplier = 0.5 + (consensus_confidence * 1.5);
        
        base_reward * tier_multiplier * confidence_multiplier
    }

    fn calculate_penalty(&self, base_penalty: f64, consensus_confidence: f64) -> f64 {
        // Higher penalties for high-confidence incorrect predictions
        let confidence_multiplier = 0.5 + (consensus_confidence * 1.5);
        base_penalty * confidence_multiplier
    }

    fn calculate_tier(&self, total_score: f64) -> ReputationTier {
        match total_score as i32 {
            0..=100 => ReputationTier::Novice,
            101..=500 => ReputationTier::Skilled,
            501..=1000 => ReputationTier::Expert,
            1001..=2500 => ReputationTier::Master,
            _ => ReputationTier::Legendary,
        }
    }

    fn calculate_recent_accuracy(&self, reputation: &EngineReputation) -> f64 {
        let recent_cutoff = Utc::now() - Duration::days(30);
        let recent_events: Vec<_> = reputation
            .stake_history
            .iter()
            .filter(|event| event.timestamp > recent_cutoff)
            .filter(|event| event.actual_result.is_some())
            .collect();

        if recent_events.is_empty() {
            return reputation.accuracy_rate;
        }

        let correct_recent = recent_events
            .iter()
            .filter(|event| {
                event.actual_result.as_ref().unwrap() == &event.prediction
            })
            .count();

        correct_recent as f64 / recent_events.len() as f64
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

impl Default for ReputationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_engine() {
        let service = ReputationService::new();
        let engine_id = "test_engine".to_string();
        
        assert!(service.register_engine(engine_id.clone()).await.is_ok());
        assert!(service.register_engine(engine_id).await.is_err());
    }

    #[tokio::test]
    async fn test_reputation_update() {
        let service = ReputationService::new();
        let engine_id = "test_engine".to_string();
        let submission_id = Uuid::new_v4();
        
        service.register_engine(engine_id.clone()).await.unwrap();
        service.record_stake(&engine_id, submission_id, 100, ThreatVerdict::Malicious).await.unwrap();
        
        let update = ReputationUpdate {
            engine_id: engine_id.clone(),
            submission_id,
            stake_amount: 100,
            prediction: ThreatVerdict::Malicious,
            actual_result: ThreatVerdict::Malicious,
            threat_type: "trojan".to_string(),
            consensus_confidence: 0.9,
        };
        
        let stats = service.update_reputation(update).await.unwrap();
        assert!(stats.reward_earned > 0.0);
        assert_eq!(stats.accuracy_rate, 1.0);
    }

    #[tokio::test]
    async fn test_minimum_stake_calculation() {
        let service = ReputationService::new();
        let engine_id = "test_engine".to_string();
        
        service.register_engine(engine_id.clone()).await.unwrap();
        let min_stake = service.calculate_minimum_stake(&engine_id).await.unwrap();
        
        assert_eq!(min_stake, 100); // Novice tier base stake
    }
}