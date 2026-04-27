use anyhow::Result;
use futures_util::StreamExt;
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

    async fn get_user_preferences(&self, user_id: Uuid) -> Result<NotificationPreferences> {
        // Query notification preferences from the database
        let prefs = sqlx::query_as::<_, NotificationPreferences>(
            r#"
            SELECT user_id, email_enabled, email_address, push_enabled,
                   webhook_enabled, websocket_enabled
            FROM notification_preferences
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(prefs.unwrap_or_default())
    }

    pub async fn start_event_listener(&self) -> Result<()> {
        info!("Starting Redis Pub/Sub event listener...");

        // Channels we're interested in for email notifications
        let channels = vec![
            "events:user_registered",
            "events:payment_processed",
        ];

        // Get a new Redis connection for Pub/Sub (must be dedicated)
        let redis_client = redis::Client::open(self.config.redis.url.clone())?;
        let conn = redis_client.get_async_connection().await?;
        let mut pubsub = conn.into_pubsub();

        // Subscribe to all channels
        for channel in &channels {
            pubsub.subscribe(channel).await?;
            info!("Subscribed to channel: {}", channel);
        }

        info!("Event listener started successfully, waiting for events...");

        // Listen for messages in an infinite loop
        loop {
            match pubsub.on_message().next().await {
                Some(msg) => {
                    let channel = msg.get_channel_name();
                    let payload: String = match msg.get_payload() {
                        Ok(p) => p,
                        Err(e) => {
                            error!("Failed to get message payload from {}: {}", channel, e);
                            continue;
                        }
                    };

                    info!("Received event from channel: {}", channel);

                    // Process the event
                    if let Err(e) = self.process_event(channel, &payload).await {
                        error!("Failed to process event from {}: {}", channel, e);
                    }
                }
                None => {
                    error!("Pub/Sub message stream ended unexpectedly");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn process_event(&self, channel: &str, payload: &str) -> Result<()> {
        use shared::messaging::event_types::{VerdyxEvent, UserRegisteredEvent, PaymentProcessedEvent, NotificationChannel, NotificationPriority, NotificationPayload};

        // Deserialize the event based on channel
        let event: VerdyxEvent = match channel {
            "events:user_registered" => {
                let user_event: UserRegisteredEvent = serde_json::from_str(payload)?;
                VerdyxEvent::UserRegistered(user_event)
            }
            "events:payment_processed" => {
                let payment_event: PaymentProcessedEvent = serde_json::from_str(payload)?;
                VerdyxEvent::PaymentProcessed(payment_event)
            }
            _ => {
                info!("Ignoring unhandled channel: {}", channel);
                return Ok(());
            }
        };

        // Extract user_id from event
        let user_id = match &event {
            VerdyxEvent::UserRegistered(e) => e.user_id,
            VerdyxEvent::PaymentProcessed(e) => e.recipient_id,
            _ => {
                error!("Unexpected event type for channel: {}", channel);
                return Ok(());
            }
        };

        // Create notification payload
        let notification_payload = NotificationPayload {
            notification_id: Uuid::new_v4(),
            user_id,
            channels: vec![NotificationChannel::Email], // Email only for now
            event: event.clone(),
            priority: match &event {
                VerdyxEvent::UserRegistered(_) => NotificationPriority::Normal,
                VerdyxEvent::PaymentProcessed(_) => NotificationPriority::High,
                _ => NotificationPriority::Normal,
            },
            created_at: chrono::Utc::now(),
        };

        info!(
            "Processing notification for user {} via email: {}",
            user_id,
            event.get_title()
        );

        // Send notification
        self.send_notification(&notification_payload).await?;

        info!("Notification sent successfully for user {}", user_id);
        Ok(())
    }

    pub async fn start_retry_worker(&self) -> Result<()> {
        info!("Starting notification retry worker...");

        loop {
            // Fetch failed notifications that haven't exceeded max retries
            let failed: Vec<NotificationRecord> = sqlx::query_as(
                r#"
                SELECT * FROM notification_records
                WHERE status = 'failed' AND retry_count < 3
                ORDER BY created_at ASC
                LIMIT 50
                "#
            )
            .fetch_all(&self.db_pool)
            .await
            .unwrap_or_default();

            if !failed.is_empty() {
                info!("Retrying {} failed notifications", failed.len());
            }

            for record in &failed {
                // Reconstruct payload from the stored record and re-attempt
                if let Some(ref payload_json) = record.payload {
                    match serde_json::from_value::<NotificationPayload>(payload_json.clone()) {
                        Ok(payload) => {
                            match self.send_notification(&payload).await {
                                Ok(_) => {
                                    // Mark as sent
                                    let _ = sqlx::query(
                                        "UPDATE notification_records SET status = 'sent', updated_at = NOW() WHERE id = $1"
                                    )
                                    .bind(record.id)
                                    .execute(&self.db_pool)
                                    .await;
                                }
                                Err(e) => {
                                    // Increment retry count
                                    let _ = sqlx::query(
                                        "UPDATE notification_records SET retry_count = retry_count + 1, updated_at = NOW() WHERE id = $1"
                                    )
                                    .bind(record.id)
                                    .execute(&self.db_pool)
                                    .await;
                                    error!("Retry failed for notification {}: {}", record.id, e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Cannot deserialize payload for notification {}: {}", record.id, e);
                        }
                    }
                }
            }

            // Sleep before next retry cycle (exponential backoff would be per-record;
            // this is a simple fixed-interval batch retry)
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }

    pub fn get_websocket_channel(&self) -> Arc<WebSocketChannel> {
        self.websocket_channel.clone()
    }
}
