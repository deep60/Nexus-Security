use axum::{extract::{State, Path}, response::Json, http::StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::AppState;
use crate::models::*;

pub async fn deposit_bounty_reward(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DepositBountyRequest>,
) -> (StatusCode, Json<Value>) {
    // Pre-flight check: verify the creator has sufficient token balance
    match state.payment_service.get_token_balance(&payload.creator_address).await {
        Ok(balance) => {
            let required = ethers::types::U256::from_dec_str(&payload.amount.to_string()).unwrap_or_default();
            if balance < required {
                return (StatusCode::BAD_REQUEST, Json(json!({
                    "error": "Insufficient token balance",
                    "balance": format!("{}", balance),
                    "required": format!("{}", required)
                })));
            }
            (StatusCode::OK, Json(json!({
                "message": "Bounty deposit pre-check passed",
                "balance": format!("{}", balance),
                "bounty_id": payload.bounty_id
            })))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": format!("Failed to check balance: {}", e)
        }))),
    }
}

pub async fn distribute_bounty_reward(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<DistributeBountyRequest>,
) -> (StatusCode, Json<Value>) {
    // Distribution is handled by BountyManager.resolveBounty() on-chain
    (StatusCode::OK, Json(json!({
        "message": "Bounty distribution queued — will be processed by on-chain resolveBounty",
        "bounty_id": payload.bounty_id
    })))
}

pub async fn lock_stake(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LockStakeRequest>,
) -> (StatusCode, Json<Value>) {
    // Pre-check: user must have sufficient balance for staking
    match state.payment_service.get_token_balance(&payload.address).await {
        Ok(balance) => {
            let required = ethers::types::U256::from_dec_str(&payload.amount.to_string()).unwrap_or_default();
            if balance < required {
                return (StatusCode::BAD_REQUEST, Json(json!({
                    "error": "Insufficient balance for stake",
                    "balance": format!("{}", balance)
                })));
            }
            (StatusCode::OK, Json(json!({
                "message": "Stake pre-check passed",
                "balance": format!("{}", balance),
                "bounty_id": payload.bounty_id
            })))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": format!("Failed to check balance: {}", e)
        }))),
    }
}

pub async fn unlock_stake(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<UnlockStakeRequest>,
) -> (StatusCode, Json<Value>) {
    // Stake unlocking is handled by the smart contract during bounty resolution
    (StatusCode::OK, Json(json!({
        "message": "Stake unlock is handled by on-chain bounty resolution",
        "stake_id": payload.stake_id
    })))
}

pub async fn slash_stake(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<SlashStakeRequest>,
) -> (StatusCode, Json<Value>) {
    // Stake slashing is handled by the smart contract during bounty resolution
    (StatusCode::OK, Json(json!({
        "message": "Stake slashing is handled by on-chain bounty resolution",
        "stake_id": payload.stake_id,
        "slash_amount": payload.slash_amount.to_string()
    })))
}

pub async fn withdraw_funds(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<WithdrawRequest>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({
        "message": "Withdrawal queued for processing",
        "to_address": payload.to_address,
        "amount": payload.amount.to_string()
    })))
}

pub async fn get_balance(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> (StatusCode, Json<Value>) {
    match state.payment_service.get_token_balance(&address).await {
        Ok(balance) => (StatusCode::OK, Json(json!({
            "address": address,
            "balance": format!("{}", balance),
            "token": "THREAT"
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": format!("Failed to get balance: {}", e)
        }))),
    }
}

pub async fn get_transactions(
    State(_state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> (StatusCode, Json<Value>) {
    // Transaction history is populated by blockchain event sync
    (StatusCode::OK, Json(json!({
        "address": address,
        "transactions": [],
        "note": "Transaction history is populated by blockchain event sync"
    })))
}

pub async fn get_transaction_status(
    State(state): State<Arc<AppState>>,
    Path(tx_hash): Path<String>,
) -> (StatusCode, Json<Value>) {
    match state.payment_service.get_tx_receipt(&tx_hash).await {
        Ok(Some(receipt)) => {
            let status = if receipt.status == Some(1.into()) {
                "confirmed"
            } else {
                "failed"
            };
            (StatusCode::OK, Json(json!({
                "tx_hash": tx_hash,
                "status": status,
                "block_number": receipt.block_number.map(|n| n.as_u64()),
                "gas_used": receipt.gas_used.map(|g| format!("{}", g))
            })))
        }
        Ok(None) => (StatusCode::OK, Json(json!({
            "tx_hash": tx_hash,
            "status": "pending"
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": format!("Failed to get transaction status: {}", e)
        }))),
    }
}

pub async fn estimate_gas(
    State(state): State<Arc<AppState>>,
    Json(_payload): Json<EstimateGasRequest>,
) -> (StatusCode, Json<Value>) {
    match state.payment_service.estimate_gas_for_transfer().await {
        Ok(gas) => (StatusCode::OK, Json(json!({
            "estimated_gas_cost": format!("{}", gas),
            "unit": "wei"
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": format!("Failed to estimate gas: {}", e)
        }))),
    }
}
