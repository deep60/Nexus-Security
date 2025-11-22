//! Observability utilities for logging, tracing, and metrics
//! 
//! Provides centralized observability setup for all services

pub mod logging;
pub mod tracing;
pub mod metrics;

pub use logging::*;
pub use tracing::*;
pub use metrics::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObservabilityError {
    #[error("Logging setup error: {0}")]
    Logging(String),
    
    #[error("Tracing setup error: {0}")]
    Tracing(String),
    
    #[error("Metrics error: {0}")]
    Metrics(String),
}

pub type ObservabilityResult<T> = Result<T, ObservabilityError>;
