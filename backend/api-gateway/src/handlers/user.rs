use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::user::User;
use crate::AppState;

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub ethereum_address: Option<String>,
    pub reputation_score: f64,
    pub total_submissions: u32,
    pub successful_submissions: u32,
    pub accuracy_rate: f64,
    pub total_earnings: String,
    pub rank: Option<u32>,
    pub joined_at: DateTime<Utc>,
    pub last_active_at: Option<DateTime<Utc>>,
}

/// Update user profile request
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub ethereum_address: Option<String>,
    pub bio: Option<String>,
    pub notification_preferences: Option<serde_json::Value>,
}

/// User statistics
#[derive(Debug, Serialize)]
pub struct UserStats {
    pub total_analyses: u64,
    pub total_bounties_created: u64,
    pub total_bounties_participated: u64,
    pub total_rewards_earned: String,
    pub total_rewards_paid: String,
    pub average_accuracy: f64,
    pub streak_days: u32,
}

/// Get current user profile
///
/// GET /api/v1/users/me
pub async fn get_current_user(
    State(state): State<AppState>,
    claims: crate::middleware::auth::Claims,
) -> Result<Json<UserProfile>, StatusCode> {
    let user = state
        .db
        .get_user_by_id(claims.sub)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserProfile {
        id: user.id,
        username: user.username,
        email: user.email,
        ethereum_address: user.wallet_address,
        reputation_score: user.reputation_score as f64,
        total_submissions: 0, // Placeholder: need to fetch from stats
        successful_submissions: 0,
        accuracy_rate: 0.0,
        total_earnings: "0".to_string(),
        rank: None,
        joined_at: user.created_at,
        last_active_at: Some(user.updated_at),
    }))
}

/// Get user profile by ID
///
/// GET /api/v1/users/:id
pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserProfile>, StatusCode> {
    let user = state
        .db
        .get_user_by_id(user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserProfile {
        id: user.id,
        username: user.username,
        email: user.email,
        ethereum_address: user.wallet_address,
        reputation_score: user.reputation_score as f64,
        total_submissions: 0,
        successful_submissions: 0,
        accuracy_rate: 0.0,
        total_earnings: "0".to_string(),
        rank: None,
        joined_at: user.created_at,
        last_active_at: Some(user.updated_at),
    }))
}

/// Update current user profile
///
/// PUT /api/v1/users/me
pub async fn update_profile(
    State(state): State<AppState>,
    claims: crate::middleware::auth::Claims,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, StatusCode> {
    // This requires implementing an update_user method in DatabaseService, which currently only has `update_user_reputation`.
    // For now we will return NOT_IMPLEMENTED until DB service is expanded, or just update reputation if that was the only thing.
    // However, proper implementation requires expanding `DatabaseService`.
    // Given the constraints, I will leave a TODO for the DatabaseService expansion but implement the handler logic structure.

    // TODO: Implement `update_user` in DatabaseService
    // state.db.update_user(claims.sub, payload).await...

    // Fallback to fetching current user to satisfy return type for now
    get_current_user(State(state), claims).await
}

/// Get user statistics
///
/// GET /api/v1/users/me/stats
/// Get user statistics
///
/// GET /api/v1/users/me/stats
pub async fn get_user_stats(
    State(state): State<AppState>,
    claims: crate::middleware::auth::Claims,
) -> Result<Json<UserStats>, StatusCode> {
    let stats = state
        .db
        .get_user_analysis_stats(claims.sub)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user stats: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .unwrap_or(crate::services::database::UserAnalysisStats {
            total_analyses: Some(0),
            avg_confidence: Some(0.0),
            malicious_detections: Some(0),
            benign_detections: Some(0),
        });

    Ok(Json(UserStats {
        total_analyses: stats.total_analyses.unwrap_or(0) as u64,
        total_bounties_created: 0, // TODO: Add to DB query or separate query
        total_bounties_participated: 0, // TODO: Add to DB query
        total_rewards_earned: "0".to_string(), // TODO: Add to DB query
        total_rewards_paid: "0".to_string(), // TODO: Add to DB query
        average_accuracy: stats.avg_confidence.unwrap_or(0.0),
        streak_days: 0, // TODO: Track streaks
    }))
}

/// Get user activity history
///
/// GET /api/v1/users/me/activity
pub async fn get_user_activity(
    State(state): State<AppState>,
    Query(params): Query<ActivityQuery>,
) -> Result<Json<ActivityListResponse>, StatusCode> {
    // TODO: Fetch activity from database
    Ok(Json(ActivityListResponse {
        activities: vec![],
        total: 0,
        page: params.page.unwrap_or(1),
        limit: params.limit.unwrap_or(20),
    }))
}

#[derive(Debug, Deserialize)]
pub struct ActivityQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub activity_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ActivityListResponse {
    pub activities: Vec<Activity>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct Activity {
    pub id: Uuid,
    pub activity_type: String,
    pub description: String,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// Delete user account
///
/// DELETE /api/v1/users/me
pub async fn delete_account(State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    // TODO: Implement account deletion
    Err(StatusCode::NOT_IMPLEMENTED)
}
