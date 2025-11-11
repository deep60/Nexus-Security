use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::models::{NotificationChannel, NotificationError, NotificationResult};
use shared::messaging::event_types::{NexusEvent, NotificationPayload, NotificationPriority};

/// Push notification channel implementation
/// Supports Firebase Cloud Messaging (FCM) and Apple Push Notification Service (APNS)
pub struct PushChannel {
    http_client: Client,
    fcm_config: Option<FcmConfig>,
    apns_config: Option<ApnsConfig>,
}

impl PushChannel {
    /// Create a new push notification channel
    pub fn new(fcm_config: Option<FcmConfig>, apns_config: Option<ApnsConfig>) -> Self {
        Self {
            http_client: Client::new(),
            fcm_config,
            apns_config,
        }
    }

    /// Send FCM push notification
    async fn send_fcm(
        &self,
        config: &FcmConfig,
        device_token: &str,
        notification: &PushNotification,
    ) -> NotificationResult<()> {
        let fcm_message = FcmMessage {
            to: device_token.to_string(),
            notification: FcmNotificationPayload {
                title: notification.title.clone(),
                body: notification.body.clone(),
                icon: notification.icon.clone(),
                sound: notification.sound.clone().unwrap_or_else(|| "default".to_string()),
                badge: notification.badge,
                click_action: notification.click_action.clone(),
            },
            data: notification.data.clone(),
            priority: match notification.priority {
                NotificationPriority::Critical | NotificationPriority::High => "high",
                _ => "normal",
            }
            .to_string(),
            time_to_live: notification.ttl.unwrap_or(86400), // 24 hours default
        };

        let response = self
            .http_client
            .post(&config.endpoint)
            .header("Authorization", format!("key={}", config.server_key))
            .header("Content-Type", "application/json")
            .json(&fcm_message)
            .send()
            .await
            .map_err(|e| NotificationError::SendError(format!("FCM request failed: {}", e)))?;

        if response.status().is_success() {
            info!("FCM push notification sent successfully to {}", device_token);
            Ok(())
        } else {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "FCM push notification failed with status {}: {}",
                status, error_body
            );
            Err(NotificationError::SendError(format!(
                "FCM error {}: {}",
                status, error_body
            )))
        }
    }

    /// Send APNS push notification
    async fn send_apns(
        &self,
        config: &ApnsConfig,
        device_token: &str,
        notification: &PushNotification,
    ) -> NotificationResult<()> {
        let apns_payload = ApnsPayload {
            aps: ApnsAps {
                alert: ApnsAlert {
                    title: notification.title.clone(),
                    body: notification.body.clone(),
                },
                badge: notification.badge,
                sound: notification.sound.clone().unwrap_or_else(|| "default".to_string()),
                thread_id: notification.thread_id.clone(),
                category: notification.category.clone(),
            },
            data: notification.data.clone(),
        };

        let url = format!(
            "{}/3/device/{}",
            config.endpoint, device_token
        );

        let response = self
            .http_client
            .post(&url)
            .header("authorization", format!("bearer {}", config.auth_token))
            .header("apns-topic", &config.bundle_id)
            .header("apns-priority", match notification.priority {
                NotificationPriority::Critical | NotificationPriority::High => "10",
                _ => "5",
            })
            .header("apns-expiration", notification.ttl.unwrap_or(86400).to_string())
            .json(&apns_payload)
            .send()
            .await
            .map_err(|e| NotificationError::SendError(format!("APNS request failed: {}", e)))?;

        if response.status().is_success() {
            info!("APNS push notification sent successfully to {}", device_token);
            Ok(())
        } else {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "APNS push notification failed with status {}: {}",
                status, error_body
            );
            Err(NotificationError::SendError(format!(
                "APNS error {}: {}",
                status, error_body
            )))
        }
    }

    /// Build push notification from event
    fn build_push_notification(
        payload: &NotificationPayload,
    ) -> PushNotification {
        let title = payload.event.get_title();
        let body = payload.event.get_description();

        let mut data = HashMap::new();
        data.insert("notification_id".to_string(), payload.notification_id.to_string());
        data.insert("user_id".to_string(), payload.user_id.to_string());
        data.insert("event_type".to_string(), Self::get_event_type(&payload.event));
        data.insert("timestamp".to_string(), payload.created_at.to_rfc3339());

        // Add event-specific data
        match &payload.event {
            NexusEvent::BountyCreated(e) => {
                data.insert("bounty_id".to_string(), e.bounty_id.to_string());
                data.insert("action".to_string(), "view_bounty".to_string());
            }
            NexusEvent::SubmissionReceived(e) => {
                data.insert("submission_id".to_string(), e.submission_id.to_string());
                data.insert("bounty_id".to_string(), e.bounty_id.to_string());
                data.insert("action".to_string(), "view_submission".to_string());
            }
            NexusEvent::PaymentProcessed(e) => {
                data.insert("bounty_id".to_string(), e.bounty_id.to_string());
                data.insert("tx_hash".to_string(), e.tx_hash.clone());
                data.insert("action".to_string(), "view_transaction".to_string());
            }
            _ => {}
        }

        PushNotification {
            title,
            body,
            icon: Some("ic_notification".to_string()),
            sound: Some("default".to_string()),
            badge: None,
            priority: payload.priority.clone(),
            data,
            click_action: Some("OPEN_APP".to_string()),
            thread_id: Some("nexus_security".to_string()),
            category: Some(Self::get_notification_category(&payload.event)),
            ttl: Some(86400), // 24 hours
        }
    }

    /// Get event type string
    fn get_event_type(event: &NexusEvent) -> String {
        match event {
            NexusEvent::BountyCreated(_) => "bounty_created",
            NexusEvent::BountyUpdated(_) => "bounty_updated",
            NexusEvent::BountyCompleted(_) => "bounty_completed",
            NexusEvent::BountyExpired(_) => "bounty_expired",
            NexusEvent::BountyCancelled(_) => "bounty_cancelled",
            NexusEvent::SubmissionReceived(_) => "submission_received",
            NexusEvent::SubmissionValidated(_) => "submission_validated",
            NexusEvent::SubmissionRejected(_) => "submission_rejected",
            NexusEvent::AnalysisStarted(_) => "analysis_started",
            NexusEvent::AnalysisCompleted(_) => "analysis_completed",
            NexusEvent::AnalysisFailed(_) => "analysis_failed",
            NexusEvent::ReputationUpdated(_) => "reputation_updated",
            NexusEvent::PaymentProcessed(_) => "payment_processed",
            NexusEvent::PaymentFailed(_) => "payment_failed",
            NexusEvent::StakeSlashed(_) => "stake_slashed",
            NexusEvent::UserRegistered(_) => "user_registered",
            NexusEvent::UserVerified(_) => "user_verified",
            NexusEvent::EngineRegistered(_) => "engine_registered",
            NexusEvent::DisputeCreated(_) => "dispute_created",
            NexusEvent::DisputeResolved(_) => "dispute_resolved",
            NexusEvent::SystemAlert(_) => "system_alert",
        }
        .to_string()
    }

    /// Get notification category for iOS
    fn get_notification_category(event: &NexusEvent) -> String {
        match event {
            NexusEvent::BountyCreated(_) => "BOUNTY_NOTIFICATION",
            NexusEvent::SubmissionReceived(_) => "SUBMISSION_NOTIFICATION",
            NexusEvent::PaymentProcessed(_) => "PAYMENT_NOTIFICATION",
            NexusEvent::ReputationUpdated(_) => "REPUTATION_NOTIFICATION",
            _ => "GENERAL_NOTIFICATION",
        }
        .to_string()
    }
}

#[async_trait]
impl NotificationChannel for PushChannel {
    async fn send(
        &self,
        payload: &NotificationPayload,
        recipient: &str,
    ) -> NotificationResult<()> {
        info!(
            "Sending push notification to {} for event: {}",
            recipient,
            payload.event.get_title()
        );

        // Parse recipient format: "platform:device_token"
        let parts: Vec<&str> = recipient.split(':').collect();
        if parts.len() != 2 {
            return Err(NotificationError::ValidationError(
                "Invalid push recipient format. Expected 'platform:device_token'".to_string(),
            ));
        }

        let platform = parts[0];
        let device_token = parts[1];

        let notification = Self::build_push_notification(payload);

        match platform {
            "fcm" | "android" => {
                if let Some(ref config) = self.fcm_config {
                    self.send_fcm(config, device_token, &notification).await
                } else {
                    Err(NotificationError::ConfigError(
                        "FCM not configured".to_string(),
                    ))
                }
            }
            "apns" | "ios" => {
                if let Some(ref config) = self.apns_config {
                    self.send_apns(config, device_token, &notification).await
                } else {
                    Err(NotificationError::ConfigError(
                        "APNS not configured".to_string(),
                    ))
                }
            }
            _ => Err(NotificationError::ValidationError(format!(
                "Unsupported platform: {}",
                platform
            ))),
        }
    }

    fn channel_type(&self) -> &'static str {
        "push"
    }

    async fn validate_recipient(&self, recipient: &str) -> NotificationResult<bool> {
        let parts: Vec<&str> = recipient.split(':').collect();
        if parts.len() != 2 {
            return Err(NotificationError::ValidationError(
                "Invalid format. Expected 'platform:device_token'".to_string(),
            ));
        }

        let platform = parts[0];
        let device_token = parts[1];

        // Validate platform
        if !["fcm", "android", "apns", "ios"].contains(&platform) {
            return Err(NotificationError::ValidationError(format!(
                "Invalid platform: {}",
                platform
            )));
        }

        // Validate device token is not empty
        if device_token.is_empty() {
            return Err(NotificationError::ValidationError(
                "Device token cannot be empty".to_string(),
            ));
        }

        Ok(true)
    }
}

// Configuration structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmConfig {
    pub endpoint: String,
    pub server_key: String,
}

impl Default for FcmConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://fcm.googleapis.com/fcm/send".to_string(),
            server_key: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApnsConfig {
    pub endpoint: String,
    pub auth_token: String,
    pub bundle_id: String,
}

impl Default for ApnsConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.push.apple.com".to_string(),
            auth_token: String::new(),
            bundle_id: "io.nexus-security.app".to_string(),
        }
    }
}

// Push notification structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PushNotification {
    title: String,
    body: String,
    icon: Option<String>,
    sound: Option<String>,
    badge: Option<i32>,
    priority: NotificationPriority,
    data: HashMap<String, String>,
    click_action: Option<String>,
    thread_id: Option<String>,
    category: Option<String>,
    ttl: Option<u64>, // Time to live in seconds
}

// FCM message structures
#[derive(Debug, Serialize)]
struct FcmMessage {
    to: String,
    notification: FcmNotificationPayload,
    data: HashMap<String, String>,
    priority: String,
    time_to_live: u64,
}

#[derive(Debug, Serialize)]
struct FcmNotificationPayload {
    title: String,
    body: String,
    icon: Option<String>,
    sound: String,
    badge: Option<i32>,
    click_action: Option<String>,
}

// APNS message structures
#[derive(Debug, Serialize)]
struct ApnsPayload {
    aps: ApnsAps,
    #[serde(flatten)]
    data: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct ApnsAps {
    alert: ApnsAlert,
    badge: Option<i32>,
    sound: String,
    #[serde(rename = "thread-id")]
    thread_id: Option<String>,
    category: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApnsAlert {
    title: String,
    body: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;
    use shared::messaging::event_types::{BountyCreatedEvent, NotificationPriority};

    #[tokio::test]
    async fn test_validate_recipient() {
        let channel = PushChannel::new(None, None);

        // Valid recipients
        assert!(channel.validate_recipient("fcm:abc123").await.is_ok());
        assert!(channel.validate_recipient("ios:xyz789").await.is_ok());

        // Invalid recipients
        assert!(channel.validate_recipient("invalid").await.is_err());
        assert!(channel.validate_recipient("fcm:").await.is_err());
        assert!(channel.validate_recipient("unknown:token").await.is_err());
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

        assert_eq!(PushChannel::get_event_type(&event), "bounty_created");
    }
}
