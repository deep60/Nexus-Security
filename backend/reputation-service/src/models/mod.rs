use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineReputation {
    pub engine_id: String,
    pub reputation_score: f64,
    pub total_analyses: i64,
    pub correct_analyses: i64,
    pub accuracy_rate: f64,
    pub total_earnings: f64,
    pub total_losses: f64,
}
