// backend/bounty-manager/src/services/consensus.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ConsensusService {
    min_submissions: u32,
    consensus_threshold: f32,
    enable_weighted_voting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub bounty_id: Uuid,
    pub final_verdict: String,
    pub confidence: f32,
    pub total_submissions: u32,
    pub verdict_distribution: HashMap<String, u32>,
    pub weighted_score: f32,
    pub consensus_reached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionData {
    pub submission_id: Uuid,
    pub verdict: String,
    pub confidence: f32,
    pub stake_amount: u64,
    pub reputation_score: f32,
}

impl ConsensusService {
    pub fn new(min_submissions: u32, consensus_threshold: f32, enable_weighted_voting: bool) -> Self {
        Self {
            min_submissions,
            consensus_threshold,
            enable_weighted_voting,
        }
    }

    /// Calculate consensus from submissions
    pub fn calculate_consensus(&self, submissions: Vec<SubmissionData>) -> ConsensusResult {
        let total_submissions = submissions.len() as u32;

        // Count verdict distribution
        let mut verdict_distribution: HashMap<String, u32> = HashMap::new();
        let mut verdict_weights: HashMap<String, f32> = HashMap::new();

        for submission in &submissions {
            *verdict_distribution.entry(submission.verdict.clone()).or_insert(0) += 1;

            if self.enable_weighted_voting {
                // Weight by stake and reputation
                let weight = (submission.stake_amount as f32 / 1000.0) 
                    * submission.reputation_score 
                    * submission.confidence;
                
                *verdict_weights.entry(submission.verdict.clone()).or_insert(0.0) += weight;
            }
        }

        // Determine final verdict
        let (final_verdict, consensus_reached) = if self.enable_weighted_voting {
            self.calculate_weighted_consensus(&verdict_weights, total_submissions)
        } else {
            self.calculate_simple_consensus(&verdict_distribution, total_submissions)
        };

        // Calculate overall confidence
        let confidence = self.calculate_confidence(&submissions, &final_verdict);

        // Calculate weighted score
        let total_weight: f32 = verdict_weights.values().sum();
        let winning_weight = verdict_weights.get(&final_verdict).copied().unwrap_or(0.0);
        let weighted_score = if total_weight > 0.0 {
            winning_weight / total_weight
        } else {
            0.0
        };

        ConsensusResult {
            bounty_id: submissions.first().map(|s| Uuid::new_v4()).unwrap_or(Uuid::new_v4()),
            final_verdict,
            confidence,
            total_submissions,
            verdict_distribution,
            weighted_score,
            consensus_reached: consensus_reached && total_submissions >= self.min_submissions,
        }
    }

    fn calculate_simple_consensus(
        &self,
        distribution: &HashMap<String, u32>,
        total: u32,
    ) -> (String, bool) {
        let (verdict, count) = distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(v, c)| (v.clone(), *c))
            .unwrap_or(("Unknown".to_string(), 0));

        let percentage = count as f32 / total as f32;
        let reached = percentage >= self.consensus_threshold;

        (verdict, reached)
    }

    fn calculate_weighted_consensus(
        &self,
        weights: &HashMap<String, f32>,
        total: u32,
    ) -> (String, bool) {
        let total_weight: f32 = weights.values().sum();
        
        let (verdict, weight) = weights
            .iter()
            .max_by(|(_, w1), (_, w2)| w1.partial_cmp(w2).unwrap())
            .map(|(v, w)| (v.clone(), *w))
            .unwrap_or(("Unknown".to_string(), 0.0));

        let percentage = if total_weight > 0.0 {
            weight / total_weight
        } else {
            0.0
        };

        let reached = percentage >= self.consensus_threshold;

        (verdict, reached)
    }

    fn calculate_confidence(&self, submissions: &[SubmissionData], final_verdict: &str) -> f32 {
        let matching: Vec<&SubmissionData> = submissions
            .iter()
            .filter(|s| s.verdict == final_verdict)
            .collect();

        if matching.is_empty() {
            return 0.0;
        }

        let total_confidence: f32 = matching.iter().map(|s| s.confidence).sum();
        total_confidence / matching.len() as f32
    }

    /// Check if consensus can be reached
    pub fn can_reach_consensus(&self, submission_count: u32) -> bool {
        submission_count >= self.min_submissions
    }

    /// Calculate accuracy score for a submission
    pub fn calculate_accuracy_score(
        &self,
        submission_verdict: &str,
        final_verdict: &str,
        confidence: f32,
    ) -> f32 {
        if submission_verdict == final_verdict {
            // Correct verdict - score based on confidence
            0.5 + (confidence * 0.5)
        } else {
            // Incorrect verdict - penalize
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_consensus() {
        let service = ConsensusService::new(3, 0.75, false);
        
        let submissions = vec![
            SubmissionData {
                submission_id: Uuid::new_v4(),
                verdict: "Malicious".to_string(),
                confidence: 0.9,
                stake_amount: 1000,
                reputation_score: 1.0,
            },
            SubmissionData {
                submission_id: Uuid::new_v4(),
                verdict: "Malicious".to_string(),
                confidence: 0.85,
                stake_amount: 1000,
                reputation_score: 1.0,
            },
            SubmissionData {
                submission_id: Uuid::new_v4(),
                verdict: "Malicious".to_string(),
                confidence: 0.8,
                stake_amount: 1000,
                reputation_score: 1.0,
            },
            SubmissionData {
                submission_id: Uuid::new_v4(),
                verdict: "Benign".to_string(),
                confidence: 0.7,
                stake_amount: 1000,
                reputation_score: 1.0,
            },
        ];

        let result = service.calculate_consensus(submissions);
        assert_eq!(result.final_verdict, "Malicious");
        assert!(result.consensus_reached);
    }
}
