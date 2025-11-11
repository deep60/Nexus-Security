// backend/bounty-manager/src/models/bounty.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BountyModel {
    pub id: Uuid,
    pub creator: String,
    pub title: String,
    pub description: String,
    pub artifact_type: String,
    pub artifact_hash: Option<String>,
    pub artifact_url: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub upload_path: Option<String>,
    pub reward_amount: i64,
    pub currency: String,
    pub min_stake: i64,
    pub max_participants: Option<i32>,
    pub deadline: DateTime<Utc>,
    pub status: String,
    pub consensus_threshold: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<sqlx::types::JsonValue>,
}

impl BountyModel {
    /// Create a new bounty in the database
    pub async fn create(pool: &PgPool, bounty: &BountyModel) -> Result<BountyModel, sqlx::Error> {
        let record = sqlx::query_as::<_, BountyModel>(
            r#"
            INSERT INTO bounties (
                id, creator, title, description, artifact_type, artifact_hash,
                artifact_url, file_name, file_size, mime_type, upload_path,
                reward_amount, currency, min_stake, max_participants, deadline,
                status, consensus_threshold, created_at, updated_at, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING *
            "#
        )
        .bind(&bounty.id)
        .bind(&bounty.creator)
        .bind(&bounty.title)
        .bind(&bounty.description)
        .bind(&bounty.artifact_type)
        .bind(&bounty.artifact_hash)
        .bind(&bounty.artifact_url)
        .bind(&bounty.file_name)
        .bind(&bounty.file_size)
        .bind(&bounty.mime_type)
        .bind(&bounty.upload_path)
        .bind(bounty.reward_amount)
        .bind(&bounty.currency)
        .bind(bounty.min_stake)
        .bind(bounty.max_participants)
        .bind(&bounty.deadline)
        .bind(&bounty.status)
        .bind(bounty.consensus_threshold)
        .bind(&bounty.created_at)
        .bind(&bounty.updated_at)
        .bind(&bounty.metadata)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Find a bounty by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<BountyModel>, sqlx::Error> {
        let record = sqlx::query_as::<_, BountyModel>(
            "SELECT * FROM bounties WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// Update bounty status
    pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE bounties SET status = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(status)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// List bounties with filters
    pub async fn list(
        pool: &PgPool,
        status: Option<&str>,
        creator: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BountyModel>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM bounties WHERE 1=1");

        if let Some(s) = status {
            query.push_str(&format!(" AND status = '{}'", s));
        }

        if let Some(c) = creator {
            query.push_str(&format!(" AND creator = '{}'", c));
        }

        query.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset));

        let records = sqlx::query_as::<_, BountyModel>(&query)
            .fetch_all(pool)
            .await?;

        Ok(records)
    }

    /// Count total bounties
    pub async fn count(pool: &PgPool, status: Option<&str>) -> Result<i64, sqlx::Error> {
        let mut query = String::from("SELECT COUNT(*) as count FROM bounties WHERE 1=1");

        if let Some(s) = status {
            query.push_str(&format!(" AND status = '{}'", s));
        }

        let result: (i64,) = sqlx::query_as(&query)
            .fetch_one(pool)
            .await?;

        Ok(result.0)
    }

    /// Get bounties by creator
    pub async fn find_by_creator(pool: &PgPool, creator: &str) -> Result<Vec<BountyModel>, sqlx::Error> {
        let records = sqlx::query_as::<_, BountyModel>(
            "SELECT * FROM bounties WHERE creator = $1 ORDER BY created_at DESC"
        )
        .bind(creator)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Get active bounties (not expired)
    pub async fn find_active(pool: &PgPool) -> Result<Vec<BountyModel>, sqlx::Error> {
        let records = sqlx::query_as::<_, BountyModel>(
            "SELECT * FROM bounties WHERE status = 'Active' AND deadline > $1 ORDER BY created_at DESC"
        )
        .bind(Utc::now())
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Delete a bounty
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM bounties WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
