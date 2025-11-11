use axum::{extract::State, response::Json, http::StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::AppState;

pub async fn send_notification(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"message": "Notification sent"})))
}

pub async fn get_notification_history(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"notifications": []})))
}

pub async fn retry_notification(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"message": "Retry queued"})))
}
