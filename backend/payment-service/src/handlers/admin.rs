use axum::{extract::{State, Path}, response::Json, http::StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::AppState;

pub async fn get_pending_payments(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"payments": []})))
}

pub async fn get_failed_payments(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"payments": []})))
}

pub async fn retry_payment(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"message": "Payment retry initiated"})))
}

pub async fn get_treasury_balance(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"balance": "0"})))
}
