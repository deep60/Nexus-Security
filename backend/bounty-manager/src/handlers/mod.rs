// backend/bounty-manager/src/handlers/mod.rs

pub mod bounty_crud;

// Re-export all public types and functions from bounty_handler
pub use bounty_crud::{
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
// Additional essential handlers
pub mod submission;
pub mod payout;
pub mod reputation_handler;
pub mod dispute;
pub mod validation;

// Re-export from additional handlers
pub use submission::{
    Submission,
    SubmissionStatus,
    AnalysisDetails,
    submit_analysis,
    get_submission,
    list_submissions_for_bounty,
    update_submission_status,
};

pub use payout::{
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

pub use dispute::{
    Dispute,
    DisputeType,
    DisputeStatus,
    DisputeSeverity,
    DisputeResolution,
    create_dispute,
    get_dispute,
    list_disputes,
    update_dispute,
    resolve_dispute,
    vote_on_dispute,
    withdraw_dispute,
    get_dispute_stats,
};

pub use validation::{
    ValidationResult,
    ValidationStatus,
    ValidationCheckType,
    QualityMetrics,
    validate_submission,
    get_validation_result,
    list_validations,
    bulk_validate_submissions,
    get_validation_stats,
    revalidate_submission,
};