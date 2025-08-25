use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Core identifier types
pub type UserId = Uuid;
pub type BountyId = Uuid;
pub type AnalysisId = Uuid;
pub type SubmissionId = Uuid;
pub type EngineId = String;

// Blockchain related types
pub type EthereumAddress = String;
pub type TransactionHash = String;
pub type BlockNumber = u64;
pub type TokenAmount = u128; // Wei amount for precision

// Analysis types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreatVerdict {
    Malicious,
    Benign,
    Suspicious,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineType {
    Human,
    Automated,
    Hybrid,
}

// File and URL analysis targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisTarget {
    File {
        filename: String,
        file_hash: String,
        file_size: u64,
        mime_type: String,
        content_url: String, // Pre-signed URL for file access
    },
    Url {
        url: String,
        domain: String,
    },
    Hash {
        hash_type: HashType,
        hash_value: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HashType {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

// Bounty related structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyInfo {
    pub id: BountyId,
    pub creator: UserId,
    pub title: String,
    pub description: String,
    pub reward_amount: TokenAmount,
    pub stake_requirement: TokenAmount,
    pub target: AnalysisTarget,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: BountyStatus,
    pub max_submissions: Option<u32>,
    pub current_submissions: u32,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BountyStatus {
    Active,
    Completed,
    Expired,
    Cancelled,
    InReview,
}

// Analysis results and submissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSubmission {
    pub id: SubmissionId,
    pub bounty_id: BountyId,
    pub engine_id: EngineId,
    pub engine_type: EngineType,
    pub submitter: UserId,
    pub verdict: ThreatVerdict,
    pub confidence_score: f32, // 0.0 to 1.0
    pub stake_amount: TokenAmount,
    pub analysis_data: AnalysisData,
    pub submitted_at: DateTime<Utc>,
    pub status: SubmissionStatus,
    pub reputation_impact: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubmissionStatus {
    Pending,
    Validated,
    Rejected,
    Rewarded,
    Slashed, // When stake is lost due to incorrect analysis
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisData {
    pub threat_families: Vec<String>,
    pub iocs: Vec<IoC>, // Indicators of Compromise
    pub yara_matches: Vec<YaraMatch>,
    pub static_analysis: Option<StaticAnalysisResult>,
    pub dynamic_analysis: Option<DynamicAnalysisResult>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub raw_output: Option<String>,
}

// Indicators of Compromise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoC {
    pub ioc_type: IoCType,
    pub value: String,
    pub description: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IoCType {
    Domain,
    IpAddress,
    Url,
    FileHash,
    EmailAddress,
    Registry,
    Mutex,
    Process,
    Service,
}

// YARA rule matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraMatch {
    pub rule_name: String,
    pub namespace: Option<String>,
    pub tags: Vec<String>,
    pub meta: HashMap<String, String>,
    pub strings: Vec<YaraString>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraString {
    pub identifier: String,
    pub matches: Vec<YaraStringMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraStringMatch {
    pub offset: u64,
    pub matched_length: u32,
    pub matched_data: Option<String>,
}

// Analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysisResult {
    pub file_type: String,
    pub entropy: f32,
    pub pe_info: Option<PEInfo>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub sections: Vec<SectionInfo>,
    pub strings: Vec<ExtractedString>,
    pub certificates: Vec<CertificateInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PEInfo {
    pub machine: String,
    pub timestamp: DateTime<Utc>,
    pub characteristics: Vec<String>,
    pub subsystem: String,
    pub entry_point: u64,
    pub image_base: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionInfo {
    pub name: String,
    pub virtual_address: u64,
    pub virtual_size: u64,
    pub raw_size: u64,
    pub characteristics: Vec<String>,
    pub entropy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedString {
    pub value: String,
    pub encoding: StringEncoding,
    pub offset: u64,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringEncoding {
    Ascii,
    Unicode,
    Base64,
    Hex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub thumbprint: String,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicAnalysisResult {
    pub execution_time: u64, // milliseconds
    pub network_activity: Vec<NetworkActivity>,
    pub file_operations: Vec<FileOperation>,
    pub registry_operations: Vec<RegistryOperation>,
    pub process_activity: Vec<ProcessActivity>,
    pub api_calls: Vec<ApiCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkActivity {
    pub protocol: String,
    pub source_ip: String,
    pub source_port: u16,
    pub dest_ip: String,
    pub dest_port: u16,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub operation: FileOpType,
    pub path: String,
    pub size: Option<u64>,
    pub hash: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOpType {
    Create,
    Read,
    Write,
    Delete,
    Move,
    Copy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryOperation {
    pub operation: RegistryOpType,
    pub key: String,
    pub value_name: Option<String>,
    pub value_data: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryOpType {
    CreateKey,
    DeleteKey,
    SetValue,
    DeleteValue,
    QueryValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessActivity {
    pub pid: u32,
    pub process_name: String,
    pub command_line: String,
    pub parent_pid: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub terminated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCall {
    pub api_name: String,
    pub module: String,
    pub parameters: Vec<String>,
    pub return_value: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// User and reputation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub ethereum_address: EthereumAddress,
    pub reputation_score: i32,
    pub total_submissions: u32,
    pub successful_submissions: u32,
    pub total_earned: TokenAmount,
    pub total_staked: TokenAmount,
    pub specializations: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub is_verified: bool,
    pub engine_info: Option<EngineInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub engine_id: EngineId,
    pub engine_name: String,
    pub engine_type: EngineType,
    pub version: String,
    pub description: String,
    pub supported_file_types: Vec<String>,
    pub api_endpoint: Option<String>,
    pub performance_metrics: EngineMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetrics {
    pub accuracy_rate: f32,
    pub response_time_avg: u64, // milliseconds
    pub total_analyses: u32,
    pub false_positives: u32,
    pub false_negatives: u32,
    pub last_updated: DateTime<Utc>,
}

// API response structures
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
}

// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketMessage {
    BountyCreated(BountyInfo),
    BountyUpdated(BountyInfo),
    SubmissionReceived {
        bounty_id: BountyId,
        submission_id: SubmissionId,
        engine_id: EngineId,
    },
    AnalysisCompleted {
        bounty_id: BountyId,
        final_verdict: ThreatVerdict,
        confidence: f32,
    },
    ReputationUpdated {
        user_id: UserId,
        old_score: i32,
        new_score: i32,
    },
    PaymentProcessed {
        bounty_id: BountyId,
        recipient: UserId,
        amount: TokenAmount,
        tx_hash: TransactionHash,
    },
}

// Configuration and constants
pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB
pub const MIN_CONFIDENCE_SCORE: f32 = 0.0;
pub const MAX_CONFIDENCE_SCORE: f32 = 1.0;
pub const DEFAULT_BOUNTY_DURATION: i64 = 24 * 60 * 60; // 24 hours in seconds
pub const MIN_STAKE_AMOUNT: TokenAmount = 1_000_000_000_000_000_000; // 1 token in wei
pub const MAX_ANALYSIS_TIME: u64 = 30 * 60 * 1000; // 30 minutes in milliseconds

// Helper functions
impl ThreatVerdict {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "malicious" => Ok(ThreatVerdict::Malicious),
            "benign" => Ok(ThreatVerdict::Benign),
            "suspicious" => Ok(ThreatVerdict::Suspicious),
            "unknown" => Ok(ThreatVerdict::Unknown),
            _ => Err(format!("Invalid threat verdict: {}", s)),
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            ThreatVerdict::Malicious => "malicious".to_string(),
            ThreatVerdict::Benign => "benign".to_string(),
            ThreatVerdict::Suspicious => "suspicious".to_string(),
            ThreatVerdict::Unknown => "unknown".to_string(),
        }
    }
}

impl Default for ThreatVerdict {
    fn default() -> Self {
        ThreatVerdict::Unknown
    }
}

impl AnalysisTarget {
    pub fn get_identifier(&self) -> String {
        match self {
            AnalysisTarget::File { file_hash, .. } => file_hash.clone(),
            AnalysisTarget::Url { url, .. } => url.clone(),
            AnalysisTarget::Hash { hash_value, .. } => hash_value.clone(),
        }
    }
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum CommonError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Parsing error: {0}")]
    ParseError(String),
}