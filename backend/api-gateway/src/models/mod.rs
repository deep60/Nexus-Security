//! This module contains all data models used throughout the API gateway,
//! including request/response structures, database models, and domain entities.
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Re-export all model modules
pub mod analysis;
pub mod error;
pub mod request;
pub mod response;

pub use error::*;
pub use request::*;
pub use response::*;

pub mod bounty;
pub mod user;

// Re-export commonly used types for convenience
pub use analysis::*;
pub use bounty::*;
pub use user::*;

/// Common response wrapper for API endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response with data
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            timestamp: Utc::now(),
        }
    }

    /// Create a successful response with data and message
    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message),
            timestamp: Utc::now(),
        }
    }
}

impl ApiResponse<()> {
    /// Create an error response
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
            timestamp: Utc::now(),
        }
    }

    /// Create a simple success response without data
    pub fn success_empty() -> Self {
        Self {
            success: true,
            data: None,
            message: None,
            timestamp: Utc::now(),
        }
    }
}

/// Pagination parameters for list endpoints
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    20
}

impl PaginationParams {
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.limit
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.page == 0 {
            return Err("Page must be greater than 0".to_string());
        }
        if self.limit == 0 || self.limit > 100 {
            return Err("Limit must be between 1 and 100".to_string());
        }
        Ok(())
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, limit: u32) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        Self {
            items,
            total,
            page,
            limit,
            total_pages,
        }
    }
}

/// Common filter parameters for list endpoints
#[derive(Debug, Deserialize)]
pub struct FilterParams {
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub updated_after: Option<DateTime<Utc>>,
    pub updated_before: Option<DateTime<Utc>>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub services: ServiceStatus,
}

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub database: String,
    pub redis: String,
    pub blockchain: String,
    pub analysis_engine: String,
}

/// Authentication token claims
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: Uuid,      // User ID
    pub exp: i64,       // Expiration timestamp
    pub iat: i64,       // Issued at timestamp
    pub role: String,   // User role
}

/// Blockchain transaction status
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransactionStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "confirmed")]
    Confirmed,
    #[serde(rename = "failed")]
    Failed,
}

/// Common error types
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Authentication error: {0}")]
    Authentication(String),
    #[error("Authorization error: {0}")]
    Authorization(String),
}

/// File type enumeration for analysis
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FileType {
    #[serde(rename = "executable")]
    Executable,
    #[serde(rename = "document")]
    Document,
    #[serde(rename = "archive")]
    Archive,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "script")]
    Script,
    #[serde(rename = "unknown")]
    Unknown,
}

/// Threat severity levels
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatSeverity {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "critical")]
    Critical,
}

/// Reputation score range
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReputationScore {
    pub score: f64,  // 0.0 to 100.0
    pub total_submissions: u32,
    pub successful_submissions: u32,
    pub last_updated: DateTime<Utc>,
}

impl ReputationScore {
    pub fn new() -> Self {
        Self {
            score: 50.0, // Start with neutral score
            total_submissions: 0,
            successful_submissions: 0,
            last_updated: Utc::now(),
        }
    }

    pub fn accuracy_rate(&self) -> f64 {
        if self.total_submissions == 0 {
            0.0
        } else {
            (self.successful_submissions as f64 / self.total_submissions as f64) * 100.0
        }
    }

    pub fn update_score(&mut self, success: bool) {
        self.total_submissions += 1;
        if success {
            self.successful_submissions += 1;
        }
        
        // Simple scoring algorithm - can be made more sophisticated
        let accuracy = self.accuracy_rate();
        let submission_bonus = (self.total_submissions as f64).min(100.0) * 0.1;
        self.score = (accuracy * 0.8) + submission_bonus;
        self.last_updated = Utc::now();
    }
}

impl Default for ReputationScore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params_offset() {
        let params = PaginationParams {
            page: 3,
            limit: 10,
        };
        assert_eq!(params.offset(), 20);
    }

    #[test]
    fn test_pagination_params_validation() {
        let invalid_page = PaginationParams {
            page: 0,
            limit: 10,
        };
        assert!(invalid_page.validate().is_err());

        let invalid_limit = PaginationParams {
            page: 1,
            limit: 0,
        };
        assert!(invalid_limit.validate().is_err());

        let valid_params = PaginationParams {
            page: 1,
            limit: 20,
        };
        assert!(valid_params.validate().is_ok());
    }

    #[test]
    fn test_reputation_score_update() {
        let mut score = ReputationScore::new();
        
        // Test successful submission
        score.update_score(true);
        assert_eq!(score.total_submissions, 1);
        assert_eq!(score.successful_submissions, 1);
        assert_eq!(score.accuracy_rate(), 100.0);
        
        // Test failed submission
        score.update_score(false);
        assert_eq!(score.total_submissions, 2);
        assert_eq!(score.successful_submissions, 1);
        assert_eq!(score.accuracy_rate(), 50.0);
    }

    #[test]
    fn test_api_response_creation() {
        let success_response = ApiResponse::success("test data");
        assert!(success_response.success);
        assert_eq!(success_response.data, Some("test data"));

        let error_response = ApiResponse::<()>::error("test error".to_string());
        assert!(!error_response.success);
        assert_eq!(error_response.message, Some("test error".to_string()));
    }
}