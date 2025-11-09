use anyhow::{Context, Result};
use reqwest::{Client, Method, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Proxy service for making HTTP requests to other microservices
/// Includes circuit breaker pattern, request retry logic, and service discovery
#[derive(Clone)]
pub struct ProxyService {
    client: Client,
    config: ProxyConfig,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    stats: Arc<RwLock<ProxyStats>>,
}

/// Proxy service configuration
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout_seconds: u64,
    pub enable_service_discovery: bool,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_seconds: 60,
            enable_service_discovery: false,
        }
    }
}

/// Service registry for microservice discovery
#[derive(Debug, Clone)]
pub struct ServiceRegistry {
    services: HashMap<String, ServiceEndpoint>,
}

/// Service endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub name: String,
    pub base_url: String,
    pub health_check_path: Option<String>,
    pub api_version: String,
    pub requires_auth: bool,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, endpoint: ServiceEndpoint) {
        info!("Registering service: {} at {}", name, endpoint.base_url);
        self.services.insert(name, endpoint);
    }

    pub fn get(&self, name: &str) -> Option<&ServiceEndpoint> {
        self.services.get(name)
    }

    pub fn default_services() -> Self {
        let mut registry = Self::new();

        // Analysis Engine
        registry.register(
            "analysis-engine".to_string(),
            ServiceEndpoint {
                name: "Analysis Engine".to_string(),
                base_url: std::env::var("ANALYSIS_ENGINE_URL")
                    .unwrap_or_else(|_| "http://localhost:8081".to_string()),
                health_check_path: Some("/health".to_string()),
                api_version: "v1".to_string(),
                requires_auth: true,
            },
        );

        // Bounty Manager
        registry.register(
            "bounty-manager".to_string(),
            ServiceEndpoint {
                name: "Bounty Manager".to_string(),
                base_url: std::env::var("BOUNTY_MANAGER_URL")
                    .unwrap_or_else(|_| "http://localhost:8082".to_string()),
                health_check_path: Some("/health".to_string()),
                api_version: "v1".to_string(),
                requires_auth: true,
            },
        );

        // Notification Service
        registry.register(
            "notification-service".to_string(),
            ServiceEndpoint {
                name: "Notification Service".to_string(),
                base_url: std::env::var("NOTIFICATION_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:8083".to_string()),
                health_check_path: Some("/health".to_string()),
                api_version: "v1".to_string(),
                requires_auth: true,
            },
        );

        registry
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,      // Normal operation
    Open,        // Failing - reject requests
    HalfOpen,    // Testing if service recovered
}

/// Circuit breaker for fault tolerance
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    last_failure_time: Option<Instant>,
    threshold: u32,
    timeout_duration: Duration,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout_seconds: u64) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            last_failure_time: None,
            threshold,
            timeout_duration: Duration::from_secs(timeout_seconds),
        }
    }

    pub fn record_success(&mut self) {
        if self.state == CircuitBreakerState::HalfOpen {
            info!("Circuit breaker recovered - transitioning to Closed");
            self.state = CircuitBreakerState::Closed;
        }
        self.failure_count = 0;
        self.last_failure_time = None;
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.threshold {
            warn!("Circuit breaker threshold reached - opening circuit");
            self.state = CircuitBreakerState::Open;
        }
    }

    pub fn can_attempt_request(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout_duration {
                        info!("Circuit breaker timeout elapsed - transitioning to HalfOpen");
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }
}

/// Proxy statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub retried_requests: u64,
    pub circuit_breaker_trips: u64,
    pub avg_response_time_ms: u64,
    pub requests_by_service: HashMap<String, u64>,
}

impl ProxyService {
    /// Create a new proxy service
    pub fn new(config: ProxyConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .context("Failed to create HTTP client")?;

        info!("Proxy service initialized with config: {:?}", config);

        Ok(Self {
            client,
            config,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ProxyStats::default())),
        })
    }

    /// Make a GET request to a service
    pub async fn get(
        &self,
        service_name: &str,
        path: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Response> {
        self.request(Method::GET, service_name, path, None::<()>, headers).await
    }

    /// Make a POST request to a service
    pub async fn post<T: Serialize>(
        &self,
        service_name: &str,
        path: &str,
        body: T,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Response> {
        self.request(Method::POST, service_name, path, Some(body), headers).await
    }

    /// Make a PUT request to a service
    pub async fn put<T: Serialize>(
        &self,
        service_name: &str,
        path: &str,
        body: T,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Response> {
        self.request(Method::PUT, service_name, path, Some(body), headers).await
    }

    /// Make a DELETE request to a service
    pub async fn delete(
        &self,
        service_name: &str,
        path: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Response> {
        self.request(Method::DELETE, service_name, path, None::<()>, headers).await
    }

    /// Make an HTTP request with circuit breaker and retry logic
    pub async fn request<T: Serialize>(
        &self,
        method: Method,
        service_name: &str,
        path: &str,
        body: Option<T>,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Response> {
        // Check circuit breaker
        {
            let mut breakers = self.circuit_breakers.write().await;
            let breaker = breakers
                .entry(service_name.to_string())
                .or_insert_with(|| {
                    CircuitBreaker::new(
                        self.config.circuit_breaker_threshold,
                        self.config.circuit_breaker_timeout_seconds,
                    )
                });

            if !breaker.can_attempt_request() {
                let mut stats = self.stats.write().await;
                stats.circuit_breaker_trips += 1;

                return Err(anyhow::anyhow!(
                    "Circuit breaker is open for service: {}",
                    service_name
                ));
            }
        }

        // Get service endpoint
        let registry = ServiceRegistry::default_services();
        let endpoint = registry
            .get(service_name)
            .ok_or_else(|| anyhow::anyhow!("Service not found: {}", service_name))?;

        let url = format!("{}{}", endpoint.base_url, path);
        debug!("Proxy request: {} {}", method, url);

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            *stats.requests_by_service.entry(service_name.to_string()).or_insert(0) += 1;
        }

        let start_time = Instant::now();
        let mut last_error = None;

        // Retry logic
        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(self.config.retry_delay_ms * attempt as u64);
                debug!("Retrying request (attempt {}/{}) after {:?}", attempt, self.config.max_retries, delay);
                tokio::time::sleep(delay).await;

                let mut stats = self.stats.write().await;
                stats.retried_requests += 1;
            }

            // Build request
            let mut request = self.client.request(method.clone(), &url);

            // Add headers
            if let Some(ref header_map) = headers {
                for (key, value) in header_map {
                    request = request.header(key, value);
                }
            }

            // Add body if provided
            if let Some(ref body_data) = body {
                request = request.json(body_data);
            }

            // Execute request
            match request.send().await {
                Ok(response) => {
                    let elapsed = start_time.elapsed();
                    let status = response.status();

                    debug!("Proxy response: {} ({:?})", status, elapsed);

                    // Update stats
                    {
                        let mut stats = self.stats.write().await;
                        let total = stats.successful_requests + stats.failed_requests;
                        let current_avg = stats.avg_response_time_ms;
                        stats.avg_response_time_ms =
                            (current_avg * total + elapsed.as_millis() as u64) / (total + 1);

                        if status.is_success() {
                            stats.successful_requests += 1;

                            // Record success in circuit breaker
                            let mut breakers = self.circuit_breakers.write().await;
                            if let Some(breaker) = breakers.get_mut(service_name) {
                                breaker.record_success();
                            }

                            return Ok(response);
                        } else {
                            stats.failed_requests += 1;
                            last_error = Some(anyhow::anyhow!("HTTP error: {}", status));
                        }
                    }

                    // Don't retry client errors (4xx), only server errors (5xx)
                    if status.is_client_error() {
                        return Ok(response);
                    }
                }
                Err(e) => {
                    error!("Proxy request failed: {}", e);
                    last_error = Some(anyhow::anyhow!("Request failed: {}", e));

                    let mut stats = self.stats.write().await;
                    stats.failed_requests += 1;
                }
            }
        }

        // All retries failed - record circuit breaker failure
        {
            let mut breakers = self.circuit_breakers.write().await;
            if let Some(breaker) = breakers.get_mut(service_name) {
                breaker.record_failure();
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Request failed after {} retries", self.config.max_retries)))
    }

    /// Check health of a service
    pub async fn health_check(&self, service_name: &str) -> Result<bool> {
        let registry = ServiceRegistry::default_services();
        let endpoint = registry
            .get(service_name)
            .ok_or_else(|| anyhow::anyhow!("Service not found: {}", service_name))?;

        let health_path = endpoint
            .health_check_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No health check path configured for service: {}", service_name))?;

        let url = format!("{}{}", endpoint.base_url, health_path);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get proxy statistics
    pub async fn get_stats(&self) -> ProxyStats {
        self.stats.read().await.clone()
    }

    /// Reset proxy statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = ProxyStats::default();
        info!("Proxy statistics reset");
    }

    /// Get circuit breaker status for a service
    pub async fn get_circuit_breaker_state(&self, service_name: &str) -> Option<CircuitBreakerState> {
        let breakers = self.circuit_breakers.read().await;
        breakers.get(service_name).map(|b| b.state.clone())
    }

    /// Manually reset circuit breaker for a service
    pub async fn reset_circuit_breaker(&self, service_name: &str) -> Result<()> {
        let mut breakers = self.circuit_breakers.write().await;

        if let Some(breaker) = breakers.get_mut(service_name) {
            breaker.state = CircuitBreakerState::Closed;
            breaker.failure_count = 0;
            breaker.last_failure_time = None;
            info!("Circuit breaker reset for service: {}", service_name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No circuit breaker found for service: {}", service_name))
        }
    }

    /// Forward request to analysis engine
    pub async fn forward_to_analysis_engine<T: Serialize>(
        &self,
        path: &str,
        body: T,
    ) -> Result<Response> {
        self.post("analysis-engine", path, body, None).await
    }

    /// Forward request to bounty manager
    pub async fn forward_to_bounty_manager<T: Serialize>(
        &self,
        path: &str,
        body: T,
    ) -> Result<Response> {
        self.post("bounty-manager", path, body, None).await
    }

    /// Send notification
    pub async fn send_notification<T: Serialize>(
        &self,
        notification: T,
    ) -> Result<Response> {
        self.post("notification-service", "/api/v1/notifications", notification, None).await
    }
}

/// Builder for proxy service configuration
pub struct ProxyServiceBuilder {
    config: ProxyConfig,
}

impl ProxyServiceBuilder {
    pub fn new() -> Self {
        Self {
            config: ProxyConfig::default(),
        }
    }

    pub fn timeout(mut self, seconds: u64) -> Self {
        self.config.timeout_seconds = seconds;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    pub fn retry_delay(mut self, delay_ms: u64) -> Self {
        self.config.retry_delay_ms = delay_ms;
        self
    }

    pub fn circuit_breaker_threshold(mut self, threshold: u32) -> Self {
        self.config.circuit_breaker_threshold = threshold;
        self
    }

    pub fn build(self) -> Result<ProxyService> {
        ProxyService::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed_state() {
        let mut breaker = CircuitBreaker::new(5, 60);
        assert_eq!(breaker.state, CircuitBreakerState::Closed);
        assert!(breaker.can_attempt_request());
    }

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let mut breaker = CircuitBreaker::new(3, 60);

        breaker.record_failure();
        assert_eq!(breaker.state, CircuitBreakerState::Closed);

        breaker.record_failure();
        assert_eq!(breaker.state, CircuitBreakerState::Closed);

        breaker.record_failure();
        assert_eq!(breaker.state, CircuitBreakerState::Open);
    }

    #[test]
    fn test_circuit_breaker_recovery() {
        let mut breaker = CircuitBreaker::new(3, 60);

        breaker.state = CircuitBreakerState::HalfOpen;
        breaker.record_success();

        assert_eq!(breaker.state, CircuitBreakerState::Closed);
        assert_eq!(breaker.failure_count, 0);
    }

    #[test]
    fn test_service_registry() {
        let mut registry = ServiceRegistry::new();

        registry.register(
            "test-service".to_string(),
            ServiceEndpoint {
                name: "Test Service".to_string(),
                base_url: "http://localhost:8080".to_string(),
                health_check_path: Some("/health".to_string()),
                api_version: "v1".to_string(),
                requires_auth: true,
            },
        );

        assert!(registry.get("test-service").is_some());
        assert!(registry.get("unknown-service").is_none());
    }

    #[test]
    fn test_proxy_config_defaults() {
        let config = ProxyConfig::default();
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
    }
}
