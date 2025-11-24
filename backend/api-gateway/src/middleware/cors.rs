use axum::{
    http::{
        header::{
            ACCEPT, AUTHORIZATION, CONTENT_TYPE, ORIGIN,
            ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_EXPOSE_HEADERS, ACCESS_CONTROL_MAX_AGE,
        },
        HeaderName, HeaderValue, Method, StatusCode,
    },
    response::{IntoResponse, Response},
};
use tower_http::cors::{Any, CorsLayer};
use std::time::Duration;

/// CORS configuration for different environments
#[derive(Debug, Clone)]
pub enum CorsConfig {
    Development,
    Staging,
    Production { allowed_origins: Vec<String> },
    Custom(CorsLayer),
}

impl CorsConfig {
    /// Create CORS configuration from environment
    pub fn from_env() -> Self {
        let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        match env.as_str() {
            "production" => {
                let origins = std::env::var("ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "https://nexus-security.com".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();

                Self::Production { allowed_origins: origins }
            }
            "staging" => Self::Staging,
            _ => Self::Development,
        }
    }

    /// Convert to CorsLayer
    pub fn into_layer(self) -> CorsLayer {
        match self {
            CorsConfig::Development => development_cors(),
            CorsConfig::Staging => staging_cors(),
            CorsConfig::Production { allowed_origins } => production_cors(allowed_origins),
            CorsConfig::Custom(layer) => layer,
        }
    }
}

/// Development CORS - Allow all origins
pub fn development_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            ACCEPT,
            ORIGIN,
            HeaderName::from_static("x-api-key"),
            HeaderName::from_static("x-request-id"),
        ])
        .expose_headers([
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-rate-limit-limit"),
            HeaderName::from_static("x-rate-limit-remaining"),
            HeaderName::from_static("x-rate-limit-reset"),
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600)) // 1 hour
}

/// Staging CORS - Allow specific staging domains
pub fn staging_cors() -> CorsLayer {
    let allowed_origins = vec![
        "https://staging.nexus-security.com",
        "https://preview.nexus-security.com",
        "http://localhost:3000",
        "http://localhost:5173", // Vite default
    ];

    CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .map(|origin| origin.parse::<HeaderValue>().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            ACCEPT,
            ORIGIN,
            HeaderName::from_static("x-api-key"),
            HeaderName::from_static("x-request-id"),
        ])
        .expose_headers([
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-rate-limit-limit"),
            HeaderName::from_static("x-rate-limit-remaining"),
            HeaderName::from_static("x-rate-limit-reset"),
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}

/// Production CORS - Strict origin validation
pub fn production_cors(allowed_origins: Vec<String>) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .filter_map(|origin| origin.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            ACCEPT,
            HeaderName::from_static("x-api-key"),
            HeaderName::from_static("x-request-id"),
        ])
        .expose_headers([
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-rate-limit-limit"),
            HeaderName::from_static("x-rate-limit-remaining"),
            HeaderName::from_static("x-rate-limit-reset"),
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(7200)) // 2 hours
}

/// Custom CORS for public API endpoints (no credentials)
pub fn public_api_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, ACCEPT])
        .expose_headers([HeaderName::from_static("x-rate-limit-remaining")])
        .allow_credentials(false)
        .max_age(Duration::from_secs(86400)) // 24 hours
}

/// CORS for webhook endpoints (allow specific services)
pub fn webhook_cors(allowed_origins: Vec<String>) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .filter_map(|origin| origin.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers([
            CONTENT_TYPE,
            HeaderName::from_static("x-webhook-signature"),
        ])
        .allow_credentials(false)
        .max_age(Duration::from_secs(3600))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_config_from_env() {
        // Test development default
        std::env::remove_var("ENVIRONMENT");
        let config = CorsConfig::from_env();
        matches!(config, CorsConfig::Development);
    }

    #[test]
    fn test_production_origins_parsing() {
        std::env::set_var("ENVIRONMENT", "production");
        std::env::set_var("ALLOWED_ORIGINS", "https://example.com,https://api.example.com");

        let config = CorsConfig::from_env();
        match config {
            CorsConfig::Production { allowed_origins } => {
                assert_eq!(allowed_origins.len(), 2);
                assert!(allowed_origins.contains(&"https://example.com".to_string()));
            }
            _ => panic!("Expected Production config"),
        }

        std::env::remove_var("ENVIRONMENT");
        std::env::remove_var("ALLOWED_ORIGINS");
    }
}
