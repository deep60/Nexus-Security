// backend/bounty-manager/src/services/blockchain_sync.rs

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;
use chrono::Utc;
use ethers::providers::Middleware;

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
        // Use the blockchain service client to fetch event logs
        // This implementation fetches BountyManager contract logs for the block
        use ethers::types::{Filter, BlockNumber, H256};

        let client = self.blockchain.get_client();
        let filter = Filter::new()
            .from_block(BlockNumber::Number(block_number.into()))
            .to_block(BlockNumber::Number(block_number.into()));

        let logs = client
            .get_logs(&filter)
            .await
            .map_err(|e| SyncError::BlockchainError(format!("Failed to fetch logs: {}", e)))?;

        let events: Vec<BlockchainEvent> = logs
            .iter()
            .filter_map(|log| {
                let event_type = classify_event_topic(log.topics.first()?);
                let tx_hash = log.transaction_hash.map(|h| format!("{:?}", h)).unwrap_or_default();

                Some(BlockchainEvent {
                    event_type: event_type?,
                    block_number,
                    transaction_hash: tx_hash,
                    timestamp: chrono::Utc::now().timestamp(),
                    data: serde_json::json!({
                        "log_index": log.log_index,
                        "address": format!("{:?}", log.address),
                    }),
                })
            })
            .collect();

        Ok(events)
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
    /// Event: BountyCreated(uint256 indexed bountyId, address indexed creator, string artifactHash, uint256 rewardAmount, uint256 deadline)
    async fn handle_bounty_created(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing BountyCreated event in block {}", event.block_number);
        
        // Topic[1] = bountyId (indexed), Topic[2] = creator (indexed)
        // Non-indexed data: artifactHash, rewardAmount, deadline
        if let Some(on_chain_id) = event.data.get("bounty_id").and_then(|v| v.as_str()) {
            info!("On-chain bounty {} created, tx: {}", on_chain_id, event.transaction_hash);
            // In production: look up DB bounty by blockchain_tx_hash and update status to Active
        }

        Ok(())
    }

    /// Handle BountyFunded event
    async fn handle_bounty_funded(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing BountyFunded event in block {}, tx: {}", event.block_number, event.transaction_hash);
        // BountyFunded is implicit in createBounty (transferFrom happens in createBounty)
        // No additional DB update needed beyond what handle_bounty_created does
        Ok(())
    }

    /// Handle SubmissionStaked event
    /// Event: AnalysisSubmitted(uint256 indexed bountyId, address indexed analyst, uint8 verdict, uint256 stakeAmount, uint256 confidence)
    async fn handle_submission_staked(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing AnalysisSubmitted event in block {}", event.block_number);
        
        let bounty_id = event.data.get("bounty_id").and_then(|v| v.as_str()).unwrap_or("unknown");
        let analyst = event.data.get("analyst").and_then(|v| v.as_str()).unwrap_or("unknown");
        info!("Analysis submitted for bounty {} by {}, tx: {}", bounty_id, analyst, event.transaction_hash);
        // In production: create or update submission record in DB with on-chain confirmation

        Ok(())
    }

    /// Handle ConsensusReached event
    /// Event: ConsensusReached(uint256 indexed bountyId, uint8 verdict, uint256 confidenceScore, uint256 totalAnalyses)
    async fn handle_consensus_reached(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing ConsensusReached event in block {}", event.block_number);
        
        let bounty_id = event.data.get("bounty_id").and_then(|v| v.as_str()).unwrap_or("unknown");
        info!("Consensus reached for bounty {}, tx: {}", bounty_id, event.transaction_hash);
        // In production: update bounty status to Completed, store consensus verdict

        Ok(())
    }

    /// Handle PayoutDistributed event
    /// Event: RewardsDistributed(uint256 indexed bountyId, address[] winners, uint256[] rewards, uint256[] stakes)
    async fn handle_payout_distributed(&self, event: &BlockchainEvent) -> Result<(), SyncError> {
        info!("Processing RewardsDistributed event in block {}", event.block_number);
        
        let bounty_id = event.data.get("bounty_id").and_then(|v| v.as_str()).unwrap_or("unknown");
        info!("Rewards distributed for bounty {}, tx: {}", bounty_id, event.transaction_hash);
        // In production: create payout records for each winner

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
        let client = self.blockchain.get_client();
        let block_number = client
            .get_block_number()
            .await
            .map_err(|e| SyncError::BlockchainError(format!("Failed to get block number: {}", e)))?;
        Ok(block_number.as_u64())
    }

    /// Get last synced block from database
    async fn get_last_synced_block_from_db(&self) -> Result<u64, SyncError> {
        // Query sync_state table for the last synced block number
        // If table doesn't exist or no row, return 0 (start from genesis)
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(MAX(block_number), 0) FROM sync_state WHERE service = 'bounty_sync'"
        )
        .fetch_one(&self.db)
        .await;

        match result {
            Ok(block) => Ok(block as u64),
            Err(e) => {
                warn!("Could not read sync state (table may not exist yet): {}", e);
                Ok(0)
            }
        }
    }

    /// Save last synced block to database
    async fn save_last_synced_block(&self, block_number: u64) -> Result<(), SyncError> {
        // Upsert sync state
        let result = sqlx::query(
            r#"
            INSERT INTO sync_state (service, block_number, updated_at)
            VALUES ('bounty_sync', $1, NOW())
            ON CONFLICT (service)
            DO UPDATE SET block_number = $1, updated_at = NOW()
            "#
        )
        .bind(block_number as i64)
        .execute(&self.db)
        .await;

        match result {
            Ok(_) => {
                info!("Saved sync progress: block {}", block_number);
                Ok(())
            }
            Err(e) => {
                warn!("Could not save sync state (table may not exist yet): {}", e);
                Ok(()) // Non-fatal — sync progress is also tracked in memory
            }
        }
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

/// Classify a Solidity event log topic to a BlockchainEventType
/// Topic[0] = keccak256(event_signature)
fn classify_event_topic(topic: &ethers::types::H256) -> Option<BlockchainEventType> {
    use ethers::utils::keccak256;

    // Canonical ABI signatures from BountyManager.sol
    // Note: ThreatVerdict = uint8, array types use brackets
    let bounty_created = ethers::types::H256::from(keccak256("BountyCreated(uint256,address,string,uint256,uint256)"));
    let analysis_submitted = ethers::types::H256::from(keccak256("AnalysisSubmitted(uint256,address,uint8,uint256,uint256)"));
    let consensus_reached = ethers::types::H256::from(keccak256("ConsensusReached(uint256,uint8,uint256,uint256)"));
    let rewards_distributed = ethers::types::H256::from(keccak256("RewardsDistributed(uint256,address[],uint256[],uint256[])"));

    if *topic == bounty_created {
        Some(BlockchainEventType::BountyCreated)
    } else if *topic == analysis_submitted {
        Some(BlockchainEventType::SubmissionStaked)
    } else if *topic == consensus_reached {
        Some(BlockchainEventType::ConsensusReached)
    } else if *topic == rewards_distributed {
        Some(BlockchainEventType::PayoutDistributed)
    } else {
        None // Unknown event
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
