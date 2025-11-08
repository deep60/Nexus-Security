use axum::{
    body::Body,
    extract::{ConnectInfo, MatchedPath},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::time::Instant;
use tracing::{error, info, warn, Span};
use uuid::Uuid;

/// Request logging middleware
pub async fn logging_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    matched_path: Option<MatchedPath>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();
    let path = matched_path
        .as_ref()
        .map(|p| p.as_str())
        .unwrap_or_else(|| uri.path());

    // Generate request ID
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Log incoming request
    info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        version = ?version,
        client_ip = %addr,
        "Incoming request"
    );

    // Process request
    let response = next.run(request).await;
    let status = response.status();
    let duration = start.elapsed();

    // Log response based on status code
    match status.as_u16() {
        200..=299 => {
            info!(
                request_id = %request_id,
                method = %method,
                path = %path,
                status = %status,
                duration_ms = %duration.as_millis(),
                "Request completed successfully"
            );
        }
        400..=499 => {
            warn!(
                request_id = %request_id,
                method = %method,
                path = %path,
                status = %status,
                duration_ms = %duration.as_millis(),
                "Client error"
            );
        }
        500..=599 => {
            error!(
                request_id = %request_id,
                method = %method,
                path = %path,
                status = %status,
                duration_ms = %duration.as_millis(),
                "Server error"
            );
        }
        _ => {
            info!(
                request_id = %request_id,
                method = %method,
                path = %path,
                status = %status,
                duration_ms = %duration.as_millis(),
                "Request completed"
            );
        }
    }

    // Add request ID to response headers
    let mut response = response;
    if let Ok(header_value) = request_id.parse() {
        response.headers_mut().insert("x-request-id", header_value);
    }

    response
}

/// Request ID injection middleware
pub async fn request_id_middleware(
    mut request: Request<Body>,
    next: Next,
) -> Response {
    // Get or generate request ID
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Insert request ID into request extensions
    request.extensions_mut().insert(RequestId(request_id.clone()));

    // Process request
    let mut response = next.run(request).await;

    // Add request ID to response headers
    if let Ok(header_value) = request_id.parse() {
        response.headers_mut().insert("x-request-id", header_value);
    }

    response
}

/// Request ID wrapper
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Structured logging for security events
pub fn log_security_event(
    event_type: &str,
    user_id: Option<Uuid>,
    ip_address: &str,
    details: &str,
) {
    warn!(
        event_type = %event_type,
        user_id = ?user_id,
        ip_address = %ip_address,
        details = %details,
        "Security event"
    );
}

/// Log authentication attempts
pub fn log_auth_attempt(
    success: bool,
    email: &str,
    ip_address: &str,
    reason: Option<&str>,
) {
    if success {
        info!(
            email = %email,
            ip_address = %ip_address,
            "Successful authentication"
        );
    } else {
        warn!(
            email = %email,
            ip_address = %ip_address,
            reason = ?reason,
            "Failed authentication attempt"
        );
    }
}

/// Log API key usage
pub fn log_api_key_usage(
    api_key_id: &str,
    endpoint: &str,
    ip_address: &str,
) {
    info!(
        api_key_id = %api_key_id,
        endpoint = %endpoint,
        ip_address = %ip_address,
        "API key usage"
    );
}

/// Log rate limit violations
pub fn log_rate_limit_exceeded(
    identifier: &str,
    endpoint: &str,
    limit: u32,
    ip_address: &str,
) {
    warn!(
        identifier = %identifier,
        endpoint = %endpoint,
        limit = %limit,
        ip_address = %ip_address,
        "Rate limit exceeded"
    );
}

/// Log blockchain transactions
pub fn log_blockchain_transaction(
    transaction_type: &str,
    user_id: Uuid,
    amount: &str,
    tx_hash: Option<&str>,
    status: &str,
) {
    info!(
        transaction_type = %transaction_type,
        user_id = %user_id,
        amount = %amount,
        tx_hash = ?tx_hash,
        status = %status,
        "Blockchain transaction"
    );
}

/// Log bounty lifecycle events
pub fn log_bounty_event(
    event_type: &str,
    bounty_id: Uuid,
    user_id: Uuid,
    details: Option<&str>,
) {
    info!(
        event_type = %event_type,
        bounty_id = %bounty_id,
        user_id = %user_id,
        details = ?details,
        "Bounty event"
    );
}

/// Log analysis submissions
pub fn log_analysis_submission(
    analysis_id: Uuid,
    engine_id: Uuid,
    verdict: &str,
    confidence: f64,
    stake_amount: &str,
) {
    info!(
        analysis_id = %analysis_id,
        engine_id = %engine_id,
        verdict = %verdict,
        confidence = %confidence,
        stake_amount = %stake_amount,
        "Analysis submission"
    );
}

/// Log consensus events
pub fn log_consensus_reached(
    analysis_id: Uuid,
    final_verdict: &str,
    confidence: f64,
    participant_count: u32,
) {
    info!(
        analysis_id = %analysis_id,
        final_verdict = %final_verdict,
        confidence = %confidence,
        participant_count = %participant_count,
        "Consensus reached"
    );
}

/// Log database errors with context
pub fn log_database_error(
    operation: &str,
    table: &str,
    error: &str,
    request_id: Option<&str>,
) {
    error!(
        operation = %operation,
        table = %table,
        error = %error,
        request_id = ?request_id,
        "Database error"
    );
}

/// Log external API calls
pub fn log_external_api_call(
    service: &str,
    endpoint: &str,
    status: u16,
    duration_ms: u64,
) {
    if status >= 200 && status < 300 {
        info!(
            service = %service,
            endpoint = %endpoint,
            status = %status,
            duration_ms = %duration_ms,
            "External API call succeeded"
        );
    } else {
        warn!(
            service = %service,
            endpoint = %endpoint,
            status = %status,
            duration_ms = %duration_ms,
            "External API call failed"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_generation() {
        let id1 = Uuid::new_v4().to_string();
        let id2 = Uuid::new_v4().to_string();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_request_id_display() {
        let request_id = RequestId("test-id-123".to_string());
        assert_eq!(format!("{}", request_id), "test-id-123");
    }
}
