use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::channels::{EmailConfig, FcmConfig, ApnsConfig, WebhookConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub email: EmailConfig,
    pub push: PushConfig,
    pub webhook: WebhookConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushConfig {
    pub fcm: Option<FcmConfig>,
    pub apns: Option<ApnsConfig>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8084".to_string())
                    .parse()?,
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")?,
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
            redis: RedisConfig {
                url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            email: EmailConfig {
                smtp_host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
                smtp_port: std::env::var("SMTP_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse()?,
                smtp_username: std::env::var("SMTP_USERNAME").unwrap_or_default(),
                smtp_password: std::env::var("SMTP_PASSWORD").unwrap_or_default(),
                from_address: std::env::var("EMAIL_FROM_ADDRESS")
                    .unwrap_or_else(|_| "noreply@nexus-security.io".to_string()),
                from_name: std::env::var("EMAIL_FROM_NAME")
                    .unwrap_or_else(|_| "Nexus Security".to_string()),
            },
            push: PushConfig {
                fcm: if std::env::var("FCM_SERVER_KEY").is_ok() {
                    Some(FcmConfig {
                        endpoint: std::env::var("FCM_ENDPOINT")
                            .unwrap_or_else(|_| "https://fcm.googleapis.com/fcm/send".to_string()),
                        server_key: std::env::var("FCM_SERVER_KEY").unwrap(),
                    })
                } else {
                    None
                },
                apns: if std::env::var("APNS_AUTH_TOKEN").is_ok() {
                    Some(ApnsConfig {
                        endpoint: std::env::var("APNS_ENDPOINT")
                            .unwrap_or_else(|_| "https://api.push.apple.com".to_string()),
                        auth_token: std::env::var("APNS_AUTH_TOKEN").unwrap(),
                        bundle_id: std::env::var("APNS_BUNDLE_ID")
                            .unwrap_or_else(|_| "io.nexus-security.app".to_string()),
                    })
                } else {
                    None
                },
            },
            webhook: WebhookConfig {
                signing_secret: std::env::var("WEBHOOK_SIGNING_SECRET").ok(),
                max_retries: std::env::var("WEBHOOK_MAX_RETRIES")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()?,
                timeout_seconds: std::env::var("WEBHOOK_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
            },
        })
    }
}
