use anyhow::{Result, Context};
use sqlx::PgPool;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use ethers::prelude::*;
use tracing::info;

use crate::config::Config;
use crate::blockchain::BlockchainProvider;

pub struct PaymentService {
    config: Config,
    db_pool: PgPool,
    redis_conn: ConnectionManager,
    provider: BlockchainProvider,
}

impl PaymentService {
    pub async fn new(
        config: Config,
        db_pool: PgPool,
        redis_conn: ConnectionManager,
        provider: BlockchainProvider,
    ) -> Result<Self> {
        Ok(Self {
            config,
            db_pool,
            redis_conn,
            provider,
        })
    }

    /// Get the provider for direct blockchain queries
    pub fn provider(&self) -> &BlockchainProvider {
        &self.provider
    }

    /// Get the database pool
    pub fn db_pool(&self) -> &PgPool {
        &self.db_pool
    }

    /// Get token balance for an address using ThreatToken.balanceOf
    pub async fn get_token_balance(&self, address: &str) -> Result<U256> {
        let addr: Address = address.parse()
            .context("Invalid Ethereum address")?;

        let balance = self.provider
            .threat_token()
            .method::<_, U256>("balanceOf", (addr,))?
            .call()
            .await
            .context("Failed to call balanceOf")?;

        Ok(balance)
    }

    /// Get transaction receipt from the chain
    pub async fn get_tx_receipt(&self, tx_hash: &str) -> Result<Option<TransactionReceipt>> {
        let hash: H256 = tx_hash.parse()
            .context("Invalid transaction hash")?;

        let receipt = self.provider
            .client()
            .get_transaction_receipt(hash)
            .await
            .context("Failed to get transaction receipt")?;

        Ok(receipt)
    }

    /// Estimate gas for a standard ERC20 transfer
    pub async fn estimate_gas_for_transfer(&self) -> Result<U256> {
        let gas_price = self.provider
            .client()
            .get_gas_price()
            .await
            .context("Failed to get gas price")?;

        // Standard ERC20 transfer gas ~ 65,000
        let estimated_gas = U256::from(65_000) * gas_price;
        Ok(estimated_gas)
    }

    /// Health check
    pub async fn health_check(&self) -> bool {
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.provider.client().get_block_number(),
        ).await {
            Ok(Ok(_)) => true,
            _ => false,
        }
    }
}
