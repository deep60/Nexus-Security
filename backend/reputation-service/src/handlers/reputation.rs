use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

fn err_response(e: ReputationError) -> (StatusCode, Json<Value>) {
    let status = match &e {
        ReputationError::NotFound(_) => StatusCode::NOT_FOUND,
        ReputationError::ValidationError(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, Json(json!({ "error": e.to_string() })))
}

fn parse_uuid(s: &str) -> Result<Uuid, (StatusCode, Json<Value>)> {
    Uuid::parse_str(s)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({ "error": "invalid id" }))))
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub limit: Option<i64>,
}

pub async fn get_user_reputation(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let user_id = match parse_uuid(&user_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match state.reputation_service.get_or_create(user_id).await {
        Ok(rep) => (StatusCode::OK, Json(json!(rep))),
        Err(e) => err_response(e),
    }
}

pub async fn get_reputation_history(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Query(q): Query<HistoryQuery>,
) -> (StatusCode, Json<Value>) {
    let user_id = match parse_uuid(&user_id) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let limit = q.limit.unwrap_or(50).clamp(1, 500);

    match state.reputation_service.get_history(user_id, limit).await {
        Ok(history) => (StatusCode::OK, Json(json!({ "history": history }))),
        Err(e) => err_response(e),
    }
}

pub async fn update_reputation(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(mut payload): Json<ReputationUpdateRequest>,
) -> (StatusCode, Json<Value>) {
    let user_id = match parse_uuid(&user_id) {
        Ok(id) => id,
        Err(e) => return e,
    };
    // Path id is authoritative.
    payload.user_id = user_id;

    match state.reputation_service.apply_update(&payload).await {
        Ok(rep) => (StatusCode::OK, Json(json!(rep))),
        Err(e) => err_response(e),
    }
}

pub async fn get_engine_reputation(
    State(state): State<Arc<AppState>>,
    Path(engine_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    // Engines are keyed by their owning user's UUID in this model.
    let user_id = match parse_uuid(&engine_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match state.reputation_service.get(user_id).await {
        Ok(Some(rep)) => (
            StatusCode::OK,
            Json(json!({
                "engine_id": engine_id,
                "reputation_score": rep.current_score,
                "accuracy_rate": rep.accuracy_rate,
                "total_submissions": rep.total_submissions,
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "engine not found" })),
        ),
        Err(e) => err_response(e),
    }
}

pub async fn get_leaderboard(
    State(state): State<Arc<AppState>>,
    Query(q): Query<LeaderboardQuery>,
) -> (StatusCode, Json<Value>) {
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    match state.reputation_service.get_leaderboard(limit).await {
        Ok(board) => (StatusCode::OK, Json(json!({ "leaderboard": board }))),
        Err(e) => err_response(e),
    }
}

pub async fn get_user_badges(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let user_id = match parse_uuid(&user_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match state.reputation_service.get_badges(user_id).await {
        Ok(badges) => (StatusCode::OK, Json(json!({ "badges": badges }))),
        Err(e) => err_response(e),
    }
}
