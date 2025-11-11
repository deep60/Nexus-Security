use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use shared::messaging::event_types::{NotificationPayload, NotificationPriority};

/// Result type for notification operations
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Notification error types
#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Send error: {0}")]
    SendError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

/// Trait for notification channels
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Send a notification through this channel
    async fn send(
        &self,
        payload: &NotificationPayload,
        recipient: &str,
    ) -> NotificationResult<()>;

    /// Get the channel type identifier
    fn channel_type(&self) -> &'static str;

    /// Validate a recipient address/token for this channel
    async fn validate_recipient(&self, recipient: &str) -> NotificationResult<bool>;
}

/// Notification record stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel: String,
    pub recipient: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: NotificationStatus,
    pub priority: NotificationPriority,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationStatus {
    Pending,
    Sending,
    Sent,
    Failed,
    Retrying,
}

impl ToString for NotificationStatus {
    fn to_string(&self) -> String {
        match self {
            NotificationStatus::Pending => "pending".to_string(),
            NotificationStatus::Sending => "sending".to_string(),
            NotificationStatus::Sent => "sent".to_string(),
            NotificationStatus::Failed => "failed".to_string(),
            NotificationStatus::Retrying => "retrying".to_string(),
        }
    }
}

/// User notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: Uuid,
    pub email_enabled: bool,
    pub email_address: Option<String>,
    pub push_enabled: bool,
    pub push_tokens: Vec<PushToken>,
    pub webhook_enabled: bool,
    pub webhook_urls: Vec<String>,
    pub websocket_enabled: bool,
    pub event_filters: Vec<String>, // Event types to receive
    pub do_not_disturb: bool,
    pub quiet_hours_start: Option<String>, // HH:MM format
    pub quiet_hours_end: Option<String>,   // HH:MM format
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushToken {
    pub platform: String, // "fcm", "apns", etc.
    pub token: String,
    pub device_name: Option<String>,
    pub added_at: DateTime<Utc>,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            email_enabled: true,
            email_address: None,
            push_enabled: true,
            push_tokens: Vec::new(),
            webhook_enabled: false,
            webhook_urls: Vec::new(),
            websocket_enabled: true,
            event_filters: Vec::new(), // Empty means all events
            do_not_disturb: false,
            quiet_hours_start: None,
            quiet_hours_end: None,
            updated_at: Utc::now(),
        }
    }
}

/// Notification statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationStats {
    pub total_sent: i64,
    pub total_failed: i64,
    pub by_channel: std::collections::HashMap<String, ChannelStats>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStats {
    pub sent: i64,
    pub failed: i64,
    pub avg_delivery_time_ms: f64,
}
