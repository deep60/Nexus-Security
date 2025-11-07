use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub submission_id: String,
    pub final_verdict: Verdict,
    pub confidence_score: f64,
    pub participating_engines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Malicious,
    Benign,
    Suspicious,
    Unknown,
}
