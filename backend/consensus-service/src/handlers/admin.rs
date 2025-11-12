use axum::{extract::{State, Path}, response::Json, http::StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::AppState;

pub async fn recalculate_consensus(
    State(_state): State<Arc<AppState>>,
    Path(_bounty_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"message": "Recalculation started"})))
}

pub async fn override_consensus(
    State(_state): State<Arc<AppState>>,
    Path(_bounty_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"message": "Consensus overridden"})))
}
