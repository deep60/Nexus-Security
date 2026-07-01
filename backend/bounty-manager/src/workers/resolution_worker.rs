// backend/bounty-manager/src/workers/resolution_worker.rs

use crate::services::blockchain::BlockchainService;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use uuid::Uuid;

/// Periodically triggers on-chain resolution for eligible bounties.
///
/// The `BountyManager` contract decouples resolution from submission (to avoid
/// making the last submitter pay to resolve for everyone and to keep submission
/// gas bounded). Something off-chain must therefore call `resolveBounty(id)` once
/// a bounty becomes eligible. This keeper fills that role: it scans for Active
/// bounties whose deadline has passed or that have collected enough analyses, and
/// submits the resolution transaction. The `blockchain_sync` service then observes
/// the resulting `ConsensusReached` / `RewardsDistributed` events and updates the DB.
pub struct ResolutionWorker {
    db: PgPool,
    blockchain: Arc<BlockchainService>,
    check_interval_seconds: u64,
    /// Number of analyses that makes a bounty eligible for early resolution,
    /// mirroring the contract's `MIN_ANALYSES_TO_RESOLVE`.
    min_analyses_to_resolve: i64,
}

impl ResolutionWorker {
    pub fn new(db: PgPool, blockchain: Arc<BlockchainService>) -> Self {
        Self {
            db,
            blockchain,
            check_interval_seconds: 60, // Check every minute
            min_analyses_to_resolve: 5,
        }
    }

    /// Start the resolution keeper loop.
    pub async fn run(&self) {
        info!("Starting bounty resolution worker...");
        let mut ticker = interval(Duration::from_secs(self.check_interval_seconds));

        loop {
            ticker.tick().await;

            if let Err(e) = self.process_resolvable_bounties().await {
                error!("Error processing resolvable bounties: {}", e);
            }
        }
    }

    /// Find and resolve all bounties that are eligible for on-chain resolution.
    async fn process_resolvable_bounties(&self) -> Result<(), WorkerError> {
        let candidates = self.find_resolvable().await?;

        if candidates.is_empty() {
            return Ok(());
        }

        info!("Found {} bounties eligible for resolution", candidates.len());

        for (bounty_id, on_chain_id) in candidates {
            if let Err(e) = self.resolve_one(bounty_id, &on_chain_id).await {
                error!("Failed to resolve bounty {} (chain id {}): {}", bounty_id, on_chain_id, e);
            }
        }

        Ok(())
    }

    /// Select Active bounties that carry an on-chain id, have not already had a
    /// resolution submitted, and are eligible (deadline passed OR enough analyses).
    async fn find_resolvable(&self) -> Result<Vec<(Uuid, String)>, WorkerError> {
        let rows: Vec<(Uuid, Option<String>)> = sqlx::query_as(
            r#"
            SELECT b.id, b.metadata->>'on_chain_id'
            FROM bounties b
            WHERE b.status = 'Active'
              AND b.metadata->>'on_chain_id' IS NOT NULL
              AND (b.metadata->>'resolution_submitted') IS NULL
              AND (
                    b.deadline < NOW()
                    OR (
                        SELECT COUNT(*) FROM submissions s WHERE s.bounty_id = b.id
                    ) >= $1
              )
            "#,
        )
        .bind(self.min_analyses_to_resolve)
        .fetch_all(&self.db)
        .await
        .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|(id, chain_id)| chain_id.map(|c| (id, c)))
            .collect())
    }

    /// Submit the resolution transaction for a single bounty and mark it so we
    /// don't resubmit on the next tick.
    async fn resolve_one(&self, bounty_id: Uuid, on_chain_id: &str) -> Result<(), WorkerError> {
        let chain_id: u64 = on_chain_id
            .parse()
            .map_err(|_| WorkerError::InvalidOnChainId(on_chain_id.to_string()))?;

        info!("Resolving bounty {} (chain id {})", bounty_id, chain_id);

        let tx_hash = self
            .blockchain
            .resolve_bounty(chain_id)
            .await
            .map_err(|e| WorkerError::BlockchainError(e.to_string()))?;

        info!("Resolution tx submitted for bounty {}: {}", bounty_id, tx_hash);

        // Mark the bounty so the keeper does not resubmit while the sync service
        // catches up and flips the status to Completed/Paid from chain events.
        self.mark_resolution_submitted(bounty_id, &tx_hash).await?;

        Ok(())
    }

    async fn mark_resolution_submitted(
        &self,
        bounty_id: Uuid,
        tx_hash: &str,
    ) -> Result<(), WorkerError> {
        sqlx::query(
            r#"
            UPDATE bounties
            SET metadata = COALESCE(metadata, '{}'::jsonb) || jsonb_build_object(
                    'resolution_submitted', true,
                    'resolution_tx', $1::text
                ),
                updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(tx_hash)
        .bind(bounty_id)
        .execute(&self.db)
        .await
        .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    #[error("Invalid on-chain id: {0}")]
    InvalidOnChainId(String),
}
