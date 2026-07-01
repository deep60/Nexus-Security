// backend/bounty-manager/src/handlers/mod.rs

pub mod bounty_crud;

// Re-export all public types and functions from bounty_handler
// Additional essential handlers
pub mod dispute;
pub mod payout;
pub mod reputation_handler;
pub mod submission;
pub mod validation;
pub mod withdrawal;

// Re-export from additional handlers
pub use submission::{
    get_submission, list_submissions_for_bounty, submit_analysis, update_submission_status,
};

pub use payout::{
    distribute_rewards, get_payout_history, handle_stake_slashing, process_bounty_completion,
};

pub use reputation_handler::{
    apply_reputation_decay, get_engine_reputation, get_leaderboard, get_reputation_history,
    register_engine, update_reputation,
};

pub use dispute::{
    create_dispute, get_dispute, get_dispute_stats, list_disputes, resolve_dispute, update_dispute,
    vote_on_dispute, withdraw_dispute,
};

pub use validation::{
    bulk_validate_submissions, get_validation_result, get_validation_stats, list_validations,
    revalidate_submission, validate_submission,
};

pub use withdrawal::get_claimable;
