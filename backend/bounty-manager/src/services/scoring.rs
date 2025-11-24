// backend/bounty-manager/src/services/scoring.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service for calculating various scores in the bounty system
#[derive(Clone)]
pub struct ScoringService {
    weights: ScoringWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    pub accuracy_weight: f32,
    pub confidence_weight: f32,
    pub stake_weight: f32,
    pub reputation_weight: f32,
    pub timeliness_weight: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            accuracy_weight: 0.40,      // 40% weight
            confidence_weight: 0.20,    // 20% weight
            stake_weight: 0.15,         // 15% weight
            reputation_weight: 0.15,    // 15% weight
            timeliness_weight: 0.10,    // 10% weight
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    pub overall_score: f32,          // 0.0 to 1.0
    pub component_scores: ComponentScores,
    pub grade: QualityGrade,
    pub feedback: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScores {
    pub accuracy_score: f32,
    pub confidence_score: f32,
    pub detail_score: f32,
    pub consistency_score: f32,
    pub timeliness_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityGrade {
    Excellent,  // 0.9-1.0
    Good,       // 0.7-0.9
    Fair,       // 0.5-0.7
    Poor,       // 0.3-0.5
    VeryPoor,   // 0.0-0.3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionScore {
    pub submission_id: String,
    pub weighted_score: f32,
    pub accuracy_contribution: f32,
    pub confidence_contribution: f32,
    pub stake_contribution: f32,
    pub reputation_contribution: f32,
    pub timeliness_contribution: f32,
    pub total_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusScore {
    pub agreement_score: f32,        // How much submissions agree
    pub confidence_score: f32,       // Average confidence
    pub participation_score: f32,    // Number of participants
    pub quality_score: f32,          // Average quality of submissions
    pub overall_consensus_strength: f32,
}

impl ScoringService {
    pub fn new() -> Self {
        Self {
            weights: ScoringWeights::default(),
        }
    }

    pub fn with_weights(weights: ScoringWeights) -> Self {
        Self { weights }
    }

    /// Calculate quality score for a submission
    pub fn calculate_quality_score(
        &self,
        has_all_fields: bool,
        confidence: f32,
        detail_level: DetailLevel,
        is_consistent: bool,
        submission_time_ms: u64,
        deadline_time_ms: u64,
    ) -> QualityScore {
        // Accuracy score (based on completeness)
        let accuracy_score = if has_all_fields { 1.0 } else { 0.5 };

        // Confidence score (normalized)
        let confidence_score = confidence.clamp(0.0, 1.0);

        // Detail score based on analysis depth
        let detail_score = match detail_level {
            DetailLevel::Minimal => 0.3,
            DetailLevel::Basic => 0.5,
            DetailLevel::Standard => 0.7,
            DetailLevel::Comprehensive => 0.9,
            DetailLevel::Expert => 1.0,
        };

        // Consistency score
        let consistency_score = if is_consistent { 1.0 } else { 0.4 };

        // Timeliness score (early submissions score higher)
        let time_ratio = submission_time_ms as f32 / deadline_time_ms as f32;
        let timeliness_score = (1.0 - time_ratio).clamp(0.0, 1.0);

        // Calculate overall score (weighted average)
        let overall_score = (
            accuracy_score * 0.3 +
            confidence_score * 0.2 +
            detail_score * 0.25 +
            consistency_score * 0.15 +
            timeliness_score * 0.10
        );

        let grade = Self::score_to_grade(overall_score);

        // Generate feedback
        let mut feedback = Vec::new();
        if !has_all_fields {
            feedback.push("Missing some required fields".to_string());
        }
        if confidence < 0.7 {
            feedback.push("Low confidence score - consider more thorough analysis".to_string());
        }
        if matches!(detail_level, DetailLevel::Minimal | DetailLevel::Basic) {
            feedback.push("Analysis could be more detailed".to_string());
        }
        if !is_consistent {
            feedback.push("Inconsistencies detected in the analysis".to_string());
        }
        if time_ratio > 0.8 {
            feedback.push("Submitted close to deadline".to_string());
        }

        QualityScore {
            overall_score,
            component_scores: ComponentScores {
                accuracy_score,
                confidence_score,
                detail_score,
                consistency_score,
                timeliness_score,
            },
            grade,
            feedback,
        }
    }

    /// Calculate weighted score for consensus
    pub fn calculate_weighted_submission_score(
        &self,
        is_correct: bool,
        confidence: f32,
        stake_amount: u64,
        reputation_score: f32,
        response_time_ms: u64,
        avg_response_time_ms: u64,
    ) -> SubmissionScore {
        let weights = &self.weights;

        // Accuracy contribution (0 if wrong, full weight if correct)
        let accuracy_contribution = if is_correct {
            weights.accuracy_weight
        } else {
            0.0
        };

        // Confidence contribution
        let confidence_contribution = confidence * weights.confidence_weight;

        // Stake contribution (normalized by log)
        let stake_normalized = ((stake_amount as f32).ln() / 15.0).min(1.0); // Normalize to 0-1
        let stake_contribution = stake_normalized * weights.stake_weight;

        // Reputation contribution (already 0-1)
        let reputation_contribution = reputation_score * weights.reputation_weight;

        // Timeliness contribution (faster is better)
        let time_ratio = if avg_response_time_ms > 0 {
            (avg_response_time_ms as f32 / response_time_ms as f32).min(2.0) / 2.0 // Cap at 2x benefit
        } else {
            0.5
        };
        let timeliness_contribution = time_ratio * weights.timeliness_weight;

        // Total weighted score
        let weighted_score = accuracy_contribution +
                           confidence_contribution +
                           stake_contribution +
                           reputation_contribution +
                           timeliness_contribution;

        let total_weight = weights.accuracy_weight +
                         weights.confidence_weight +
                         weights.stake_weight +
                         weights.reputation_weight +
                         weights.timeliness_weight;

        SubmissionScore {
            submission_id: "".to_string(), // Set by caller
            weighted_score,
            accuracy_contribution,
            confidence_contribution,
            stake_contribution,
            reputation_contribution,
            timeliness_contribution,
            total_weight,
        }
    }

    /// Calculate consensus strength score
    pub fn calculate_consensus_score(
        &self,
        total_submissions: u32,
        matching_submissions: u32,
        avg_confidence: f32,
        avg_quality: f32,
    ) -> ConsensusScore {
        // Agreement score (what % agree with consensus)
        let agreement_score = if total_submissions > 0 {
            matching_submissions as f32 / total_submissions as f32
        } else {
            0.0
        };

        // Confidence score (average confidence of matching submissions)
        let confidence_score = avg_confidence.clamp(0.0, 1.0);

        // Participation score (more participants = stronger consensus)
        let participation_score = ((total_submissions as f32).ln() / 5.0).min(1.0);

        // Quality score
        let quality_score = avg_quality.clamp(0.0, 1.0);

        // Overall consensus strength (weighted combination)
        let overall_consensus_strength = (
            agreement_score * 0.4 +
            confidence_score * 0.3 +
            participation_score * 0.15 +
            quality_score * 0.15
        );

        ConsensusScore {
            agreement_score,
            confidence_score,
            participation_score,
            quality_score,
            overall_consensus_strength,
        }
    }

    /// Calculate reward distribution scores
    pub fn calculate_reward_distribution(
        &self,
        submission_scores: Vec<SubmissionScore>,
        total_reward: u64,
    ) -> HashMap<String, u64> {
        let mut distribution = HashMap::new();

        // Calculate total weight
        let total_weight: f32 = submission_scores
            .iter()
            .map(|s| s.weighted_score)
            .sum();

        if total_weight == 0.0 {
            return distribution;
        }

        // Distribute rewards proportionally
        for score in submission_scores {
            let reward_share = (score.weighted_score / total_weight) * total_reward as f32;
            distribution.insert(score.submission_id.clone(), reward_share as u64);
        }

        distribution
    }

    /// Calculate penalty amount for incorrect submission
    pub fn calculate_penalty(
        &self,
        stake_amount: u64,
        confidence: f32,
        severity: PenaltySeverity,
    ) -> u64 {
        let base_penalty: f32 = match severity {
            PenaltySeverity::Minor => 0.10,      // 10% slash
            PenaltySeverity::Moderate => 0.25,   // 25% slash
            PenaltySeverity::Severe => 0.50,     // 50% slash
            PenaltySeverity::Critical => 1.0,    // 100% slash
        };

        // Increase penalty for high confidence incorrect submissions
        let confidence_multiplier: f32 = if confidence > 0.8 {
            1.5 // Penalize overconfidence
        } else {
            1.0
        };

        let penalty_rate = (base_penalty * confidence_multiplier).min(1.0_f32);
        (stake_amount as f32 * penalty_rate) as u64
    }

    /// Convert score to grade
    fn score_to_grade(score: f32) -> QualityGrade {
        match score {
            s if s >= 0.9 => QualityGrade::Excellent,
            s if s >= 0.7 => QualityGrade::Good,
            s if s >= 0.5 => QualityGrade::Fair,
            s if s >= 0.3 => QualityGrade::Poor,
            _ => QualityGrade::VeryPoor,
        }
    }
}

impl Default for ScoringService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetailLevel {
    Minimal,
    Basic,
    Standard,
    Comprehensive,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PenaltySeverity {
    Minor,
    Moderate,
    Severe,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_score_calculation() {
        let service = ScoringService::new();
        
        let score = service.calculate_quality_score(
            true,                    // has all fields
            0.95,                    // high confidence
            DetailLevel::Expert,     // expert level detail
            true,                    // consistent
            100000,                  // submitted at 100s
            1000000,                 // deadline at 1000s
        );

        assert!(score.overall_score >= 0.9);
        assert_eq!(score.grade, QualityGrade::Excellent);
    }

    #[test]
    fn test_weighted_submission_score() {
        let service = ScoringService::new();
        
        let score = service.calculate_weighted_submission_score(
            true,    // correct
            0.9,     // 90% confidence
            10000,   // stake
            0.85,    // reputation
            500,     // response time
            1000,    // avg response time
        );

        assert!(score.weighted_score > 0.5);
    }

    #[test]
    fn test_consensus_score() {
        let service = ScoringService::new();
        
        let score = service.calculate_consensus_score(
            10,   // total submissions
            8,    // matching submissions
            0.85, // avg confidence
            0.8,  // avg quality
        );

        assert_eq!(score.agreement_score, 0.8);
        assert!(score.overall_consensus_strength >= 0.6);
    }

    #[test]
    fn test_penalty_calculation() {
        let service = ScoringService::new();
        
        // Minor penalty
        let penalty = service.calculate_penalty(
            10000,
            0.5,
            PenaltySeverity::Minor,
        );
        assert_eq!(penalty, 1000); // 10% of 10000

        // Severe penalty with high confidence
        let penalty = service.calculate_penalty(
            10000,
            0.95,
            PenaltySeverity::Severe,
        );
        assert!(penalty >= 5000); // At least 50%
    }

    #[test]
    fn test_reward_distribution() {
        let service = ScoringService::new();
        
        let scores = vec![
            SubmissionScore {
                submission_id: "sub1".to_string(),
                weighted_score: 0.8,
                accuracy_contribution: 0.4,
                confidence_contribution: 0.2,
                stake_contribution: 0.1,
                reputation_contribution: 0.05,
                timeliness_contribution: 0.05,
                total_weight: 1.0,
            },
            SubmissionScore {
                submission_id: "sub2".to_string(),
                weighted_score: 0.2,
                accuracy_contribution: 0.1,
                confidence_contribution: 0.05,
                stake_contribution: 0.03,
                reputation_contribution: 0.01,
                timeliness_contribution: 0.01,
                total_weight: 1.0,
            },
        ];

        let distribution = service.calculate_reward_distribution(scores, 100000);
        
        assert_eq!(*distribution.get("sub1").unwrap(), 80000);
        assert_eq!(*distribution.get("sub2").unwrap(), 20000);
    }
}
