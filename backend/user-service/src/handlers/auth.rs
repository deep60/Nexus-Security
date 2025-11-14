use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::Claims;
use crate::models::*;
use crate::AppState;

// ============= Request/Response Types =============

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyWalletRequest {
    pub address: String,
    pub signature: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

// ============= Handlers =============

/// Register a new user
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.user_service.register(req).await?;
    Ok(Json(response))
}

/// Login user
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.user_service.login(req).await?;
    Ok(Json(response))
}

/// Logout user
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<MessageResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    state.user_service.logout(user_id).await?;

    Ok(Json(MessageResponse {
        message: "Successfully logged out".to_string(),
    }))
}

/// Refresh access token
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.user_service.refresh_token(&req.refresh_token).await?;
    Ok(Json(response))
}

/// Verify email
pub async fn verify_email(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<VerifyEmailRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    state.user_service.verify_email(user_id, &req.token).await?;

    Ok(Json(MessageResponse {
        message: "Email verified successfully".to_string(),
    }))
}

/// Forgot password - send reset email
pub async fn forgot_password(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    // Generate reset token
    // TODO: Implement password reset email via notification service

    Ok(Json(MessageResponse {
        message: "Password reset email sent".to_string(),
    }))
}

/// Reset password with token
pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    // TODO: Implement password reset logic

    Ok(Json(MessageResponse {
        message: "Password reset successfully".to_string(),
    }))
}

/// Verify wallet signature and link to account
pub async fn verify_wallet(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<VerifyWalletRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    state
        .user_service
        .link_wallet(user_id, &req.address, &req.signature, &req.message)
        .await?;

    Ok(Json(MessageResponse {
        message: "Wallet verified and linked successfully".to_string(),
    }))
}

// ============= Additional Types =============

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

// ============= Error Handling =============

#[derive(Debug)]
pub enum AppError {
    UserError(UserError),
    Unauthorized(String),
    InternalError(String),
}

impl From<UserError> for AppError {
    fn from(err: UserError) -> Self {
        AppError::UserError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::UserError(UserError::ValidationError(msg)) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::UserError(UserError::AuthenticationError(msg)) => {
                (StatusCode::UNAUTHORIZED, msg)
            }
            AppError::UserError(UserError::Unauthorized(msg)) => (StatusCode::FORBIDDEN, msg),
            AppError::UserError(UserError::NotFound) => {
                (StatusCode::NOT_FOUND, "Resource not found".to_string())
            }
            AppError::UserError(UserError::AlreadyExists) => {
                (StatusCode::CONFLICT, "Resource already exists".to_string())
            }
            AppError::UserError(UserError::DatabaseError(msg)) => {
                tracing::error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::UserError(UserError::InvalidToken) => {
                (StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string())
            }
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
