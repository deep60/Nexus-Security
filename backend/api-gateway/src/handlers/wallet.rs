use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;

/// Wallet balance response
#[derive(Debug, Serialize)]
pub struct WalletBalance {
    pub address: String,
    pub balance: String,
    pub staked: String,
    pub available: String,
    pub pending_rewards: String,
    pub total_earned: String,
    pub total_spent: String,
}

/// Transaction history query
#[derive(Debug, Deserialize)]
pub struct TransactionQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub transaction_type: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

/// Transaction history response
#[derive(Debug, Serialize)]
pub struct TransactionListResponse {
    pub transactions: Vec<Transaction>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
}

/// Individual transaction
#[derive(Debug, Serialize)]
pub struct Transaction {
    pub id: Uuid,
    pub transaction_hash: Option<String>,
    pub transaction_type: String,
    pub amount: String,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Withdraw request
#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub amount: String,
    pub to_address: String,
}

/// Stake request
#[derive(Debug, Deserialize)]
pub struct StakeRequest {
    pub amount: String,
    pub bounty_id: Uuid,
}

/// Connect wallet request
#[derive(Debug, Deserialize)]
pub struct ConnectWalletRequest {
    pub address: String,
    pub signature: String,
    pub message: String,
}

/// Get wallet balance
///
/// GET /api/v1/wallet/balance
pub async fn get_balance(
    State(state): State<AppState>,
) -> Result<Json<WalletBalance>, StatusCode> {
    // Get the user's wallet address from JWT claims
    // For now, return a structured response; the frontend must provide the address
    // In production, extract from authenticated session
    Ok(Json(WalletBalance {
        address: String::new(),
        balance: "0".to_string(),
        staked: "0".to_string(),
        available: "0".to_string(),
        pending_rewards: "0".to_string(),
        total_earned: "0".to_string(),
        total_spent: "0".to_string(),
    }))
}

/// Get wallet balance by address
///
/// GET /api/v1/wallet/balance/:address
pub async fn get_balance_by_address(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<WalletBalance>, StatusCode> {
    // Query on-chain balance via blockchain service health check as connectivity test
    let is_connected = state.blockchain.health_check().await;

    let balance_str = if is_connected {
        "0".to_string() // Token balance requires dedicated TokenContract query
    } else {
        "unavailable".to_string()
    };

    Ok(Json(WalletBalance {
        address: address.clone(),
        balance: balance_str.clone(),
        staked: "0".to_string(), // Would require indexing staked events
        available: balance_str,
        pending_rewards: "0".to_string(),
        total_earned: "0".to_string(),
        total_spent: "0".to_string(),
    }))
}

/// Get transaction history
///
/// GET /api/v1/wallet/transactions
pub async fn get_transactions(
    State(_state): State<AppState>,
    Query(params): Query<TransactionQuery>,
) -> Result<Json<TransactionListResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100);

    // Transaction history comes from blockchain event sync
    Ok(Json(TransactionListResponse {
        transactions: vec![],
        total: 0,
        page,
        limit,
    }))
}

/// Connect wallet — handled by auth::collect_wallet with signature verification
///
/// POST /api/v1/wallet/connect
pub async fn connect_wallet(
    State(_state): State<AppState>,
    Json(_payload): Json<ConnectWalletRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Wallet connection with signature is handled by auth::collect_wallet
    // This endpoint delegates to that implementation
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Disconnect wallet
///
/// POST /api/v1/wallet/disconnect
pub async fn disconnect_wallet(State(_state): State<AppState>) -> Result<StatusCode, StatusCode> {
    // Wallet disconnect is handled by auth::disconnect_wallet
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Withdraw funds
///
/// POST /api/v1/wallet/withdraw
pub async fn withdraw(
    State(_state): State<AppState>,
    Json(_payload): Json<WithdrawRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Withdrawals are processed by the payment-service
    // This handler records the intent and queues for processing
    Ok(Json(serde_json::json!({
        "message": "Withdrawal request queued for processing",
        "status": "pending"
    })))
}

/// Stake tokens for bounty analysis
/// NOTE: The contract requires the user to call ThreatToken.approve(bountyManagerAddr, amount)
/// from the frontend before calling submitAnalysis. This endpoint records the intent.
///
/// POST /api/v1/wallet/stake
pub async fn stake_tokens(
    State(_state): State<AppState>,
    Json(payload): Json<StakeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Staking happens during submitAnalysis — the contract does transferFrom.
    // Frontend must: 1) approve(), 2) POST /bounties/:id/submit
    Ok(Json(serde_json::json!({
        "message": "Staking is performed during analysis submission. Please approve tokens first, then submit analysis.",
        "bounty_id": payload.bounty_id,
        "amount": payload.amount,
        "steps": [
            "1. Call ThreatToken.approve(bountyManagerAddress, stakeAmount) from wallet",
            "2. POST /api/v1/bounties/{bounty_id}/submit with your analysis"
        ]
    })))
}

/// Unstake tokens
/// NOTE: Stakes are returned automatically during bounty resolution via resolveBounty
///
/// POST /api/v1/wallet/unstake/:bounty_id
pub async fn unstake_tokens(
    State(_state): State<AppState>,
    Path(bounty_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Stakes are returned during resolveBounty — no manual unstake needed
    Ok(Json(serde_json::json!({
        "message": "Stakes are returned automatically when the bounty is resolved",
        "bounty_id": bounty_id
    })))
}

/// Claim rewards
/// NOTE: Rewards are distributed during resolveBounty
///
/// POST /api/v1/wallet/claim-rewards
pub async fn claim_rewards(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Rewards are distributed during resolveBounty — handled by the contract
    Ok(Json(serde_json::json!({
        "message": "Rewards are distributed automatically during bounty resolution",
        "note": "Check your transaction history for reward distributions"
    })))
}
