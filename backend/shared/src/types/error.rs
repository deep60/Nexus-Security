//! Common error types for Nexus Security

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommonError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Redis error: {0}")]
    Redis(String),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("External service error: {0}")]
    ExternalService(String),
    
    #[error("Blockchain error: {0}")]
    Blockchain(String),
    
    #[error("Cryptography error: {0}")]
    Crypto(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl CommonError {
    pub fn http_status_code(&self) -> u16 {
        match self {
            CommonError::ValidationFailed(_) => 400,
            CommonError::InvalidInput(_) => 400,
            CommonError::AuthenticationFailed(_) => 401,
            CommonError::AuthorizationFailed(_) => 403,
            CommonError::NotFound(_) => 404,
            CommonError::AlreadyExists(_) => 409,
            CommonError::RateLimitExceeded(_) => 429,
            CommonError::Timeout(_) => 504,
            _ => 500,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            CommonError::Timeout(_) | CommonError::ExternalService(_)
        )
    }
}

impl From<sqlx::Error> for CommonError {
    fn from(err: sqlx::Error) -> Self {
        CommonError::Database(err.to_string())
    }
}

impl From<redis::RedisError> for CommonError {
    fn from(err: redis::RedisError) -> Self {
        CommonError::Redis(err.to_string())
    }
}

impl From<std::io::Error> for CommonError {
    fn from(err: std::io::Error) -> Self {
        CommonError::Internal(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_status_codes() {
        assert_eq!(CommonError::NotFound("test".to_string()).http_status_code(), 404);
        assert_eq!(CommonError::AuthenticationFailed("test".to_string()).http_status_code(), 401);
        assert_eq!(CommonError::ValidationFailed("test".to_string()).http_status_code(), 400);
    }

    #[test]
    fn test_retryable() {
        assert!(CommonError::Timeout("test".to_string()).is_retryable());
        assert!(CommonError::ExternalService("test".to_string()).is_retryable());
        assert!(!CommonError::ValidationFailed("test".to_string()).is_retryable());
    }
}
