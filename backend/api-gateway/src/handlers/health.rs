use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::AppState;

/// Health check response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: i64,
    pub services: HashMap<String, ServiceHealth>,
}

/// Individual service health status
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: String,
    pub response_time_ms: Option<u64>,
    pub message: Option<String>,
}

impl ServiceHealth {
    pub fn healthy(response_time_ms: u64) -> Self {
        Self {
            status: "healthy".to_string(),
            response_time_ms: Some(response_time_ms),
            message: None,
        }
    }

    pub fn unhealthy(message: String) -> Self {
        Self {
            status: "unhealthy".to_string(),
            response_time_ms: None,
            message: Some(message),
        }
    }

    pub fn degraded(response_time_ms: u64, message: String) -> Self {
        Self {
            status: "degraded".to_string(),
            response_time_ms: Some(response_time_ms),
            message: Some(message),
        }
    }
}

/// Main health check endpoint
///
/// GET /api/v1/health
pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    let mut services = HashMap::new();

    // Check database health
    let db_start = std::time::Instant::now();
    let db_health = match state.db.health_check().await {
        true => ServiceHealth::healthy(db_start.elapsed().as_millis() as u64),
        false => ServiceHealth::unhealthy("Database connection failed".to_string()),
    };
    services.insert("database".to_string(), db_health);

    // Check Redis health
    let redis_start = std::time::Instant::now();
    let redis_health = match state.redis.health_check().await {
        true => ServiceHealth::healthy(redis_start.elapsed().as_millis() as u64),
        false => ServiceHealth::unhealthy("Redis connection failed".to_string()),
    };
    services.insert("redis".to_string(), redis_health);

    // Check blockchain health
    let blockchain_start = std::time::Instant::now();
    let blockchain_health = match state.blockchain.health_check().await {
        true => ServiceHealth::healthy(blockchain_start.elapsed().as_millis() as u64),
        false => ServiceHealth::unhealthy("Blockchain connection failed".to_string()),
    };
    services.insert("blockchain".to_string(), blockchain_health);

    // Determine overall status
    let all_healthy = services.values().all(|s| s.status == "healthy");
    let any_unhealthy = services.values().any(|s| s.status == "unhealthy");

    let overall_status = if all_healthy {
        "healthy"
    } else if any_unhealthy {
        "unhealthy"
    } else {
        "degraded"
    };

    let response = HealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now(),
        uptime_seconds: start_time.elapsed().as_secs() as i64, // TODO: Track actual uptime
        services,
    };

    let status_code = match overall_status {
        "healthy" => StatusCode::OK,
        "degraded" => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    Ok((status_code, Json(response)).into())
}

/// Readiness check endpoint
///
/// GET /api/v1/ready
pub async fn readiness_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check if all critical services are ready
    let db_ready = state.db.health_check().await;
    let redis_ready = state.redis.health_check().await;

    if db_ready && redis_ready {
        Ok(Json(serde_json::json!({
            "ready": true,
            "timestamp": Utc::now()
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Liveness check endpoint
///
/// GET /api/v1/alive
pub async fn liveness_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "alive": true,
        "timestamp": Utc::now()
    }))
}

/// Detailed metrics endpoint (for monitoring systems)
///
/// GET /api/v1/metrics
pub async fn metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement actual metrics collection
    Ok(Json(serde_json::json!({
        "timestamp": Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "metrics": {
            "requests_total": 0,
            "requests_active": 0,
            "errors_total": 0,
            "response_time_avg_ms": 0,
        }
    })))
}
