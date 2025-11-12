use axum::{extract::{State, Path}, response::Json, http::StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::AppState;
use crate::models::*;

pub async fn create_dispute(
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<CreateDisputeRequest>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"message": "Dispute created"})))
}

pub async fn get_dispute(
    State(_state): State<Arc<AppState>>,
    Path(_dispute_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"dispute": {}})))
}

pub async fn resolve_dispute(
    State(_state): State<Arc<AppState>>,
    Path(_dispute_id): Path<String>,
    Json(_payload): Json<ResolveDisputeRequest>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"message": "Dispute resolved"})))
}

pub async fn get_bounty_disputes(
    State(_state): State<Arc<AppState>>,
    Path(_bounty_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"disputes": []})))
}
