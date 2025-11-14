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
    // TODO: Implement file upload logic with S3 or similar
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Avatar upload endpoint - implementation pending",
        "user_id": user_id,
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
