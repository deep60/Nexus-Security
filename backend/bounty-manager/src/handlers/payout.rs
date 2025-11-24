// backend/bounty-manager/src/handlers/payout_handler.rs

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
use shared::types::ApiResponse;
use super::bounty_crud::PaginationParams;
use crate::handlers::bounty_crud::{BountyManagerState, ThreatVerdict};
use crate::handlers::submission::{Submission, SubmissionStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutInfo {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub total_reward_pool: u64,
    pub consensus_verdict: ThreatVerdict,
    pub consensus_confidence: f32,
    pub total_correct_submissions: u32,
    pub total_incorrect_submissions: u32,
    pub reward_distributions: Vec<RewardDistribution>,
    pub slashed_stakes: Vec<SlashedStake>,
    pub status: PayoutStatus,
    pub processing_started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub blockchain_transactions: Vec<PayoutTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PayoutStatus {
    Calculating,    // Determining consensus and rewards
    Processing,     // Executing blockchain transactions
    Completed,      // All payouts processed
    Failed,         // Error in processing
    PartialFailure, // Some transactions failed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    pub engine_id: String,
    pub submission_id: Uuid,
    pub base_reward: u64,         // Share of bounty reward
    pub accuracy_bonus: u64,      // Bonus for high confidence in correct verdict
    pub reputation_multiplier: f32, // Reputation-based multiplier
    pub stake_return: u64,        // Original stake returned
    pub total_payout: u64,        // Total amount to be paid
    pub transaction_hash: Option<String>,
    pub processed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashedStake {
    pub engine_id: String,
    pub submission_id: Uuid,
    pub stake_amount: u64,
    pub slashing_reason: SlashingReason,
    pub redistributed_amount: u64, // Amount redistributed to correct submissions
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashingReason {
    IncorrectVerdict,     // Wrong consensus verdict
    LowConfidence,        // High confidence in wrong verdict
    MaliciousActivity,    // Detected coordinated false submissions
    TechnicalViolation,   // Violated submission rules
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutTransaction {
    pub transaction_hash: String,
    pub transaction_type: TransactionType,
    pub recipient: String,
    pub amount: u64,
    pub gas_used: Option<u64>,
    pub status: TransactionStatus,
    pub block_number: Option<u64>,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    RewardPayout,
    StakeReturn,
    StakeSlashing,
    BountyRefund,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Debug, Serialize)]
pub struct PayoutSummary {
    pub total_rewards_distributed: u64,
    pub total_stakes_slashed: u64,
    pub successful_engines: u32,
    pub slashed_engines: u32,
    pub consensus_accuracy: f32,
    pub average_confidence: f32,
}

#[derive(Debug, Deserialize)]
pub struct PayoutFilters {
    pub status: Option<PayoutStatus>,
    pub bounty_id: Option<Uuid>,
    pub engine_id: Option<String>,
    pub min_amount: Option<u64>,
}

// Handler implementations
pub async fn process_bounty_completion(
    State(state): State<BountyManagerState>,
    Path(bounty_id): Path<Uuid>,
) -> Result<Json<ApiResponse<PayoutInfo>>, StatusCode> {
    // TODO: Fetch bounty and all submissions
    // TODO: Calculate consensus based on weighted voting
    // TODO: Determine reward distribution
    
    let payout_id = Uuid::new_v4();
    let now = Utc::now();

    // Mock consensus calculation
    let consensus_verdict = ThreatVerdict::Malicious;
    let consensus_confidence = 0.87;

    // Mock reward calculation
    let reward_distributions = vec![
        RewardDistribution {
            engine_id: "engine_123".to_string(),
            submission_id: Uuid::new_v4(),
            base_reward: 60000, // 60% of reward pool
            accuracy_bonus: 10000,
            reputation_multiplier: 1.2,
            stake_return: 50000,
            total_payout: 134000, // (60000 + 10000) * 1.2 + 50000
            transaction_hash: None,
            processed: false,
        },
        RewardDistribution {
            engine_id: "engine_456".to_string(),
            submission_id: Uuid::new_v4(),
            base_reward: 40000, // 40% of reward pool
            accuracy_bonus: 5000,
            reputation_multiplier: 1.0,
            stake_return: 30000,
            total_payout: 75000, // (40000 + 5000) * 1.0 + 30000
            transaction_hash: None,
            processed: false,
        },
    ];

    let slashed_stakes = vec![
        SlashedStake {
            engine_id: "engine_789".to_string(),
            submission_id: Uuid::new_v4(),
            stake_amount: 25000,
            slashing_reason: SlashingReason::IncorrectVerdict,
            redistributed_amount: 20000, // 80% redistributed, 20% burned
            transaction_hash: None,
        },
    ];

    let payout_info = PayoutInfo {
        id: payout_id,
        bounty_id,
        total_reward_pool: 100000,
        consensus_verdict,
        consensus_confidence,
        total_correct_submissions: 2,
        total_incorrect_submissions: 1,
        reward_distributions,
        slashed_stakes,
        status: PayoutStatus::Calculating,
        processing_started_at: now,
        completed_at: None,
        blockchain_transactions: vec![],
    };

    // TODO: Save payout info to database
    // TODO: Start async payout processing

    Ok(Json(ApiResponse::success(payout_info)))
}

pub async fn distribute_rewards(
    State(state): State<BountyManagerState>,
    Path(payout_id): Path<Uuid>,
) -> Result<Json<ApiResponse<PayoutInfo>>, StatusCode> {
    // TODO: Process blockchain transactions for rewards
    // TODO: Update submission statuses
    // TODO: Update engine reputations
    
    let mut payout_info = create_mock_payout_info(payout_id);
    
    // Simulate processing transactions
    for distribution in &mut payout_info.reward_distributions {
        // TODO: Execute actual blockchain transaction
        distribution.transaction_hash = Some(format!("0x{:x}", rand::random::<u64>()));
        distribution.processed = true;
        
        payout_info.blockchain_transactions.push(PayoutTransaction {
            transaction_hash: distribution.transaction_hash.clone().unwrap(),
            transaction_type: TransactionType::RewardPayout,
            recipient: distribution.engine_id.clone(),
            amount: distribution.total_payout,
            gas_used: Some(21000),
            status: TransactionStatus::Confirmed,
            block_number: Some(18500000),
            processed_at: Utc::now(),
        });
    }

    payout_info.status = PayoutStatus::Completed;
    payout_info.completed_at = Some(Utc::now());

    Ok(Json(ApiResponse::success(payout_info)))
}

pub async fn handle_stake_slashing(
    State(state): State<BountyManagerState>,
    Path(payout_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<SlashedStake>>>, StatusCode> {
    // TODO: Process stake slashing transactions
    // TODO: Redistribute slashed amounts to correct submissions
    // TODO: Update reputation scores negatively

    let slashed_stakes = vec![
        SlashedStake {
            engine_id: "engine_789".to_string(),
            submission_id: Uuid::new_v4(),
            stake_amount: 25000,
            slashing_reason: SlashingReason::IncorrectVerdict,
            redistributed_amount: 20000,
            transaction_hash: Some("0xslash123...".to_string()),
        },
    ];

    Ok(Json(ApiResponse::success(slashed_stakes)))
}

pub async fn get_payout_history(
    State(_state): State<BountyManagerState>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<PayoutFilters>,
    Extension(engine_id): Extension<Option<String>>, // Optional - admin vs engine view
) -> Result<Json<ApiResponse<Vec<PayoutInfo>>>, StatusCode> {
    // TODO: Fetch payout history from database
    // TODO: Filter based on permissions (engine can only see their own)
    
    let payouts = vec![
        create_mock_payout_info(Uuid::new_v4()),
    ];

    Ok(Json(ApiResponse::success(payouts)))
}

pub async fn get_payout_summary(
    State(_state): State<BountyManagerState>,
    Path(bounty_id): Path<Uuid>,
) -> Result<Json<ApiResponse<PayoutSummary>>, StatusCode> {
    // TODO: Calculate real summary from database

    let summary = PayoutSummary {
        total_rewards_distributed: 209000,
        total_stakes_slashed: 25000,
        successful_engines: 2,
        slashed_engines: 1,
        consensus_accuracy: 0.87,
        average_confidence: 0.82,
    };

    Ok(Json(ApiResponse::success(summary)))
}

// Internal helper functions for consensus calculation
pub fn calculate_weighted_consensus(submissions: &[Submission], reputation_weights: &HashMap<String, f32>) -> (ThreatVerdict, f32) {
    let mut verdict_scores: HashMap<String, f32> = HashMap::new();
    let mut total_weight = 0.0;

    for submission in submissions {
        if submission.status != SubmissionStatus::Active {
            continue;
        }

        let reputation_weight = reputation_weights.get(&submission.engine_id).unwrap_or(&1.0);
        let stake_weight = submission.stake_amount as f32 / 100000.0; // Normalize stake
        let confidence_weight = submission.confidence;
        
        let combined_weight = reputation_weight * stake_weight * confidence_weight;
        
        let verdict_key = format!("{:?}", submission.verdict);
        *verdict_scores.entry(verdict_key).or_insert(0.0) += combined_weight;
        total_weight += combined_weight;
    }

    // Find consensus verdict
    let unknown_str = "Unknown".to_string();
    let (consensus_verdict_str, consensus_score) = verdict_scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap_or((&unknown_str, &0.0));

    let consensus_confidence = consensus_score / total_weight;

    let consensus_verdict = match consensus_verdict_str.as_str() {
        "Malicious" => ThreatVerdict::Malicious,
        "Benign" => ThreatVerdict::Benign,
        "Suspicious" => ThreatVerdict::Suspicious,
        _ => ThreatVerdict::Unknown,
    };

    (consensus_verdict, consensus_confidence)
}

pub fn calculate_reward_distribution(
    total_reward: u64,
    correct_submissions: &[Submission],
    reputation_weights: &HashMap<String, f32>,
) -> Vec<RewardDistribution> {
    let mut distributions = Vec::new();
    let total_stake: u64 = correct_submissions.iter().map(|s| s.stake_amount).sum();
    
    for submission in correct_submissions {
        let reputation_multiplier = reputation_weights.get(&submission.engine_id).unwrap_or(&1.0);
        
        // Base reward proportional to stake
        let stake_proportion = submission.stake_amount as f32 / total_stake as f32;
        let base_reward = (total_reward as f32 * stake_proportion) as u64;
        
        // Accuracy bonus for high confidence
        let accuracy_bonus = if submission.confidence > 0.9 {
            (base_reward as f32 * 0.2) as u64
        } else if submission.confidence > 0.8 {
            (base_reward as f32 * 0.1) as u64
        } else {
            0
        };

        let total_payout = ((base_reward + accuracy_bonus) as f32 * reputation_multiplier) as u64 + submission.stake_amount;

        distributions.push(RewardDistribution {
            engine_id: submission.engine_id.clone(),
            submission_id: submission.id,
            base_reward,
            accuracy_bonus,
            reputation_multiplier: *reputation_multiplier,
            stake_return: submission.stake_amount,
            total_payout,
            transaction_hash: None,
            processed: false,
        });
    }

    distributions
}

// Mock data helper
fn create_mock_payout_info(id: Uuid) -> PayoutInfo {
    PayoutInfo {
        id,
        bounty_id: Uuid::new_v4(),
        total_reward_pool: 100000,
        consensus_verdict: ThreatVerdict::Malicious,
        consensus_confidence: 0.87,
        total_correct_submissions: 2,
        total_incorrect_submissions: 1,
        reward_distributions: vec![],
        slashed_stakes: vec![],
        status: PayoutStatus::Completed,
        processing_started_at: Utc::now() - chrono::Duration::hours(1),
        completed_at: Some(Utc::now() - chrono::Duration::minutes(30)),
        blockchain_transactions: vec![],
    }
}