use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn get_pending_payments(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    match state.payment_service.list_pending().await {
        Ok(payments) => (StatusCode::OK, Json(json!({ "payments": payments }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_failed_payments(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    let rows = sqlx::query_as::<_, crate::models::Payment>(
        "SELECT * FROM payments WHERE status = 'failed' ORDER BY updated_at DESC LIMIT 100",
    )
    .fetch_all(state.payment_service.db_pool())
    .await;

    match rows {
        Ok(payments) => (StatusCode::OK, Json(json!({ "payments": payments }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn retry_payment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let payment_id = match uuid::Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "invalid id" })),
            )
        }
    };

    // Reset a failed payment to pending so the processor picks it up again.
    let res = sqlx::query(
        "UPDATE payments SET status = 'pending', updated_at = NOW() WHERE id = $1 AND status = 'failed'",
    )
    .bind(payment_id)
    .execute(state.payment_service.db_pool())
    .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "message": "payment requeued" })),
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "no failed payment with that id" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_treasury_balance(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    let treasury = state
        .payment_service
        .config()
        .blockchain
        .treasury_address
        .clone();

    match state.payment_service.get_token_balance(&treasury).await {
        Ok(balance) => (
            StatusCode::OK,
            Json(json!({ "address": treasury, "balance": format!("{}", balance) })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("failed to read treasury balance: {}", e) })),
        ),
    }
}
