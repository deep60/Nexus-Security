use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

fn err_response(e: ConsensusError) -> (StatusCode, Json<Value>) {
    let status = match &e {
        ConsensusError::NotFound(_) => StatusCode::NOT_FOUND,
        ConsensusError::ValidationError(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, Json(json!({ "error": e.to_string() })))
}

fn parse_uuid(s: &str) -> Result<Uuid, (StatusCode, Json<Value>)> {
    Uuid::parse_str(s)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({ "error": "invalid id" }))))
}

/// Extract the acting user from the `x-user-id` header (set by the gateway),
/// falling back to a nil UUID when absent.
fn actor(headers: &HeaderMap) -> Uuid {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .unwrap_or_else(Uuid::nil)
}

pub async fn create_dispute(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateDisputeRequest>,
) -> (StatusCode, Json<Value>) {
    if payload.reason.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "reason is required" })),
        );
    }

    let initiator = actor(&headers);
    match state
        .consensus_service
        .create_dispute(&payload, initiator)
        .await
    {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({ "dispute_id": id, "status": "open" })),
        ),
        Err(e) => err_response(e),
    }
}

pub async fn get_dispute(
    State(state): State<Arc<AppState>>,
    Path(dispute_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let dispute_id = match parse_uuid(&dispute_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match state.consensus_service.get_dispute(dispute_id).await {
        Ok(Some(d)) => (StatusCode::OK, Json(json!(d))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "dispute not found" })),
        ),
        Err(e) => err_response(e),
    }
}

pub async fn resolve_dispute(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(dispute_id): Path<String>,
    Json(payload): Json<ResolveDisputeRequest>,
) -> (StatusCode, Json<Value>) {
    let dispute_id = match parse_uuid(&dispute_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let resolver = actor(&headers);
    match state
        .consensus_service
        .resolve_dispute(dispute_id, &payload, resolver)
        .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({ "dispute_id": dispute_id, "status": "resolved" })),
        ),
        Err(e) => err_response(e),
    }
}

pub async fn get_bounty_disputes(
    State(state): State<Arc<AppState>>,
    Path(bounty_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let bounty_id = match parse_uuid(&bounty_id) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match state.consensus_service.get_bounty_disputes(bounty_id).await {
        Ok(disputes) => (StatusCode::OK, Json(json!({ "disputes": disputes }))),
        Err(e) => err_response(e),
    }
}
