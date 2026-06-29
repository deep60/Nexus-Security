// backend/bounty-manager/src/models/validation_result.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ValidationResultModel {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub bounty_id: Uuid,
    pub validator_id: String,
    pub validator_type: String,
    pub validation_status: String,
    pub quality_score: f32,
    pub checks_performed: sqlx::types::JsonValue,
    pub issues_found: sqlx::types::JsonValue,
    pub recommendations: sqlx::types::JsonValue,
    pub validated_at: DateTime<Utc>,
    pub metadata: Option<sqlx::types::JsonValue>,
}

impl ValidationResultModel {
    pub async fn create(
        pool: &PgPool,
        result: &ValidationResultModel,
    ) -> Result<ValidationResultModel, sqlx::Error> {
        let record = sqlx::query_as::<_, ValidationResultModel>(
            r#"
            INSERT INTO validation_results (
                id, submission_id, bounty_id, validator_id, validator_type,
                validation_status, quality_score, checks_performed, issues_found,
                recommendations, validated_at, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(result.id)
        .bind(result.submission_id)
        .bind(result.bounty_id)
        .bind(&result.validator_id)
        .bind(&result.validator_type)
        .bind(&result.validation_status)
        .bind(result.quality_score)
        .bind(&result.checks_performed)
        .bind(&result.issues_found)
        .bind(&result.recommendations)
        .bind(result.validated_at)
        .bind(&result.metadata)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<ValidationResultModel>, sqlx::Error> {
        let record = sqlx::query_as::<_, ValidationResultModel>(
            "SELECT * FROM validation_results WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_submission(
        pool: &PgPool,
        submission_id: Uuid,
    ) -> Result<Vec<ValidationResultModel>, sqlx::Error> {
        let records = sqlx::query_as::<_, ValidationResultModel>(
            "SELECT * FROM validation_results WHERE submission_id = $1 ORDER BY validated_at DESC",
        )
        .bind(submission_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    pub async fn find_latest_by_submission(
        pool: &PgPool,
        submission_id: Uuid,
    ) -> Result<Option<ValidationResultModel>, sqlx::Error> {
        let record = sqlx::query_as::<_, ValidationResultModel>(
            "SELECT * FROM validation_results WHERE submission_id = $1 ORDER BY validated_at DESC LIMIT 1"
        )
        .bind(submission_id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    pub async fn list(
        pool: &PgPool,
        bounty_id: Option<Uuid>,
        submission_id: Option<Uuid>,
        status: Option<&str>,
        min_quality: Option<f32>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ValidationResultModel>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM validation_results WHERE 1=1");

        if let Some(bid) = bounty_id {
            query.push_str(&format!(" AND bounty_id = '{bid}'"));
        }
        if let Some(sid) = submission_id {
            query.push_str(&format!(" AND submission_id = '{sid}'"));
        }
        if let Some(s) = status {
            query.push_str(&format!(" AND validation_status = '{s}'"));
        }
        if let Some(mq) = min_quality {
            query.push_str(&format!(" AND quality_score >= {mq}"));
        }

        query.push_str(&format!(
            " ORDER BY validated_at DESC LIMIT {limit} OFFSET {offset}"
        ));

        let records = sqlx::query_as::<_, ValidationResultModel>(&query)
            .fetch_all(pool)
            .await?;

        Ok(records)
    }

    pub async fn count(
        pool: &PgPool,
        bounty_id: Option<Uuid>,
        status: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let mut query = String::from("SELECT COUNT(*) FROM validation_results WHERE 1=1");

        if let Some(bid) = bounty_id {
            query.push_str(&format!(" AND bounty_id = '{bid}'"));
        }
        if let Some(s) = status {
            query.push_str(&format!(" AND validation_status = '{s}'"));
        }

        let result: (i64,) = sqlx::query_as(&query).fetch_one(pool).await?;

        Ok(result.0)
    }

    pub async fn get_stats(pool: &PgPool) -> Result<ValidationStats, sqlx::Error> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM validation_results")
            .fetch_one(pool)
            .await?;
        let passed: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM validation_results WHERE validation_status IN ('Passed', 'PassedWithWarnings')"
        ).fetch_one(pool).await?;
        let failed: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM validation_results WHERE validation_status = 'Failed'",
        )
        .fetch_one(pool)
        .await?;
        let avg_quality: (Option<f64>,) =
            sqlx::query_as("SELECT AVG(quality_score) FROM validation_results")
                .fetch_one(pool)
                .await?;

        Ok(ValidationStats {
            total_validations: total.0,
            passed_count: passed.0,
            failed_count: failed.0,
            avg_quality_score: avg_quality.0.unwrap_or(0.0) as f32,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    pub total_validations: i64,
    pub passed_count: i64,
    pub failed_count: i64,
    pub avg_quality_score: f32,
}
