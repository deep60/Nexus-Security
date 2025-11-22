//! Blockchain transaction utilities

use ethers::types::{Transaction, TransactionReceipt, H256, U256};
use super::BlockchainError;

/// Transaction status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    Pending,
    Confirmed(u64), // Block number
    Failed(String), // Error message
}

/// Extended transaction info
#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub hash: H256,
    pub from: String,
    pub to: Option<String>,
    pub value: U256,
    pub gas_used: Option<U256>,
    pub gas_price: U256,
    pub nonce: U256,
    pub block_number: Option<u64>,
    pub status: TransactionStatus,
}

impl From<Transaction> for TransactionInfo {
    fn from(tx: Transaction) -> Self {
        Self {
            hash: tx.hash,
            from: format!("{:?}", tx.from),
            to: tx.to.map(|addr| format!("{:?}", addr)),
            value: tx.value,
            gas_used: None,
            gas_price: tx.gas_price.unwrap_or_default(),
            nonce: tx.nonce,
            block_number: tx.block_number.map(|n| n.as_u64()),
            status: if tx.block_number.is_some() {
                TransactionStatus::Confirmed(tx.block_number.unwrap().as_u64())
            } else {
                TransactionStatus::Pending
            },
        }
    }
}

impl TransactionInfo {
    /// Update with receipt information
    pub fn with_receipt(mut self, receipt: &TransactionReceipt) -> Self {
        self.gas_used = Some(receipt.gas_used.unwrap_or_default());
        self.block_number = receipt.block_number.map(|n| n.as_u64());
        
        self.status = if receipt.status == Some(1.into()) {
            TransactionStatus::Confirmed(receipt.block_number.unwrap().as_u64())
        } else {
            TransactionStatus::Failed("Transaction reverted".to_string())
        };
        
        self
    }

    /// Calculate transaction cost in wei
    pub fn total_cost(&self) -> U256 {
        let gas_cost = self.gas_used.unwrap_or(U256::zero()) * self.gas_price;
        self.value + gas_cost
    }

    /// Check if transaction is confirmed
    pub fn is_confirmed(&self) -> bool {
        matches!(self.status, TransactionStatus::Confirmed(_))
    }

    /// Check if transaction failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, TransactionStatus::Failed(_))
    }
}

/// Transaction builder helper
pub struct TransactionBuilder {
    pub to: Option<String>,
    pub value: U256,
    pub data: Vec<u8>,
    pub gas_limit: Option<U256>,
    pub gas_price: Option<U256>,
    pub nonce: Option<U256>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            to: None,
            value: U256::zero(),
            data: Vec::new(),
            gas_limit: None,
            gas_price: None,
            nonce: None,
        }
    }

    pub fn to(mut self, address: impl Into<String>) -> Self {
        self.to = Some(address.into());
        self
    }

    pub fn value(mut self, value: U256) -> Self {
        self.value = value;
        self
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
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

    pub fn nonce(mut self, nonce: U256) -> Self {
        self.nonce = Some(nonce);
        self
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_builder() {
        let tx = TransactionBuilder::new()
            .to("0x1234567890123456789012345678901234567890")
            .value(U256::from(1000))
            .gas_limit(U256::from(21000))
            .build();
        
        assert_eq!(tx.value, U256::from(1000));
        assert_eq!(tx.gas_limit, Some(U256::from(21000)));
    }
}
