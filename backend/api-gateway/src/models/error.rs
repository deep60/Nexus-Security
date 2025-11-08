use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

/// Application-wide error types
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    #[error("Permission denied: {0}")]
    Forbidden(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Blockchain error: {0}")]
    Blockchain(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("File too large: {0}")]
    FileTooLarge(String),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),

    #[error("Analysis timeout")]
    AnalysisTimeout,

    #[error("Consensus not reached")]
    ConsensusNotReached,

    #[error("Bounty expired")]
    BountyExpired,

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Reputation too low: required {required}, actual {actual}")]
    InsufficientReputation { required: i32, actual: i32 },

    #[error("Maximum participants reached")]
    MaxParticipantsReached,

    #[error("Stake amount too low: minimum {minimum}, provided {provided}")]
    StakeTooLow { minimum: String, provided: String },

    #[error("External API error: {0}")]
    ExternalApi(String),
}

/// Error response structure for JSON API
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorInfo,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn to_error_response(&self, request_id: Option<String>) -> ErrorResponse {
        ErrorResponse {
            error: ErrorInfo {
                code: self.error_code(),
                message: self.to_string(),
                details: self.error_details(),
            },
            timestamp: chrono::Utc::now(),
            request_id,
        }
    }

    fn error_code(&self) -> String {
        match self {
            ApiError::Unauthorized(_) => "UNAUTHORIZED".to_string(),
            ApiError::Forbidden(_) => "FORBIDDEN".to_string(),
            ApiError::NotFound(_) => "NOT_FOUND".to_string(),
            ApiError::BadRequest(_) => "BAD_REQUEST".to_string(),
            ApiError::Validation(_) => "VALIDATION_ERROR".to_string(),
            ApiError::Conflict(_) => "CONFLICT".to_string(),
            ApiError::Database(_) => "DATABASE_ERROR".to_string(),
            ApiError::Blockchain(_) => "BLOCKCHAIN_ERROR".to_string(),
            ApiError::Internal(_) => "INTERNAL_ERROR".to_string(),
            ApiError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE".to_string(),
            ApiError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED".to_string(),
            ApiError::FileTooLarge(_) => "FILE_TOO_LARGE".to_string(),
            ApiError::UnsupportedFileType(_) => "UNSUPPORTED_FILE_TYPE".to_string(),
            ApiError::InsufficientFunds(_) => "INSUFFICIENT_FUNDS".to_string(),
            ApiError::AnalysisTimeout => "ANALYSIS_TIMEOUT".to_string(),
            ApiError::ConsensusNotReached => "CONSENSUS_NOT_REACHED".to_string(),
            ApiError::BountyExpired => "BOUNTY_EXPIRED".to_string(),
            ApiError::InvalidSignature(_) => "INVALID_SIGNATURE".to_string(),
            ApiError::InsufficientReputation { .. } => "INSUFFICIENT_REPUTATION".to_string(),
            ApiError::MaxParticipantsReached => "MAX_PARTICIPANTS_REACHED".to_string(),
            ApiError::StakeTooLow { .. } => "STAKE_TOO_LOW".to_string(),
            ApiError::ExternalApi(_) => "EXTERNAL_API_ERROR".to_string(),
        }
    }

    fn error_details(&self) -> Option<serde_json::Value> {
        match self {
            ApiError::InsufficientReputation { required, actual } => {
                Some(json!({
                    "required_reputation": required,
                    "actual_reputation": actual,
                    "deficit": required - actual,
                }))
            }
            ApiError::StakeTooLow { minimum, provided } => {
                Some(json!({
                    "minimum_stake": minimum,
                    "provided_stake": provided,
                }))
            }
            _ => None,
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Blockchain(_) => StatusCode::BAD_GATEWAY,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            ApiError::FileTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::UnsupportedFileType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ApiError::InsufficientFunds(_) => StatusCode::PAYMENT_REQUIRED,
            ApiError::AnalysisTimeout => StatusCode::REQUEST_TIMEOUT,
            ApiError::ConsensusNotReached => StatusCode::CONFLICT,
            ApiError::BountyExpired => StatusCode::GONE,
            ApiError::InvalidSignature(_) => StatusCode::UNAUTHORIZED,
            ApiError::InsufficientReputation { .. } => StatusCode::FORBIDDEN,
            ApiError::MaxParticipantsReached => StatusCode::CONFLICT,
            ApiError::StakeTooLow { .. } => StatusCode::BAD_REQUEST,
            ApiError::ExternalApi(_) => StatusCode::BAD_GATEWAY,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = self.to_error_response(None);

        (status, Json(error_response)).into_response()
    }
}

// Convert from common error types
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("Resource not found".to_string()),
            sqlx::Error::Database(db_err) => {
                // Check for unique constraint violations
                if let Some(constraint) = db_err.constraint() {
                    ApiError::Conflict(format!("Constraint violation: {}", constraint))
                } else {
                    ApiError::Database(db_err.to_string())
                }
            }
            _ => ApiError::Database(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(format!("JSON parsing error: {}", err))
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::Internal(format!("IO error: {}", err))
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(err: redis::RedisError) -> Self {
        ApiError::ServiceUnavailable(format!("Redis error: {}", err))
    }
}

/// Result type alias for API operations
pub type ApiResult<T> = Result<T, ApiError>;

/// Validation error builder
pub struct ValidationErrorBuilder {
    errors: Vec<FieldError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
    pub code: String,
}

impl ValidationErrorBuilder {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error(mut self, field: impl Into<String>, message: impl Into<String>) -> Self {
        self.errors.push(FieldError {
            field: field.into(),
            message: message.into(),
            code: "VALIDATION_ERROR".to_string(),
        });
        self
    }

    pub fn build(self) -> Result<(), ApiError> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(ApiError::Validation(
                serde_json::to_string(&self.errors).unwrap_or_else(|_| "Validation failed".to_string())
            ))
        }
    }
}

impl Default for ValidationErrorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_generation() {
        let error = ApiError::NotFound("User not found".to_string());
        assert_eq!(error.error_code(), "NOT_FOUND");
    }

    #[test]
    fn test_validation_error_builder() {
        let result = ValidationErrorBuilder::new()
            .add_error("email", "Invalid email format")
            .add_error("password", "Password too short")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_insufficient_reputation_details() {
        let error = ApiError::InsufficientReputation {
            required: 100,
            actual: 50,
        };

        let details = error.error_details();
        assert!(details.is_some());
    }
}
