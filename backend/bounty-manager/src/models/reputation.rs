// backend/bounty-manager/src/models/reputation.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReputationModel {
    pub engine_id: String,
    pub reputation_score: f32,
    pub total_submissions: i32,
    pub correct_submissions: i32,
    pub accuracy_rate: f32,
    pub average_confidence: f32,
    pub total_stake: i64,
    pub rewards_earned: i64,
    pub penalties_incurred: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ReputationModel {
    pub async fn create(pool: &PgPool, reputation: &ReputationModel) -> Result<ReputationModel, sqlx::Error> {
        let record = sqlx::query_as::<_, ReputationModel>(
            r#"
            INSERT INTO reputations (
                engine_id, reputation_score, total_submissions, correct_submissions,
                accuracy_rate, average_confidence, total_stake, rewards_earned,
                penalties_incurred, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#
        )
        .bind(&reputation.engine_id)
        .bind(reputation.reputation_score)
        .bind(reputation.total_submissions)
        .bind(reputation.correct_submissions)
        .bind(reputation.accuracy_rate)
        .bind(reputation.average_confidence)
        .bind(reputation.total_stake)
        .bind(reputation.rewards_earned)
        .bind(reputation.penalties_incurred)
        .bind(&reputation.created_at)
        .bind(&reputation.updated_at)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_id(pool: &PgPool, engine_id: &str) -> Result<Option<ReputationModel>, sqlx::Error> {
        let record = sqlx::query_as::<_, ReputationModel>(
            "SELECT * FROM reputations WHERE engine_id = $1"
        )
        .bind(engine_id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    pub async fn update(pool: &PgPool, reputation: &ReputationModel) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE reputations SET
                reputation_score = $1, total_submissions = $2, correct_submissions = $3,
                accuracy_rate = $4, average_confidence = $5, total_stake = $6,
                rewards_earned = $7, penalties_incurred = $8, updated_at = $9
            WHERE engine_id = $10
            "#
        )
        .bind(reputation.reputation_score)
        .bind(reputation.total_submissions)
        .bind(reputation.correct_submissions)
        .bind(reputation.accuracy_rate)
        .bind(reputation.average_confidence)
        .bind(reputation.total_stake)
        .bind(reputation.rewards_earned)
        .bind(reputation.penalties_incurred)
        .bind(Utc::now())
        .bind(&reputation.engine_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_leaderboard(pool: &PgPool, limit: i64) -> Result<Vec<ReputationModel>, sqlx::Error> {
        let records = sqlx::query_as::<_, ReputationModel>(
            "SELECT * FROM reputations ORDER BY reputation_score DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    pub async fn increment_submission(pool: &PgPool, engine_id: &str, is_correct: bool) -> Result<(), sqlx::Error> {
        if is_correct {
            sqlx::query(
                r#"
                UPDATE reputations SET
                    total_submissions = total_submissions + 1,
                    correct_submissions = correct_submissions + 1,
                    accuracy_rate = CAST(correct_submissions + 1 AS FLOAT) / CAST(total_submissions + 1 AS FLOAT),
                    updated_at = $1
                WHERE engine_id = $2
                "#
            )
            .bind(Utc::now())
            .bind(engine_id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                r#"
                UPDATE reputations SET
                    total_submissions = total_submissions + 1,
                    accuracy_rate = CAST(correct_submissions AS FLOAT) / CAST(total_submissions + 1 AS FLOAT),
                    updated_at = $1
                WHERE engine_id = $2
                "#
            )
            .bind(Utc::now())
            .bind(engine_id)
            .execute(pool)
            .await?;
        }

        Ok(())
    }
}
