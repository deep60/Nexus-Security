use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};

/// Circuit breaker pattern implementation for fault tolerance
/// Prevents cascading failures by failing fast when a service is unhealthy
#[derive(Clone)]
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitBreakerState>>,
    config: CircuitBreakerConfig,
    stats: Arc<RwLock<CircuitBreakerStats>>,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    /// Normal operation - requests are allowed through
    Closed,
    /// Service is failing - requests are rejected immediately
    Open,
    /// Testing if service has recovered - limited requests allowed
    HalfOpen,
}

/// Internal state with timestamps
#[derive(Debug, Clone)]
struct CircuitBreakerState {
    state: State,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    last_state_change: Instant,
    consecutive_successes: u32,
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self {
            state: State::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            last_state_change: Instant::now(),
            consecutive_successes: 0,
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Time to wait before attempting to recover (in seconds)
    pub timeout_seconds: u64,
    /// Number of successful requests needed to close circuit from half-open
    pub success_threshold: u32,
    /// Window for counting failures (in seconds)
    pub failure_window_seconds: u64,
    /// Name/identifier for this circuit breaker
    pub name: String,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout_seconds: 60,
            success_threshold: 3,
            failure_window_seconds: 60,
            name: "default".to_string(),
        }
    }
}

impl CircuitBreakerConfig {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn with_thresholds(mut self, failure_threshold: u32, success_threshold: u32) -> Self {
        self.failure_threshold = failure_threshold;
        self.success_threshold = success_threshold;
        self
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Aggressive configuration - fails fast
    pub fn aggressive() -> Self {
        Self {
            failure_threshold: 2,
            timeout_seconds: 30,
            success_threshold: 5,
            failure_window_seconds: 30,
            name: "aggressive".to_string(),
        }
    }

    /// Lenient configuration - more tolerant to failures
    pub fn lenient() -> Self {
        Self {
            failure_threshold: 10,
            timeout_seconds: 120,
            success_threshold: 2,
            failure_window_seconds: 120,
            name: "lenient".to_string(),
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CircuitBreakerStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rejected_requests: u64,
    pub state_transitions: u64,
    pub current_state: String,
    pub last_failure_time: Option<String>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with given configuration
    pub fn new(config: CircuitBreakerConfig) -> Self {
        info!("Initializing circuit breaker: {}", config.name);

        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState::default())),
            config,
            stats: Arc::new(RwLock::new(CircuitBreakerStats {
                current_state: "Closed".to_string(),
                ..Default::default()
            })),
        }
    }

    /// Create a circuit breaker with default configuration
    pub fn default_with_name(name: String) -> Self {
        Self::new(CircuitBreakerConfig::new(name))
    }

    /// Check if a request is allowed through the circuit breaker
    pub async fn is_request_allowed(&self) -> bool {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;

        stats.total_requests += 1;

        match state.state {
            State::Closed => {
                // Check if we should reset failure count based on time window
                if let Some(last_failure) = state.last_failure_time {
                    if last_failure.elapsed() > Duration::from_secs(self.config.failure_window_seconds) {
                        debug!("Failure window expired, resetting failure count for {}", self.config.name);
                        state.failure_count = 0;
                        state.last_failure_time = None;
                    }
                }
                true
            }
            State::Open => {
                // Check if timeout has elapsed to transition to half-open
                if state.last_state_change.elapsed() >= Duration::from_secs(self.config.timeout_seconds) {
                    info!("Circuit breaker {} timeout elapsed - transitioning to HalfOpen", self.config.name);
                    self.transition_to_half_open(&mut state, &mut stats);
                    true
                } else {
                    stats.rejected_requests += 1;
                    debug!("Circuit breaker {} is Open - rejecting request", self.config.name);
                    false
                }
            }
            State::HalfOpen => {
                // Allow limited requests through to test service health
                debug!("Circuit breaker {} is HalfOpen - allowing test request", self.config.name);
                true
            }
        }
    }

    /// Record a successful operation
    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;

        stats.successful_requests += 1;
        state.success_count += 1;
        state.consecutive_successes += 1;

        match state.state {
            State::HalfOpen => {
                if state.consecutive_successes >= self.config.success_threshold {
                    info!(
                        "Circuit breaker {} recovered - transitioning to Closed ({}  successes)",
                        self.config.name, state.consecutive_successes
                    );
                    self.transition_to_closed(&mut state, &mut stats);
                } else {
                    debug!(
                        "Circuit breaker {} HalfOpen - success {}/{}",
                        self.config.name, state.consecutive_successes, self.config.success_threshold
                    );
                }
            }
            State::Closed => {
                // Reset failure count on success
                if state.failure_count > 0 {
                    debug!("Circuit breaker {} - resetting failure count on success", self.config.name);
                    state.failure_count = 0;
                    state.last_failure_time = None;
                }
            }
            State::Open => {
                // Should not happen as Open state rejects requests
                warn!("Circuit breaker {} recorded success while Open (unexpected)", self.config.name);
            }
        }
    }

    /// Record a failed operation
    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;

        stats.failed_requests += 1;
        state.failure_count += 1;
        state.consecutive_successes = 0;
        state.last_failure_time = Some(Instant::now());

        match state.state {
            State::Closed => {
                if state.failure_count >= self.config.failure_threshold {
                    warn!(
                        "Circuit breaker {} failure threshold reached ({}/{}) - transitioning to Open",
                        self.config.name, state.failure_count, self.config.failure_threshold
                    );
                    self.transition_to_open(&mut state, &mut stats);
                } else {
                    debug!(
                        "Circuit breaker {} - failure {}/{}",
                        self.config.name, state.failure_count, self.config.failure_threshold
                    );
                }
            }
            State::HalfOpen => {
                warn!(
                    "Circuit breaker {} failed during recovery - transitioning back to Open",
                    self.config.name
                );
                self.transition_to_open(&mut state, &mut stats);
            }
            State::Open => {
                // Already open, just increment counter
                debug!("Circuit breaker {} - additional failure while Open", self.config.name);
            }
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        if !self.is_request_allowed().await {
            return Err(CircuitBreakerError::Open);
        }

        match f.await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(error) => {
                self.record_failure().await;
                Err(CircuitBreakerError::ServiceError(error))
            }
        }
    }

    /// Get current state of the circuit breaker
    pub async fn get_state(&self) -> State {
        self.state.read().await.state.clone()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> CircuitBreakerStats {
        self.stats.read().await.clone()
    }

    /// Reset the circuit breaker to closed state
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;

        info!("Manually resetting circuit breaker {}", self.config.name);

        self.transition_to_closed(&mut state, &mut stats);
        state.failure_count = 0;
        state.success_count = 0;
        state.consecutive_successes = 0;
        state.last_failure_time = None;
    }

    /// Force circuit breaker to open state (for testing/maintenance)
    pub async fn force_open(&self) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;

        warn!("Forcibly opening circuit breaker {}", self.config.name);
        self.transition_to_open(&mut state, &mut stats);
    }

    // Transition helpers
    fn transition_to_closed(&self, state: &mut CircuitBreakerState, stats: &mut CircuitBreakerStats) {
        state.state = State::Closed;
        state.failure_count = 0;
        state.consecutive_successes = 0;
        state.last_state_change = Instant::now();
        stats.state_transitions += 1;
        stats.current_state = "Closed".to_string();
    }

    fn transition_to_open(&self, state: &mut CircuitBreakerState, stats: &mut CircuitBreakerStats) {
        state.state = State::Open;
        state.consecutive_successes = 0;
        state.last_state_change = Instant::now();
        stats.state_transitions += 1;
        stats.current_state = "Open".to_string();

        if let Some(last_failure) = state.last_failure_time {
            stats.last_failure_time = Some(format!("{:?} ago", last_failure.elapsed()));
        }
    }

    fn transition_to_half_open(&self, state: &mut CircuitBreakerState, stats: &mut CircuitBreakerStats) {
        state.state = State::HalfOpen;
        state.consecutive_successes = 0;
        state.last_state_change = Instant::now();
        stats.state_transitions += 1;
        stats.current_state = "HalfOpen".to_string();
    }
}

/// Circuit breaker errors
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    /// Circuit breaker is open - request rejected
    Open,
    /// The underlying service returned an error
    ServiceError(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "Circuit breaker is open - request rejected"),
            Self::ServiceError(e) => write!(f, "Service error: {}", e),
        }
    }
}

impl<E: std::error::Error> std::error::Error for CircuitBreakerError<E> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout_seconds: 1,
            failure_window_seconds: 60,
            name: "test".to_string(),
        };

        let cb = CircuitBreaker::new(config);

        // Initially closed
        assert_eq!(cb.get_state().await, State::Closed);
        assert!(cb.is_request_allowed().await);

        // Record failures to open circuit
        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_failure().await;

        // Should be open now
        assert_eq!(cb.get_state().await, State::Open);
        assert!(!cb.is_request_allowed().await);

        // Wait for timeout
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should transition to half-open
        assert!(cb.is_request_allowed().await);
        assert_eq!(cb.get_state().await, State::HalfOpen);

        // Record successes to close circuit
        cb.record_success().await;
        cb.record_success().await;

        // Should be closed now
        assert_eq!(cb.get_state().await, State::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_call() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());

        // Successful call
        let result = cb.call(async { Ok::<i32, String>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Failed call
        let result = cb.call(async { Err::<i32, String>("error".to_string()) }).await;
        assert!(matches!(result, Err(CircuitBreakerError::ServiceError(_))));
    }

    #[tokio::test]
    async fn test_circuit_breaker_stats() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());

        cb.record_success().await;
        cb.record_failure().await;

        let stats = cb.get_stats().await;
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.failed_requests, 1);
    }

    #[tokio::test]
    async fn test_reset() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            ..CircuitBreakerConfig::default()
        });

        cb.record_failure().await;
        cb.record_failure().await;

        assert_eq!(cb.get_state().await, State::Open);

        cb.reset().await;

        assert_eq!(cb.get_state().await, State::Closed);
    }
}
