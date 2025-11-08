use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message.into()),
            timestamp: Utc::now(),
        }
    }
}

impl ApiResponse<()> {
    /// Create an error response
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.into()),
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

    /// Create a success response with only a message
    pub fn success_message(message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: None,
            message: Some(message.into()),
            timestamp: Utc::now(),
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
    pub has_more: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, limit: u32) -> Self {
        let total_pages = if limit > 0 {
            ((total as f64) / (limit as f64)).ceil() as u32
        } else {
            0
        };
        let has_more = page < total_pages;

        Self {
            items,
            total,
            page,
            limit,
            total_pages,
            has_more,
        }
    }

    pub fn empty(page: u32, limit: u32) -> Self {
        Self {
            items: Vec::new(),
            total: 0,
            page,
            limit,
            total_pages: 0,
            has_more: false,
        }
    }
}

/// Authentication responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub ethereum_address: Option<String>,
    pub reputation_score: f64,
    pub total_submissions: u32,
    pub successful_submissions: u32,
    pub accuracy_rate: f64,
    pub rank: Option<u32>,
    pub is_verified: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

/// Bounty responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: String,
    pub total_reward: String,
    pub minimum_stake: String,
    pub current_participants: u32,
    pub max_participants: Option<u32>,
    pub required_consensus: f64,
    pub minimum_reputation: f64,
    pub deadline: Option<DateTime<Utc>>,
    pub creator_address: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub time_remaining: Option<String>,
    pub participation_rate: f64,
    pub can_participate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyDetailsResponse {
    pub bounty: BountyResponse,
    pub participants: Vec<ParticipantResponse>,
    pub submissions: Vec<SubmissionResponse>,
    pub stats: BountyStatsResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantResponse {
    pub engine_id: Uuid,
    pub engine_name: String,
    pub engine_address: String,
    pub stake_amount: String,
    pub joined_at: DateTime<Utc>,
    pub submission_count: u32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionResponse {
    pub id: Uuid,
    pub engine_id: Uuid,
    pub engine_name: String,
    pub verdict: String,
    pub confidence: f64,
    pub stake_amount: String,
    pub submitted_at: DateTime<Utc>,
    pub is_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyStatsResponse {
    pub total_submissions: u32,
    pub unique_participants: u32,
    pub consensus_reached: bool,
    pub consensus_verdict: Option<String>,
    pub consensus_confidence: Option<f64>,
    pub total_stake_pool: String,
    pub avg_response_time: Option<f64>,
}

/// Analysis responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub id: Uuid,
    pub file_hash: String,
    pub file_name: Option<String>,
    pub status: String,
    pub verdict: Option<String>,
    pub confidence: Option<f64>,
    pub threat_types: Vec<String>,
    pub risk_score: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub processing_time_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisDetailsResponse {
    pub analysis: AnalysisResponse,
    pub engine_results: Vec<EngineResultResponse>,
    pub threat_indicators: Vec<ThreatIndicatorResponse>,
    pub file_metadata: Option<FileMetadataResponse>,
    pub yara_matches: Vec<YaraMatchResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineResultResponse {
    pub engine_id: Uuid,
    pub engine_name: String,
    pub verdict: String,
    pub confidence: f64,
    pub threat_types: Vec<String>,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicatorResponse {
    pub indicator_type: String,
    pub value: String,
    pub severity: String,
    pub confidence: f64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadataResponse {
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub file_type: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub entropy: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraMatchResponse {
    pub rule_name: String,
    pub rule_namespace: Option<String>,
    pub tags: Vec<String>,
    pub severity: String,
}

/// Wallet responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletResponse {
    pub address: String,
    pub balance: String,
    pub staked: String,
    pub available: String,
    pub pending_rewards: String,
    pub total_earned: String,
    pub total_spent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub transaction_hash: Option<String>,
    pub transaction_type: String,
    pub amount: String,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub confirmations: Option<u32>,
    pub metadata: serde_json::Value,
}

/// Reputation responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub rank: u32,
    pub user_id: Uuid,
    pub username: String,
    pub reputation_score: f64,
    pub total_submissions: u32,
    pub successful_submissions: u32,
    pub accuracy_rate: f64,
    pub total_earnings: String,
    pub badges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationHistoryResponse {
    pub id: Uuid,
    pub event_type: String,
    pub score_change: f64,
    pub new_score: f64,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub tier: String,
    pub earned_at: Option<DateTime<Utc>>,
    pub progress: Option<f64>,
}

/// Statistics responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatsResponse {
    pub total_bounties: u64,
    pub active_bounties: u64,
    pub total_analyses: u64,
    pub total_users: u64,
    pub total_engines: u64,
    pub total_rewards_distributed: String,
    pub avg_analysis_time_seconds: f64,
    pub consensus_accuracy_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatsResponse {
    pub total_analyses: u64,
    pub total_bounties_created: u64,
    pub total_bounties_participated: u64,
    pub total_rewards_earned: String,
    pub total_rewards_paid: String,
    pub average_accuracy: f64,
    pub streak_days: u32,
    pub rank: u32,
    pub percentile: f64,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: DateTime<Utc>,
    pub services: ServiceStatusResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatusResponse {
    pub database: ServiceHealth,
    pub redis: ServiceHealth,
    pub blockchain: ServiceHealth,
    pub analysis_engine: ServiceHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: String, // "healthy", "degraded", "unhealthy"
    pub response_time_ms: Option<u64>,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
}

/// Webhook responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub delivery_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDeliveryResponse {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_type: String,
    pub status: String,
    pub status_code: Option<u16>,
    pub error_message: Option<String>,
    pub attempt_number: u32,
    pub triggered_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Batch operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse<T> {
    pub results: Vec<BatchOperationResult<T>>,
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationResult<T> {
    pub index: usize,
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// Notification response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// API key response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub key: String, // Only shown once during creation
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Rate limit info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
    pub window_seconds: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
    }

    #[test]
    fn test_paginated_response() {
        let response = PaginatedResponse::new(
            vec!["item1", "item2"],
            10,
            1,
            5,
        );
        assert_eq!(response.total_pages, 2);
        assert!(response.has_more);
    }

    #[test]
    fn test_empty_paginated_response() {
        let response: PaginatedResponse<String> = PaginatedResponse::empty(1, 20);
        assert_eq!(response.total, 0);
        assert!(!response.has_more);
    }
}
