// Transaction building and execution
use anyhow::Result;
use ethers::prelude::*;
use tracing::{info, warn};

pub struct TransactionBuilder {
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub data: Option<Bytes>,
    pub gas_limit: Option<U256>,
    pub gas_price: Option<U256>,
    pub nonce: Option<U256>,
}

impl TransactionBuilder {
    pub fn new(from: Address, to: Address) -> Self {
        Self {
            from,
            to,
            value: U256::zero(),
            data: None,
            gas_limit: None,
            gas_price: None,
            nonce: None,
        }
    }

    pub fn value(mut self, value: U256) -> Self {
        self.value = value;
        self
    }

    pub fn data(mut self, data: Bytes) -> Self {
        self.data = Some(data);
        self
    }

    pub fn gas_limit(mut self, limit: U256) -> Self {
        self.gas_limit = Some(limit);
        self
    }

    pub fn gas_price(mut self, price: U256) -> Self {
        self.gas_price = Some(price);
        self
    }

    pub fn build(self) -> TransactionRequest {
        TransactionRequest::new()
            .from(self.from)
            .to(self.to)
            .value(self.value)
            .data(self.data.unwrap_or_default())
    }
}

pub async fn send_transaction(
    provider: &Provider<Ws>,
    tx: TransactionRequest,
) -> Result<H256> {
    info!("Sending transaction...");
    let pending_tx = provider.send_transaction(tx, None).await?;
    Ok(pending_tx.tx_hash())
}

pub async fn wait_for_confirmation(
    provider: &Provider<Ws>,
    tx_hash: H256,
    confirmations: usize,
) -> Result<TransactionReceipt> {
    info!("Waiting for confirmation of tx {}", tx_hash);
    
    let receipt = provider
        .get_transaction_receipt(tx_hash)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Transaction not found"))?;

    Ok(receipt)
}
