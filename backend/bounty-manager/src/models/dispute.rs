// backend/bounty-manager/src/models/dispute.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DisputeModel {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub submission_id: Option<Uuid>,
    pub disputer_id: String,
    pub dispute_type: String,
    pub severity: String,
    pub status: String,
    pub title: String,
    pub description: String,
    pub evidence: Option<sqlx::types::JsonValue>,
    pub stake_amount: i64,
    pub resolution: Option<String>,
    pub resolution_details: Option<String>,
    pub resolver_id: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<sqlx::types::JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DisputeVoteModel {
    pub id: Uuid,
    pub dispute_id: Uuid,
    pub voter_id: String,
    pub vote: String,
    pub voting_power: f32,
    pub rationale: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl DisputeModel {
    pub async fn create(
        pool: &PgPool,
        dispute: &DisputeModel,
    ) -> Result<DisputeModel, sqlx::Error> {
        let record = sqlx::query_as::<_, DisputeModel>(
            r#"
            INSERT INTO disputes (
                id, bounty_id, submission_id, disputer_id, dispute_type, severity,
                status, title, description, evidence, stake_amount, resolution,
                resolution_details, resolver_id, resolved_at, created_at, updated_at, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#,
        )
        .bind(&dispute.id)
        .bind(&dispute.bounty_id)
        .bind(&dispute.submission_id)
        .bind(&dispute.disputer_id)
        .bind(&dispute.dispute_type)
        .bind(&dispute.severity)
        .bind(&dispute.status)
        .bind(&dispute.title)
        .bind(&dispute.description)
        .bind(&dispute.evidence)
        .bind(dispute.stake_amount)
        .bind(&dispute.resolution)
        .bind(&dispute.resolution_details)
        .bind(&dispute.resolver_id)
        .bind(&dispute.resolved_at)
        .bind(&dispute.created_at)
        .bind(&dispute.updated_at)
        .bind(&dispute.metadata)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<DisputeModel>, sqlx::Error> {
        let record = sqlx::query_as::<_, DisputeModel>("SELECT * FROM disputes WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(record)
    }

    pub async fn find_by_bounty(
        pool: &PgPool,
        bounty_id: Uuid,
    ) -> Result<Vec<DisputeModel>, sqlx::Error> {
        let records = sqlx::query_as::<_, DisputeModel>(
            "SELECT * FROM disputes WHERE bounty_id = $1 ORDER BY created_at DESC",
        )
        .bind(bounty_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    pub async fn find_by_disputer(
        pool: &PgPool,
        disputer_id: &str,
    ) -> Result<Vec<DisputeModel>, sqlx::Error> {
        let records = sqlx::query_as::<_, DisputeModel>(
            "SELECT * FROM disputes WHERE disputer_id = $1 ORDER BY created_at DESC",
        )
        .bind(disputer_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    pub async fn list(
        pool: &PgPool,
        status: Option<&str>,
        bounty_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DisputeModel>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM disputes WHERE 1=1");

        if let Some(s) = status {
            query.push_str(&format!(" AND status = '{}'", s));
        }
        if let Some(bid) = bounty_id {
            query.push_str(&format!(" AND bounty_id = '{}'", bid));
        }

        query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT {} OFFSET {}",
            limit, offset
        ));

        let records = sqlx::query_as::<_, DisputeModel>(&query)
            .fetch_all(pool)
            .await?;

        Ok(records)
    }

    pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE disputes SET status = $1, updated_at = $2 WHERE id = $3")
            .bind(status)
            .bind(Utc::now())
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn resolve(
        pool: &PgPool,
        id: Uuid,
        resolution: &str,
        resolution_details: &str,
        resolver_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE disputes SET
                status = 'Resolved', resolution = $1, resolution_details = $2,
                resolver_id = $3, resolved_at = $4, updated_at = $4
            WHERE id = $5
            "#,
        )
        .bind(resolution)
        .bind(resolution_details)
        .bind(resolver_id)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn count_by_status(pool: &PgPool, status: Option<&str>) -> Result<i64, sqlx::Error> {
        let mut query = String::from("SELECT COUNT(*) FROM disputes WHERE 1=1");

        if let Some(s) = status {
            query.push_str(&format!(" AND status = '{}'", s));
        }

        let result: (i64,) = sqlx::query_as(&query).fetch_one(pool).await?;

        Ok(result.0)
    }

    /// Check if a disputer already has an active dispute for a bounty
    pub async fn has_active_dispute(
        pool: &PgPool,
        bounty_id: Uuid,
        disputer_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM disputes WHERE bounty_id = $1 AND disputer_id = $2 AND status NOT IN ('Resolved', 'Withdrawn')"
        )
        .bind(bounty_id)
        .bind(disputer_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }

    /// Get dispute statistics
    pub async fn get_stats(pool: &PgPool) -> Result<DisputeStats, sqlx::Error> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM disputes")
            .fetch_one(pool)
            .await?;
        let open: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM disputes WHERE status IN ('Open', 'UnderReview', 'Voting')",
        )
        .fetch_one(pool)
        .await?;
        let resolved: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM disputes WHERE status = 'Resolved'")
                .fetch_one(pool)
                .await?;
        let upheld: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM disputes WHERE resolution = 'DisputeUpheld'")
                .fetch_one(pool)
                .await?;
        let rejected: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM disputes WHERE resolution = 'DisputeRejected'")
                .fetch_one(pool)
                .await?;
        let avg_resolution: (Option<f64>,) = sqlx::query_as(
            "SELECT AVG(EXTRACT(EPOCH FROM (resolved_at - created_at)) / 3600.0) FROM disputes WHERE resolved_at IS NOT NULL"
        ).fetch_one(pool).await?;

        Ok(DisputeStats {
            total_disputes: total.0,
            open_disputes: open.0,
            resolved_disputes: resolved.0,
            upheld_disputes: upheld.0,
            rejected_disputes: rejected.0,
            avg_resolution_time_hours: avg_resolution.0.unwrap_or(0.0) as f32,
        })
    }
}

impl DisputeVoteModel {
    pub async fn create(
        pool: &PgPool,
        vote: &DisputeVoteModel,
    ) -> Result<DisputeVoteModel, sqlx::Error> {
        let record = sqlx::query_as::<_, DisputeVoteModel>(
            r#"
            INSERT INTO dispute_votes (id, dispute_id, voter_id, vote, voting_power, rationale, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(&vote.id)
        .bind(&vote.dispute_id)
        .bind(&vote.voter_id)
        .bind(&vote.vote)
        .bind(vote.voting_power)
        .bind(&vote.rationale)
        .bind(&vote.created_at)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn has_voted(
        pool: &PgPool,
        dispute_id: Uuid,
        voter_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM dispute_votes WHERE dispute_id = $1 AND voter_id = $2",
        )
        .bind(dispute_id)
        .bind(voter_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }

    pub async fn get_vote_tallies(
        pool: &PgPool,
        dispute_id: Uuid,
    ) -> Result<(f32, f32), sqlx::Error> {
        let uphold: (Option<f64>,) = sqlx::query_as(
            "SELECT COALESCE(SUM(voting_power), 0) FROM dispute_votes WHERE dispute_id = $1 AND vote = 'Uphold'"
        )
        .bind(dispute_id)
        .fetch_one(pool)
        .await?;

        let reject: (Option<f64>,) = sqlx::query_as(
            "SELECT COALESCE(SUM(voting_power), 0) FROM dispute_votes WHERE dispute_id = $1 AND vote = 'Reject'"
        )
        .bind(dispute_id)
        .fetch_one(pool)
        .await?;

        Ok((
            uphold.0.unwrap_or(0.0) as f32,
            reject.0.unwrap_or(0.0) as f32,
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeStats {
    pub total_disputes: i64,
    pub open_disputes: i64,
    pub resolved_disputes: i64,
    pub upheld_disputes: i64,
    pub rejected_disputes: i64,
    pub avg_resolution_time_hours: f32,
}
