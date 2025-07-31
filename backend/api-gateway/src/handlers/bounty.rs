use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::models::bounty::{Bounty, BountyStatus, BountySubmission, EngineVerdict};
use crate::services::{database::DatabaseService, blockchain::BlockchainService};

// Request/Response DTOs
#[derive(Deserialize)]
pub struct CreateBountyRequest {
    pub title: String,
    pub description: String,
    pub file_hash: Option<String>,
    pub url: Option<String>,
    pub reward_amount: u64,       // Amount in Wei
    pub deadline: DateTime<Utc>,
    pub required_consensus: u8,          // Minimum number of engines needed
    pub confidence_threshold: f32,       // Minimum confidence score (0.0-1.0)
}

#[derive(Deserialize)]
pub struct SubmitAnalysisRequest {
    pub engine_id: String,
    pub verdict: String,          // "malicious", "benign", "suspicious"
    pub confidence: f32,          // 0.0-1.0
    pub analysis_details: serde_json::Value,
    pub stake
}