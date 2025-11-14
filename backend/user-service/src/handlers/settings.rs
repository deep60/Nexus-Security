use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::Claims;
use crate::handlers::auth::{AppError, MessageResponse};
use crate::models::*;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub email_notifications: Option<bool>,
    pub push_notifications: Option<bool>,
    pub webhook_notifications: Option<bool>,
    pub privacy_public_profile: Option<bool>,
    pub privacy_show_email: Option<bool>,
    pub privacy_show_stats: Option<bool>,
    pub language: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Enable2FAResponse {
    pub secret: String,
    pub qr_code_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct Disable2FARequest {
    pub code: String,
}

/// Get user settings
pub async fn get_settings(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<UserSettings>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let settings = state.user_service.get_settings(user_id).await?;

    Ok(Json(settings))
}

/// Update user settings
pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<UserSettings>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    // Build SQL update query dynamically
    let settings = sqlx::query_as::<_, UserSettings>(
        r#"
        UPDATE user_settings
        SET email_notifications = COALESCE($1, email_notifications),
            push_notifications = COALESCE($2, push_notifications),
            webhook_notifications = COALESCE($3, webhook_notifications),
            privacy_public_profile = COALESCE($4, privacy_public_profile),
            privacy_show_email = COALESCE($5, privacy_show_email),
            privacy_show_stats = COALESCE($6, privacy_show_stats),
            language = COALESCE($7, language),
            timezone = COALESCE($8, timezone),
            updated_at = NOW()
        WHERE user_id = $9
        RETURNING *
        "#,
    )
    .bind(req.email_notifications)
    .bind(req.push_notifications)
    .bind(req.webhook_notifications)
    .bind(req.privacy_public_profile)
    .bind(req.privacy_show_email)
    .bind(req.privacy_show_stats)
    .bind(&req.language)
    .bind(&req.timezone)
    .bind(user_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(Json(settings))
}

/// Change password
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    state.user_service.change_password(user_id, req).await?;

    Ok(Json(MessageResponse {
        message: "Password changed successfully".to_string(),
    }))
}

/// Enable 2FA
pub async fn enable_2fa(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Enable2FAResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let secret = state.user_service.enable_2fa(user_id).await?;

    // Generate QR code URL for authenticator apps
    let qr_code_url = format!(
        "otpauth://totp/NexusSecurity:{}?secret={}&issuer=NexusSecurity",
        claims.email, secret
    );

    Ok(Json(Enable2FAResponse { secret, qr_code_url }))
}

/// Verify and activate 2FA
pub async fn verify_2fa(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<Verify2FARequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    state.user_service.verify_2fa(user_id, &req.code).await?;

    Ok(Json(MessageResponse {
        message: "2FA enabled successfully".to_string(),
    }))
}

/// Disable 2FA
pub async fn disable_2fa(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<Disable2FARequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    state.user_service.disable_2fa(user_id, &req.code).await?;

    Ok(Json(MessageResponse {
        message: "2FA disabled successfully".to_string(),
    }))
}
