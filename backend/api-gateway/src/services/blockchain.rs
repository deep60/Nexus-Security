//! Handles all blockchain interactions including:
//! - Smart contract deployments and interactions
//! - Bounty creation and management
//! - Token transfers and staking
//! - Reputation system updates
//! - Transaction monitoring

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Mutex};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use ethers::{
    providers::{Provider, Http, Middleware, ProviderError},
    types::{Address, U256, H256, TransactionRequest, Transaction, TransactionReceipt},
    contract::{Contract, ContractError},
    abi::{Abi, Token},                      //(Abi - Application Binary Interface)
    signers::{LocalWallet, Signer},
    middleware::SignerMiddleware, 
};

use crate::models::{TransactionStatus, ReputationScore, ThreatSeverity};
use crate::config;

/// Blockchain client wrapper
type BlockchainClient = SignerMiddleware<Provider<Http>, LocalWallet>;

/// Smart contract instances
#[derive(Clone)]
pub struct ContractInstances {
    pub bounty_manager: Contract<BlockchainClient>,
    pub threat_token: Contract<BlockchainClient>,
    pub reputation_system: Contract<BlockchainClient>,
}

/// Blockchain service for handling Web3 operations
pub struct BlockchainService {
    client: Arc<BlockchainClient>,
    contracts: ContractInstances,
    config: config::BlockchainConfig,
    pending_transactions: Arc<RwLock<HashMap<H256, PendingTransaction>>>,
    nonce_manager: Arc<Mutex<u64>>,
}

/// Pending transaction tracking
#[derive(Debug, Clone)]
struct PendingTransaction {
    pub hash: H256,
    pub tx_type: TransactionType,
    pub created_at: DateTime<Utc>,
    pub retry_count: u32,
    pub user_id: Option<Uuid>,
    pub bounty_id: Option<Uuid>,
}

/// Types of blockchain transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    CreateBounty,
    SubmitAnalysis,
    StakeTokens,
    ClaimReward,
    UpdateReputation,
    TokenTransfer,
}

/// Bounty creation parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBountyParams {
    pub bounty_id: Uuid,
    pub creator: Address,
    pub reward_amount: U256,
    pub file_hash: H256,
    pub deadline: u64,
    pub minimum_stake: U256,
    pub required_consensus: u8,
}

/// Analysis submission parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitAnalysisParams {
    pub bounty_id: Uuid,
    pub analyst: Address,
    pub is_malicious: bool,
    pub confidence: u8,
    pub stake_amount: U256,
    pub analysis_data: String,
}

/// Blockchain transaction result
#[derive(Debug, Serialize)]
pub struct TransactionResult {
    pub hash: H256,
    pub status: TransactionStatus,
    pub block_number: Option<u64>,
    pub gas_used: Option<U256>,
    pub confirmations: u64,
}

/// Bounty status from blockchain
#[derive(Debug, Serialize, Deserialize)]
pub struct BountyStatus {
    pub id: Uuid,
    pub creator: Address,
    pub reward_amount: U256,
    pub total_stake: U256,
    pub submissions_count: u32,
    pub consensus_reached: bool,
    pub is_resolved: bool,
    pub resolution: Option<bool>,
    pub deadline: u64,
}

impl BlockchainService {
    /// Create a new blockchain service instance
    pub async fn new(config: config::BlockchainConfig) -> Result<Self> {
        // Setup provider
        let provider = Provider::<Http>::try_from(&config.rpc_url)
            .context("Failed to create HTTP provider")?
            .interval(Duration::from_millis(1000));

        // Setup wallet
        let wallet = config.private_key.parse::<LocalWallet>()
            .context("Invalid private key")?
            .with_chain_id(config.chain_id);

        // Create client
        let client = Arc::new(SignerMiddleware::new(provider, wallet));

        // Get current nonce
        let address = client.address();
        let nonce = client.get_transaction_count(address, None).await
            .context("Failed to get transaction count")?;

        // Load contract ABIs and create instances
        let contracts = Self::load_contracts(&client, &config).await?;

        Ok(Self {
            client,
            contracts,
            config,
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
            nonce_manager: Arc::new(Mutex::new(nonce.as_u64())),
        })
    }

    /// Load smart contract instances
    async fn load_contracts(
        client: &Arc<BlockchainClient>,
        config: &config::BlockchainConfig,
    ) -> Result<ContractInstances> {
        // In a real implementation, you would load the ABIs from files or embedded resources
        // For now, we'll create placeholder contract instances

        let bounty_manager_address: Address = config.contracts.bounty_manager.parse()
            .context("Invalid bounty manager address")?;

        let threat_token_address: Address = config.contracts.threat_token.parse()
            .context("Invalid threat token address")?;

        let reputation_system_address: Address = config.contracts.reputation_system.parse()
            .context("Invalid reputation system address")?;

        // Load ABIs (in practice, these would come from JSON files)
        let bounty_manager_abi = Self::get_bounty_manager_abi();
        let threat_token_abi = Self::get_threat_token_abi();
        let reputation_system_abi = Self::get_reputation_system_abi();

        let bounty_manager = Contract::new(
            bounty_manager_address,
            bounty_manager_abi,
            Arc::clone(client),
        );

        let threat_token = Contract::new(
            threat_token_address,
            threat_token_abi,
            Arc::clone(client),
        );

        let reputation_system = Contract::new(
            reputation_system_address,
            reputation_system_abi,
            Arc::clone(client),
        );

        Ok(ContractInstances {
            bounty_manager,
            threat_token,
            reputation_system,
        })
    }

    /// Create a new bounty on the blockchain
    pub async fn create_bounty(&self, params: CreateBountyParams) -> Result<H256> {
        let bounty_id_bytes = *params.bounty_id.as_bytes();

        let tx = self.contracts.bounty_manager
            .method::<_, H256>("createBounty", (
                bounty_id_bytes,
                params.reward_amount,
                params.file_hash,
                params.deadline,
                params.minimum_stake,
                params.required_consensus,
            ))?
            .gas(self.config.gas_limit)
            .gas_price(self.config.gas_price_gwei * 1_000_000_000);

        let pending_tx = tx.send().await
            .context("Failed to send create bounty transaction")?;

        let tx_hash = pending_tx.tx_hash();
        
        // Track pending transaction
        self.track_pending_transaction(
            tx_hash,
            TransactionType::CreateBounty,
            None,
            Some(params.bounty_id),
        ).await;

        tracing::info!("Created bounty transaction: {:?}", tx_hash);
        Ok(tx_hash)
    }

    /// Submit analysis for a bounty
    pub async fn submit_analysis(&self, params: SubmitAnalysisParams) -> Result<H256> {
        let bounty_id_bytes = *params.bounty_id.as_bytes();

        let tx = self.contracts.bounty_manager
            .method::<_, H256>("submitAnalysis", (
                bounty_id_bytes,
                params.is_malicious,
                params.confidence,
                params.analysis_data,
            ))?
            .value(params.stake_amount)
            .gas(self.config.gas_limit)
            .gas_price(self.config.gas_price_gwei * 1_000_000_000);

        let pending_tx = tx.send().await
            .context("Failed to send submit analysis transaction")?;

        let tx_hash = pending_tx.tx_hash();
        
        self.track_pending_transaction(
            tx_hash,
            TransactionType::SubmitAnalysis,
            None,
            Some(params.bounty_id),
        ).await;

        tracing::info!("Submitted analysis transaction: {:?}", tx_hash);
        Ok(tx_hash)
    }

    /// Stake tokens for analysis submission
    pub async fn stake_tokens(&self, bounty_id: Uuid, amount: U256, user_id: Uuid) -> Result<H256> {
        let bounty_id_bytes = *bounty_id.as_bytes();

        let tx = self.contracts.threat_token
            .method::<_, H256>("approve", (
                self.config.contracts.bounty_manager.parse::<Address>()?,
                amount,
            ))?
            .gas(self.config.gas_limit)
            .gas_price(self.config.gas_price_gwei * 1_000_000_000);

        let pending_tx = tx.send().await
            .context("Failed to send stake tokens transaction")?;

        let tx_hash = pending_tx.tx_hash();
        
        self.track_pending_transaction(
            tx_hash,
            TransactionType::StakeTokens,
            Some(user_id),
            Some(bounty_id),
        ).await;

        tracing::info!("Staked tokens transaction: {:?}", tx_hash);
        Ok(tx_hash)
    }

    /// Claim rewards for successful analysis
    pub async fn claim_reward(&self, bounty_id: Uuid, user_id: Uuid) -> Result<H256> {
        let bounty_id_bytes = *bounty_id.as_bytes();

        let tx = self.contracts.bounty_manager
            .method::<_, H256>("claimReward", (bounty_id_bytes,))?
            .gas(self.config.gas_limit)
            .gas_price(self.config.gas_price_gwei * 1_000_000_000);

        let pending_tx = tx.send().await
            .context("Failed to send claim reward transaction")?;

        let tx_hash = pending_tx.tx_hash();
        
        self.track_pending_transaction(
            tx_hash,
            TransactionType::ClaimReward,
            Some(user_id),
            Some(bounty_id),
        ).await;

        tracing::info!("Claimed reward transaction: {:?}", tx_hash);
        Ok(tx_hash)
    }

    /// Update user reputation on blockchain
    pub async fn update_reputation(&self, user_address: Address, new_score: f64) -> Result<H256> {
        let score_scaled = U256::from((new_score * 100.0) as u64); // Scale to avoid decimals
        
        let tx = self.contracts.reputation_system
            .method::<_, H256>("updateReputation", (user_address, score_scaled))?
            .gas(self.config.gas_limit)
            .gas_price(self.config.gas_price_gwei * 1_000_000_000);

        let pending_tx = tx.send().await
            .context("Failed to send update reputation transaction")?;

        let tx_hash = pending_tx.tx_hash();
        
        self.track_pending_transaction(
            tx_hash,
            TransactionType::UpdateReputation,
            None,
            None,
        ).await;

        tracing::info!("Updated reputation transaction: {:?}", tx_hash);
        Ok(tx_hash)
    }

    /// Get bounty status from blockchain
    pub async fn get_bounty_status(&self, bounty_id: Uuid) -> Result<BountyStatus> {
        let bounty_id_bytes = *bounty_id.as_bytes();

        let result: (Address, U256, U256, u32, bool, bool, bool, u64) = self.contracts.bounty_manager
            .method("getBountyStatus", (bounty_id_bytes,))?
            .call()
            .await
            .context("Failed to get bounty status")?;

        let (creator, reward_amount, total_stake, submissions_count, consensus_reached, is_resolved, resolution, deadline) = result;

        Ok(BountyStatus {
            id: bounty_id,
            creator,
            reward_amount,
            total_stake,
            submissions_count,
            consensus_reached,
            is_resolved,
            resolution: if is_resolved { Some(resolution) } else { None },
            deadline,
        })
    }

    /// Get user's token balance
    pub async fn get_token_balance(&self, user_address: Address) -> Result<U256> {
        let balance: U256 = self.contracts.threat_token
            .method("balanceOf", (user_address,))?
            .call()
            .await
            .context("Failed to get token balance")?;

        Ok(balance)
    }

    /// Get user's reputation score from blockchain
    pub async fn get_reputation_score(&self, user_address: Address) -> Result<ReputationScore> {
        let result: (u64, u32, u32, u64) = self.contracts.reputation_system
            .method("getReputation", (user_address,))?
            .call()
            .await
            .context("Failed to get reputation score")?;

        let (score_scaled, total_submissions, successful_submissions, last_updated_timestamp) = result;
        
        let last_updated = DateTime::from_timestamp(last_updated_timestamp as i64, 0)
            .unwrap_or_else(Utc::now);

        Ok(ReputationScore {
            score: (score_scaled as f64) / 100.0, // Unscale the score
            total_submissions,
            successful_submissions,
            last_updated,
        })
    }

    /// Check transaction status
    pub async fn get_transaction_status(&self, tx_hash: H256) -> Result<TransactionResult> {
        let receipt = self.client.get_transaction_receipt(tx_hash).await?;
        let current_block = self.client.get_block_number().await?;
        
        match receipt {
            Some(receipt) => {
                let confirmations = current_block.saturating_sub(receipt.block_number.unwrap_or_default());
                let status = if receipt.status == Some(1.into()) {
                    if confirmations >= self.config.confirmation_blocks.into() {
                        TransactionStatus::Confirmed
                    } else {
                        TransactionStatus::Pending
                    }
                } else {
                    TransactionStatus::Failed
                };

                Ok(TransactionResult {
                    hash: tx_hash,
                    status,
                    block_number: receipt.block_number.map(|n| n.as_u64()),
                    gas_used: receipt.gas_used,
                    confirmations: confirmations.as_u64(),
                })
            }
            None => Ok(TransactionResult {
                hash: tx_hash,
                status: TransactionStatus::Pending,
                block_number: None,
                gas_used: None,
                confirmations: 0,
            }),
        }
    }

    /// Monitor pending transactions and update their status
    pub async fn monitor_transactions(&self) -> Result<()> {
        let mut pending_txs = self.pending_transactions.write().await;
        let mut completed_hashes = Vec::new();

        for (hash, pending_tx) in pending_txs.iter_mut() {
            match self.get_transaction_status(*hash).await {
                Ok(result) => {
                    match result.status {
                        TransactionStatus::Confirmed | TransactionStatus::Failed => {
                            tracing::info!("Transaction {} completed with status: {:?}", hash, result.status);
                            completed_hashes.push(*hash);
                        }
                        TransactionStatus::Pending => {
                            // Check if transaction is too old and needs retry
                            let age = Utc::now().signed_duration_since(pending_tx.created_at);
                            if age.num_minutes() > 10 && pending_tx.retry_count < self.config.retry_attempts {
                                pending_tx.retry_count += 1;
                                tracing::warn!("Transaction {} is taking too long, retry count: {}", hash, pending_tx.retry_count);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to check transaction status for {}: {}", hash, e);
                }
            }
        }

        // Remove completed transactions
        for hash in completed_hashes {
            pending_txs.remove(&hash);
        }

        Ok(())
    }

    /// Track a pending transaction
    async fn track_pending_transaction(
        &self,
        hash: H256,
        tx_type: TransactionType,
        user_id: Option<Uuid>,
        bounty_id: Option<Uuid>,
    ) {
        let pending_tx = PendingTransaction {
            hash,
            tx_type,
            created_at: Utc::now(),
            retry_count: 0,
            user_id,
            bounty_id,
        };

        let mut pending_txs = self.pending_transactions.write().await;
        pending_txs.insert(hash, pending_tx);
    }

    /// Get next nonce for transactions
    async fn get_next_nonce(&self) -> u64 {
        let mut nonce_guard = self.nonce_manager.lock().await;
        let nonce = *nonce_guard;
        *nonce_guard += 1;
        nonce
    }

    // Load real ABIs from compiled contract artifacts
    fn get_bounty_manager_abi() -> Abi {
        crate::services::abi_loader::load_bounty_manager_abi()
            .expect("Failed to load BountyManager ABI - run blockchain/scripts/extract-abis.sh")
    }

    fn get_threat_token_abi() -> Abi {
        crate::services::abi_loader::load_threat_token_abi()
            .expect("Failed to load ThreatToken ABI - run blockchain/scripts/extract-abis.sh")
    }

    fn get_reputation_system_abi() -> Abi {
        crate::services::abi_loader::load_reputation_system_abi()
            .expect("Failed to load ReputationSystem ABI - run blockchain/scripts/extract-abis.sh")
    }
}

/// Helper functions for blockchain operations
impl BlockchainService {
    /// Convert reputation score to blockchain format
    pub fn encode_reputation_score(score: &ReputationScore) -> U256 {
        U256::from((score.score * 100.0) as u64)
    }

    /// Convert blockchain format to reputation score
    pub fn decode_reputation_score(
        score_scaled: U256,
        total_submissions: u32,
        successful_submissions: u32,
        last_updated_timestamp: u64,
    ) -> ReputationScore {
        let last_updated = DateTime::from_timestamp(last_updated_timestamp as i64, 0)
            .unwrap_or_else(Utc::now);

        ReputationScore {
            score: score_scaled.as_u64() as f64 / 100.0,
            total_submissions,
            successful_submissions,
            last_updated,
        }
    }

    /// Calculate gas price based on network conditions
    pub async fn estimate_gas_price(&self) -> Result<U256> {
        let gas_price = self.client.get_gas_price().await
            .context("Failed to get gas price")?;
        
        // Add 10% buffer
        Ok(gas_price * 110 / 100)
    }

    /// Validate Ethereum address
    pub fn validate_address(address: &str) -> Result<Address> {
        address.parse::<Address>()
            .context("Invalid Ethereum address format")
    }

    /// Health check for blockchain service
    /// TODO: Implement actual blockchain connectivity check
    pub async fn health_check(&self) -> bool {
        // Stub implementation - always returns true
        // In production, this should ping the RPC endpoint
        true
    }

    /// Create analysis bounty on blockchain
    /// TODO: Implement actual blockchain transaction
    pub async fn create_analysis_bounty(&self, _params: CreateBountyParams) -> Result<H256> {
        // Stub implementation
        anyhow::bail!("create_analysis_bounty not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_reputation_score() {
        let original_score = ReputationScore {
            score: 85.5,
            total_submissions: 100,
            successful_submissions: 85,
            last_updated: Utc::now(),
        };

        let encoded = BlockchainService::encode_reputation_score(&original_score);
        let decoded = BlockchainService::decode_reputation_score(
            encoded,
            original_score.total_submissions,
            original_score.successful_submissions,
            original_score.last_updated.timestamp() as u64,
        );

        assert_eq!((original_score.score * 100.0) as u64, (decoded.score * 100.0) as u64);
        assert_eq!(original_score.total_submissions, decoded.total_submissions);
        assert_eq!(original_score.successful_submissions, decoded.successful_submissions);
    }

    #[test]
    fn test_validate_address() {
        let valid_address = "0x742b15C2d1f7a9fE9a8d2F1B22d7e3aF95c30B34";
        assert!(BlockchainService::validate_address(valid_address).is_ok());

        let invalid_address = "invalid_address";
        assert!(BlockchainService::validate_address(invalid_address).is_err());
    }
}