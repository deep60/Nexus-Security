use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;

fn parse_uuid(s: &str) -> Result<Uuid, (StatusCode, Json<Value>)> {
    Uuid::parse_str(s)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({ "error": "invalid id" }))))
}

/// Recompute ranks (and re-evaluate this user's badges).
pub async fn recalculate_reputation(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let user_id = match parse_uuid(&user_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    if let Err(e) = state.reputation_service.recompute_ranks().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        );
    }
    match state.reputation_service.evaluate_badges(user_id).await {
        Ok(awarded) => (
            StatusCode::OK,
            Json(json!({ "message": "recalculated", "badges_awarded": awarded })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn reset_reputation(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let user_id = match parse_uuid(&user_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match state.reputation_service.reset(user_id).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "message": "reputation reset" }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct AwardBadgeRequest {
    pub user_id: Uuid,
}

/// Trigger badge evaluation for a user (badges are criteria-based).
pub async fn award_badge(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AwardBadgeRequest>,
) -> (StatusCode, Json<Value>) {
    match state.reputation_service.evaluate_badges(payload.user_id).await {
        Ok(awarded) => (
            StatusCode::OK,
            Json(json!({ "message": "evaluated", "badges_awarded": awarded })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
