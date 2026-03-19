use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;

/// Leaderboard entry
#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub user_id: Uuid,
    pub username: Option<String>,
    pub reputation_score: i32,
}

/// Leaderboard query parameters
#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Leaderboard response
#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

/// User reputation response
#[derive(Debug, Serialize)]
pub struct UserReputation {
    pub user_id: Uuid,
    pub username: Option<String>,
    pub reputation_score: i32,
    pub rank: Option<i64>,
}

/// Reputation history entry
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReputationHistoryEntry {
    pub id: Uuid,
    pub event_type: String,
    pub score_change: f64,
    pub new_score: f64,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

/// Reputation history response
#[derive(Debug, Serialize)]
pub struct ReputationHistoryResponse {
    pub entries: Vec<ReputationHistoryEntry>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub current_score: i32,
}

/// DB row for leaderboard
#[derive(sqlx::FromRow)]
struct LeaderboardRow {
    id: Uuid,
    username: Option<String>,
    reputation_score: i32,
}

/// Get global leaderboard
pub async fn get_leaderboard(
    State(state): State<AppState>,
    Query(params): Query<LeaderboardQuery>,
) -> Result<Json<LeaderboardResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = (page.saturating_sub(1)) * limit;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_active = true")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    let rows = sqlx::query_as::<_, LeaderboardRow>(
        "SELECT id, username, reputation_score FROM users WHERE is_active = true
         ORDER BY reputation_score DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("DB error fetching leaderboard: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let base_rank = offset;
    let entries: Vec<LeaderboardEntry> = rows
        .into_iter()
        .enumerate()
        .map(|(i, row)| LeaderboardEntry {
            rank: base_rank + i as u32 + 1,
            user_id: row.id,
            username: row.username,
            reputation_score: row.reputation_score,
        })
        .collect();

    Ok(Json(LeaderboardResponse {
        entries,
        total,
        page,
        limit,
    }))
}

/// Get top analysts (top 10)
pub async fn get_top_analysts(
    State(state): State<AppState>,
) -> Result<Json<Vec<LeaderboardEntry>>, StatusCode> {
    let rows = sqlx::query_as::<_, LeaderboardRow>(
        "SELECT id, username, reputation_score FROM users WHERE is_active = true
         ORDER BY reputation_score DESC LIMIT 10"
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("DB error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let entries: Vec<LeaderboardEntry> = rows
        .into_iter()
        .enumerate()
        .map(|(i, row)| LeaderboardEntry {
            rank: i as u32 + 1,
            user_id: row.id,
            username: row.username,
            reputation_score: row.reputation_score,
        })
        .collect();

    Ok(Json(entries))
}

/// Get user reputation by ID
pub async fn get_user_reputation(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserReputation>, StatusCode> {
    let user = sqlx::query_as::<_, LeaderboardRow>(
        "SELECT id, username, reputation_score FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("DB error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Calculate rank
    let rank: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*) + 1 FROM users WHERE reputation_score > $1 AND is_active = true"
    )
    .bind(user.reputation_score)
    .fetch_optional(state.db.pool())
    .await
    .unwrap_or(None);

    Ok(Json(UserReputation {
        user_id: user.id,
        username: user.username,
        reputation_score: user.reputation_score,
        rank,
    }))
}

/// Get reputation history for a user
pub async fn get_reputation_history(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<LeaderboardQuery>,
) -> Result<Json<ReputationHistoryResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = (page.saturating_sub(1)) * limit;

    // Get current score
    let current_score: i32 = sqlx::query_scalar(
        "SELECT reputation_score FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("DB error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .unwrap_or(0);

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM reputation_history WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let entries = sqlx::query_as::<_, ReputationHistoryEntry>(
        "SELECT id, event_type, score_change::float8, new_score::float8, reason, created_at
         FROM reputation_history WHERE user_id = $1
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(user_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("DB error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ReputationHistoryResponse {
        entries,
        total,
        page,
        limit,
        current_score,
    }))
}

/// List available badges (static)
pub async fn list_available_badges(
    State(_state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    Ok(Json(vec![
        serde_json::json!({"id": "first_analysis", "name": "First Analysis", "description": "Complete your first analysis", "tier": "bronze"}),
        serde_json::json!({"id": "ten_analyses", "name": "Veteran Analyst", "description": "Complete 10 analyses", "tier": "silver"}),
        serde_json::json!({"id": "high_accuracy", "name": "Sharp Eye", "description": "Achieve 90% accuracy over 20 analyses", "tier": "gold"}),
        serde_json::json!({"id": "bounty_hunter", "name": "Bounty Hunter", "description": "Win 5 bounties", "tier": "gold"}),
        serde_json::json!({"id": "top_ten", "name": "Elite", "description": "Reach the top 10 leaderboard", "tier": "platinum"}),
    ]))
}

/// Claim badge
pub async fn claim_badge(
    State(_state): State<AppState>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}
