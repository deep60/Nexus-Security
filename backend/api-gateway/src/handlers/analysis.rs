use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;

/// Query parameters for listing analyses
#[derive(Debug, Deserialize)]
pub struct ListAnalysesQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub status: Option<String>,
    pub verdict: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

/// Response for analysis list
#[derive(Debug, Serialize)]
pub struct AnalysisListResponse {
    pub analyses: Vec<AnalysisSummary>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

/// Summary of an analysis
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AnalysisSummary {
    pub id: Uuid,
    pub file_hash: Option<String>,
    pub status: Option<String>,
    pub verdict: Option<String>,
    pub confidence: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Analysis stats
#[derive(Debug, Serialize)]
pub struct AnalysisStats {
    pub total_analyses: i64,
    pub pending: i64,
    pub completed: i64,
    pub malicious_count: i64,
    pub benign_count: i64,
    pub suspicious_count: i64,
}

/// Get analysis by ID
pub async fn get_analysis(
    State(state): State<AppState>,
    Path(analysis_id): Path<Uuid>,
) -> Result<Json<AnalysisSummary>, StatusCode> {
    let row = sqlx::query_as::<_, AnalysisSummary>(
        "SELECT id, file_hash, status, verdict, confidence::float8 as confidence, created_at, completed_at FROM analyses WHERE id = $1"
    )
    .bind(analysis_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("DB error fetching analysis: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(analysis) => Ok(Json(analysis)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Get analysis details (same as get_analysis for now)
pub async fn get_analysis_details(
    state: State<AppState>,
    path: Path<Uuid>,
) -> Result<Json<AnalysisSummary>, StatusCode> {
    get_analysis(state, path).await
}

/// List all analyses with filters
pub async fn list_analyses(
    State(state): State<AppState>,
    Query(params): Query<ListAnalysesQuery>,
) -> Result<Json<AnalysisListResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = (page.saturating_sub(1)) * limit;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analyses")
        .fetch_one(state.db.pool())
        .await
        .map_err(|e| {
            tracing::error!("DB error counting analyses: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let analyses = sqlx::query_as::<_, AnalysisSummary>(
        "SELECT id, file_hash, status, verdict, confidence::float8 as confidence, created_at, completed_at
         FROM analyses ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("DB error listing analyses: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(AnalysisListResponse {
        analyses,
        total,
        page,
        limit,
    }))
}

/// Get analysis statistics
pub async fn get_analysis_stats(
    State(state): State<AppState>,
) -> Result<Json<AnalysisStats>, StatusCode> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analyses")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    let pending: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analyses WHERE status = 'pending'")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    let completed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analyses WHERE status = 'completed'")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    let malicious: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analyses WHERE verdict = 'malicious'")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    let benign: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analyses WHERE verdict = 'benign'")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    let suspicious: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analyses WHERE verdict = 'suspicious'")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    Ok(Json(AnalysisStats {
        total_analyses: total,
        pending,
        completed,
        malicious_count: malicious,
        benign_count: benign,
        suspicious_count: suspicious,
    }))
}

/// Get analyses by bounty
pub async fn get_analyses_by_bounty(
    State(state): State<AppState>,
    Path(bounty_id): Path<Uuid>,
) -> Result<Json<Vec<AnalysisSummary>>, StatusCode> {
    let analyses = sqlx::query_as::<_, AnalysisSummary>(
        "SELECT id, file_hash, status, verdict, confidence::float8 as confidence, created_at, completed_at
         FROM analyses WHERE bounty_id = $1 ORDER BY created_at DESC"
    )
    .bind(bounty_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("DB error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(analyses))
}

/// Get analyses by file hash
pub async fn get_analyses_by_hash(
    State(state): State<AppState>,
    Path(file_hash): Path<String>,
) -> Result<Json<Vec<AnalysisSummary>>, StatusCode> {
    let analyses = sqlx::query_as::<_, AnalysisSummary>(
        "SELECT id, file_hash, status, verdict, confidence::float8 as confidence, created_at, completed_at
         FROM analyses WHERE file_hash = $1 ORDER BY created_at DESC"
    )
    .bind(&file_hash)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("DB error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(analyses))
}

/// Submit analysis
pub async fn submit_analysis(
    State(_state): State<AppState>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Dispute analysis
pub async fn dispute_analysis(
    State(_state): State<AppState>,
    Path(_analysis_id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}
