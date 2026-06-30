//! Minimal, dependency-light Prometheus exposition helper shared by every
//! service. Deliberately std-only (no axum/tower coupling) so it works across
//! every service regardless of their axum/tower-http versions. Each service
//! wires a tiny `/metrics` handler that calls [`MetricsRegistry::render_prometheus`].
//!
//! Metrics emitted for every service:
//!   - `verdyx_service_up`              (gauge, always 1 while serving)
//!   - `verdyx_service_uptime_seconds`  (gauge)
//!   - `verdyx_build_info{version=...}` (gauge, value 1)
//!   - `verdyx_http_requests_total`     (counter)
//!   - `verdyx_http_errors_total`       (counter)

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Process-wide, cheap metrics registry. Clone freely (Arc inside).
#[derive(Clone)]
pub struct MetricsRegistry {
    inner: Arc<Inner>,
}

struct Inner {
    service: String,
    version: String,
    started: Instant,
    http_requests_total: AtomicU64,
    http_errors_total: AtomicU64,
}

impl MetricsRegistry {
    pub fn new(service: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(Inner {
                service: service.into(),
                version: version.into(),
                started: Instant::now(),
                http_requests_total: AtomicU64::new(0),
                http_errors_total: AtomicU64::new(0),
            }),
        }
    }

    pub fn inc_request(&self) {
        self.inner
            .http_requests_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_error(&self) {
        self.inner.http_errors_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.inner.started.elapsed().as_secs()
    }

    /// Render the current state in Prometheus text exposition format.
    pub fn render_prometheus(&self) -> String {
        let svc = &self.inner.service;
        let requests = self.inner.http_requests_total.load(Ordering::Relaxed);
        let errors = self.inner.http_errors_total.load(Ordering::Relaxed);
        let uptime = self.uptime_seconds();

        let mut out = String::with_capacity(512);

        out.push_str("# HELP verdyx_service_up 1 if the service is serving requests\n");
        out.push_str("# TYPE verdyx_service_up gauge\n");
        out.push_str(&format!("verdyx_service_up{{service=\"{svc}\"}} 1\n"));

        out.push_str("# HELP verdyx_service_uptime_seconds Seconds since process start\n");
        out.push_str("# TYPE verdyx_service_uptime_seconds gauge\n");
        out.push_str(&format!(
            "verdyx_service_uptime_seconds{{service=\"{svc}\"}} {uptime}\n"
        ));

        out.push_str("# HELP verdyx_build_info Build metadata\n");
        out.push_str("# TYPE verdyx_build_info gauge\n");
        out.push_str(&format!(
            "verdyx_build_info{{service=\"{svc}\",version=\"{}\"}} 1\n",
            self.inner.version
        ));

        out.push_str("# HELP verdyx_http_requests_total Total HTTP requests handled\n");
        out.push_str("# TYPE verdyx_http_requests_total counter\n");
        out.push_str(&format!(
            "verdyx_http_requests_total{{service=\"{svc}\"}} {requests}\n"
        ));

        out.push_str("# HELP verdyx_http_errors_total Total HTTP 5xx responses\n");
        out.push_str("# TYPE verdyx_http_errors_total counter\n");
        out.push_str(&format!(
            "verdyx_http_errors_total{{service=\"{svc}\"}} {errors}\n"
        ));

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_expected_metrics() {
        let m = MetricsRegistry::new("test-service", "1.2.3");
        m.inc_request();
        m.inc_request();
        m.inc_error();

        let text = m.render_prometheus();
        assert!(text.contains("verdyx_service_up{service=\"test-service\"} 1"));
        assert!(text.contains("verdyx_http_requests_total{service=\"test-service\"} 2"));
        assert!(text.contains("verdyx_http_errors_total{service=\"test-service\"} 1"));
        assert!(text.contains("version=\"1.2.3\""));
    }
}
