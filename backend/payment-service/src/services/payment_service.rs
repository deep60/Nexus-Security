use anyhow::{Context, Result};
use ethers::middleware::SignerMiddleware;
use ethers::prelude::*;
use ethers::signers::{LocalWallet, Signer};
use redis::aio::ConnectionManager;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::blockchain::{BlockchainProvider, TokenContract};
use crate::config::Config;
use crate::models::*;

/// Signer-bound client used for treasury-authored transactions.
pub type SignerClient = SignerMiddleware<BlockchainProvider, LocalWallet>;

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

    pub fn db_pool(&self) -> &PgPool {
        &self.db_pool
    }

    /// Build a treasury-signing client (provider + treasury wallet).
    fn signer_client(&self) -> Result<Arc<SignerClient>> {
        let wallet = self
            .config
            .blockchain
            .treasury_private_key
            .parse::<LocalWallet>()
            .context("Invalid treasury private key")?
            .with_chain_id(self.config.blockchain.chain_id);

        let client = SignerMiddleware::new(self.provider.clone(), wallet);
        Ok(Arc::new(client))
    }

    fn token_contract(&self) -> Result<TokenContract<Provider<Ws>>> {
        let addr: Address = self
            .config
            .blockchain
            .token_contract_address
            .parse()
            .context("Invalid token contract address")?;
        Ok(TokenContract::new(addr, self.provider.clone()))
    }

    fn token_contract_signed(&self) -> Result<TokenContract<SignerClient>> {
        let addr: Address = self
            .config
            .blockchain
            .token_contract_address
            .parse()
            .context("Invalid token contract address")?;
        Ok(TokenContract::new(addr, self.signer_client()?))
    }

    // -- Reads ------------------------------------------------------------

    pub async fn get_token_balance(&self, address: &str) -> Result<U256> {
        let addr: Address = address.parse().context("Invalid Ethereum address")?;
        let token = self.token_contract()?;
        token
            .balance_of(addr)
            .call()
            .await
            .context("Failed to call balanceOf")
    }

    pub async fn get_tx_receipt(
        &self,
        tx_hash: &str,
    ) -> Result<Option<ethers::types::TransactionReceipt>> {
        let hash: H256 = tx_hash.parse().context("Invalid transaction hash")?;
        self.provider
            .get_transaction_receipt(hash)
            .await
            .context("Failed to get transaction receipt")
    }

    pub async fn estimate_gas_for_transfer(&self) -> Result<U256> {
        let gas_price = self
            .provider
            .get_gas_price()
            .await
            .context("Failed to get gas price")?;
        Ok(U256::from(65_000) * gas_price)
    }

    pub async fn health_check(&self) -> bool {
        matches!(
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                self.provider.get_block_number(),
            )
            .await,
            Ok(Ok(_))
        )
    }

    // -- Treasury-signed transfers ---------------------------------------

    /// Transfer tokens FROM the treasury to `to`, waiting for confirmations.
    /// Returns the transaction receipt on success.
    async fn treasury_transfer(
        &self,
        to: &str,
        amount: U256,
    ) -> Result<ethers::types::TransactionReceipt> {
        let to_addr: Address = to.parse().context("Invalid recipient address")?;
        let token = self.token_contract_signed()?;

        // Ensure the treasury can cover it.
        let treasury: Address = self
            .config
            .blockchain
            .treasury_address
            .parse()
            .context("Invalid treasury address")?;
        let balance = token.balance_of(treasury).call().await?;
        if balance < amount {
            anyhow::bail!("Treasury balance {} below required {}", balance, amount);
        }

        let call = token.transfer(to_addr, amount);
        let pending = call.send().await.context("Failed to broadcast transfer")?;

        info!("Treasury transfer broadcast: {:?}", pending.tx_hash());

        let confirmations = self.config.blockchain.confirmation_blocks as usize;
        let receipt = pending
            .confirmations(confirmations.max(1))
            .await
            .context("Failed waiting for confirmations")?
            .ok_or_else(|| anyhow::anyhow!("Transfer dropped from mempool"))?;

        if receipt.status != Some(1u64.into()) {
            anyhow::bail!("Transfer reverted on-chain: {:?}", receipt.transaction_hash);
        }

        Ok(receipt)
    }

    // -- Persistence helpers ---------------------------------------------

    /// Insert a payment row and return its id.
    async fn insert_payment(
        &self,
        bounty_id: Option<Uuid>,
        payer: &str,
        recipient: &str,
        amount: Decimal,
        payment_type: PaymentType,
        status: PaymentStatus,
    ) -> PaymentResult<Uuid> {
        let token_addr = self.config.blockchain.token_contract_address.clone();
        sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO payments
                (bounty_id, payer_address, recipient_address, amount,
                 token_address, status, payment_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(bounty_id.unwrap_or_else(Uuid::nil))
        .bind(payer)
        .bind(recipient)
        .bind(amount)
        .bind(token_addr)
        .bind(status.to_string())
        .bind(payment_type.to_string())
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| PaymentError::DatabaseError(e.to_string()))
    }

    async fn mark_payment(
        &self,
        id: Uuid,
        status: PaymentStatus,
        tx_hash: Option<&str>,
        block_number: Option<i64>,
        error: Option<&str>,
    ) -> PaymentResult<()> {
        let completed = matches!(status, PaymentStatus::Completed | PaymentStatus::Confirmed);
        sqlx::query(
            r#"
            UPDATE payments
            SET status = $2,
                transaction_hash = COALESCE($3, transaction_hash),
                metadata = COALESCE(metadata, '{}'::jsonb)
                    || jsonb_build_object('block_number', $4::bigint, 'error', $5::text),
                updated_at = NOW(),
                completed_at = CASE WHEN $6 THEN NOW() ELSE completed_at END
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status.to_string())
        .bind(tx_hash)
        .bind(block_number)
        .bind(error)
        .bind(completed)
        .execute(&self.db_pool)
        .await
        .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    fn to_wei(amount: &Decimal) -> PaymentResult<U256> {
        // Amounts are stored as whole-token decimals; assume 18 decimals.
        let scaled = amount.round_dp(0).to_string();
        U256::from_dec_str(&scaled)
            .map_err(|e| PaymentError::ValidationError(format!("invalid amount: {e}")))
    }

    // -- High-level operations -------------------------------------------

    /// Distribute a bounty reward: treasury → winner, real transfer.
    pub async fn distribute_reward(
        &self,
        req: &DistributeBountyRequest,
    ) -> PaymentResult<PaymentResponse> {
        let amount_wei = Self::to_wei(&req.amount)?;
        let payment_id = self
            .insert_payment(
                Some(req.bounty_id),
                &self.config.blockchain.treasury_address,
                &req.winner_address,
                req.amount,
                PaymentType::BountyReward,
                PaymentStatus::Processing,
            )
            .await?;

        match self
            .treasury_transfer(&req.winner_address, amount_wei)
            .await
        {
            Ok(receipt) => {
                let tx_hash = format!("{:?}", receipt.transaction_hash);
                let block = receipt.block_number.map(|b| b.as_u64() as i64);
                self.mark_payment(
                    payment_id,
                    PaymentStatus::Completed,
                    Some(&tx_hash),
                    block,
                    None,
                )
                .await?;
                Ok(PaymentResponse {
                    success: true,
                    payment_id: Some(payment_id),
                    tx_hash: Some(tx_hash),
                    message: "Bounty reward distributed".to_string(),
                    estimated_completion_time: None,
                })
            }
            Err(e) => {
                self.mark_payment(
                    payment_id,
                    PaymentStatus::Failed,
                    None,
                    None,
                    Some(&e.to_string()),
                )
                .await?;
                Err(PaymentError::TransactionFailed(e.to_string()))
            }
        }
    }

    /// Process a withdrawal: treasury → user, minus fee.
    pub async fn process_withdrawal(
        &self,
        req: &WithdrawRequest,
    ) -> PaymentResult<PaymentResponse> {
        let min = U256::from_dec_str(&self.config.payment.min_withdraw_amount).unwrap_or_default();
        let max = U256::from_dec_str(&self.config.payment.max_withdraw_amount).unwrap_or(U256::MAX);
        let amount_wei = Self::to_wei(&req.amount)?;

        if amount_wei < min {
            return Err(PaymentError::ValidationError(format!(
                "amount below minimum withdrawal ({min})"
            )));
        }
        if amount_wei > max {
            return Err(PaymentError::ValidationError(format!(
                "amount above maximum withdrawal ({max})"
            )));
        }

        // Apply withdrawal fee.
        let fee_bps = (self.config.payment.withdraw_fee_percentage * 100.0) as u64;
        let fee = amount_wei * U256::from(fee_bps) / U256::from(10_000u64);
        let net = amount_wei.saturating_sub(fee);

        let payment_id = self
            .insert_payment(
                None,
                &self.config.blockchain.treasury_address,
                &req.to_address,
                req.amount,
                PaymentType::Withdrawal,
                PaymentStatus::Processing,
            )
            .await?;

        match self.treasury_transfer(&req.to_address, net).await {
            Ok(receipt) => {
                let tx_hash = format!("{:?}", receipt.transaction_hash);
                let block = receipt.block_number.map(|b| b.as_u64() as i64);
                self.mark_payment(
                    payment_id,
                    PaymentStatus::Completed,
                    Some(&tx_hash),
                    block,
                    None,
                )
                .await?;
                Ok(PaymentResponse {
                    success: true,
                    payment_id: Some(payment_id),
                    tx_hash: Some(tx_hash),
                    message: format!("Withdrawal sent (fee {fee} wei)"),
                    estimated_completion_time: None,
                })
            }
            Err(e) => {
                self.mark_payment(
                    payment_id,
                    PaymentStatus::Failed,
                    None,
                    None,
                    Some(&e.to_string()),
                )
                .await?;
                Err(PaymentError::TransactionFailed(e.to_string()))
            }
        }
    }

    /// Record a bounty deposit intent. The actual transfer must be signed by
    /// the creator's wallet client-side; here we verify funds and persist a
    /// pending record that the transaction monitor reconciles once the
    /// on-chain deposit event arrives.
    pub async fn record_deposit_intent(
        &self,
        req: &DepositBountyRequest,
    ) -> PaymentResult<PaymentResponse> {
        let amount_wei = Self::to_wei(&req.amount)?;
        let balance = self
            .get_token_balance(&req.creator_address)
            .await
            .map_err(|e| PaymentError::BlockchainError(e.to_string()))?;
        if balance < amount_wei {
            return Err(PaymentError::InsufficientBalance(format!(
                "creator balance {balance} < required {amount_wei}"
            )));
        }

        let payment_id = self
            .insert_payment(
                Some(req.bounty_id),
                &req.creator_address,
                &self.config.blockchain.treasury_address,
                req.amount,
                PaymentType::BountyDeposit,
                PaymentStatus::Pending,
            )
            .await?;

        Ok(PaymentResponse {
            success: true,
            payment_id: Some(payment_id),
            tx_hash: None,
            message: "Deposit recorded; awaiting on-chain confirmation".to_string(),
            estimated_completion_time: None,
        })
    }

    /// Record a stake lock intent (user-signed on-chain).
    pub async fn record_stake_lock(
        &self,
        req: &LockStakeRequest,
    ) -> PaymentResult<PaymentResponse> {
        let amount_wei = Self::to_wei(&req.amount)?;
        let balance = self
            .get_token_balance(&req.address)
            .await
            .map_err(|e| PaymentError::BlockchainError(e.to_string()))?;
        if balance < amount_wei {
            return Err(PaymentError::InsufficientBalance(format!(
                "balance {balance} < stake {amount_wei}"
            )));
        }

        let payment_id = self
            .insert_payment(
                Some(req.bounty_id),
                &req.address,
                &self.config.blockchain.treasury_address,
                req.amount,
                PaymentType::StakeLock,
                PaymentStatus::Pending,
            )
            .await?;

        Ok(PaymentResponse {
            success: true,
            payment_id: Some(payment_id),
            tx_hash: None,
            message: "Stake lock recorded; awaiting on-chain confirmation".to_string(),
            estimated_completion_time: None,
        })
    }

    /// Slash a stake: treasury transfers the slashed portion out of the
    /// slashed user. In this model slashing is enforced by transferring the
    /// slashed amount from treasury custody to the bounty pool address.
    pub async fn slash_stake(&self, req: &SlashStakeRequest) -> PaymentResult<PaymentResponse> {
        let amount_wei = Self::to_wei(&req.slash_amount)?;
        let payment_id = self
            .insert_payment(
                None,
                &self.config.blockchain.treasury_address,
                &self.config.blockchain.treasury_address,
                req.slash_amount,
                PaymentType::StakeSlash,
                PaymentStatus::Processing,
            )
            .await?;

        // Slashed funds are moved to the payment/escrow contract address.
        let dest = self.config.blockchain.payment_contract_address.clone();
        match self.treasury_transfer(&dest, amount_wei).await {
            Ok(receipt) => {
                let tx_hash = format!("{:?}", receipt.transaction_hash);
                let block = receipt.block_number.map(|b| b.as_u64() as i64);
                self.mark_payment(
                    payment_id,
                    PaymentStatus::Completed,
                    Some(&tx_hash),
                    block,
                    None,
                )
                .await?;
                Ok(PaymentResponse {
                    success: true,
                    payment_id: Some(payment_id),
                    tx_hash: Some(tx_hash),
                    message: format!("Stake {} slashed", req.stake_id),
                    estimated_completion_time: None,
                })
            }
            Err(e) => {
                self.mark_payment(
                    payment_id,
                    PaymentStatus::Failed,
                    None,
                    None,
                    Some(&e.to_string()),
                )
                .await?;
                Err(PaymentError::TransactionFailed(e.to_string()))
            }
        }
    }

    /// Mark a stake as unlocked (no transfer needed; funds were never moved).
    pub async fn unlock_stake(&self, stake_id: Uuid) -> PaymentResult<PaymentResponse> {
        Ok(PaymentResponse {
            success: true,
            payment_id: None,
            tx_hash: None,
            message: format!("Stake {stake_id} marked for unlock"),
            estimated_completion_time: None,
        })
    }

    // -- Worker support ---------------------------------------------------

    /// Pending payments awaiting on-chain confirmation.
    pub async fn list_pending(&self) -> PaymentResult<Vec<Payment>> {
        sqlx::query_as::<_, Payment>(
            "SELECT * FROM payments WHERE status IN ('pending','processing') ORDER BY created_at ASC LIMIT 100",
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| PaymentError::DatabaseError(e.to_string()))
    }

    /// Reconcile a processing payment that has a tx hash by checking its receipt.
    pub async fn reconcile_payment(&self, payment: &Payment) -> PaymentResult<()> {
        let Some(tx_hash) = &payment.tx_hash else {
            return Ok(());
        };
        match self.get_tx_receipt(tx_hash).await {
            Ok(Some(receipt)) => {
                let status = if receipt.status == Some(1u64.into()) {
                    PaymentStatus::Completed
                } else {
                    PaymentStatus::Failed
                };
                let block = receipt.block_number.map(|b| b.as_u64() as i64);
                self.mark_payment(payment.id, status, Some(tx_hash), block, None)
                    .await?;
            }
            Ok(None) => { /* still pending */ }
            Err(e) => warn!("reconcile {} failed: {}", payment.id, e),
        }
        Ok(())
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    #[allow(dead_code)]
    fn redis(&self) -> ConnectionManager {
        self.redis_conn.clone()
    }
}
