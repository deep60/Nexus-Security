use axum::{body::Body, extract::MatchedPath, http::Request, middleware::Next, response::Response};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Global metrics collector
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    pub total_requests: Arc<AtomicU64>,
    pub active_requests: Arc<AtomicU64>,
    pub total_errors: Arc<AtomicU64>,
    pub request_durations: Arc<RwLock<Vec<u64>>>,
    pub endpoint_stats: Arc<RwLock<HashMap<String, EndpointMetrics>>>,
    pub status_code_counts: Arc<RwLock<HashMap<u16, u64>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            active_requests: Arc::new(AtomicU64::new(0)),
            total_errors: Arc::new(AtomicU64::new(0)),
            request_durations: Arc::new(RwLock::new(Vec::new())),
            endpoint_stats: Arc::new(RwLock::new(HashMap::new())),
            status_code_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a request
    pub async fn record_request(&self, endpoint: String, duration_ms: u64, status_code: u16) {
        // Increment total requests
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        // Record duration
        let mut durations = self.request_durations.write().await;
        durations.push(duration_ms);

        // Keep only last 10000 requests for memory efficiency
        if durations.len() > 10000 {
            durations.drain(0..5000);
        }
        drop(durations);

        // Update endpoint stats
        let mut endpoint_stats = self.endpoint_stats.write().await;
        let stats = endpoint_stats
            .entry(endpoint)
            .or_insert_with(EndpointMetrics::new);
        stats.record_request(duration_ms, status_code);
        drop(endpoint_stats);

        // Update status code counts
        let mut status_counts = self.status_code_counts.write().await;
        *status_counts.entry(status_code).or_insert(0) += 1;

        // Track errors
        if status_code >= 500 {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get current metrics snapshot
    pub async fn get_snapshot(&self) -> MetricsSnapshot {
        let durations = self.request_durations.read().await;
        let avg_duration = if !durations.is_empty() {
            durations.iter().sum::<u64>() / durations.len() as u64
        } else {
            0
        };

        let p95_duration = calculate_percentile(&durations, 95.0);
        let p99_duration = calculate_percentile(&durations, 99.0);

        drop(durations);

        MetricsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            active_requests: self.active_requests.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            avg_response_time_ms: avg_duration,
            p95_response_time_ms: p95_duration,
            p99_response_time_ms: p99_duration,
            error_rate: self.calculate_error_rate().await,
        }
    }

    /// Calculate error rate
    async fn calculate_error_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        let errors = self.total_errors.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            (errors as f64 / total as f64) * 100.0
        }
    }

    /// Get endpoint-specific metrics
    pub async fn get_endpoint_metrics(&self) -> HashMap<String, EndpointMetrics> {
        self.endpoint_stats.read().await.clone()
    }

    /// Get status code distribution
    pub async fn get_status_code_distribution(&self) -> HashMap<u16, u64> {
        self.status_code_counts.read().await.clone()
    }

    /// Reset all metrics (useful for testing)
    pub async fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.active_requests.store(0, Ordering::Relaxed);
        self.total_errors.store(0, Ordering::Relaxed);
        self.request_durations.write().await.clear();
        self.endpoint_stats.write().await.clear();
        self.status_code_counts.write().await.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-endpoint metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct EndpointMetrics {
    pub total_requests: u64,
    pub total_errors: u64,
    pub avg_duration_ms: u64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub last_request_at: Option<std::time::SystemTime>,
    durations: Vec<u64>,
}

impl EndpointMetrics {
    fn new() -> Self {
        Self {
            total_requests: 0,
            total_errors: 0,
            avg_duration_ms: 0,
            min_duration_ms: u64::MAX,
            max_duration_ms: 0,
            last_request_at: None,
            durations: Vec::new(),
        }
    }

    fn record_request(&mut self, duration_ms: u64, status_code: u16) {
        self.total_requests += 1;
        self.durations.push(duration_ms);

        if status_code >= 500 {
            self.total_errors += 1;
        }

        self.min_duration_ms = self.min_duration_ms.min(duration_ms);
        self.max_duration_ms = self.max_duration_ms.max(duration_ms);

        // Recalculate average
        self.avg_duration_ms = if !self.durations.is_empty() {
            self.durations.iter().sum::<u64>() / self.durations.len() as u64
        } else {
            0
        };

        // Keep only last 1000 requests
        if self.durations.len() > 1000 {
            self.durations.drain(0..500);
        }

        self.last_request_at = Some(std::time::SystemTime::now());
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.total_errors as f64 / self.total_requests as f64) * 100.0
        }
    }
}

/// Metrics snapshot for reporting
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub active_requests: u64,
    pub total_errors: u64,
    pub avg_response_time_ms: u64,
    pub p95_response_time_ms: u64,
    pub p99_response_time_ms: u64,
    pub error_rate: f64,
}

/// Metrics collection middleware
pub async fn metrics_middleware(
    collector: Arc<MetricsCollector>,
    matched_path: Option<MatchedPath>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let path = matched_path
        .as_ref()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

    // Increment active requests
    collector.active_requests.fetch_add(1, Ordering::Relaxed);

    // Process request
    let response = next.run(request).await;
    let status = response.status();

    // Decrement active requests
    collector.active_requests.fetch_sub(1, Ordering::Relaxed);

    // Record metrics
    let duration = start.elapsed().as_millis() as u64;
    let collector_clone = collector.clone();
    tokio::spawn(async move {
        collector_clone
            .record_request(path, duration, status.as_u16())
            .await;
    });

    response
}

/// Calculate percentile from sorted durations
fn calculate_percentile(durations: &[u64], percentile: f64) -> u64 {
    if durations.is_empty() {
        return 0;
    }

    let mut sorted = durations.to_vec();
    sorted.sort_unstable();

    let index = ((percentile / 100.0) * sorted.len() as f64).ceil() as usize - 1;
    sorted.get(index).copied().unwrap_or(0)
}

/// Prometheus-compatible metrics export
pub fn export_prometheus_metrics(
    snapshot: &MetricsSnapshot,
    endpoint_metrics: &HashMap<String, EndpointMetrics>,
) -> String {
    let mut output = String::new();

    // Total requests
    output.push_str(&format!(
        "# HELP http_requests_total Total number of HTTP requests\n"
    ));
    output.push_str(&format!("# TYPE http_requests_total counter\n"));
    output.push_str(&format!(
        "http_requests_total {}\n\n",
        snapshot.total_requests
    ));

    // Active requests
    output.push_str(&format!(
        "# HELP http_requests_active Number of active HTTP requests\n"
    ));
    output.push_str(&format!("# TYPE http_requests_active gauge\n"));
    output.push_str(&format!(
        "http_requests_active {}\n\n",
        snapshot.active_requests
    ));

    // Total errors
    output.push_str(&format!(
        "# HELP http_requests_errors_total Total number of HTTP errors\n"
    ));
    output.push_str(&format!("# TYPE http_requests_errors_total counter\n"));
    output.push_str(&format!(
        "http_requests_errors_total {}\n\n",
        snapshot.total_errors
    ));

    // Average response time
    output.push_str(&format!(
        "# HELP http_response_time_ms_avg Average response time in milliseconds\n"
    ));
    output.push_str(&format!("# TYPE http_response_time_ms_avg gauge\n"));
    output.push_str(&format!(
        "http_response_time_ms_avg {}\n\n",
        snapshot.avg_response_time_ms
    ));

    // P95 response time
    output.push_str(&format!(
        "# HELP http_response_time_ms_p95 95th percentile response time\n"
    ));
    output.push_str(&format!("# TYPE http_response_time_ms_p95 gauge\n"));
    output.push_str(&format!(
        "http_response_time_ms_p95 {}\n\n",
        snapshot.p95_response_time_ms
    ));

    // Error rate
    output.push_str(&format!(
        "# HELP http_error_rate_percent Error rate percentage\n"
    ));
    output.push_str(&format!("# TYPE http_error_rate_percent gauge\n"));
    output.push_str(&format!(
        "http_error_rate_percent {}\n\n",
        snapshot.error_rate
    ));

    // Per-endpoint metrics
    output.push_str(&format!(
        "# HELP http_requests_by_endpoint_total Requests per endpoint\n"
    ));
    output.push_str(&format!("# TYPE http_requests_by_endpoint_total counter\n"));
    for (endpoint, metrics) in endpoint_metrics {
        output.push_str(&format!(
            "http_requests_by_endpoint_total{{endpoint=\"{}\"}} {}\n",
            endpoint, metrics.total_requests
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        collector
            .record_request("/api/test".to_string(), 100, 200)
            .await;
        collector
            .record_request("/api/test".to_string(), 150, 200)
            .await;

        let snapshot = collector.get_snapshot().await;
        assert_eq!(snapshot.total_requests, 2);
        assert_eq!(snapshot.total_errors, 0);
    }

    #[tokio::test]
    async fn test_error_tracking() {
        let collector = MetricsCollector::new();

        collector
            .record_request("/api/test".to_string(), 100, 200)
            .await;
        collector
            .record_request("/api/test".to_string(), 150, 500)
            .await;

        let snapshot = collector.get_snapshot().await;
        assert_eq!(snapshot.total_errors, 1);
        assert!(snapshot.error_rate > 0.0);
    }

    #[test]
    fn test_percentile_calculation() {
        let durations = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        let p95 = calculate_percentile(&durations, 95.0);
        assert!(p95 >= 90);
    }
}
