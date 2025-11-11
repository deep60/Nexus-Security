// backend/bounty-manager/src/models/payout.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PayoutModel {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub submission_id: Option<Uuid>,
    pub recipient: String,
    pub amount: i64,
    pub currency: String,
    pub payout_type: String,
    pub status: String,
    pub transaction_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub metadata: Option<sqlx::types::JsonValue>,
}

impl PayoutModel {
    pub async fn create(pool: &PgPool, payout: &PayoutModel) -> Result<PayoutModel, sqlx::Error> {
        let record = sqlx::query_as::<_, PayoutModel>(
            r#"
            INSERT INTO payouts (
                id, bounty_id, submission_id, recipient, amount, currency,
                payout_type, status, transaction_hash, created_at, processed_at, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#
        )
        .bind(&payout.id)
        .bind(&payout.bounty_id)
        .bind(&payout.submission_id)
        .bind(&payout.recipient)
        .bind(payout.amount)
        .bind(&payout.currency)
        .bind(&payout.payout_type)
        .bind(&payout.status)
        .bind(&payout.transaction_hash)
        .bind(&payout.created_at)
        .bind(&payout.processed_at)
        .bind(&payout.metadata)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<PayoutModel>, sqlx::Error> {
        let record = sqlx::query_as::<_, PayoutModel>(
            "SELECT * FROM payouts WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_bounty(pool: &PgPool, bounty_id: Uuid) -> Result<Vec<PayoutModel>, sqlx::Error> {
        let records = sqlx::query_as::<_, PayoutModel>(
            "SELECT * FROM payouts WHERE bounty_id = $1 ORDER BY created_at DESC"
        )
        .bind(bounty_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    pub async fn find_by_recipient(pool: &PgPool, recipient: &str) -> Result<Vec<PayoutModel>, sqlx::Error> {
        let records = sqlx::query_as::<_, PayoutModel>(
            "SELECT * FROM payouts WHERE recipient = $1 ORDER BY created_at DESC"
        )
        .bind(recipient)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: &str,
        transaction_hash: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE payouts SET status = $1, transaction_hash = $2, processed_at = $3 WHERE id = $4"
        )
        .bind(status)
        .bind(transaction_hash)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_pending(pool: &PgPool) -> Result<Vec<PayoutModel>, sqlx::Error> {
        let records = sqlx::query_as::<_, PayoutModel>(
            "SELECT * FROM payouts WHERE status = 'Pending' ORDER BY created_at ASC"
        )
        .fetch_all(pool)
        .await?;

        Ok(records)
    }
}
