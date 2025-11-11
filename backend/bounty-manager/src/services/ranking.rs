// backend/bounty-manager/src/services/ranking.rs

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Service for ranking and leaderboard management
#[derive(Clone)]
pub struct RankingService {
    db: PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRanking {
    pub rank: u32,
    pub engine_id: String,
    pub engine_name: Option<String>,
    pub score: f32,
    pub total_submissions: u32,
    pub correct_submissions: u32,
    pub accuracy_rate: f32,
    pub total_rewards: i64,
    pub avg_response_time_ms: Option<u64>,
    pub last_active: DateTime<Utc>,
    pub tier: RankingTier,
    pub badges: Vec<Badge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RankingTier {
    Novice,      // 0-20 score
    Apprentice,  // 20-40 score
    Expert,      // 40-60 score
    Master,      // 60-80 score
    GrandMaster, // 80-100 score
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub badge_type: BadgeType,
    pub earned_at: DateTime<Utc>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BadgeType {
    FirstSubmission,
    HundredSubmissions,
    ThousandSubmissions,
    PerfectAccuracy,     // 100% accuracy for 10+ submissions
    SpeedDemon,          // Fastest avg response time
    ConsistentPerformer, // 90%+ accuracy over 50+ submissions
    TopContributor,      // Most submissions in a month
    DisputeWinner,       // Won a dispute
    MalwareHunter,       // Specialization in malware detection
    ZeroDayFinder,       // Detected unknown threats
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardFilters {
    pub time_period: TimePeriod,
    pub category: Option<LeaderboardCategory>,
    pub tier: Option<RankingTier>,
    pub min_submissions: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimePeriod {
    AllTime,
    ThisMonth,
    ThisWeek,
    Today,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeaderboardCategory {
    OverallScore,
    Accuracy,
    TotalRewards,
    ResponseTime,
    Submissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub rankings: Vec<EngineRanking>,
    pub total_engines: u32,
    pub time_period: TimePeriod,
    pub category: LeaderboardCategory,
    pub updated_at: DateTime<Utc>,
}

impl RankingService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get global leaderboard
    pub async fn get_leaderboard(
        &self,
        filters: LeaderboardFilters,
        limit: u32,
    ) -> Result<LeaderboardResponse, RankingError> {
        // TODO: Implement database query with filters
        // For now, return mock data
        
        let rankings = self.get_mock_rankings(limit);

        Ok(LeaderboardResponse {
            rankings,
            total_engines: 100,
            time_period: filters.time_period,
            category: filters.category.unwrap_or(LeaderboardCategory::OverallScore),
            updated_at: Utc::now(),
        })
    }

    /// Get ranking for a specific engine
    pub async fn get_engine_ranking(&self, engine_id: &str) -> Result<EngineRanking, RankingError> {
        // TODO: Implement database query
        Ok(self.create_mock_ranking(1, engine_id))
    }

    /// Calculate tier based on score
    pub fn calculate_tier(score: f32) -> RankingTier {
        match score {
            s if s >= 80.0 => RankingTier::GrandMaster,
            s if s >= 60.0 => RankingTier::Master,
            s if s >= 40.0 => RankingTier::Expert,
            s if s >= 20.0 => RankingTier::Apprentice,
            _ => RankingTier::Novice,
        }
    }

    /// Calculate ranking score from metrics
    pub fn calculate_ranking_score(
        accuracy_rate: f32,
        total_submissions: u32,
        avg_response_time_ms: Option<u64>,
        total_rewards: i64,
    ) -> f32 {
        // Weighted scoring formula
        let accuracy_score = accuracy_rate * 40.0; // Max 40 points

        // Experience score (logarithmic scaling)
        let experience_score = ((total_submissions as f32).ln() * 3.0).min(25.0); // Max 25 points

        // Speed score (faster is better, capped)
        let speed_score = if let Some(time) = avg_response_time_ms {
            let normalized = 1000.0 / (time as f32 + 100.0); // Normalize response time
            (normalized * 15.0).min(15.0) // Max 15 points
        } else {
            0.0
        };

        // Rewards score (logarithmic scaling)
        let reward_score = ((total_rewards as f32 + 1.0).ln() * 2.0).min(20.0); // Max 20 points

        // Total score (0-100)
        (accuracy_score + experience_score + speed_score + reward_score).min(100.0)
    }

    /// Check and award badges
    pub async fn check_and_award_badges(&self, engine_id: &str) -> Result<Vec<Badge>, RankingError> {
        let mut new_badges = Vec::new();

        // TODO: Implement badge checking logic
        // Example checks:
        // - First submission
        // - Milestone submissions (100, 1000)
        // - Perfect accuracy streak
        // - Top rankings

        Ok(new_badges)
    }

    /// Get top performers for a time period
    pub async fn get_top_performers(
        &self,
        time_period: TimePeriod,
        limit: u32,
    ) -> Result<Vec<EngineRanking>, RankingError> {
        // TODO: Implement time-based filtering
        Ok(self.get_mock_rankings(limit))
    }

    /// Get category-specific rankings
    pub async fn get_category_rankings(
        &self,
        category: LeaderboardCategory,
        limit: u32,
    ) -> Result<Vec<EngineRanking>, RankingError> {
        // TODO: Implement category-specific sorting
        let mut rankings = self.get_mock_rankings(limit);

        // Sort based on category
        match category {
            LeaderboardCategory::Accuracy => {
                rankings.sort_by(|a, b| b.accuracy_rate.partial_cmp(&a.accuracy_rate).unwrap());
            }
            LeaderboardCategory::TotalRewards => {
                rankings.sort_by(|a, b| b.total_rewards.cmp(&a.total_rewards));
            }
            LeaderboardCategory::Submissions => {
                rankings.sort_by(|a, b| b.total_submissions.cmp(&a.total_submissions));
            }
            _ => {}
        }

        Ok(rankings)
    }

    /// Update ranking after a new submission
    pub async fn update_ranking_after_submission(
        &self,
        engine_id: &str,
        was_correct: bool,
        response_time_ms: u64,
    ) -> Result<(), RankingError> {
        // TODO: Implement incremental ranking update
        // This would:
        // 1. Update submission count
        // 2. Update accuracy rate
        // 3. Update avg response time
        // 4. Recalculate score
        // 5. Update rank position
        // 6. Check for new badges

        Ok(())
    }

    /// Get comparative stats (engine vs average)
    pub async fn get_comparative_stats(&self, engine_id: &str) -> Result<ComparativeStats, RankingError> {
        // TODO: Implement comparison with global averages
        
        Ok(ComparativeStats {
            engine_accuracy: 0.85,
            average_accuracy: 0.75,
            engine_submissions: 150,
            average_submissions: 50,
            engine_rewards: 50000,
            average_rewards: 25000,
            percentile: 75.0, // Top 25%
        })
    }

    // Mock data helpers
    fn get_mock_rankings(&self, limit: u32) -> Vec<EngineRanking> {
        (1..=limit)
            .map(|i| self.create_mock_ranking(i, &format!("engine_{}", i)))
            .collect()
    }

    fn create_mock_ranking(&self, rank: u32, engine_id: &str) -> EngineRanking {
        let score = 95.0 - (rank as f32 * 2.0);
        let tier = Self::calculate_tier(score);

        EngineRanking {
            rank,
            engine_id: engine_id.to_string(),
            engine_name: Some(format!("Engine {}", rank)),
            score,
            total_submissions: 100 + (rank * 10),
            correct_submissions: 90 + (rank * 8),
            accuracy_rate: 0.9 - (rank as f32 * 0.01),
            total_rewards: 100000 - (rank as i64 * 1000),
            avg_response_time_ms: Some(1000 + (rank as u64 * 100)),
            last_active: Utc::now(),
            tier,
            badges: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparativeStats {
    pub engine_accuracy: f32,
    pub average_accuracy: f32,
    pub engine_submissions: u32,
    pub average_submissions: u32,
    pub engine_rewards: i64,
    pub average_rewards: i64,
    pub percentile: f32, // 0-100, higher is better
}

#[derive(Debug, thiserror::Error)]
pub enum RankingError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Engine not found: {0}")]
    EngineNotFound(String),

    #[error("Invalid ranking parameters: {0}")]
    InvalidParameters(String),
}

impl From<sqlx::Error> for RankingError {
    fn from(err: sqlx::Error) -> Self {
        RankingError::DatabaseError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_calculation() {
        assert_eq!(RankingService::calculate_tier(85.0), RankingTier::GrandMaster);
        assert_eq!(RankingService::calculate_tier(65.0), RankingTier::Master);
        assert_eq!(RankingService::calculate_tier(45.0), RankingTier::Expert);
        assert_eq!(RankingService::calculate_tier(25.0), RankingTier::Apprentice);
        assert_eq!(RankingService::calculate_tier(10.0), RankingTier::Novice);
    }

    #[test]
    fn test_ranking_score_calculation() {
        let score = RankingService::calculate_ranking_score(
            0.95,      // 95% accuracy
            100,       // 100 submissions
            Some(500), // 500ms avg response
            50000,     // 50k rewards
        );

        assert!(score >= 70.0 && score <= 100.0);
    }

    #[test]
    fn test_perfect_accuracy_score() {
        let score = RankingService::calculate_ranking_score(
            1.0,       // 100% accuracy
            1000,      // Many submissions
            Some(100), // Very fast
            100000,    // High rewards
        );

        assert!(score >= 90.0);
    }
}
