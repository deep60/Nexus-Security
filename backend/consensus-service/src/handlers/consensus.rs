use crate::models::*;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

/// Map a ConsensusError to an HTTP status + JSON body.
fn err_response(e: ConsensusError) -> (StatusCode, Json<Value>) {
    let status = match &e {
        ConsensusError::NotFound(_) => StatusCode::NOT_FOUND,
        ConsensusError::ValidationError(_) => StatusCode::BAD_REQUEST,
        ConsensusError::InsufficientSubmissions { .. } => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, Json(json!({ "error": e.to_string() })))
}

fn parse_uuid(s: &str) -> Result<Uuid, (StatusCode, Json<Value>)> {
    Uuid::parse_str(s).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid id" })),
        )
    })
}

pub async fn get_bounty_consensus(
    State(state): State<Arc<AppState>>,
    Path(bounty_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let bounty_id = match parse_uuid(&bounty_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match state.consensus_service.get_stored(bounty_id).await {
        Ok(Some(resp)) => (StatusCode::OK, Json(json!(resp))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "no consensus for bounty" })),
        ),
        Err(e) => err_response(e),
    }
}

pub async fn calculate_consensus(
    State(state): State<Arc<AppState>>,
    Path(bounty_id): Path<String>,
    Json(payload): Json<ConsensusCalculationRequest>,
) -> (StatusCode, Json<Value>) {
    let bounty_id = match parse_uuid(&bounty_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let _ = payload.force_recalculate; // recalculation is always fresh
    match state
        .consensus_service
        .calculate_and_store(bounty_id, false)
        .await
    {
        Ok(resp) => (StatusCode::OK, Json(json!(resp))),
        Err(e) => err_response(e),
    }
}

pub async fn get_submission_consensus(
    State(state): State<Arc<AppState>>,
    Path(submission_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    // A submission belongs to a bounty; we treat the path id as bounty id here
    // since votes are aggregated per bounty.
    let id = match parse_uuid(&submission_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match state.consensus_service.get_stored(id).await {
        Ok(Some(resp)) => (StatusCode::OK, Json(json!(resp))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "no consensus found" })),
        ),
        Err(e) => err_response(e),
    }
}

pub async fn get_consensus_stats(
    State(state): State<Arc<AppState>>,
    Path(bounty_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let bounty_id = match parse_uuid(&bounty_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match state.consensus_service.load_votes(bounty_id).await {
        Ok(votes) => (
            StatusCode::OK,
            Json(json!({
                "bounty_id": bounty_id,
                "total_votes": votes.len(),
                "engines": votes.iter().map(|v| v.engine_id.clone()).collect::<Vec<_>>(),
            })),
        ),
        Err(e) => err_response(e),
    }
}
