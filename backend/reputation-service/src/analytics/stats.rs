use crate::models::ReputationStats;
use rust_decimal::Decimal;

pub fn calculate_stats() -> ReputationStats {
    ReputationStats {
        total_users: 0,
        avg_score: Decimal::new(0, 0),
        median_score: Decimal::new(0, 0),
        avg_accuracy: Decimal::new(0, 0),
        score_distribution: vec![],
    }
}
