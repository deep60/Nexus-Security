use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Analysis status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "analysis_status", rename_all = "lowercase")]
pub enum AnalysisStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Disputed,
}

// Threat verdict enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "threat_verdict", rename_all = "lowercase")]
pub enum ThreatVerdict {
    Benign,
    Suspicious,
    Malicious,
    Unknown,
}

// Analysis engine type
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "engine_type", rename_all = "lowercase")]
pub enum EngineType {
    Automated,
    Human,
    Hybrid,
}

// Main analysis record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Analysis {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub submitter_address: String,
    pub file_hash: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub file_type: Option<String>,
    pub url: Option<String>,
    pub status: AnalysisStatus,
    pub consensus_verdict: Option<ThreatVerdict>,
    pub confidence_score: Option<f64>,
    pub total_engines: i32,
    pub completed_engines: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// Individual engine analysis result
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EngineAnalysis {
    pub id: Uuid,
    pub analysis_id: Uuid,
    pub engine_id: Uuid,
    pub engine_name: String,
    pub engine_type: EngineType,
    pub engine_address: String,
    pub verdict: ThreatVerdict,
    pub confidence: f64,
    pub stake_amount: String, // Use string for precise decimal handling
    pub details: serde_json::Value,
    pub signatures: Vec<String>,
    pub is_winner: Option<bool>,
    pub reward_earned: Option<String>,
    pub penalty_applied: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

// Analysis statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisStats {
    pub analysis_id: Uuid,
    pub benign_votes: i32,
    pub suspicious_votes: i32,
    pub malicious_votes: i32,
    pub unknown_votes: i32,
    pub total_stake: String,
    pub avg_confidence: f64,
    pub consensus_threshold: f64,
    pub dispute_count: i32,
}

// YARA rule matches
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct YaraMatch {
    pub id: Uuid,
    pub engine_analysis_id: Uuid,
    pub rule_name: String,
    pub rule_namespace: Option<String>,
    pub rule_tags: Vec<String>,
    pub match_offset: i64,
    pub match_length: i64,
    pub matched_data: String,
    pub severity: String,
}

// File analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub ssdeep: Option<String>,
    pub file_type: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub entropy: Option<f64>,
    pub pe_info: Option<PeInfo>,
    pub strings: Option<Vec<String>>,
    pub imports: Option<Vec<String>>,
    pub exports: Option<Vec<String>>,
    pub last_analysis: Option<DateTime<Utc>>,
}

// PE file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeInfo {
    pub compile_time: Option<DateTime<Utc>>,
    pub entry_point: Option<String>,
    pub sections: Option<Vec<PeSection>>,
    pub imports: Option<Vec<String>>,
    pub exports: Option<Vec<String>>,
    pub digital_signature: Option<DigitalSignature>,
    pub version_info: Option<serde_json::Value>,
}

// PE section info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeSection {
    pub name: String,
    pub virtual_address: String,
    pub virtual_size: i64,
    pub raw_size: i64,
    pub entropy: f64,
    pub md5: String,
}

// Digital signature info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalSignature {
    pub is_signed: bool,
    pub is_valid: Option<bool>,
    pub signer: Option<String>,
    pub issuer: Option<String>,
    pub serial_number: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

// URL analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlMetadata {
    pub domain: String,
    pub subdomain: Option<String>,
    pub path: String,
    pub query_params: Option<serde_json::Value>,
    pub scheme: String,
    pub port: Option<i32>,
    pub ip_address: Option<String>,
    pub whois_info: Option<serde_json::Value>,
    pub ssl_info: Option<SslInfo>,
    pub http_headers: Option<serde_json::Value>,
    pub response_code: Option<i32>,
    pub page_content_hash: Option<String>,
}

// SSL certificate info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslInfo {
    pub is_valid: bool,
    pub issuer: Option<String>,
    pub subject: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub algorithm: Option<String>,
    pub key_size: Option<i32>,
}

// Analysis result for consensus and database storage
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisResult {
    pub id: Uuid,
    pub bounty_id: Uuid,
    pub analysis_id: Uuid,
    pub engine_id: Uuid,
    pub engine_name: String,
    pub verdict: ThreatVerdict,
    pub confidence: f64,
    pub stake_amount: String,
    pub details: serde_json::Value,
    pub threat_indicators: Vec<ThreatIndicator>,
    pub submitted_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

// Threat indicators found in analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    pub id: Uuid,
    pub analysis_result_id: Uuid,
    pub indicator_type: String, // "hash", "domain", "ip", "url", "signature", etc.
    pub value: String,
    pub severity: String, // "low", "medium", "high", "critical"
    pub confidence: f64,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Analysis request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub bounty_id: Uuid,
    pub file_data: Option<Vec<u8>>,
    pub file_name: Option<String>,
    pub url: Option<String>,
    pub priority: Option<i32>,
    pub max_engines: Option<i32>,
    pub required_engine_types: Option<Vec<EngineType>>,
    pub metadata: Option<serde_json::Value>,
}

// Analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub analysis: Analysis,
    pub engine_results: Vec<EngineAnalysis>,
    pub stats: AnalysisStats,
    pub yara_matches: Vec<YaraMatch>,
    pub file_metadata: Option<FileMetadata>,
    pub url_metadata: Option<UrlMetadata>,
}

// Engine registration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisEngine {
    pub id: Uuid,
    pub name: String,
    pub engine_type: EngineType,
    pub address: String,
    pub endpoint_url: Option<String>,
    pub capabilities: Vec<String>,
    pub reputation_score: f64,
    pub total_analyses: i64,
    pub successful_analyses: i64,
    pub total_rewards: String,
    pub total_penalties: String,
    pub is_active: bool,
    pub stake_required: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Engine performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EngineMetrics {
    pub engine_id: Uuid,
    pub time_period: String, // daily, weekly, monthly
    pub total_submissions: i32,
    pub correct_verdicts: i32,
    pub false_positives: i32,
    pub false_negatives: i32,
    pub avg_response_time: f64,
    pub accuracy_rate: f64,
    pub reputation_change: f64,
    pub rewards_earned: String,
    pub penalties_incurred: String,
    pub calculated_at: DateTime<Utc>,
}

// Analysis dispute
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisDispute {
    pub id: Uuid,
    pub analysis_id: Uuid,
    pub disputer_address: String,
    pub disputed_engine_id: Uuid,
    pub reason: String,
    pub evidence: serde_json::Value,
    pub stake_amount: String,
    pub status: DisputeStatus,
    pub resolution: Option<String>,
    pub resolved_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "dispute_status", rename_all = "lowercase")]
pub enum DisputeStatus {
    Open,
    UnderReview,
    Resolved,
    Rejected,
}

impl Analysis {
    pub fn new(bounty_id: Uuid, submitter_address: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            bounty_id,
            submitter_address,
            file_hash: None,
            file_name: None,
            file_size: None,
            file_type: None,
            url: None,
            status: AnalysisStatus::Pending,
            consensus_verdict: None,
            confidence_score: None,
            total_engines: 0,
            completed_engines: 0,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.status, AnalysisStatus::Completed)
    }

    pub fn completion_percentage(&self) -> f64 {
        if self.total_engines == 0 {
            0.0
        } else {
            (self.completed_engines as f64 / self.total_engines as f64) * 100.0
        }
    }
}

impl EngineAnalysis {
    pub fn new(
        analysis_id: Uuid,
        engine_id: Uuid,
        engine_name: String,
        engine_type: EngineType,
        engine_address: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            analysis_id,
            engine_id,
            engine_name,
            engine_type,
            engine_address,
            verdict: ThreatVerdict::Unknown,
            confidence: 0.0,
            stake_amount: "0".to_string(),
            details: serde_json::Value::Object(serde_json::Map::new()),
            signatures: Vec::new(),
            is_winner: None,
            reward_earned: None,
            penalty_applied: None,
            submitted_at: Utc::now(),
            verified_at: None,
        }
    }
}

// Helper functions for consensus calculation
impl AnalysisStats {
    pub fn calculate_consensus(&self) -> (ThreatVerdict, f64) {
        let total_votes = self.benign_votes + self.suspicious_votes + 
                         self.malicious_votes + self.unknown_votes;
        
        if total_votes == 0 {
            return (ThreatVerdict::Unknown, 0.0);
        }

        let max_votes = [
            self.benign_votes,
            self.suspicious_votes,
            self.malicious_votes,
            self.unknown_votes,
        ].iter().max().unwrap();

        let confidence = (*max_votes as f64 / total_votes as f64) * 100.0;

        let verdict = if self.malicious_votes == *max_votes {
            ThreatVerdict::Malicious
        } else if self.suspicious_votes == *max_votes {
            ThreatVerdict::Suspicious
        } else if self.benign_votes == *max_votes {
            ThreatVerdict::Benign
        } else {
            ThreatVerdict::Unknown
        };

        (verdict, confidence)
    }
}