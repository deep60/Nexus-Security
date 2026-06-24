use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;

fn user_from_headers(headers: &HeaderMap) -> Option<Uuid> {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
}

#[derive(Debug, Deserialize)]
pub struct RegisterWebhookRequest {
    pub webhook_url: String,
    pub webhook_secret: Option<String>,
}

pub async fn register_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RegisterWebhookRequest>,
) -> (StatusCode, Json<Value>) {
    let Some(user_id) = user_from_headers(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "missing x-user-id" })),
        );
    };

    if !req.webhook_url.starts_with("https://") {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "webhook_url must be https" })),
        );
    }

    let res = sqlx::query(
        r#"
        INSERT INTO notification_preferences
            (user_id, webhook_enabled, webhook_url, webhook_secret)
        VALUES ($1, true, $2, $3)
        ON CONFLICT (user_id) DO UPDATE SET
            webhook_enabled = true,
            webhook_url = $2,
            webhook_secret = $3,
            updated_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(&req.webhook_url)
    .bind(&req.webhook_secret)
    .execute(&state.db_pool)
    .await;

    match res {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "webhook registered" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn unregister_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let Some(user_id) = user_from_headers(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "missing x-user-id" })),
        );
    };

    let res = sqlx::query(
        r#"
        UPDATE notification_preferences
        SET webhook_enabled = false, webhook_url = NULL, webhook_secret = NULL, updated_at = NOW()
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(&state.db_pool)
    .await;

    match res {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "webhook unregistered" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
