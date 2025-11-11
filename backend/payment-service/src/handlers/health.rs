use axum::{response::Json, http::StatusCode};
use serde_json::{json, Value};

pub async fn health_check() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "payment-service",
            "version": env!("CARGO_PKG_VERSION")
        }))
    )
}
