// backend/bounty-manager/src/services/blockchain.rs

use std::sync::Arc;
use std::time::Duration;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ethers::{
    providers::{Provider, Http, Middleware},
    types::{Address, U256, H256},
    contract::Contract,
    abi::Abi,
    signers::{LocalWallet, Signer},
    middleware::SignerMiddleware,
};

type BlockchainClient = SignerMiddleware<Provider<Http>, LocalWallet>;

#[derive(Debug, Clone)]
pub struct BlockchainService {
    client: Arc<BlockchainClient>,
    bounty_manager: Contract<BlockchainClient>,
    threat_token: Contract<BlockchainClient>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StakeTransaction {
    pub transaction_hash: String,
    pub from_address: String,
    pub amount: u64,
    pub bounty_id: Uuid,
    pub block_number: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayoutTransaction {
    pub transaction_hash: String,
    pub to_address: String,
    pub amount: u64,
    pub bounty_id: Uuid,
    pub block_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Contract error: {0}")]
    ContractError(String),
}

impl From<anyhow::Error> for BlockchainError {
    fn from(e: anyhow::Error) -> Self {
        BlockchainError::ContractError(e.to_string())
    }
}

impl BlockchainService {
    pub async fn new(
        rpc_url: &str,
        private_key: &str,
        chain_id: u64,
        bounty_manager_address: &str,
        threat_token_address: &str,
        bounty_manager_abi: Abi,
        threat_token_abi: Abi,
    ) -> Result<Self, BlockchainError> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| BlockchainError::ConnectionError(e.to_string()))?
            .interval(Duration::from_millis(1000));

        let wallet: LocalWallet = private_key
            .parse::<LocalWallet>()
            .map_err(|e| BlockchainError::ConnectionError(format!("Invalid private key: {}", e)))?
            .with_chain_id(chain_id);

        let client = Arc::new(SignerMiddleware::new(provider, wallet));

        let bm_addr: Address = bounty_manager_address
            .parse()
            .map_err(|_| BlockchainError::InvalidAddress(bounty_manager_address.to_string()))?;

        let tt_addr: Address = threat_token_address
            .parse()
            .map_err(|_| BlockchainError::InvalidAddress(threat_token_address.to_string()))?;

        let bounty_manager = Contract::new(bm_addr, bounty_manager_abi, Arc::clone(&client));
        let threat_token = Contract::new(tt_addr, threat_token_abi, Arc::clone(&client));

        Ok(Self {
            client,
            bounty_manager,
            threat_token,
        })
    }

    /// Approve and stake tokens for a bounty analysis submission
    /// Calls ThreatToken.approve(bountyManagerAddress, amount)
    pub async fn create_stake_transaction(
        &self,
        _from_address: &str,
        bounty_id: Uuid,
        amount: u64,
    ) -> Result<StakeTransaction, BlockchainError> {
        let amount_u256 = U256::from(amount);

        // Approve BountyManager to spend tokens
        let bm_address = self.bounty_manager.address();
        let tx = self.threat_token
            .method::<_, bool>("approve", (bm_address, amount_u256))
            .map_err(|e| BlockchainError::ContractError(e.to_string()))?;

        let pending_tx = tx.send().await
            .map_err(|e| BlockchainError::TransactionFailed(e.to_string()))?;

        let tx_hash = pending_tx.tx_hash();

        // Wait for receipt
        let receipt = pending_tx.await
            .map_err(|e| BlockchainError::TransactionFailed(e.to_string()))?;

        let block_number = receipt.and_then(|r| r.block_number.map(|n| n.as_u64()));

        Ok(StakeTransaction {
            transaction_hash: format!("{:?}", tx_hash),
            from_address: format!("{:?}", self.client.address()),
            amount,
            bounty_id,
            block_number,
        })
    }

    /// Distribute payout: Transfer tokens to an analyst
    /// Calls ThreatToken.transfer(to, amount)
    pub async fn create_payout_transaction(
        &self,
        to_address: &str,
        bounty_id: Uuid,
        amount: u64,
    ) -> Result<PayoutTransaction, BlockchainError> {
        let to: Address = to_address
            .parse()
            .map_err(|_| BlockchainError::InvalidAddress(to_address.to_string()))?;

        let amount_u256 = U256::from(amount);

        let tx = self.threat_token
            .method::<_, bool>("transfer", (to, amount_u256))
            .map_err(|e| BlockchainError::ContractError(e.to_string()))?;

        let pending_tx = tx.send().await
            .map_err(|e| BlockchainError::TransactionFailed(e.to_string()))?;

        let tx_hash = pending_tx.tx_hash();

        let receipt = pending_tx.await
            .map_err(|e| BlockchainError::TransactionFailed(e.to_string()))?;

        let block_number = receipt.and_then(|r| r.block_number.map(|n| n.as_u64()));

        Ok(PayoutTransaction {
            transaction_hash: format!("{:?}", tx_hash),
            to_address: to_address.to_string(),
            amount,
            bounty_id,
            block_number,
        })
    }

    /// Verify a transaction by checking its receipt
    pub async fn verify_transaction(&self, tx_hash: &str) -> Result<bool, BlockchainError> {
        let hash: H256 = tx_hash
            .parse()
            .map_err(|_| BlockchainError::ContractError("Invalid transaction hash".to_string()))?;

        let receipt = self.client
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| BlockchainError::ConnectionError(e.to_string()))?;

        match receipt {
            Some(r) => Ok(r.status == Some(1.into())),
            None => Ok(false), // Transaction not yet mined
        }
    }

    /// Get transaction status from chain
    pub async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus, BlockchainError> {
        let hash: H256 = tx_hash
            .parse()
            .map_err(|_| BlockchainError::ContractError("Invalid transaction hash".to_string()))?;

        let receipt = self.client
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| BlockchainError::ConnectionError(e.to_string()))?;

        match receipt {
            Some(r) => {
                if r.status == Some(1.into()) {
                    Ok(TransactionStatus::Confirmed)
                } else {
                    Ok(TransactionStatus::Failed)
                }
            }
            None => Ok(TransactionStatus::Pending),
        }
    }

    /// Get token balance for an address
    pub async fn get_balance(&self, address: &str) -> Result<u64, BlockchainError> {
        let addr: Address = address
            .parse()
            .map_err(|_| BlockchainError::InvalidAddress(address.to_string()))?;

        let balance: U256 = self.threat_token
            .method("balanceOf", (addr,))
            .map_err(|e| BlockchainError::ContractError(e.to_string()))?
            .call()
            .await
            .map_err(|e| BlockchainError::ConnectionError(e.to_string()))?;

        Ok(balance.as_u64())
    }

    /// Expose the client for use by sync services
    pub fn get_client(&self) -> &BlockchainClient {
        &self.client
    }

    /// Health check — verifies RPC connectivity
    pub async fn health_check(&self) -> bool {
        match tokio::time::timeout(
            Duration::from_secs(5),
            self.client.get_block_number(),
        ).await {
            Ok(Ok(_)) => true,
            _ => false,
        }
    }
}
