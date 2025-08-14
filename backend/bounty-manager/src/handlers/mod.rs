// backend/bounty-manager/src/handlers/mod.rs

pub mod bounty_handler;

// Re-export all public types and functions from bounty_handler
pub use bounty_handler::{
    // Core types
    Bounty,
    ArtifactType,
    ArtifactData,
    BountyStatus,
    SubmissionSummary,
    ThreatVerdict,
    BountyManagerState,
    
    // Request/Response DTOs
    CreateBountyRequest,
    UpdateBountyRequest,
    BountyFilters,
    BountyListResponse,
    BountyStatsResponse,
    CurrencyStats,
    SubmissionRequest,
    SubmissionResponse,
    AnalysisData,
    
    // Handler functions
    create_bounty,
    get_bounty,
    list_bounties,
    update_bounty,
    cancel_bounty,
    get_bounty_stats,
    submit_to_bounty,
};
/ Additional essential handlers
pub mod submission_handler;
pub mod payout_handler;
pub mod reputation_handler;

// Re-export from additional handlers
pub use submission_handler::{
    Submission,
    SubmissionStatus,
    AnalysisDetails,
    submit_analysis,
    get_submission,
    list_submissions_for_bounty,
    update_submission_status,
};

pub use payout_handler::{
    PayoutInfo,
    PayoutStatus,
    RewardDistribution,
    process_bounty_completion,
    distribute_rewards,
    handle_stake_slashing,
    get_payout_history,
};

pub use reputation_handler::{
    ReputationUpdate,
    ReputationMetrics,
    update_engine_reputation,
    get_engine_reputation,
    calculate_accuracy_score,
    get_reputation_leaderboard,
};