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

// Additional handlers that might be added later
// pub mod submission_handler;
// pub mod payout_handler;
// pub mod reputation_handler;