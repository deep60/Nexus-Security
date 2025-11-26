use anyhow::Result;
use ethers::prelude::*;
use std::sync::Arc;
use tracing::info;

use crate::config::BlockchainConfig;

pub type BlockchainProvider = Arc<Provider<Ws>>;

/// Create blockchain provider
pub async fn create_provider(config: &BlockchainConfig) -> Result<BlockchainProvider> {
    info!("Connecting to blockchain at {}", config.rpc_url);

    // Use WebSocket provider for real-time event listening
    let ws_url = config.ws_url.as_ref().unwrap_or(&config.rpc_url);
    let provider = Provider::<Ws>::connect(ws_url).await?;

    // Verify connection
    let chain_id = provider.get_chainid().await?;
    info!("Connected to blockchain with chain ID: {}", chain_id);

    if chain_id.as_u64() != config.chain_id {
        return Err(anyhow::anyhow!(
            "Chain ID mismatch: expected {}, got {}",
            config.chain_id,
            chain_id
        ));
    }

    Ok(Arc::new(provider))
}

/// Get current gas price with multiplier
pub async fn get_gas_price(
    provider: &Provider<Ws>,
    multiplier: f64,
    max_gwei: u64,
) -> Result<U256> {
    let base_gas_price = provider.get_gas_price().await?;
    let adjusted = (base_gas_price.as_u128() as f64 * multiplier) as u128;
    let max_price = U256::from(max_gwei) * U256::from(1_000_000_000u64);

    Ok(U256::from(adjusted).min(max_price))
}

/// Wait for transaction confirmation
pub async fn wait_for_confirmations(
    provider: &Provider<Ws>,
    tx_hash: H256,
    confirmations: usize,
) -> Result<Option<TransactionReceipt>> {
    info!("Waiting for {} confirmations for tx {}", confirmations, tx_hash);

    let receipt = provider
        .get_transaction_receipt(tx_hash)
        .await?;

    if let Some(ref receipt) = receipt {
        let current_block = provider.get_block_number().await?;
        let tx_block = receipt.block_number.unwrap_or_default();

        if current_block.as_u64() >= tx_block.as_u64() + confirmations as u64 {
            return Ok(Some(receipt.clone()));
        }
    }

    Ok(None)
}
