use axum::{extract::State, response::Json, http::StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::AppState;

pub async fn get_preferences(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"preferences": {}})))
}

pub async fn update_preferences(
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"message": "Preferences updated"})))
}
