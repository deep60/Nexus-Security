// backend/bounty-manager/src/models/submission.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubmissionModel {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub engine_id: String,
    pub engine_type: String,
    pub verdict: String,
    pub confidence: f32,
    pub stake_amount: i64,
    pub analysis_details: sqlx::types::JsonValue,
    pub status: String,
    pub transaction_hash: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub accuracy_score: Option<f32>,
}

impl SubmissionModel {
    pub async fn create(pool: &PgPool, submission: &SubmissionModel) -> Result<SubmissionModel, sqlx::Error> {
        let record = sqlx::query_as::<_, SubmissionModel>(
            r#"
            INSERT INTO submissions (
                id, bounty_id, engine_id, engine_type, verdict, confidence,
                stake_amount, analysis_details, status, transaction_hash,
                submitted_at, processed_at, accuracy_score
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#
        )
        .bind(&submission.id)
        .bind(&submission.bounty_id)
        .bind(&submission.engine_id)
        .bind(&submission.engine_type)
        .bind(&submission.verdict)
        .bind(submission.confidence)
        .bind(submission.stake_amount)
        .bind(&submission.analysis_details)
        .bind(&submission.status)
        .bind(&submission.transaction_hash)
        .bind(&submission.submitted_at)
        .bind(&submission.processed_at)
        .bind(submission.accuracy_score)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<SubmissionModel>, sqlx::Error> {
        let record = sqlx::query_as::<_, SubmissionModel>(
            "SELECT * FROM submissions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_bounty(pool: &PgPool, bounty_id: Uuid) -> Result<Vec<SubmissionModel>, sqlx::Error> {
        let records = sqlx::query_as::<_, SubmissionModel>(
            "SELECT * FROM submissions WHERE bounty_id = $1 ORDER BY submitted_at DESC"
        )
        .bind(bounty_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    pub async fn find_by_engine(pool: &PgPool, engine_id: &str) -> Result<Vec<SubmissionModel>, sqlx::Error> {
        let records = sqlx::query_as::<_, SubmissionModel>(
            "SELECT * FROM submissions WHERE engine_id = $1 ORDER BY submitted_at DESC"
        )
        .bind(engine_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE submissions SET status = $1, processed_at = $2 WHERE id = $3"
        )
        .bind(status)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_accuracy_score(pool: &PgPool, id: Uuid, score: f32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE submissions SET accuracy_score = $1 WHERE id = $2"
        )
        .bind(score)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn count_by_bounty(pool: &PgPool, bounty_id: Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM submissions WHERE bounty_id = $1"
        )
        .bind(bounty_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }
}
