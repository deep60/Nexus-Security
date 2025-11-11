use axum::{extract::State, response::Json, http::StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::AppState;

pub async fn register_webhook(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"message": "Webhook registered"})))
}

pub async fn unregister_webhook(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"message": "Webhook unregistered"})))
}
