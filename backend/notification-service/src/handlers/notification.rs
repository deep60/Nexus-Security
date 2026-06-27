use crate::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use shared::messaging::event_types::{
    NotificationChannel, NotificationPayload, NotificationPriority, PaymentProcessedEvent,
    PaymentType, UserRegisteredEvent, VerdyxEvent,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct TestEmailRequest {
    pub email: String,
    pub event_type: String, // "user_registered" or "payment_processed"
}

#[derive(Debug, Serialize)]
pub struct TestEmailResponse {
    pub success: bool,
    pub message: String,
}

/// Test endpoint to send email notifications
pub async fn send_notification(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TestEmailRequest>,
) -> (StatusCode, Json<TestEmailResponse>) {
    let test_user_id = Uuid::new_v4();

    // Create test event based on request
    let event = match req.event_type.as_str() {
        "user_registered" => VerdyxEvent::UserRegistered(UserRegisteredEvent {
            user_id: test_user_id,
            username: "test_user".to_string(),
            email: req.email.clone(),
            ethereum_address: "0x0000000000000000000000000000000000000000".to_string(),
            registered_at: Utc::now(),
        }),
        "payment_processed" => VerdyxEvent::PaymentProcessed(PaymentProcessedEvent {
            bounty_id: Uuid::new_v4(),
            recipient_id: test_user_id,
            amount: 1000u128,
            tx_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                .to_string(),
            payment_type: PaymentType::BountyReward,
            processed_at: Utc::now(),
        }),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(TestEmailResponse {
                    success: false,
                    message: "Invalid event_type. Use 'user_registered' or 'payment_processed'"
                        .to_string(),
                }),
            );
        }
    };

    // Create notification payload
    let notification_payload = NotificationPayload {
        notification_id: Uuid::new_v4(),
        user_id: test_user_id,
        channels: vec![NotificationChannel::Email],
        event: event.clone(),
        priority: NotificationPriority::Normal,
        created_at: Utc::now(),
    };

    // Send notification
    match state
        .notification_manager
        .send_notification(&notification_payload)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(TestEmailResponse {
                success: true,
                message: format!(
                    "Test email sent to {} for event: {}",
                    req.email, req.event_type
                ),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TestEmailResponse {
                success: false,
                message: format!("Failed to send email: {}", e),
            }),
        ),
    }
}

pub async fn get_notification_history(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    let user_id = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok());

    let Some(user_id) = user_id else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "missing x-user-id" })),
        );
    };

    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            Value,
            String,
            Option<chrono::DateTime<Utc>>,
        ),
    >(
        r#"
        SELECT id, channel, event_type, payload, status, sent_at
        FROM notification_history
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db_pool)
    .await;

    match rows {
        Ok(rows) => {
            let items: Vec<Value> = rows
                .into_iter()
                .map(|(id, channel, event_type, payload, status, sent_at)| {
                    json!({
                        "id": id,
                        "channel": channel,
                        "event_type": event_type,
                        "payload": payload,
                        "status": status,
                        "sent_at": sent_at,
                    })
                })
                .collect();
            (StatusCode::OK, Json(json!({ "notifications": items })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn retry_notification(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"message": "Retry queued"})))
}
