use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn validate_submission(
    State(_state): State<Arc<AppState>>,
    Path(_submission_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"valid": true})))
}

pub async fn batch_validate(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"results": []})))
}
