// Pre-production scaffolding: some items are intentionally unused while
// features are wired up. This crate-level allow keeps `clippy -D warnings`
// green without deleting code we are about to use. Remove before GA.
#![allow(dead_code)]

//! Shared utilities and types for Verdyx backend services

// Re-export common dependencies
pub use anyhow;
pub use chrono;
pub use serde;
pub use serde_json;
pub use thiserror;
pub use tracing;
pub use uuid;

// Common error types
#[derive(Debug, thiserror::Error)]
pub enum VerdyxError {
    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("External service error: {0}")]
    ExternalService(String),
}

pub type Result<T> = std::result::Result<T, VerdyxError>;

// Export modules
pub mod messaging;
pub mod types;
