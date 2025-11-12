use axum::{extract::{State, Path}, response::Json, http::StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::AppState;
use crate::models::*;

pub async fn get_bounty_consensus(
    State(_state): State<Arc<AppState>>,
    Path(_bounty_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"verdict": "malicious"})))
}

pub async fn calculate_consensus(
    State(_state): State<Arc<AppState>>,
    Path(_bounty_id): Path<String>,
    Json(_payload): Json<ConsensusCalculationRequest>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"message": "Consensus calculated"})))
}

pub async fn get_submission_consensus(
    State(_state): State<Arc<AppState>>,
    Path(_submission_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"verdict": "malicious"})))
}

pub async fn get_consensus_stats(
    State(_state): State<Arc<AppState>>,
    Path(_bounty_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"stats": {}})))
}
