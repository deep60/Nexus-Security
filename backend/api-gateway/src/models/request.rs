use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Common pagination request parameters
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    pub page: u32,

    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
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

    pub fn validate_and_normalize(&mut self) -> Result<(), String> {
        if self.page == 0 {
            return Err("Page must be at least 1".to_string());
        }
        if self.limit == 0 || self.limit > 100 {
            return Err("Limit must be between 1 and 100".to_string());
        }
        Ok(())
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            limit: default_limit(),
        }
    }
}

/// Sorting parameters
#[derive(Debug, Clone, Deserialize)]
pub struct SortParams {
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Desc
    }
}

/// Date range filter
#[derive(Debug, Clone, Deserialize)]
pub struct DateRangeParams {
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

impl DateRangeParams {
    pub fn validate(&self) -> Result<(), String> {
        if let (Some(from), Some(to)) = (self.from_date, self.to_date) {
            if from > to {
                return Err("from_date must be before to_date".to_string());
            }
        }
        Ok(())
    }
}

/// Authentication requests
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50, message = "Username must be 3-50 characters"))]
    pub username: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    pub ethereum_address: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,

    pub remember_me: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogoutRequest {
    pub logout_all_devices: Option<bool>,
}

/// Wallet connection request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ConnectWalletRequest {
    #[validate(length(equal = 42, message = "Invalid Ethereum address"))]
    pub address: String,

    pub signature: String,

    #[validate(length(min = 1, message = "Message is required"))]
    pub message: String,

    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DisconnectWalletRequest {
    pub address: String,
}

/// File upload request (metadata)
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct FileUploadRequest {
    #[validate(length(min = 1, max = 255, message = "Filename must be 1-255 characters"))]
    pub filename: String,

    pub file_size: u64,

    pub mime_type: Option<String>,

    pub description: Option<String>,

    pub tags: Option<Vec<String>>,
}

/// URL submission request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UrlSubmissionRequest {
    #[validate(url(message = "Invalid URL format"))]
    pub url: String,

    pub priority: Option<String>,

    pub description: Option<String>,

    pub tags: Option<Vec<String>>,
}

/// Bounty creation request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateBountyRequest {
    #[validate(length(min = 5, max = 200, message = "Title must be 5-200 characters"))]
    pub title: String,

    #[validate(length(min = 20, max = 5000, message = "Description must be 20-5000 characters"))]
    pub description: String,

    pub bounty_type: String,

    pub priority: Option<String>,

    #[validate(length(min = 1, message = "Total reward is required"))]
    pub total_reward: String,

    pub minimum_stake: Option<String>,

    pub distribution_method: Option<String>,

    pub max_participants: Option<i32>,

    pub required_consensus: Option<f64>,

    pub minimum_reputation: Option<f64>,

    pub deadline_hours: Option<i32>,

    pub auto_finalize: Option<bool>,

    pub requires_human_analysis: Option<bool>,

    pub file_types_allowed: Option<Vec<String>>,

    pub max_file_size: Option<i64>,

    pub tags: Option<Vec<String>>,

    pub metadata: Option<serde_json::Value>,
}

/// Analysis submission request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SubmitAnalysisRequest {
    pub bounty_id: Uuid,

    pub verdict: String, // "benign", "malicious", "suspicious", "unknown"

    #[validate(range(min = 0.0, max = 1.0, message = "Confidence must be between 0 and 1"))]
    pub confidence: f64,

    #[validate(length(min = 1, message = "Stake amount is required"))]
    pub stake_amount: String,

    pub threat_types: Option<Vec<String>>,

    pub threat_indicators: Option<Vec<ThreatIndicatorRequest>>,

    pub signatures: Option<Vec<String>>,

    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThreatIndicatorRequest {
    pub indicator_type: String,
    pub value: String,
    pub severity: String,
    pub confidence: f64,
    pub description: Option<String>,
}

/// Stake management
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct StakeRequest {
    pub bounty_id: Uuid,

    #[validate(length(min = 1, message = "Amount is required"))]
    pub amount: String,

    pub auto_participate: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnstakeRequest {
    pub bounty_id: Uuid,
    pub amount: Option<String>, // If None, unstake all
}

/// Dispute management
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateDisputeRequest {
    pub analysis_id: Uuid,

    pub disputed_engine_id: Uuid,

    #[validate(length(min = 20, max = 2000, message = "Reason must be 20-2000 characters"))]
    pub reason: String,

    pub evidence: Option<serde_json::Value>,

    #[validate(length(min = 1, message = "Stake amount is required"))]
    pub stake_amount: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ResolveDisputeRequest {
    pub dispute_id: Uuid,

    pub resolution: String, // "accepted", "rejected"

    #[validate(length(min = 10, max = 1000, message = "Resolution details required"))]
    pub resolution_details: String,
}

/// Webhook management
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RegisterWebhookRequest {
    #[validate(url(message = "Invalid webhook URL"))]
    pub url: String,

    #[validate(length(min = 1, message = "At least one event is required"))]
    pub events: Vec<String>,

    pub description: Option<String>,

    pub headers: Option<serde_json::Value>,

    pub retry_policy: Option<WebhookRetryPolicy>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebhookRetryPolicy {
    pub max_attempts: u32,
    pub retry_interval_seconds: u64,
    pub exponential_backoff: bool,
}

impl Default for WebhookRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            retry_interval_seconds: 60,
            exponential_backoff: true,
        }
    }
}

/// Profile update
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 3, max = 50, message = "Username must be 3-50 characters"))]
    pub username: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,

    pub ethereum_address: Option<String>,

    #[validate(length(max = 500, message = "Bio must be under 500 characters"))]
    pub bio: Option<String>,

    pub avatar_url: Option<String>,

    pub notification_preferences: Option<serde_json::Value>,
}

/// Reputation update (admin only)
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateReputationRequest {
    pub user_id: Uuid,

    pub score_change: f64,

    #[validate(length(min = 5, max = 200, message = "Reason must be 5-200 characters"))]
    pub reason: String,

    pub metadata: Option<serde_json::Value>,
}

/// Search and filtering requests
#[derive(Debug, Clone, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub filters: Option<serde_json::Value>,
    pub pagination: Option<PaginationParams>,
    pub sort: Option<SortParams>,
}

/// Batch operations
#[derive(Debug, Clone, Deserialize)]
pub struct BatchRequest<T> {
    pub operations: Vec<T>,
    pub fail_on_error: Option<bool>,
}

/// Export request
#[derive(Debug, Clone, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub filters: Option<serde_json::Value>,
    pub fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Csv,
    Pdf,
}

/// API key management
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateApiKeyRequest {
    #[validate(length(min = 3, max = 100, message = "Name must be 3-100 characters"))]
    pub name: String,

    pub permissions: Option<Vec<String>>,

    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RevokeApiKeyRequest {
    pub api_key_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_offset() {
        let params = PaginationParams {
            page: 3,
            limit: 20,
        };
        assert_eq!(params.offset(), 40);
    }

    #[test]
    fn test_date_range_validation() {
        let valid_range = DateRangeParams {
            from_date: Some(Utc::now() - chrono::Duration::days(7)),
            to_date: Some(Utc::now()),
        };
        assert!(valid_range.validate().is_ok());

        let invalid_range = DateRangeParams {
            from_date: Some(Utc::now()),
            to_date: Some(Utc::now() - chrono::Duration::days(7)),
        };
        assert!(invalid_range.validate().is_err());
    }
}
