use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;

fn parse_uuid(s: &str) -> Result<Uuid, (StatusCode, Json<Value>)> {
    Uuid::parse_str(s)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({ "error": "invalid id" }))))
}

/// Force a fresh consensus calculation for a bounty (admin-triggered).
pub async fn recalculate_consensus(
    State(state): State<Arc<AppState>>,
    Path(bounty_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let bounty_id = match parse_uuid(&bounty_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match state
        .consensus_service
        .calculate_and_store(bounty_id, false)
        .await
    {
        Ok(resp) => (StatusCode::OK, Json(json!(resp))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

/// Admin override: finalize a bounty consensus immediately with current votes.
pub async fn override_consensus(
    State(state): State<Arc<AppState>>,
    Path(bounty_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let bounty_id = match parse_uuid(&bounty_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match state
        .consensus_service
        .calculate_and_store(bounty_id, true)
        .await
    {
        Ok(resp) => (
            StatusCode::OK,
            Json(json!({ "message": "consensus finalized", "result": resp })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
