use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use std::sync::Arc;
use tracing::{info, warn, error};

// Reputation scoring weights and parameters
const ACCURACY_WEIGHT: f64 = 0.40;
const TIMELINESS_WEIGHT: f64 = 0.25;
const CONSISTENCY_WEIGHT: f64 = 0.20;
const VOLUME_WEIGHT: f64 = 0.15;

const MIN_SUBMISSIONS_FOR_RELIABLE_SCORE: u32 = 10;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub tier: Option<ReputationTier>,
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

// Application state for reputation management
pub type ReputationStore = Arc<RwLock<HashMap<Uuid, EngineReputation>>>;

impl EngineReputation {
    pub fn new(engine_id: Uuid, engine_name: String) -> Self {
        Self {
            engine_id,
            engine_name,
            current_score: 100.0, // Starting score
            historical_high: 100.0,
            total_submissions: 0,
            successful_submissions: 0,
            false_positives: 0,
            false_negatives: 0,
            average_response_time: 0.0,
            specialty_areas: Vec::new(),
            tier: ReputationTier::Bronze,
            last_updated: Utc::now(),
            monthly_scores: Vec::new(),
            performance_metrics: PerformanceMetrics::default(),
        }
    }

    pub fn calculate_tier(&self) -> ReputationTier {
        match self.current_score {
            s if s >= 801.0 => ReputationTier::Diamond,
            s if s >= 601.0 => ReputationTier::Platinum,
            s if s >= 401.0 => ReputationTier::Gold,
            s if s >= 201.0 => ReputationTier::Silver,
            _ => ReputationTier::Bronze,
        }
    }

    pub fn update_score(&mut self, update: &ReputationUpdateRequest) {
        self.total_submissions += 1;
        
        // Update accuracy metrics
        if update.was_accurate {
            self.successful_submissions += 1;
        } else {
            if update.confidence_score > 0.8 {
                self.false_positives += 1;
            } else {
                self.false_negatives += 1;
            }
        }

        // Update response time
        self.average_response_time = (
            self.average_response_time * (self.total_submissions - 1) as f64 + 
            update.response_time_minutes
        ) / self.total_submissions as f64;

        // Update specialty areas
        if !self.specialty_areas.contains(&update.threat_category) {
            self.specialty_areas.push(update.threat_category.clone());
        }

        // Recalculate performance metrics
        self.calculate_performance_metrics();
        
        // Calculate new reputation score
        let new_score = self.calculate_reputation_score();
        self.current_score = new_score.min(MAX_REPUTATION_SCORE);
        
        if self.current_score > self.historical_high {
            self.historical_high = self.current_score;
        }

        self.tier = self.calculate_tier();
        self.last_updated = Utc::now();
    }

    fn calculate_performance_metrics(&mut self) {
        let accuracy = if self.total_submissions > 0 {
            self.successful_submissions as f64 / self.total_submissions as f64
        } else {
            0.0
        };

        let precision = if self.successful_submissions + self.false_positives > 0 {
            self.successful_submissions as f64 / 
            (self.successful_submissions + self.false_positives) as f64
        } else {
            0.0
        };

        let recall = if self.successful_submissions + self.false_negatives > 0 {
            self.successful_submissions as f64 / 
            (self.successful_submissions + self.false_negatives) as f64
        } else {
            0.0
        };

        let f1 = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        // Timeliness score (inverse of response time, capped)
        let timeliness = if self.average_response_time > 0.0 {
            (60.0 / self.average_response_time).min(1.0) // 60 minutes = perfect score
        } else {
            1.0
        };

        // Consistency score based on variance in recent performance
        let consistency = self.calculate_consistency_score();

        // Specialization bonus for focused expertise
        let specialization = (self.specialty_areas.len() as f64 * 0.1).min(0.5);

        self.performance_metrics = PerformanceMetrics {
            accuracy_rate: accuracy,
            precision,
            recall,
            f1_score: f1,
            consistency_score: consistency,
            timeliness_score: timeliness,
            specialization_bonus: specialization,
        };
    }

    fn calculate_consistency_score(&self) -> f64 {
        // Simplified consistency calculation based on historical variance
        // In a real implementation, this would analyze score variance over time
        if self.total_submissions < MIN_SUBMISSIONS_FOR_RELIABLE_SCORE {
            return 0.5; // Neutral score for new engines
        }
        
        // Mock calculation - in reality, you'd calculate standard deviation
        // of recent accuracy rates
        0.8 // Placeholder
    }

    fn calculate_reputation_score(&self) -> f64 {
        let base_score = 100.0;
        
        let accuracy_component = self.performance_metrics.accuracy_rate * ACCURACY_WEIGHT * 1000.0;
        let timeliness_component = self.performance_metrics.timeliness_score * TIMELINESS_WEIGHT * 1000.0;
        let consistency_component = self.performance_metrics.consistency_score * CONSISTENCY_WEIGHT * 1000.0;
        
        // Volume component rewards active participation
        let volume_component = if self.total_submissions > 0 {
            (self.total_submissions as f64).ln() * VOLUME_WEIGHT * 100.0
        } else {
            0.0
        };

        let total_score = base_score + accuracy_component + timeliness_component + 
                         consistency_component + volume_component + 
                         (self.performance_metrics.specialization_bonus * 100.0);

        // Apply penalties for poor performance
        let penalty_factor = if self.performance_metrics.accuracy_rate < 0.5 {
            0.5 // Severe penalty for poor accuracy
        } else if self.performance_metrics.accuracy_rate < 0.7 {
            0.8 // Moderate penalty
        } else {
            1.0 // No penalty
        };

        total_score * penalty_factor
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

// Handler functions

/// Update engine reputation based on submission results
pub async fn update_reputation(
    State(store): State<ReputationStore>,
    Json(update_req): Json<ReputationUpdateRequest>,
) -> Result<Json<EngineReputation>, StatusCode> {
    info!("Updating reputation for engine: {}", update_req.engine_id);
    
    let mut store = store.write().await;
    
    let engine = store.entry(update_req.engine_id).or_insert_with(|| {
        info!("Creating new reputation entry for engine: {}", update_req.engine_id);
        EngineReputation::new(update_req.engine_id, format!("Engine-{}", update_req.engine_id))
    });

    let old_score = engine.current_score;
    engine.update_score(&update_req);
    
    info!(
        "Reputation updated for engine {}: {} -> {} (change: {:.2})",
        update_req.engine_id,
        old_score,
        engine.current_score,
        engine.current_score - old_score
    );
    
    Ok(Json(engine.clone()))
}

/// Get reputation for a specific engine
pub async fn get_engine_reputation(
    State(store): State<ReputationStore>,
    Path(engine_id): Path<Uuid>,
) -> Result<Json<EngineReputation>, StatusCode> {
    info!("Retrieving reputation for engine: {}", engine_id);
    
    let store = store.read().await;
    
    match store.get(&engine_id) {
        Some(reputation) => {
            info!("Found reputation for engine {}: score {:.2}", engine_id, reputation.current_score);
            Ok(Json(reputation.clone()))
        },
        None => {
            warn!("Reputation not found for engine: {}", engine_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Get reputation leaderboard
pub async fn get_leaderboard(
    State(store): State<ReputationStore>,
    Query(query): Query<ReputationQuery>,
) -> Result<Json<ReputationLeaderboard>, StatusCode> {
    info!("Fetching reputation leaderboard with filters: {:?}", query);
    
    let store = store.read().await;
    
    let mut engines: Vec<EngineReputation> = store.values().cloned().collect();
    let total_engines = engines.len();
    
    // Apply filters
    if let Some(tier) = &query.tier {
        engines.retain(|e| matches!((&e.tier, tier), 
            (ReputationTier::Bronze, ReputationTier::Bronze) |
            (ReputationTier::Silver, ReputationTier::Silver) |
            (ReputationTier::Gold, ReputationTier::Gold) |
            (ReputationTier::Platinum, ReputationTier::Platinum) |
            (ReputationTier::Diamond, ReputationTier::Diamond)
        ));
        info!("Filtered by tier {:?}: {} engines remaining", tier, engines.len());
    }
    
    if let Some(specialty) = &query.specialty {
        engines.retain(|e| e.specialty_areas.contains(specialty));
        info!("Filtered by specialty {:?}: {} engines remaining", specialty, engines.len());
    }
    
    if let Some(min_score) = query.min_score {
        engines.retain(|e| e.current_score >= min_score);
        info!("Filtered by min score {}: {} engines remaining", min_score, engines.len());
    }
    
    // Sort by reputation score (descending)
    engines.sort_by(|a, b| b.current_score.partial_cmp(&a.current_score).unwrap());
    
    // Apply limit
    if let Some(limit) = query.limit {
        engines.truncate(limit);
        info!("Limited results to {} engines", limit);
    }
    
    let leaderboard = ReputationLeaderboard {
        total_count: engines.len(),
        engines,
        last_updated: Utc::now(),
    };
    
    info!("Returning leaderboard with {} engines (from {} total)", leaderboard.total_count, total_engines);
    
    Ok(Json(leaderboard))
}

/// Get reputation history for an engine
pub async fn get_reputation_history(
    State(store): State<ReputationStore>,
    Path(engine_id): Path<Uuid>,
) -> Result<Json<ReputationHistory>, StatusCode> {
    let store = store.read().await;
    
    match store.get(&engine_id) {
        Some(reputation) => {
            // In a real implementation, this would fetch from a time-series database
            let history = ReputationHistory {
                engine_id,
                score_history: vec![
                    ScoreHistoryEntry {
                        timestamp: reputation.last_updated,
                        score: reputation.current_score,
                        change_reason: "Current score".to_string(),
                        submission_id: None,
                    }
                ],
                milestone_achievements: generate_milestones(reputation),
            };
            Ok(Json(history))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Apply monthly reputation decay
pub async fn apply_reputation_decay(
    State(store): State<ReputationStore>,
) -> Result<Json<HashMap<String, u32>>, StatusCode> {
    let mut store = store.write().await;
    let mut decay_stats = HashMap::new();
    let mut decayed_count = 0u32;
    
    for (_, engine) in store.iter_mut() {
        let old_score = engine.current_score;
        engine.current_score *= REPUTATION_DECAY_FACTOR;
        engine.tier = engine.calculate_tier();
        engine.last_updated = Utc::now();
        
        if old_score != engine.current_score {
            decayed_count += 1;
        }
    }
    
    decay_stats.insert("engines_affected".to_string(), decayed_count);
    decay_stats.insert("total_engines".to_string(), store.len() as u32);
    
    Ok(Json(decay_stats))
}

/// Register a new engine for reputation tracking
pub async fn register_engine(
    State(store): State<ReputationStore>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<EngineReputation>, StatusCode> {
    let engine_id = Uuid::new_v4();
    let engine_name = req.get("name")
        .and_then(|n| n.as_str())
        .unwrap_or(&format!("Engine-{}", engine_id))
        .to_string();
    
    let mut store = store.write().await;
    let reputation = EngineReputation::new(engine_id, engine_name);
    
    store.insert(engine_id, reputation.clone());
    
    Ok(Json(reputation))
}

// Helper functions

fn generate_milestones(reputation: &EngineReputation) -> Vec<MilestoneAchievement> {
    let mut milestones = Vec::new();
    
    // Add milestone achievements based on current stats
    if reputation.total_submissions >= 100 {
        milestones.push(MilestoneAchievement {
            milestone: "Century Submitter".to_string(),
            achieved_at: reputation.last_updated,
            score_at_achievement: reputation.current_score,
        });
    }
    
    if reputation.current_score >= 500.0 {
        milestones.push(MilestoneAchievement {
            milestone: "Gold Tier Achievement".to_string(),
            achieved_at: reputation.last_updated,
            score_at_achievement: reputation.current_score,
        });
    }
    
    if reputation.performance_metrics.accuracy_rate >= 0.95 {
        milestones.push(MilestoneAchievement {
            milestone: "Accuracy Expert".to_string(),
            achieved_at: reputation.last_updated,
            score_at_achievement: reputation.current_score,
        });
    }
    
    milestones
}