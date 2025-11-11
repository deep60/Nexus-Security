// backend/bounty-manager/src/services/blockchain_sync.rs

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;
use chrono::Utc;

use crate::services::blockchain::BlockchainService;
use crate::models::{bounty::BountyModel, submission::SubmissionModel, payout::PayoutModel};

/// Service for synchronizing blockchain state with database
#[derive(Clone)]
pub struct BlockchainSyncService {
    db: PgPool,
    blockchain: Arc<BlockchainService>,
    last_synced_block: Arc<RwLock<u64>>,
    sync_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub last_synced_block: u64,
    pub current_block: u64,
    pub blocks_behind: u64,
    pub is_syncing: bool,
    pub last_sync_time: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainEvent {
    pub event_type: BlockchainEventType,
    pub block_number: u64,
    pub transaction_hash: String,
    pub timestamp: i64,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockchainEventType {
    BountyCreated,
    BountyFunded,
    SubmissionStaked,
    ConsensusReached,
    PayoutDistributed,
    StakeSlashed,
    DisputeRaised,
    DisputeResolved,
}

impl BlockchainSyncService {
    pub fn new(db: PgPool, blockchain: Arc<BlockchainService>) -> Self {
        Self {
            db,
            blockchain,
            last_synced_block: Arc::new(RwLock::new(0)),
            sync_interval_seconds: 15, // Sync every 15 seconds
        }
    }

    /// Start the blockchain sync service
    pub async fn start(&self) -> Result<(), SyncError> {
        info!("Starting blockchain sync service...");
        
        // Initialize last synced block from database
        if let Ok(block) = self.get_last_synced_block_from_db().await {
            *self.last_synced_block.write().await = block;
            info!("Resuming sync from block {}", block);
        }

        let service = self.clone();
        tokio::spawn(async move {
            service.sync_loop().await;
        });

        Ok(())
    }

    /// Main sync loop
    async fn sync_loop(&self) {
        let mut ticker = interval(Duration::from_secs(self.sync_interval_seconds));

        loop {
            ticker.tick().await;

            if let Err(e) = self.sync_blocks().await {
                error!("Error syncing blocks: {}", e);
            }
        }
    }

    /// Sync new blocks from blockchain
    async fn sync_blocks(&self) -> Result<(), SyncError> {
        let last_synced = *self.last_synced_block.read().await;
        
        // Get current block number from blockchain
        let current_block = self.get_current_block_number().await?;

        if current_block <= last_synced {
            return Ok(());
        }

        info!(
            "Syncing blocks {} to {} ({} blocks)",
            last_synced + 1,
            current_block,
            current_block - last_synced
        );

        // Sync blocks in batches to avoid overwhelming the system
        let batch_size = 100;
        for start_block in (last_synced + 1..=current_block).step_by(batch_size) {
            let end_block = (start_block + batch_size as u64 - 1).min(current_block);
            
            if let Err(e) = self.sync_block_range(start_block, end_block).await {
                error!("Error syncing blocks {}-{}: {}", start_block, end_block, e);
                return Err(e);
            }

            // Update last synced block
            *self.last_synced_block.write().await = end_block;
            self.save_last_synced_block(end_block).await?;
        }

        Ok(())
    }

    /// Sync a range of blocks
    async fn sync_block_range(&self, start: u64, end: u64) -> Result<(), SyncError> {
        for block_num in start..=end {
            // Get events from this block
            let events = self.get_block_events(block_num).await?;

            for event in events {
                if let Err(e) = self.process_event(&event).await {
                    error!("Error processing event: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Get events from a specific block
    async fn get_block_events(&self, block_number: u64) -> Result<Vec<BlockchainEvent>, SyncError> {
        // TODO: Implement actual blockchain event fetching
        // This is a placeholder that returns empty events
        Ok(Vec::new())
    }

    /// Process a blockchain event
    async fn process_event(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        match &event.event_type {
            BlockchainEventType::BountyCreated => {
                self.handle_bounty_created(event).await?;
            }
            BlockchainEventType::BountyFunded => {
                self.handle_bounty_funded(event).await?;
            }
            BlockchainEventType::SubmissionStaked => {
                self.handle_submission_staked(event).await?;
            }
            BlockchainEventType::ConsensusReached => {
                self.handle_consensus_reached(event).await?;
            }
            BlockchainEventType::PayoutDistributed => {
                self.handle_payout_distributed(event).await?;
            }
            BlockchainEventType::StakeSlashed => {
                self.handle_stake_slashed(event).await?;
            }
            BlockchainEventType::DisputeRaised => {
                self.handle_dispute_raised(event).await?;
            }
            BlockchainEventType::DisputeResolved => {
                self.handle_dispute_resolved(event).await?;
            }
        }

        Ok(())
    }

    /// Handle BountyCreated event
    async fn handle_bounty_created(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing BountyCreated event in block {}", event.block_number);
        
        // TODO: Extract bounty data from event and create/update in database
        // Example:
        // let bounty_id = event.data["bounty_id"].as_str().unwrap();
        // BountyModel::update_status(&self.db, bounty_id, "Active").await?;

        Ok(())
    }

    /// Handle BountyFunded event
    async fn handle_bounty_funded(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing BountyFunded event in block {}", event.block_number);
        // TODO: Update bounty funding status
        Ok(())
    }

    /// Handle SubmissionStaked event
    async fn handle_submission_staked(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing SubmissionStaked event in block {}", event.block_number);
        
        // TODO: Update submission with transaction hash
        // if let Some(submission_id) = event.data["submission_id"].as_str() {
        //     let uuid = Uuid::parse_str(submission_id)?;
        //     SubmissionModel::update_status(&self.db, uuid, "Active").await?;
        // }

        Ok(())
    }

    /// Handle ConsensusReached event
    async fn handle_consensus_reached(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing ConsensusReached event in block {}", event.block_number);
        // TODO: Update bounty status to completed
        Ok(())
    }

    /// Handle PayoutDistributed event
    async fn handle_payout_distributed(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing PayoutDistributed event in block {}", event.block_number);
        
        // TODO: Update payout status
        // if let Some(payout_id) = event.data["payout_id"].as_str() {
        //     let uuid = Uuid::parse_str(payout_id)?;
        //     PayoutModel::update_status(
        //         &self.db,
        //         uuid,
        //         "Completed",
        //         Some(&event.transaction_hash),
        //     ).await?;
        // }

        Ok(())
    }

    /// Handle StakeSlashed event
    async fn handle_stake_slashed(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing StakeSlashed event in block {}", event.block_number);
        // TODO: Update submission status to Slashed
        Ok(())
    }

    /// Handle DisputeRaised event
    async fn handle_dispute_raised(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing DisputeRaised event in block {}", event.block_number);
        // TODO: Create dispute record
        Ok(())
    }

    /// Handle DisputeResolved event
    async fn handle_dispute_resolved(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing DisputeResolved event in block {}", event.block_number);
        // TODO: Update dispute status
        Ok(())
    }

    /// Get current block number from blockchain
    async fn get_current_block_number(&self) -> Result<u64, SyncError> {
        // TODO: Implement actual blockchain query
        // For now, return a mock value
        Ok(1000)
    }

    /// Get last synced block from database
    async fn get_last_synced_block_from_db(&self) -> Result<u64, SyncError> {
        // TODO: Implement database query to get last synced block
        // Could store in a sync_status table
        Ok(0)
    }

    /// Save last synced block to database
    async fn save_last_synced_block(&self, block_number: u64) -> Result<(), SyncError> {
        // TODO: Implement database update
        info!("Saved sync progress: block {}", block_number);
        Ok(())
    }

    /// Get current sync status
    pub async fn get_sync_status(&self) -> Result<SyncStatus, SyncError> {
        let last_synced = *self.last_synced_block.read().await;
        let current_block = self.get_current_block_number().await?;

        Ok(SyncStatus {
            last_synced_block: last_synced,
            current_block,
            blocks_behind: current_block.saturating_sub(last_synced),
            is_syncing: current_block > last_synced,
            last_sync_time: Utc::now(),
        })
    }

    /// Force sync from a specific block
    pub async fn force_sync_from_block(&self, block_number: u64) -> Result<(), SyncError> {
        info!("Force syncing from block {}", block_number);
        *self.last_synced_block.write().await = block_number;
        self.save_last_synced_block(block_number).await?;
        Ok(())
    }

    /// Resync recent blocks (useful for reorg handling)
    pub async fn resync_recent_blocks(&self, num_blocks: u64) -> Result<(), SyncError> {
        let last_synced = *self.last_synced_block.read().await;
        let resync_from = last_synced.saturating_sub(num_blocks);
        
        warn!("Resyncing last {} blocks from block {}", num_blocks, resync_from);
        self.force_sync_from_block(resync_from).await?;
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Event processing error: {0}")]
    EventProcessingError(String),
}

impl From<sqlx::Error> for SyncError {
    fn from(err: sqlx::Error) -> Self {
        SyncError::DatabaseError(err.to_string())
    }
}

impl From<uuid::Error> for SyncError {
    fn from(err: uuid::Error) -> Self {
        SyncError::ParseError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_creation() {
        let status = SyncStatus {
            last_synced_block: 100,
            current_block: 150,
            blocks_behind: 50,
            is_syncing: true,
            last_sync_time: Utc::now(),
        };

        assert_eq!(status.blocks_behind, 50);
        assert!(status.is_syncing);
    }
}
