use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};
use chrono::{DateTime, Utc};

use crate::models::{NotificationChannel, NotificationError, NotificationResult};
use shared::messaging::event_types::{NexusEvent, NotificationPayload};

/// Webhook notification channel implementation
/// Sends HTTP POST requests to registered webhook URLs
pub struct WebhookChannel {
    http_client: Client,
    signing_secret: Option<String>,
}

impl WebhookChannel {
    /// Create a new webhook channel
    pub fn new(signing_secret: Option<String>) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("NexusSecurity-Webhook/1.0")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            http_client,
            signing_secret,
        }
    }

    /// Generate HMAC signature for webhook payload
    fn generate_signature(&self, payload: &str) -> Option<String> {
        use ring::hmac;

        if let Some(secret) = &self.signing_secret {
            let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
            let signature = hmac::sign(&key, payload.as_bytes());
            Some(format!("sha256={}", hex::encode(signature.as_ref())))
        } else {
            None
        }
    }

    /// Build webhook payload from notification
    fn build_webhook_payload(notification: &NotificationPayload) -> WebhookPayload {
        WebhookPayload {
            id: notification.notification_id,
            event_type: Self::get_event_type(&notification.event),
            user_id: notification.user_id,
            timestamp: notification.created_at,
            event: notification.event.clone(),
            version: "1.0".to_string(),
        }
    }

    /// Get event type string
    fn get_event_type(event: &NexusEvent) -> String {
        match event {
            NexusEvent::BountyCreated(_) => "bounty.created",
            NexusEvent::BountyUpdated(_) => "bounty.updated",
            NexusEvent::BountyCompleted(_) => "bounty.completed",
            NexusEvent::BountyExpired(_) => "bounty.expired",
            NexusEvent::BountyCancelled(_) => "bounty.cancelled",
            NexusEvent::SubmissionReceived(_) => "submission.received",
            NexusEvent::SubmissionValidated(_) => "submission.validated",
            NexusEvent::SubmissionRejected(_) => "submission.rejected",
            NexusEvent::AnalysisStarted(_) => "analysis.started",
            NexusEvent::AnalysisCompleted(_) => "analysis.completed",
            NexusEvent::AnalysisFailed(_) => "analysis.failed",
            NexusEvent::ReputationUpdated(_) => "reputation.updated",
            NexusEvent::PaymentProcessed(_) => "payment.processed",
            NexusEvent::PaymentFailed(_) => "payment.failed",
            NexusEvent::StakeSlashed(_) => "stake.slashed",
            NexusEvent::UserRegistered(_) => "user.registered",
            NexusEvent::UserVerified(_) => "user.verified",
            NexusEvent::EngineRegistered(_) => "engine.registered",
            NexusEvent::DisputeCreated(_) => "dispute.created",
            NexusEvent::DisputeResolved(_) => "dispute.resolved",
            NexusEvent::SystemAlert(_) => "system.alert",
        }
        .to_string()
    }

    /// Send webhook with retry logic
    async fn send_with_retry(
        &self,
        url: &str,
        payload: &WebhookPayload,
        max_retries: u32,
    ) -> NotificationResult<WebhookResponse> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                let backoff = Duration::from_secs(2_u64.pow(attempt - 1));
                warn!(
                    "Retrying webhook to {} (attempt {}/{}), waiting {:?}",
                    url, attempt, max_retries, backoff
                );
                tokio::time::sleep(backoff).await;
            }

            match self.send_webhook_request(url, payload).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    error!("Webhook attempt {} failed: {:?}", attempt + 1, last_error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            NotificationError::SendError("All retry attempts failed".to_string())
        }))
    }

    /// Send a single webhook request
    async fn send_webhook_request(
        &self,
        url: &str,
        payload: &WebhookPayload,
    ) -> NotificationResult<WebhookResponse> {
        let payload_json = serde_json::to_string(payload)
            .map_err(|e| NotificationError::SerializationError(format!("Failed to serialize payload: {}", e)))?;

        let mut request = self
            .http_client
            .post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-Nexus-Event", &payload.event_type)
            .header("X-Nexus-Delivery-ID", payload.id.to_string())
            .header("X-Nexus-Timestamp", payload.timestamp.to_rfc3339());

        // Add signature if configured
        if let Some(signature) = self.generate_signature(&payload_json) {
            request = request.header("X-Nexus-Signature", signature);
        }

        let start_time = Utc::now();
        let response = request
            .body(payload_json)
            .send()
            .await
            .map_err(|e| NotificationError::SendError(format!("Webhook request failed: {}", e)))?;

        let status_code = response.status().as_u16();
        let response_body = response
            .text()
            .await
            .unwrap_or_else(|_| String::new());

        let delivery_time_ms = (Utc::now() - start_time).num_milliseconds() as u64;

        if status_code >= 200 && status_code < 300 {
            info!(
                "Webhook delivered successfully to {} in {}ms (status: {})",
                url, delivery_time_ms, status_code
            );
            Ok(WebhookResponse {
                status_code,
                response_body,
                delivery_time_ms,
            })
        } else {
            error!(
                "Webhook failed to {} with status {}: {}",
                url, status_code, response_body
            );
            Err(NotificationError::SendError(format!(
                "Webhook returned error status {}: {}",
                status_code, response_body
            )))
        }
    }

    /// Validate webhook URL
    fn validate_url(url: &str) -> NotificationResult<()> {
        let parsed_url = url::Url::parse(url)
            .map_err(|e| NotificationError::ValidationError(format!("Invalid URL: {}", e)))?;

        // Only allow HTTP and HTTPS
        if !["http", "https"].contains(&parsed_url.scheme()) {
            return Err(NotificationError::ValidationError(
                "Only HTTP and HTTPS schemes are allowed".to_string(),
            ));
        }

        // Prevent internal network calls for security
        if let Some(host) = parsed_url.host_str() {
            if host == "localhost"
                || host == "127.0.0.1"
                || host.starts_with("192.168.")
                || host.starts_with("10.")
                || host.starts_with("172.16.")
                || host.starts_with("172.17.")
                || host.starts_with("172.18.")
                || host.starts_with("172.19.")
                || host.starts_with("172.2")
                || host.starts_with("172.30.")
                || host.starts_with("172.31.")
            {
                warn!("Attempted to send webhook to internal address: {}", host);
                return Err(NotificationError::ValidationError(
                    "Webhooks to internal/private networks are not allowed".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl NotificationChannel for WebhookChannel {
    async fn send(
        &self,
        payload: &NotificationPayload,
        recipient: &str,
    ) -> NotificationResult<()> {
        info!(
            "Sending webhook notification to {} for event: {}",
            recipient,
            payload.event.get_title()
        );

        // Validate URL
        Self::validate_url(recipient)?;

        // Build webhook payload
        let webhook_payload = Self::build_webhook_payload(payload);

        // Send with retry (3 attempts)
        let _response = self.send_with_retry(recipient, &webhook_payload, 3).await?;

        Ok(())
    }

    fn channel_type(&self) -> &'static str {
        "webhook"
    }

    async fn validate_recipient(&self, recipient: &str) -> NotificationResult<bool> {
        Self::validate_url(recipient)?;
        Ok(true)
    }
}

// Webhook payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Unique ID for this webhook delivery
    pub id: uuid::Uuid,
    /// Event type (e.g., "bounty.created")
    pub event_type: String,
    /// User ID associated with the event
    pub user_id: uuid::Uuid,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// The actual event data
    pub event: NexusEvent,
    /// API version
    pub version: String,
}

/// Webhook response information
#[derive(Debug, Clone)]
pub struct WebhookResponse {
    pub status_code: u16,
    pub response_body: String,
    pub delivery_time_ms: u64,
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Optional signing secret for HMAC signatures
    pub signing_secret: Option<String>,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            signing_secret: None,
            max_retries: 3,
            timeout_seconds: 30,
        }
    }
}

/// Webhook registration information stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRegistration {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub url: String,
    pub events: Vec<String>, // Event types to listen for
    pub is_active: bool,
    pub secret: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub total_deliveries: i64,
    pub failed_deliveries: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;
    use shared::messaging::event_types::{BountyCreatedEvent, NotificationPriority};

    #[test]
    fn test_validate_url() {
        // Valid URLs
        assert!(WebhookChannel::validate_url("https://example.com/webhook").is_ok());
        assert!(WebhookChannel::validate_url("http://api.example.com/events").is_ok());

        // Invalid URLs
        assert!(WebhookChannel::validate_url("not-a-url").is_err());
        assert!(WebhookChannel::validate_url("ftp://example.com").is_err());
        assert!(WebhookChannel::validate_url("https://localhost/webhook").is_err());
        assert!(WebhookChannel::validate_url("https://127.0.0.1/webhook").is_err());
        assert!(WebhookChannel::validate_url("https://192.168.1.1/webhook").is_err());
        assert!(WebhookChannel::validate_url("https://10.0.0.1/webhook").is_err());
    }

    #[test]
    fn test_get_event_type() {
        let event = NexusEvent::BountyCreated(BountyCreatedEvent {
            bounty_id: Uuid::new_v4(),
            creator_id: Uuid::new_v4(),
            title: "Test".to_string(),
            description: "Test bounty".to_string(),
            reward_amount: 1000,
            stake_requirement: 100,
            expires_at: Utc::now(),
            target_type: "file".to_string(),
            tags: vec![],
            created_at: Utc::now(),
        });

        assert_eq!(WebhookChannel::get_event_type(&event), "bounty.created");
    }

    #[test]
    fn test_generate_signature() {
        let channel = WebhookChannel::new(Some("test_secret".to_string()));
        let payload = "test payload";

        let signature = channel.generate_signature(payload);
        assert!(signature.is_some());
        assert!(signature.unwrap().starts_with("sha256="));
    }

    #[tokio::test]
    async fn test_validate_recipient() {
        let channel = WebhookChannel::new(None);

        assert!(channel.validate_recipient("https://example.com/webhook").await.is_ok());
        assert!(channel.validate_recipient("https://localhost/webhook").await.is_err());
    }
}
