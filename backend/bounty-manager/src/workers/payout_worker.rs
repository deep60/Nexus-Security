// backend/bounty-manager/src/workers/payout_worker.rs

use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use crate::services::blockchain::BlockchainService;
use crate::services::notification::NotificationService;
use crate::models::payout::PayoutModel;

pub struct PayoutWorker {
    db: PgPool,
    blockchain_service: Arc<BlockchainService>,
    notification_service: Arc<NotificationService>,
    check_interval_seconds: u64,
}

impl PayoutWorker {
    pub fn new(
        db: PgPool,
        blockchain_service: Arc<BlockchainService>,
        notification_service: Arc<NotificationService>,
    ) -> Self {
        Self {
            db,
            blockchain_service,
            notification_service,
            check_interval_seconds: 30, // Check every 30 seconds
        }
    }

    /// Start the payout worker
    pub async fn run(&self) {
        info!("Starting payout worker...");
        let mut ticker = interval(Duration::from_secs(self.check_interval_seconds));

        loop {
            ticker.tick().await;
            
            if let Err(e) = self.process_pending_payouts().await {
                error!("Error processing payouts: {}", e);
            }
        }
    }

    /// Process all pending payouts
    async fn process_pending_payouts(&self) -> Result<(), WorkerError> {
        let pending_payouts = PayoutModel::get_pending(&self.db)
            .await
            .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

        if !pending_payouts.is_empty() {
            info!("Processing {} pending payouts", pending_payouts.len());
        }

        for payout in pending_payouts {
            if let Err(e) = self.process_payout(&payout).await {
                error!("Error processing payout {}: {}", payout.id, e);
            }
        }

        Ok(())
    }

    /// Process a single payout
    async fn process_payout(&self, payout: &PayoutModel) -> Result<(), WorkerError> {
        info!(
            "Processing payout {} for recipient {} (amount: {})",
            payout.id, payout.recipient, payout.amount
        );

        // Create blockchain transaction
        let transaction = self.blockchain_service
            .create_payout_transaction(
                &payout.recipient,
                payout.bounty_id,
                payout.amount as u64,
            )
            .await
            .map_err(|e| WorkerError::BlockchainError(e.to_string()))?;

        // Update payout status
        PayoutModel::update_status(
            &self.db,
            payout.id,
            "Completed",
            Some(&transaction.transaction_hash),
        )
        .await
        .map_err(|e| WorkerError::DatabaseError(e.to_string()))?;

        // Send notification
        if let Err(e) = self.notification_service
            .notify_payout_processed(
                &payout.recipient,
                payout.amount as u64,
                &transaction.transaction_hash,
            )
            .await
        {
            error!("Failed to send payout notification: {}", e);
        }

        info!(
            "Payout {} completed with tx hash: {}",
            payout.id, transaction.transaction_hash
        );

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Blockchain error: {0}")]
    BlockchainError(String),
}
