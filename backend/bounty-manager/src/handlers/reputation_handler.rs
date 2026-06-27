use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::handlers::bounty_crud::BountyManagerState;
use crate::models::ReputationModel;

// Reputation scoring weights and parameters
const ACCURACY_WEIGHT: f64 = 0.40;
const TIMELINESS_WEIGHT: f64 = 0.25;
const CONSISTENCY_WEIGHT: f64 = 0.20;
const VOLUME_WEIGHT: f64 = 0.15;

const MIN_SUBMISSIONS_FOR_RELIABLE_SCORE: i32 = 10;
const REPUTATION_DECAY_FACTOR: f64 = 0.95; // Monthly decay
const MAX_REPUTATION_SCORE: f64 = 1000.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineReputation {
    pub engine_id: Uuid,
    pub engine_name: String,
    pub current_score: f64,
    pub historical_high: f64,
    pub total_submissions: u32,
    pub successful_submissions: u32,
    pub false_positives: u32,
    pub false_negatives: u32,
    pub average_response_time: f64, // in minutes
    pub specialty_areas: Vec<ThreatCategory>,
    pub tier: ReputationTier,
    pub last_updated: DateTime<Utc>,
    pub monthly_scores: Vec<MonthlyScore>,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyScore {
    pub month: String, // YYYY-MM format
    pub score: f64,
    pub submissions_count: u32,
    pub accuracy_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub accuracy_rate: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub consistency_score: f64,
    pub timeliness_score: f64,
    pub specialization_bonus: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReputationTier {
    Bronze,   // 0-200
    Silver,   // 201-400
    Gold,     // 401-600
    Platinum, // 601-800
    Diamond,  // 801-1000
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreatCategory {
    Malware,
    Phishing,
    Ransomware,
    APT,
    Botnet,
    Cryptocurrency,
    IoT,
    Mobile,
    WebApplication,
    Network,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReputationUpdateRequest {
    pub submission_id: Uuid,
    pub engine_id: Uuid,
    pub was_accurate: bool,
    pub response_time_minutes: f64,
    pub threat_category: ThreatCategory,
    pub confidence_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReputationQuery {
    pub tier: Option<String>,
    pub specialty: Option<ThreatCategory>,
    pub min_score: Option<f64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReputationLeaderboard {
    pub engines: Vec<EngineReputation>,
    pub total_count: usize,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReputationHistory {
    pub engine_id: Uuid,
    pub score_history: Vec<ScoreHistoryEntry>,
    pub milestone_achievements: Vec<MilestoneAchievement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub score: f64,
    pub change_reason: String,
    pub submission_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneAchievement {
    pub milestone: String,
    pub achieved_at: DateTime<Utc>,
    pub score_at_achievement: f64,
}

// ── Conversion helpers ──────────────────────────────────────

fn calculate_tier_from_score(score: f64) -> ReputationTier {
    match score as i32 {
        801..=i32::MAX => ReputationTier::Diamond,
        601..=800 => ReputationTier::Platinum,
        401..=600 => ReputationTier::Gold,
        201..=400 => ReputationTier::Silver,
        _ => ReputationTier::Bronze,
    }
}

/// Calculate consistency score from real submission data.
/// Uses coefficient of variation of recent accuracy windows from the DB.
async fn calculate_consistency_score(pool: &PgPool, engine_id: &str) -> f64 {
    // Query the last 5 batches of 10 submissions each and compute accuracy variance.
    // If the engine has fewer than MIN_SUBMISSIONS_FOR_RELIABLE_SCORE, return neutral 0.5.
    let row: Option<(Option<i32>,)> =
        sqlx::query_as("SELECT total_submissions FROM reputations WHERE engine_id = $1")
            .bind(engine_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    let total = row.and_then(|r| r.0).unwrap_or(0);
    if total < MIN_SUBMISSIONS_FOR_RELIABLE_SCORE {
        return 0.5; // Neutral for new engines
    }

    // Compute standard deviation of accuracy across monthly windows.
    // If no monthly data exists, fall back to overall accuracy as a proxy.
    let variance_row: Option<(Option<f64>,)> = sqlx::query_as(
        r#"
        SELECT STDDEV(monthly_accuracy) FROM (
            SELECT
                DATE_TRUNC('month', s.submitted_at) AS month,
                CAST(SUM(CASE WHEN s.accuracy_score > 0 THEN 1 ELSE 0 END) AS FLOAT)
                    / NULLIF(COUNT(*), 0) AS monthly_accuracy
            FROM submissions s
            WHERE s.engine_id = $1 AND s.processed_at IS NOT NULL
            GROUP BY DATE_TRUNC('month', s.submitted_at)
            HAVING COUNT(*) >= 3
        ) monthly_stats
        "#,
    )
    .bind(engine_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let stddev = variance_row.and_then(|r| r.0).unwrap_or(0.0);

    // Convert stddev to a 0-1 score: low variance = high consistency
    // stddev of 0 => 1.0, stddev of 0.5 => 0.0
    (1.0 - (stddev * 2.0)).clamp(0.0, 1.0)
}

fn db_to_engine_reputation(rep: &ReputationModel, consistency: f64) -> EngineReputation {
    let score = rep.reputation_score as f64;
    let tier = calculate_tier_from_score(score);

    let accuracy = rep.accuracy_rate as f64;
    let total = rep.total_submissions as u32;
    let correct = rep.correct_submissions as u32;
    let incorrect = total.saturating_sub(correct);

    // Simple precision/recall approximation: treat correct as TP,
    // and split incorrect between FP and FN equally when we don't have detailed data
    let fp = incorrect / 2;
    let fn_ = incorrect - fp;

    let precision = if correct + fp > 0 {
        correct as f64 / (correct + fp) as f64
    } else {
        0.0
    };
    let recall = if correct + fn_ > 0 {
        correct as f64 / (correct + fn_) as f64
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };

    let timeliness = 0.7; // Default until we have per-submission timing in the model
    let specialization = 0.0; // Default until we track categories per engine

    EngineReputation {
        engine_id: Uuid::parse_str(&rep.engine_id).unwrap_or(Uuid::nil()),
        engine_name: rep.engine_id.clone(),
        current_score: score,
        historical_high: score, // Would require a historical_high column for real tracking
        total_submissions: total,
        successful_submissions: correct,
        false_positives: fp,
        false_negatives: fn_,
        average_response_time: 0.0,
        specialty_areas: Vec::new(),
        tier,
        last_updated: rep.updated_at,
        monthly_scores: Vec::new(), // Populated separately if needed
        performance_metrics: PerformanceMetrics {
            accuracy_rate: accuracy,
            precision,
            recall,
            f1_score: f1,
            consistency_score: consistency,
            timeliness_score: timeliness,
            specialization_bonus: specialization,
        },
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            accuracy_rate: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
            consistency_score: 0.5,
            timeliness_score: 0.5,
            specialization_bonus: 0.0,
        }
    }
}

// ── Handler functions (all DB-backed) ──────────────────────

/// Update engine reputation based on submission results
pub async fn update_reputation(
    State(state): State<BountyManagerState>,
    Json(update_req): Json<ReputationUpdateRequest>,
) -> Result<Json<EngineReputation>, StatusCode> {
    let engine_id_str = update_req.engine_id.to_string();
    info!("Updating reputation for engine: {}", engine_id_str);

    // Fetch or create the reputation record
    let mut rep = match ReputationModel::find_by_id(&state.db, &engine_id_str).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            info!(
                "Creating new reputation entry for engine: {}",
                engine_id_str
            );
            let now = Utc::now();
            let new_rep = ReputationModel {
                engine_id: engine_id_str.clone(),
                reputation_score: 100.0, // Starting score
                total_submissions: 0,
                correct_submissions: 0,
                accuracy_rate: 0.0,
                average_confidence: 0.0,
                total_stake: 0,
                rewards_earned: 0,
                penalties_incurred: 0,
                created_at: now,
                updated_at: now,
            };
            ReputationModel::create(&state.db, &new_rep)
                .await
                .map_err(|e| {
                    error!("Failed to create reputation: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
        }
        Err(e) => {
            error!("DB error fetching reputation: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let old_score = rep.reputation_score;

    // Update metrics
    rep.total_submissions += 1;
    if update_req.was_accurate {
        rep.correct_submissions += 1;
    }
    rep.accuracy_rate = rep.correct_submissions as f32 / rep.total_submissions as f32;
    rep.average_confidence = (rep.average_confidence * (rep.total_submissions - 1) as f32
        + update_req.confidence_score as f32)
        / rep.total_submissions as f32;

    // Recalculate composite score
    let accuracy_component = rep.accuracy_rate as f64 * ACCURACY_WEIGHT * MAX_REPUTATION_SCORE;
    let volume_component = if rep.total_submissions > 0 {
        (rep.total_submissions as f64).ln() * VOLUME_WEIGHT * 100.0
    } else {
        0.0
    };
    let consistency = calculate_consistency_score(&state.db, &engine_id_str).await;
    let consistency_component = consistency * CONSISTENCY_WEIGHT * MAX_REPUTATION_SCORE;
    let timeliness_component = if update_req.response_time_minutes > 0.0 {
        ((60.0 / update_req.response_time_minutes).min(1.0))
            * TIMELINESS_WEIGHT
            * MAX_REPUTATION_SCORE
    } else {
        TIMELINESS_WEIGHT * MAX_REPUTATION_SCORE
    };

    let new_score = (100.0
        + accuracy_component
        + volume_component
        + consistency_component
        + timeliness_component)
        .min(MAX_REPUTATION_SCORE);

    // Apply penalty factor for poor accuracy
    let penalty_factor = if rep.accuracy_rate < 0.5 {
        0.5
    } else if rep.accuracy_rate < 0.7 {
        0.8
    } else {
        1.0
    };

    rep.reputation_score = (new_score * penalty_factor) as f32;
    rep.updated_at = Utc::now();

    ReputationModel::update(&state.db, &rep)
        .await
        .map_err(|e| {
            error!("Failed to update reputation: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(
        "Reputation updated for engine {}: {:.2} -> {:.2} (change: {:.2})",
        engine_id_str,
        old_score,
        rep.reputation_score,
        rep.reputation_score - old_score
    );

    let engine_rep = db_to_engine_reputation(&rep, consistency);
    Ok(Json(engine_rep))
}

/// Get reputation for a specific engine from DB
pub async fn get_engine_reputation(
    State(state): State<BountyManagerState>,
    Path(engine_id): Path<Uuid>,
) -> Result<Json<EngineReputation>, StatusCode> {
    let engine_id_str = engine_id.to_string();
    info!("Retrieving reputation for engine: {}", engine_id_str);

    let rep = ReputationModel::find_by_id(&state.db, &engine_id_str)
        .await
        .map_err(|e| {
            error!("DB error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            warn!("Reputation not found for engine: {}", engine_id_str);
            StatusCode::NOT_FOUND
        })?;

    let consistency = calculate_consistency_score(&state.db, &engine_id_str).await;
    let engine_rep = db_to_engine_reputation(&rep, consistency);

    info!(
        "Found reputation for engine {}: score {:.2}",
        engine_id_str, rep.reputation_score
    );
    Ok(Json(engine_rep))
}

/// Get reputation leaderboard from DB with filters
pub async fn get_leaderboard(
    State(state): State<BountyManagerState>,
    Query(query): Query<ReputationQuery>,
) -> Result<Json<ReputationLeaderboard>, StatusCode> {
    info!("Fetching reputation leaderboard");

    let limit = query.limit.unwrap_or(50) as i64;

    // Build dynamic query based on filters
    let mut sql = String::from("SELECT * FROM reputations WHERE 1=1");
    let mut bind_idx = 1;
    let mut binds: Vec<String> = Vec::new();

    if let Some(min_score) = query.min_score {
        bind_idx += 1;
        sql.push_str(&format!(" AND reputation_score >= {}", min_score));
    }

    // Tier filtering via score ranges
    if let Some(ref tier_str) = query.tier {
        let (low, high) = match tier_str.as_str() {
            "Bronze" => (0.0, 200.0),
            "Silver" => (201.0, 400.0),
            "Gold" => (401.0, 600.0),
            "Platinum" => (601.0, 800.0),
            "Diamond" => (801.0, 1000.0),
            _ => (0.0, 1000.0),
        };
        sql.push_str(&format!(
            " AND reputation_score >= {} AND reputation_score <= {}",
            low, high
        ));
    }

    sql.push_str(&format!(" ORDER BY reputation_score DESC LIMIT {}", limit));

    let records = sqlx::query_as::<_, ReputationModel>(&sql)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Leaderboard query failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM reputations")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    let engines: Vec<EngineReputation> = records
        .iter()
        .map(|r| db_to_engine_reputation(r, 0.7)) // Use default consistency for bulk listing
        .collect();

    let leaderboard = ReputationLeaderboard {
        total_count: engines.len(),
        engines,
        last_updated: Utc::now(),
    };

    info!(
        "Returning leaderboard with {} engines (from {} total)",
        leaderboard.total_count, total_count.0
    );
    Ok(Json(leaderboard))
}

/// Get reputation history for an engine from DB
pub async fn get_reputation_history(
    State(state): State<BountyManagerState>,
    Path(engine_id): Path<Uuid>,
) -> Result<Json<ReputationHistory>, StatusCode> {
    let engine_id_str = engine_id.to_string();

    // Verify the engine exists
    let rep = ReputationModel::find_by_id(&state.db, &engine_id_str)
        .await
        .map_err(|e| {
            error!("DB error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Fetch score change history from submissions processed for this engine.
    // Each processed submission represents a score change event.
    let history_rows: Vec<(DateTime<Utc>, Option<f32>, Uuid)> = sqlx::query_as(
        r#"
        SELECT processed_at, accuracy_score, id
        FROM submissions
        WHERE engine_id = $1 AND processed_at IS NOT NULL
        ORDER BY processed_at DESC
        LIMIT 50
        "#,
    )
    .bind(&engine_id_str)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut score_history: Vec<ScoreHistoryEntry> = history_rows
        .into_iter()
        .map(|(ts, score, sub_id)| ScoreHistoryEntry {
            timestamp: ts,
            score: score.unwrap_or(0.0) as f64,
            change_reason: "Submission processed".to_string(),
            submission_id: Some(sub_id),
        })
        .collect();

    // Always include the current score as the most recent entry
    score_history.insert(
        0,
        ScoreHistoryEntry {
            timestamp: rep.updated_at,
            score: rep.reputation_score as f64,
            change_reason: "Current score".to_string(),
            submission_id: None,
        },
    );

    let milestones = generate_milestones_from_db(&rep);

    let history = ReputationHistory {
        engine_id,
        score_history,
        milestone_achievements: milestones,
    };

    Ok(Json(history))
}

/// Apply monthly reputation decay via bulk SQL update
pub async fn apply_reputation_decay(
    State(state): State<BountyManagerState>,
) -> Result<Json<HashMap<String, u32>>, StatusCode> {
    let cutoff = Utc::now() - chrono::Duration::days(30);

    let result = sqlx::query(
        r#"
        UPDATE reputations
        SET reputation_score = reputation_score * $1,
            updated_at = NOW()
        WHERE updated_at < $2
        "#,
    )
    .bind(REPUTATION_DECAY_FACTOR as f32)
    .bind(cutoff)
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Reputation decay failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM reputations")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    let mut decay_stats = HashMap::new();
    decay_stats.insert(
        "engines_affected".to_string(),
        result.rows_affected() as u32,
    );
    decay_stats.insert("total_engines".to_string(), total.0 as u32);

    info!(
        "Applied reputation decay: {} engines affected out of {} total",
        result.rows_affected(),
        total.0
    );

    Ok(Json(decay_stats))
}

/// Register a new engine for reputation tracking in DB
pub async fn register_engine(
    State(state): State<BountyManagerState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<EngineReputation>, StatusCode> {
    let engine_id = Uuid::new_v4();
    let engine_id_str = engine_id.to_string();
    let engine_name = req
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or(&format!("Engine-{}", engine_id))
        .to_string();

    let now = Utc::now();
    let new_rep = ReputationModel {
        engine_id: engine_id_str.clone(),
        reputation_score: 100.0, // Starting score
        total_submissions: 0,
        correct_submissions: 0,
        accuracy_rate: 0.0,
        average_confidence: 0.0,
        total_stake: 0,
        rewards_earned: 0,
        penalties_incurred: 0,
        created_at: now,
        updated_at: now,
    };

    let created = ReputationModel::create(&state.db, &new_rep)
        .await
        .map_err(|e| {
            error!("Failed to register engine: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(engine_id = %engine_id_str, name = %engine_name, "Registered new engine");

    let engine_rep = db_to_engine_reputation(&created, 0.5);
    Ok(Json(engine_rep))
}

// ── Helper functions ──────────────────────────────────────

fn generate_milestones_from_db(rep: &ReputationModel) -> Vec<MilestoneAchievement> {
    let mut milestones = Vec::new();

    if rep.total_submissions >= 100 {
        milestones.push(MilestoneAchievement {
            milestone: "Century Submitter".to_string(),
            achieved_at: rep.updated_at,
            score_at_achievement: rep.reputation_score as f64,
        });
    }

    if rep.reputation_score >= 500.0 {
        milestones.push(MilestoneAchievement {
            milestone: "Gold Tier Achievement".to_string(),
            achieved_at: rep.updated_at,
            score_at_achievement: rep.reputation_score as f64,
        });
    }

    if rep.accuracy_rate >= 0.95 && rep.total_submissions >= 10 {
        milestones.push(MilestoneAchievement {
            milestone: "Accuracy Expert".to_string(),
            achieved_at: rep.updated_at,
            score_at_achievement: rep.reputation_score as f64,
        });
    }

    if rep.total_submissions >= 1000 {
        milestones.push(MilestoneAchievement {
            milestone: "Legendary Submitter".to_string(),
            achieved_at: rep.updated_at,
            score_at_achievement: rep.reputation_score as f64,
        });
    }

    milestones
}
