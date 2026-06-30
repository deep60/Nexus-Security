//! Axum middleware that wires HTTP request/error counters into
//! [`crate::MetricsRegistry`]. Enable with the `axum-mw` feature.
//!
//! Each service installs the layer once and the registry's
//! `verdyx_http_requests_total` and `verdyx_http_errors_total` counters
//! become live, exposed via that service's `/metrics` endpoint.

use crate::MetricsRegistry;
use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// Increments `requests_total` for every request and `errors_total` for any
/// response with HTTP status >= 500. Intentionally a flat counter (no path
/// labels) — Prometheus cardinality stays bounded and the per-endpoint
/// breakdown lives in the gateway's own `MetricsCollector`.
pub async fn track(
    axum::extract::State(metrics): axum::extract::State<MetricsRegistry>,
    req: Request<Body>,
    next: Next,
) -> Response {
    metrics.inc_request();
    let response = next.run(req).await;
    if response.status().is_server_error() {
        metrics.inc_error();
    }
    response
}

/// Variant for services whose router state is not the bare `MetricsRegistry`.
/// Pass the registry in directly. Use with `axum::middleware::from_fn`:
///
/// ```ignore
/// let registry = shared::MetricsRegistry::new("svc", "0.1.0");
/// let app = Router::new()
///     .layer(axum::middleware::from_fn({
///         let registry = registry.clone();
///         move |req, next| shared::metrics_mw::track_with(registry.clone(), req, next)
///     }));
/// ```
pub async fn track_with(metrics: MetricsRegistry, req: Request<Body>, next: Next) -> Response {
    metrics.inc_request();
    let response = next.run(req).await;
    if response.status().is_server_error() {
        metrics.inc_error();
    }
    response
}

/// True when the status is a server error (5xx). Tiny helper kept here so
/// downstream code can match the same definition.
pub fn is_server_error(status: StatusCode) -> bool {
    status.is_server_error()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pure-logic check on the helper exported alongside the middleware.
    // Driving an axum Router through this middleware requires `tower::Service`,
    // which would add a dev-dep just to test trivial counter increments — not
    // worth it. The end-to-end behavior is exercised by the smoke test.
    #[test]
    fn server_error_classifier_matches_expected_range() {
        assert!(!is_server_error(StatusCode::OK));
        assert!(!is_server_error(StatusCode::BAD_REQUEST));
        assert!(!is_server_error(StatusCode::NOT_FOUND));
        assert!(is_server_error(StatusCode::INTERNAL_SERVER_ERROR));
        assert!(is_server_error(StatusCode::BAD_GATEWAY));
        assert!(is_server_error(StatusCode::SERVICE_UNAVAILABLE));
    }

    #[test]
    fn registry_counters_increment_in_isolation() {
        let registry = MetricsRegistry::new("test-svc", "0.0.1");
        registry.inc_request();
        registry.inc_request();
        registry.inc_error();
        let exposed = registry.render_prometheus();
        assert!(
            exposed.contains("verdyx_http_requests_total{service=\"test-svc\"} 2"),
            "requests counter wrong: {exposed}"
        );
        assert!(
            exposed.contains("verdyx_http_errors_total{service=\"test-svc\"} 1"),
            "errors counter wrong: {exposed}"
        );
    }
}
