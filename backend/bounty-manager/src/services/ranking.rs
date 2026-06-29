// backend/bounty-manager/src/services/ranking.rs

use crate::models::ReputationModel;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

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

    /// Get global leaderboard backed by the reputations table
    pub async fn get_leaderboard(
        &self,
        filters: LeaderboardFilters,
        limit: u32,
    ) -> Result<LeaderboardResponse, RankingError> {
        let time_cutoff = self.time_period_to_cutoff(&filters.time_period);
        let category = filters
            .category
            .clone()
            .unwrap_or(LeaderboardCategory::OverallScore);

        // Query reputations table with optional time filter
        let mut query = String::from("SELECT * FROM reputations WHERE updated_at >= $1");

        if let Some(min_subs) = filters.min_submissions {
            query.push_str(&format!(" AND total_submissions >= {min_subs}"));
        }

        let order_clause = match &category {
            LeaderboardCategory::Accuracy => " ORDER BY accuracy_rate DESC",
            LeaderboardCategory::TotalRewards => " ORDER BY rewards_earned DESC",
            LeaderboardCategory::Submissions => " ORDER BY total_submissions DESC",
            _ => " ORDER BY reputation_score DESC",
        };
        query.push_str(order_clause);
        query.push_str(&format!(" LIMIT {limit}"));

        let records = sqlx::query_as::<_, ReputationModel>(&query)
            .bind(time_cutoff)
            .fetch_all(&self.db)
            .await?;

        let total_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM reputations WHERE updated_at >= $1")
                .bind(time_cutoff)
                .fetch_one(&self.db)
                .await
                .unwrap_or((0,));

        let rankings: Vec<EngineRanking> = records
            .into_iter()
            .enumerate()
            .map(|(idx, rep)| self.reputation_to_ranking(idx as u32 + 1, rep))
            .collect();

        Ok(LeaderboardResponse {
            rankings,
            total_engines: total_count.0 as u32,
            time_period: filters.time_period,
            category,
            updated_at: Utc::now(),
        })
    }

    /// Get ranking for a specific engine from the reputations table
    pub async fn get_engine_ranking(&self, engine_id: &str) -> Result<EngineRanking, RankingError> {
        let reputation = ReputationModel::find_by_id(&self.db, engine_id)
            .await?
            .ok_or_else(|| RankingError::EngineNotFound(engine_id.to_string()))?;

        // Calculate rank position by counting engines with higher scores
        let rank_row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) + 1 FROM reputations WHERE reputation_score > $1")
                .bind(reputation.reputation_score)
                .fetch_one(&self.db)
                .await
                .unwrap_or((1,));

        Ok(self.reputation_to_ranking(rank_row.0 as u32, reputation))
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

    /// Check and award badges based on engine performance from DB
    pub async fn check_and_award_badges(
        &self,
        engine_id: &str,
    ) -> Result<Vec<Badge>, RankingError> {
        let mut new_badges = Vec::new();
        let now = Utc::now();

        let reputation = match ReputationModel::find_by_id(&self.db, engine_id).await? {
            Some(rep) => rep,
            None => return Ok(new_badges),
        };

        // First submission badge
        if reputation.total_submissions >= 1 {
            new_badges.push(Badge {
                badge_type: BadgeType::FirstSubmission,
                earned_at: now,
                description: "Submitted your first analysis".to_string(),
            });
        }

        // Milestone badges
        if reputation.total_submissions >= 100 {
            new_badges.push(Badge {
                badge_type: BadgeType::HundredSubmissions,
                earned_at: now,
                description: "Completed 100 submissions".to_string(),
            });
        }

        if reputation.total_submissions >= 1000 {
            new_badges.push(Badge {
                badge_type: BadgeType::ThousandSubmissions,
                earned_at: now,
                description: "Completed 1,000 submissions".to_string(),
            });
        }

        // Perfect accuracy (100% over 10+ submissions)
        if reputation.total_submissions >= 10 && reputation.accuracy_rate >= 1.0 {
            new_badges.push(Badge {
                badge_type: BadgeType::PerfectAccuracy,
                earned_at: now,
                description: "Perfect accuracy over 10+ submissions".to_string(),
            });
        }

        // Consistent performer (90%+ over 50+ submissions)
        if reputation.total_submissions >= 50 && reputation.accuracy_rate >= 0.9 {
            new_badges.push(Badge {
                badge_type: BadgeType::ConsistentPerformer,
                earned_at: now,
                description: "90%+ accuracy over 50+ submissions".to_string(),
            });
        }

        Ok(new_badges)
    }

    /// Get top performers for a time period from DB
    pub async fn get_top_performers(
        &self,
        time_period: TimePeriod,
        limit: u32,
    ) -> Result<Vec<EngineRanking>, RankingError> {
        let cutoff = self.time_period_to_cutoff(&time_period);

        let records = sqlx::query_as::<_, ReputationModel>(
            "SELECT * FROM reputations WHERE updated_at >= $1 ORDER BY reputation_score DESC LIMIT $2"
        )
        .bind(cutoff)
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        let rankings = records
            .into_iter()
            .enumerate()
            .map(|(idx, rep)| self.reputation_to_ranking(idx as u32 + 1, rep))
            .collect();

        Ok(rankings)
    }

    /// Get category-specific rankings from DB
    pub async fn get_category_rankings(
        &self,
        category: LeaderboardCategory,
        limit: u32,
    ) -> Result<Vec<EngineRanking>, RankingError> {
        let order_clause = match &category {
            LeaderboardCategory::Accuracy => "accuracy_rate DESC",
            LeaderboardCategory::TotalRewards => "rewards_earned DESC",
            LeaderboardCategory::Submissions => "total_submissions DESC",
            LeaderboardCategory::ResponseTime => "reputation_score DESC", // No response time in model; fallback
            LeaderboardCategory::OverallScore => "reputation_score DESC",
        };

        let query = format!("SELECT * FROM reputations ORDER BY {order_clause} LIMIT $1");

        let records = sqlx::query_as::<_, ReputationModel>(&query)
            .bind(limit as i64)
            .fetch_all(&self.db)
            .await?;

        let rankings = records
            .into_iter()
            .enumerate()
            .map(|(idx, rep)| self.reputation_to_ranking(idx as u32 + 1, rep))
            .collect();

        Ok(rankings)
    }

    /// Update ranking after a new submission — increments counts and recalculates score
    pub async fn update_ranking_after_submission(
        &self,
        engine_id: &str,
        was_correct: bool,
        _response_time_ms: u64,
    ) -> Result<(), RankingError> {
        // 1. Increment submission count in the reputations table
        ReputationModel::increment_submission(&self.db, engine_id, was_correct)
            .await
            .map_err(|e| RankingError::DatabaseError(e.to_string()))?;

        // 2. Refetch updated reputation and recalculate composite score
        if let Some(rep) = ReputationModel::find_by_id(&self.db, engine_id).await? {
            let new_score = Self::calculate_ranking_score(
                rep.accuracy_rate,
                rep.total_submissions as u32,
                None,
                rep.rewards_earned,
            );

            // Normalize to 0-1 for the reputation_score column
            let normalized = new_score / 100.0;
            sqlx::query("UPDATE reputations SET reputation_score = $1, updated_at = $2 WHERE engine_id = $3")
                .bind(normalized)
                .bind(Utc::now())
                .bind(engine_id)
                .execute(&self.db)
                .await?;
        }

        // 3. Check for new badge awards
        let _new_badges = self.check_and_award_badges(engine_id).await?;

        Ok(())
    }

    /// Get comparative stats (engine vs global averages) from DB
    pub async fn get_comparative_stats(
        &self,
        engine_id: &str,
    ) -> Result<ComparativeStats, RankingError> {
        let reputation = ReputationModel::find_by_id(&self.db, engine_id)
            .await?
            .ok_or_else(|| RankingError::EngineNotFound(engine_id.to_string()))?;

        // Fetch global averages
        let avg_row: (Option<f64>, Option<f64>, Option<f64>) = sqlx::query_as(
            "SELECT AVG(accuracy_rate), AVG(total_submissions), AVG(rewards_earned) FROM reputations"
        )
        .fetch_one(&self.db)
        .await
        .unwrap_or((Some(0.0), Some(0.0), Some(0.0)));

        // Calculate percentile: % of engines this engine scores better than
        let total_engines: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM reputations")
            .fetch_one(&self.db)
            .await
            .unwrap_or((1,));

        let engines_below: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM reputations WHERE reputation_score < $1")
                .bind(reputation.reputation_score)
                .fetch_one(&self.db)
                .await
                .unwrap_or((0,));

        let percentile = if total_engines.0 > 0 {
            (engines_below.0 as f32 / total_engines.0 as f32) * 100.0
        } else {
            50.0
        };

        Ok(ComparativeStats {
            engine_accuracy: reputation.accuracy_rate,
            average_accuracy: avg_row.0.unwrap_or(0.0) as f32,
            engine_submissions: reputation.total_submissions as u32,
            average_submissions: avg_row.1.unwrap_or(0.0) as u32,
            engine_rewards: reputation.rewards_earned,
            average_rewards: avg_row.2.unwrap_or(0.0) as i64,
            percentile,
        })
    }

    // ── Private helpers ──────────────────────────────────────

    fn time_period_to_cutoff(&self, period: &TimePeriod) -> DateTime<Utc> {
        match period {
            TimePeriod::AllTime => DateTime::<Utc>::MIN_UTC,
            TimePeriod::ThisMonth => Utc::now() - Duration::days(30),
            TimePeriod::ThisWeek => Utc::now() - Duration::days(7),
            TimePeriod::Today => Utc::now() - Duration::days(1),
        }
    }

    fn reputation_to_ranking(&self, rank: u32, rep: ReputationModel) -> EngineRanking {
        let score = Self::calculate_ranking_score(
            rep.accuracy_rate,
            rep.total_submissions as u32,
            None,
            rep.rewards_earned,
        );
        let tier = Self::calculate_tier(score);

        EngineRanking {
            rank,
            engine_id: rep.engine_id.clone(),
            engine_name: Some(rep.engine_id),
            score,
            total_submissions: rep.total_submissions as u32,
            correct_submissions: rep.correct_submissions as u32,
            accuracy_rate: rep.accuracy_rate,
            total_rewards: rep.rewards_earned,
            avg_response_time_ms: None,
            last_active: rep.updated_at,
            tier,
            badges: Vec::new(), // Badges loaded separately if needed
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
        assert_eq!(
            RankingService::calculate_tier(85.0),
            RankingTier::GrandMaster
        );
        assert_eq!(RankingService::calculate_tier(65.0), RankingTier::Master);
        assert_eq!(RankingService::calculate_tier(45.0), RankingTier::Expert);
        assert_eq!(
            RankingService::calculate_tier(25.0),
            RankingTier::Apprentice
        );
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

        assert!((70.0..=100.0).contains(&score));
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
