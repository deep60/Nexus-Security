use anyhow::Result;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::channels::{EmailChannel, PushChannel, WebhookChannel, WebSocketChannel};
use crate::config::Config;
use crate::models::{NotificationChannel, NotificationPreferences, NotificationRecord, NotificationStatus};
use shared::messaging::event_types::NotificationPayload;

pub struct NotificationManager {
    config: Config,
    db_pool: PgPool,
    redis_conn: ConnectionManager,
    email_channel: Arc<EmailChannel>,
    push_channel: Arc<PushChannel>,
    webhook_channel: Arc<WebhookChannel>,
    websocket_channel: Arc<WebSocketChannel>,
}

impl NotificationManager {
    pub async fn new(
        config: Config,
        db_pool: PgPool,
        redis_conn: ConnectionManager,
    ) -> Result<Self> {
        let email_channel = Arc::new(EmailChannel::new(config.email.clone())?);
        let push_channel = Arc::new(PushChannel::new(
            config.push.fcm.clone(),
            config.push.apns.clone(),
        ));
        let webhook_channel = Arc::new(WebhookChannel::new(config.webhook.signing_secret.clone()));
        let websocket_channel = Arc::new(WebSocketChannel::new());

        Ok(Self {
            config,
            db_pool,
            redis_conn,
            email_channel,
            push_channel,
            webhook_channel,
            websocket_channel,
        })
    }

    pub async fn send_notification(&self, payload: &NotificationPayload) -> Result<()> {
        info!("Processing notification for user {}", payload.user_id);

        // Get user preferences
        let prefs = self.get_user_preferences(payload.user_id).await?;

        // Send through each enabled channel
        for channel in &payload.channels {
            match channel {
                shared::messaging::event_types::NotificationChannel::Email if prefs.email_enabled => {
                    if let Some(email) = &prefs.email_address {
                        let _ = self.email_channel.send(payload, email).await;
                    }
                }
                shared::messaging::event_types::NotificationChannel::Push if prefs.push_enabled => {
                    for token in &prefs.push_tokens {
                        let recipient = format!("{}:{}", token.platform, token.token);
                        let _ = self.push_channel.send(payload, &recipient).await;
                    }
                }
                shared::messaging::event_types::NotificationChannel::Webhook if prefs.webhook_enabled => {
                    for url in &prefs.webhook_urls {
                        let _ = self.webhook_channel.send(payload, url).await;
                    }
                }
                shared::messaging::event_types::NotificationChannel::WebSocket if prefs.websocket_enabled => {
                    let _ = self.websocket_channel.send(payload, &payload.user_id.to_string()).await;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn get_user_preferences(&self, _user_id: Uuid) -> Result<NotificationPreferences> {
        // TODO: Fetch from database
        Ok(NotificationPreferences::default())
    }

    pub async fn start_event_listener(&self) -> Result<()> {
        info!("Starting event listener...");
        // TODO: Listen to Redis pub/sub or Kafka for events
        Ok(())
    }

    pub async fn start_retry_worker(&self) -> Result<()> {
        info!("Starting retry worker...");
        // TODO: Retry failed notifications
        Ok(())
    }

    pub fn get_websocket_channel(&self) -> Arc<WebSocketChannel> {
        self.websocket_channel.clone()
    }
}
