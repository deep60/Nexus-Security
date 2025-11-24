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
    // TODO: Extract user from JWT claims
) -> Result<Json<UserProfile>, StatusCode> {
    // TODO: Fetch user from database
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Get user profile by ID
///
/// GET /api/v1/users/:id
pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserProfile>, StatusCode> {
    // TODO: Fetch user from database
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Update current user profile
///
/// PUT /api/v1/users/me
pub async fn update_profile(
    State(state): State<AppState>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, StatusCode> {
    // TODO: Update user in database
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Get user statistics
///
/// GET /api/v1/users/me/stats
pub async fn get_user_stats(State(state): State<AppState>) -> Result<Json<UserStats>, StatusCode> {
    // TODO: Calculate user statistics
    Ok(Json(UserStats {
        total_analyses: 0,
        total_bounties_created: 0,
        total_bounties_participated: 0,
        total_rewards_earned: "0".to_string(),
        total_rewards_paid: "0".to_string(),
        average_accuracy: 0.0,
        streak_days: 0,
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
