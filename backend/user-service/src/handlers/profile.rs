use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::Claims;
use crate::handlers::auth::AppError;
use crate::models::*;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub user: UserPublic,
    pub profile: UserProfile,
}

/// Get current user's profile
pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ProfileResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let user = state.user_service.get_user_by_id(user_id).await?;
    let profile = state.user_service.get_profile(user_id).await?;

    Ok(Json(ProfileResponse {
        user: user.into(),
        profile,
    }))
}

/// Update current user's profile
pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let profile = state.user_service.update_profile(user_id, req).await?;

    Ok(Json(profile))
}

/// Upload avatar
pub async fn upload_avatar(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Avatar upload requires multipart form handling and S3/object storage.
    // Integration path: add axum::extract::Multipart, upload to S3Client,
    // then store the URL in the user's profile via user_service.update_avatar().
    tracing::info!(user_id = %user_id, "Avatar upload requested — S3 integration pending");

    Ok(Json(serde_json::json!({
        "message": "Avatar upload requires S3 object storage integration",
        "user_id": user_id,
        "status": "not_implemented",
    })))
}

/// Get public profile of any user
pub async fn get_user_profile(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ProfileResponse>, AppError> {
    let user = state.user_service.get_user_by_id(user_id).await?;
    let profile = state.user_service.get_profile(user_id).await?;

    // Check privacy settings
    let settings = state.user_service.get_settings(user_id).await?;

    if !settings.privacy_public_profile {
        return Err(AppError::Unauthorized(
            "This profile is private".to_string(),
        ));
    }

    Ok(Json(ProfileResponse {
        user: user.into(),
        profile,
    }))
}
