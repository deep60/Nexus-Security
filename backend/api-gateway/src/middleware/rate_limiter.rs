use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use chrono::Utc;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_window: u32,
    pub window_duration: Duration,
    pub burst_size: Option<u32>,
}

impl RateLimitConfig {
    /// Default: 100 requests per minute
    pub fn default() -> Self {
        Self {
            requests_per_window: 100,
            window_duration: Duration::from_secs(60),
            burst_size: Some(20),
        }
    }

    /// Strict: 10 requests per minute
    pub fn strict() -> Self {
        Self {
            requests_per_window: 10,
            window_duration: Duration::from_secs(60),
            burst_size: Some(5),
        }
    }

    /// Relaxed: 1000 requests per minute
    pub fn relaxed() -> Self {
        Self {
            requests_per_window: 1000,
            window_duration: Duration::from_secs(60),
            burst_size: Some(200),
        }
    }

    /// Authentication endpoints: 5 requests per 15 minutes
    pub fn auth() -> Self {
        Self {
            requests_per_window: 5,
            window_duration: Duration::from_secs(900), // 15 minutes
            burst_size: None,
        }
    }

    /// API endpoints: 1000 requests per hour
    pub fn api() -> Self {
        Self {
            requests_per_window: 1000,
            window_duration: Duration::from_secs(3600), // 1 hour
            burst_size: Some(100),
        }
    }
}

/// Rate limiter state
#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
    last_request: Instant,
}

impl RateLimitEntry {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            count: 0,
            window_start: now,
            last_request: now,
        }
    }

    fn should_reset(&self, window_duration: Duration) -> bool {
        self.window_start.elapsed() >= window_duration
    }

    fn reset(&mut self) {
        let now = Instant::now();
        self.count = 0;
        self.window_start = now;
        self.last_request = now;
    }
}

/// Rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiter {
    entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if request is allowed
    pub async fn check_rate_limit(&self, identifier: &str) -> RateLimitResult {
        let mut entries = self.entries.write().await;
        let entry = entries.entry(identifier.to_string()).or_insert_with(RateLimitEntry::new);

        // Reset window if expired
        if entry.should_reset(self.config.window_duration) {
            entry.reset();
        }

        // Check if limit exceeded
        if entry.count >= self.config.requests_per_window {
            let retry_after = self.config.window_duration
                .checked_sub(entry.window_start.elapsed())
                .unwrap_or(Duration::from_secs(0));

            return RateLimitResult::Limited {
                retry_after_secs: retry_after.as_secs(),
                limit: self.config.requests_per_window,
                remaining: 0,
                reset_at: entry.window_start + self.config.window_duration,
            };
        }

        // Increment counter
        entry.count += 1;
        entry.last_request = Instant::now();

        let remaining = self.config.requests_per_window.saturating_sub(entry.count);

        RateLimitResult::Allowed {
            limit: self.config.requests_per_window,
            remaining,
            reset_at: entry.window_start + self.config.window_duration,
        }
    }

    /// Clean up old entries (call periodically)
    pub async fn cleanup_old_entries(&self) {
        let mut entries = self.entries.write().await;
        let threshold = self.config.window_duration * 2;

        entries.retain(|_, entry| entry.last_request.elapsed() < threshold);
    }
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed {
        limit: u32,
        remaining: u32,
        reset_at: Instant,
    },
    Limited {
        retry_after_secs: u64,
        limit: u32,
        remaining: u32,
        reset_at: Instant,
    },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }

    pub fn headers(&self) -> Vec<(String, String)> {
        match self {
            RateLimitResult::Allowed { limit, remaining, reset_at } => {
                vec![
                    ("X-RateLimit-Limit".to_string(), limit.to_string()),
                    ("X-RateLimit-Remaining".to_string(), remaining.to_string()),
                    ("X-RateLimit-Reset".to_string(), format!("{}", reset_at.elapsed().as_secs())),
                ]
            }
            RateLimitResult::Limited { retry_after_secs, limit, remaining, .. } => {
                vec![
                    ("X-RateLimit-Limit".to_string(), limit.to_string()),
                    ("X-RateLimit-Remaining".to_string(), remaining.to_string()),
                    ("Retry-After".to_string(), retry_after_secs.to_string()),
                ]
            }
        }
    }
}

/// Rate limit error response
#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimitError {
    pub error: String,
    pub message: String,
    pub retry_after_seconds: u64,
    pub limit: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// IP-based rate limiting middleware
pub async fn ip_rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(limiter): State<Arc<RateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let ip = addr.ip().to_string();

    match limiter.check_rate_limit(&ip).await {
        RateLimitResult::Allowed { .. } => {
            let mut response = next.run(request).await;

            // Add rate limit headers
            // TODO: Add headers to response

            Ok(response)
        }
        RateLimitResult::Limited { retry_after_secs, limit, .. } => {
            let error_response = RateLimitError {
                error: "RATE_LIMIT_EXCEEDED".to_string(),
                message: format!("Rate limit of {} requests exceeded", limit),
                retry_after_seconds: retry_after_secs,
                limit,
                timestamp: Utc::now(),
            };

            let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(error_response)).into_response();

            // Add retry-after header
            if let Ok(value) = retry_after_secs.to_string().parse() {
                response.headers_mut().insert("Retry-After", value);
            }

            Err(response)
        }
    }
}

/// User-based rate limiting middleware (requires authentication)
pub async fn user_rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    // TODO: Extract user ID from request extensions (set by auth middleware)
    let user_id = "anonymous"; // Placeholder

    match limiter.check_rate_limit(user_id).await {
        RateLimitResult::Allowed { limit, remaining, .. } => {
            let mut response = next.run(request).await;

            // Add rate limit headers
            if let Ok(limit_value) = limit.to_string().parse() {
                response.headers_mut().insert("X-RateLimit-Limit", limit_value);
            }
            if let Ok(remaining_value) = remaining.to_string().parse() {
                response.headers_mut().insert("X-RateLimit-Remaining", remaining_value);
            }

            Ok(response)
        }
        RateLimitResult::Limited { retry_after_secs, limit, .. } => {
            let error_response = RateLimitError {
                error: "RATE_LIMIT_EXCEEDED".to_string(),
                message: format!("User rate limit of {} requests exceeded", limit),
                retry_after_seconds: retry_after_secs,
                limit,
                timestamp: Utc::now(),
            };

            let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(error_response)).into_response();

            if let Ok(value) = retry_after_secs.to_string().parse() {
                response.headers_mut().insert("Retry-After", value);
            }

            Err(response)
        }
    }
}

/// API key-based rate limiting
pub async fn api_key_rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    // Extract API key from header
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous");

    match limiter.check_rate_limit(api_key).await {
        RateLimitResult::Allowed { .. } => Ok(next.run(request).await),
        RateLimitResult::Limited { retry_after_secs, limit, .. } => {
            let error_response = RateLimitError {
                error: "RATE_LIMIT_EXCEEDED".to_string(),
                message: format!("API key rate limit of {} requests exceeded", limit),
                retry_after_seconds: retry_after_secs,
                limit,
                timestamp: Utc::now(),
            };

            Err((StatusCode::TOO_MANY_REQUESTS, Json(error_response)).into_response())
        }
    }
}

/// Adaptive rate limiter that adjusts based on server load
#[derive(Debug, Clone)]
pub struct AdaptiveRateLimiter {
    base_limiter: RateLimiter,
    current_multiplier: Arc<RwLock<f64>>,
}

impl AdaptiveRateLimiter {
    pub fn new(base_config: RateLimitConfig) -> Self {
        Self {
            base_limiter: RateLimiter::new(base_config),
            current_multiplier: Arc::new(RwLock::new(1.0)),
        }
    }

    /// Adjust rate limit based on server load (0.5 = stricter, 2.0 = more lenient)
    pub async fn adjust_for_load(&self, load_percentage: f64) {
        let mut multiplier = self.current_multiplier.write().await;

        *multiplier = if load_percentage > 90.0 {
            0.5 // Reduce to 50% capacity under high load
        } else if load_percentage > 75.0 {
            0.75 // Reduce to 75% capacity
        } else if load_percentage < 25.0 {
            1.5 // Increase to 150% capacity under low load
        } else {
            1.0 // Normal capacity
        };
    }

    pub async fn check_rate_limit(&self, identifier: &str) -> RateLimitResult {
        // Use base limiter with adjusted multiplier
        self.base_limiter.check_rate_limit(identifier).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(RateLimitConfig {
            requests_per_window: 5,
            window_duration: Duration::from_secs(60),
            burst_size: None,
        });

        for _ in 0..5 {
            let result = limiter.check_rate_limit("test_user").await;
            assert!(result.is_allowed());
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(RateLimitConfig {
            requests_per_window: 3,
            window_duration: Duration::from_secs(60),
            burst_size: None,
        });

        for _ in 0..3 {
            limiter.check_rate_limit("test_user").await;
        }

        let result = limiter.check_rate_limit("test_user").await;
        assert!(!result.is_allowed());
    }

    #[tokio::test]
    async fn test_rate_limiter_resets_after_window() {
        let limiter = RateLimiter::new(RateLimitConfig {
            requests_per_window: 2,
            window_duration: Duration::from_millis(100),
            burst_size: None,
        });

        limiter.check_rate_limit("test_user").await;
        limiter.check_rate_limit("test_user").await;

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        let result = limiter.check_rate_limit("test_user").await;
        assert!(result.is_allowed());
    }
}
