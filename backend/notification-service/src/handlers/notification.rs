use axum::{extract::State, response::Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::AppState;
use shared::messaging::event_types::{
    NexusEvent, UserRegisteredEvent, PaymentProcessedEvent,
    NotificationPayload, NotificationChannel, NotificationPriority, PaymentType,
};

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
        "user_registered" => {
            NexusEvent::UserRegistered(UserRegisteredEvent {
                user_id: test_user_id,
                username: "test_user".to_string(),
                email: req.email.clone(),
                ethereum_address: "0x0000000000000000000000000000000000000000".to_string(),
                registered_at: Utc::now(),
            })
        }
        "payment_processed" => {
            NexusEvent::PaymentProcessed(PaymentProcessedEvent {
                bounty_id: Uuid::new_v4(),
                recipient_id: test_user_id,
                amount: 1000.0,
                tx_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
                payment_type: PaymentType::BountyReward,
                processed_at: Utc::now(),
            })
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(TestEmailResponse {
                    success: false,
                    message: "Invalid event_type. Use 'user_registered' or 'payment_processed'".to_string(),
                })
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
    match state.notification_manager.send_notification(&notification_payload).await {
        Ok(_) => {
            (
                StatusCode::OK,
                Json(TestEmailResponse {
                    success: true,
                    message: format!("Test email sent to {} for event: {}", req.email, req.event_type),
                })
            )
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TestEmailResponse {
                    success: false,
                    message: format!("Failed to send email: {}", e),
                })
            )
        }
    }
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
