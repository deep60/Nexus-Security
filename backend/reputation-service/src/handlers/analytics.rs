use crate::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn get_reputation_trends(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"trends": []})))
}

pub async fn get_score_distribution(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"distribution": []})))
}

pub async fn get_accuracy_stats(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"stats": {}})))
}
