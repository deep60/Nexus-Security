use crate::config::ReputationConfig;
use crate::models::{ReputationUpdateRequest, UserReputation};
use rust_decimal::Decimal;

pub struct ReputationScorer {
    config: ReputationConfig,
}

impl ReputationScorer {
    pub fn new(config: ReputationConfig) -> Self {
        Self { config }
    }

    /// Calculate reputation change based on submission result
    pub fn calculate_score_change(
        &self,
        current_reputation: &UserReputation,
        update: &ReputationUpdateRequest,
    ) -> i32 {
        let mut base_change = if update.was_correct {
            self.config.correct_analysis_points
        } else {
            self.config.incorrect_analysis_penalty
        };

        // Apply streak bonus
        if update.was_correct && current_reputation.current_streak > 0 {
            let streak_multiplier = 1.0 + (current_reputation.current_streak as f64 * 0.1);
            base_change = (base_change as f64 * streak_multiplier.min(self.config.streak_bonus_multiplier)) as i32;
        }

        // Apply consensus bonus
        if update.in_consensus {
            base_change += self.config.consensus_bonus;
        }

        // Apply early submission bonus
        if update.was_early {
            base_change += self.config.early_submission_bonus;
        }

        // Apply confidence multiplier
        let confidence_multiplier = update.confidence_score.to_string().parse::<f64>().unwrap_or(1.0);
        base_change = (base_change as f64 * confidence_multiplier) as i32;

        base_change
    }

    /// Calculate accuracy rate
    pub fn calculate_accuracy(correct: i32, total: i32) -> Decimal {
        if total == 0 {
            return Decimal::new(0, 0);
        }
        Decimal::from(correct) / Decimal::from(total)
    }

    /// Apply time decay to reputation score
    pub fn apply_decay(&self, current_score: i32, days_inactive: f64) -> i32 {
        let decay_factor = 1.0 - (self.config.decay_rate_per_day * days_inactive);
        let new_score = (current_score as f64 * decay_factor.max(0.0)) as i32;
        new_score.max(self.config.min_score)
    }

    /// Calculate percentile rank
    pub fn calculate_percentile(user_rank: i32, total_users: i32) -> Decimal {
        if total_users == 0 {
            return Decimal::new(0, 0);
        }
        let percentile = ((total_users - user_rank + 1) as f64 / total_users as f64) * 100.0;
        Decimal::try_from(percentile).unwrap_or(Decimal::new(0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

    fn test_config() -> ReputationConfig {
        ReputationConfig {
            base_score: 1000,
            correct_analysis_points: 50,
            incorrect_analysis_penalty: -100,
            streak_bonus_multiplier: 1.5,
            decay_rate_per_day: 0.001,
            min_score: 0,
            max_score: 10000,
            consensus_bonus: 25,
            early_submission_bonus: 10,
        }
    }

    #[test]
    fn test_score_calculation() {
        let scorer = ReputationScorer::new(test_config());
        let reputation = UserReputation {
            user_id: Uuid::new_v4(),
            current_score: 1000,
            highest_score: 1000,
            lowest_score: 1000,
            total_submissions: 10,
            correct_submissions: 8,
            incorrect_submissions: 2,
            accuracy_rate: Decimal::new(80, 2),
            current_streak: 3,
            best_streak: 5,
            total_earned: Decimal::new(0, 0),
            rank: Some(1),
            percentile: Some(Decimal::new(95, 0)),
            last_updated: Utc::now(),
            created_at: Utc::now(),
        };

        let update = ReputationUpdateRequest {
            user_id: Uuid::new_v4(),
            submission_id: Uuid::new_v4(),
            bounty_id: Uuid::new_v4(),
            was_correct: true,
            confidence_score: Decimal::new(90, 2),
            in_consensus: true,
            was_early: true,
        };

        let change = scorer.calculate_score_change(&reputation, &update);
        assert!(change > 50); // Should be more than base due to bonuses
    }

    #[test]
    fn test_accuracy_calculation() {
        let accuracy = ReputationScorer::calculate_accuracy(8, 10);
        assert_eq!(accuracy, Decimal::new(80, 2));
    }
}
