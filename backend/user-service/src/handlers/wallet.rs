use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::Claims;
use crate::handlers::auth::{AppError, MessageResponse};
use crate::models::*;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct LinkWalletRequest {
    pub address: String,
    pub signature: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct WalletInfo {
    pub address: String,
    pub linked_at: chrono::DateTime<chrono::Utc>,
}

/// Link Ethereum wallet
pub async fn link_wallet(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<LinkWalletRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    state
        .user_service
        .link_wallet(user_id, &req.address, &req.signature, &req.message)
        .await?;

    Ok(Json(MessageResponse {
        message: "Wallet linked successfully".to_string(),
    }))
}

/// Unlink Ethereum wallet
pub async fn unlink_wallet(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<MessageResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    // Remove wallet address
    sqlx::query("UPDATE users SET ethereum_address = NULL WHERE id = $1")
        .bind(user_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(Json(MessageResponse {
        message: "Wallet unlinked successfully".to_string(),
    }))
}

/// List linked wallets
pub async fn list_wallets(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<WalletInfo>>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let user = state.user_service.get_user_by_id(user_id).await?;

    let wallets = if let Some(address) = user.ethereum_address {
        vec![WalletInfo {
            address,
            linked_at: user.updated_at,
        }]
    } else {
        vec![]
    };

    Ok(Json(wallets))
}
