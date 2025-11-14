use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::Claims;
use crate::handlers::auth::{AppError, MessageResponse};
use crate::models::*;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct SubmitKycResponse {
    pub kyc_id: Uuid,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadDocumentsRequest {
    pub document_front_url: String,
    pub document_back_url: Option<String>,
    pub selfie_url: String,
}

/// Submit KYC verification request
pub async fn submit_kyc(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<SubmitKycRequest>,
) -> Result<Json<SubmitKycResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let kyc_id = state.user_service.submit_kyc(user_id, req).await?;

    Ok(Json(SubmitKycResponse {
        kyc_id,
        message: "KYC verification submitted successfully. Please upload required documents.".to_string(),
    }))
}

/// Get KYC status
pub async fn get_kyc_status(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Option<KycVerification>>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let kyc = state.user_service.get_kyc_status(user_id).await?;

    Ok(Json(kyc))
}

/// Upload KYC documents
pub async fn upload_documents(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UploadDocumentsRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    // Get latest KYC submission
    let kyc = state
        .user_service
        .get_kyc_status(user_id)
        .await?
        .ok_or(AppError::UserError(UserError::NotFound))?;

    // Update document URLs
    sqlx::query(
        r#"
        UPDATE kyc_verifications
        SET document_front_url = $1,
            document_back_url = $2,
            selfie_url = $3,
            status = 'under_review'
        WHERE id = $4
        "#,
    )
    .bind(&req.document_front_url)
    .bind(&req.document_back_url)
    .bind(&req.selfie_url)
    .bind(kyc.id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

    // Update user KYC status
    sqlx::query("UPDATE users SET kyc_status = 'under_review' WHERE id = $1")
        .bind(user_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(Json(MessageResponse {
        message: "Documents uploaded successfully. Your KYC is now under review.".to_string(),
    }))
}
