use chrono;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod analysis_result;

// Re-export commonly used types for convenience
pub use analysis_result::{
    AnalysisResult,
    AnalysisStatus,
    BehavioralAnalysis,
    DetectionResult,
    EngineType,
    ExecutableInfo,
    FileMetadata,
    FileOperation,
    NetworkIndicators,
    NetworkOperation,
    ProcessOperation,
    RegistryOperation,
    SectionInfo,
    SeverityLevel,
    SignatureInfo,
    ThreatCategory,
    ThreatVerdict,
    YaraMatch,
    YaraString,
    YaraStringInstance,
};

/// Configuration for an analysis engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Unique identifier for the engine
    pub engine_id: Uuid,
    /// Human-readable name of the engine
    pub name: String,
    /// Engine version
    pub version: String,
    /// Type of analysis this engine performs
    pub engine_type: EngineType,
    /// Whether the engine is currently enabled
    pub enabled: bool,
    /// Priority/weight for consensus calculation (1-10)
    pub priority: u8,
    /// Maximum timeout for this engine in seconds
    pub timeout_seconds: u32,
    /// Engine-specific configuration parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Resource limits for the engine
    pub resource_limits: ResourceLimits,
    /// Supported file types (MIME types)
    pub supported_file_types: Vec<String>,
    /// Maximum file size this engine can handle (in bytes)
    pub max_file_size: u64,
    /// Engine reputation score (0.0 to 1.0)
    pub reputation_score: f32,
}

/// Resource limits for analysis engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: u32,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: u8,
    /// Maximum disk space for temporary files in MB
    pub max_disk_mb: u32,
    /// Maximum network bandwidth in KB/s
    pub max_network_kbps: Option<u32>,
}

/// Analysis request submitted to the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    /// Unique identifier for this analysis request
    pub request_id: Uuid,
    /// Submission ID from the API gateway
    pub submission_id: Uuid,
    /// Bounty ID if this is part of a bounty program
    pub bounty_id: Option<Uuid>,
    /// File information to analyze
    pub file_info: FileSubmission,
    /// Priority of this analysis (1-10, 10 being highest)
    pub priority: u8,
    /// Specific engines to use (if None, use all applicable engines)
    pub requested_engines: Option<Vec<String>>,
    /// Analysis options
    pub options: AnalysisOptions,
    /// Requester information
    pub requester: RequesterInfo,
}

/// File submission information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSubmission {
    /// Original filename (if available)
    pub filename: Option<String>,
    /// File path in the storage system
    pub file_path: String,
    /// File size in bytes
    pub file_size: u64,
    /// MIME type
    pub mime_type: String,
    /// Pre-computed hashes (optional, will be computed if missing)
    pub hashes: Option<FileHashes>,
    /// File upload timestamp
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
}

/// Pre-computed file hashes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashes {
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: Option<String>,
}

/// Analysis options and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    /// Perform deep static analysis
    pub deep_static_analysis: bool,
    /// Run behavioral analysis (requires sandbox)
    pub behavioral_analysis: bool,
    /// Maximum analysis time in seconds
    pub max_analysis_time: u32,
    /// Include network indicators extraction
    pub extract_network_indicators: bool,
    /// Run YARA rules
    pub yara_analysis: bool,
    /// Custom YARA rules to include
    pub custom_yara_rules: Vec<String>,
    /// Generate detailed report
    pub detailed_report: bool,
    /// Store analysis artifacts
    pub store_artifacts: bool,
}

/// Information about the analysis requester
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequesterInfo {
    /// User ID (if authenticated)
    pub user_id: Option<Uuid>,
    /// Organization ID (if applicable)
    pub organization_id: Option<Uuid>,
    /// API key used for the request
    pub api_key_id: Option<Uuid>,
    /// IP address of the requester
    pub ip_address: String,
    /// User agent string
    pub user_agent: Option<String>,
}

/// Engine execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineExecution {
    /// Engine that performed the analysis
    pub engine_id: Uuid,
    /// Engine name
    pub engine_name: String,
    /// Execution status
    pub status: ExecutionStatus,
    /// Start time
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// End time (if completed)
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Processing time in milliseconds
    pub processing_time_ms: Option<u64>,
    /// Detection result (if successful)
    pub result: Option<DetectionResult>,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Resource usage during execution
    pub resource_usage: Option<ResourceUsage>,
}

/// Execution status of an engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Timeout,
    Cancelled,
    Skipped,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Peak memory usage in MB
    pub peak_memory_mb: u32,
    /// Average CPU usage percentage
    pub avg_cpu_percent: f32,
    /// Disk space used in MB
    pub disk_usage_mb: u32,
    /// Network data transferred in KB
    pub network_usage_kb: Option<u32>,
}

/// Analysis queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEntry {
    /// Queue entry ID
    pub entry_id: Uuid,
    /// Analysis request
    pub request: AnalysisRequest,
    /// Queue priority (calculated from request priority and other factors)
    pub queue_priority: u32,
    /// When this entry was added to the queue
    pub queued_at: chrono::DateTime<chrono::Utc>,
    /// Number of retry attempts
    pub retry_count: u8,
    /// Maximum retry attempts allowed
    pub max_retries: u8,
    /// Engines assigned to this analysis
    pub assigned_engines: Vec<Uuid>,
    /// Current status
    pub status: QueueStatus,
}

/// Queue entry status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueueStatus {
    Pending,
    Assigned,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Analysis statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    /// Total number of analyses performed
    pub total_analyses: u64,
    /// Number of analyses by verdict
    pub verdict_counts: HashMap<ThreatVerdict, u64>,
    /// Average processing time by engine type
    pub avg_processing_times: HashMap<EngineType, f64>,
    /// Engine accuracy rates
    pub engine_accuracy: HashMap<String, f32>,
    /// File type distribution
    pub file_type_distribution: HashMap<String, u64>,
    /// Analysis volume over time
    pub hourly_volumes: Vec<HourlyVolume>,
}

/// Hourly analysis volume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyVolume {
    pub hour: chrono::DateTime<chrono::Utc>,
    pub count: u64,
    pub avg_processing_time_ms: f64,
}

/// Cache entry for analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// File hash (SHA256)
    pub file_hash: String,
    /// Cached analysis result
    pub analysis_result: AnalysisResult,
    /// When this entry was cached
    pub cached_at: chrono::DateTime<chrono::Utc>,
    /// Cache expiry time
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Number of times this cache entry was used
    pub hit_count: u32,
}

/// Error types specific to the analysis engine
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    #[error("File too large: {size} bytes (max: {max_size} bytes)")]
    FileTooLarge { size: u64, max_size: u64 },
    #[error("Unsupported file type: {mime_type}")]
    UnsupportedFileType { mime_type: String },
    #[error("Engine '{engine_name}' timed out after {timeout_seconds} seconds")]
    EngineTimeout { engine_name: String, timeout_seconds: u32 },
    #[error("Engine '{engine_name}' error: {error}")]
    EngineError { engine_name: String, error: String },
    #[error("Insufficient resources: {resource_type}")]
    InsufficientResources { resource_type: String },
    #[error("Invalid request: {reason}")]
    InvalidRequest { reason: String },
    #[error("Database error: {0}")]
    DatabaseError(#[from] anyhow::Error),
    #[error("Storage error: {0}")]
    StorageError(#[from] anyhow::Error),
    #[error("Network error: {0}")]
    NetworkError(#[from] anyhow::Error),
    #[error("Configuration error: {error}")]
    ConfigurationError { error: String },
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisError::FileNotFound { path } => {
                write!(f, "File not found: {}", path)
            }
            AnalysisError::FileTooLarge { size, max_size } => {
                write!(f, "File too large: {} bytes (max: {} bytes)", size, max_size)
            }
            AnalysisError::UnsupportedFileType { mime_type } => {
                write!(f, "Unsupported file type: {}", mime_type)
            }
            AnalysisError::EngineTimeout { engine_name, timeout_seconds } => {
                write!(f, "Engine '{}' timed out after {} seconds", engine_name, timeout_seconds)
            }
            AnalysisError::EngineError { engine_name, error } => {
                write!(f, "Engine '{}' error: {}", engine_name, error)
            }
            AnalysisError::InsufficientResources { resource_type } => {
                write!(f, "Insufficient resources: {}", resource_type)
            }
            AnalysisError::InvalidRequest { reason } => {
                write!(f, "Invalid request: {}", reason)
            }
            AnalysisError::DatabaseError { error } => {
                write!(f, "Database error: {}", error)
            }
            AnalysisError::StorageError { error } => {
                write!(f, "Storage error: {}", error)
            }
            AnalysisError::NetworkError { error } => {
                write!(f, "Network error: {}", error)
            }
            AnalysisError::ConfigurationError { error } => {
                write!(f, "Configuration error: {}", error)
            }
        }
    }
}

impl std::error::Error for AnalysisError {}

/// Result type for analysis operations
pub type AnalysisEngineResult<T> = Result<T, AnalysisError>;

// Default implementations for common configurations
impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            deep_static_analysis: true,
            behavioral_analysis: false,
            max_analysis_time: 300, // 5 minutes
            extract_network_indicators: true,
            yara_analysis: true,
            custom_yara_rules: Vec::new(),
            detailed_report: true,
            store_artifacts: false,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024, // 1GB
            max_cpu_percent: 80,
            max_disk_mb: 512, // 512MB
            max_network_kbps: Some(1024), // 1MB/s
        }
    }
}

impl EngineConfig {
    /// Create a new engine configuration
    pub fn new(name: String, engine_type: EngineType) -> Self {
        Self {
            engine_id: Uuid::new_v4(),
            name,
            version: "1.0.0".to_string(),
            engine_type,
            enabled: true,
            priority: 5,
            timeout_seconds: 300,
            parameters: HashMap::new(),
            resource_limits: ResourceLimits::default(),
            supported_file_types: vec!["*/*".to_string()], // Accept all by default
            max_file_size: 100 * 1024 * 1024, // 100MB
            reputation_score: 0.5, // Neutral starting reputation
        }
    }

    /// Check if this engine supports the given file type
    pub fn supports_file_type(&self, mime_type: &str) -> bool {
        self.supported_file_types.contains(&"*/*".to_string()) ||
        self.supported_file_types.iter().any(|supported| {
            // Support wildcard matching like "application/*"
            if supported.ends_with("/*") {
                let prefix = &supported[..supported.len() - 2];
                mime_type.starts_with(prefix)
            } else {
                supported == mime_type
            }
        })
    }

    /// Check if this engine can handle the given file size
    pub fn can_handle_file_size(&self, file_size: u64) -> bool {
        file_size <= self.max_file_size
    }
}

impl AnalysisRequest {
    /// Create a new analysis request with default options
    pub fn new(
        submission_id: Uuid,
        file_info: FileSubmission,
        requester: RequesterInfo,
    ) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            submission_id,
            bounty_id: None,
            file_info,
            priority: 5,
            requested_engines: None,
            options: AnalysisOptions::default(),
            requester,
        }
    }

    /// Set bounty ID for this analysis
    pub fn with_bounty(mut self, bounty_id: Uuid) -> Self {
        self.bounty_id = Some(bounty_id);
        self
    }

    /// Set analysis priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10).max(1);
        self
    }

    /// Set specific engines to use
    pub fn with_engines(mut self, engines: Vec<String>) -> Self {
        self.requested_engines = Some(engines);
        self
    }

    /// Set custom analysis options
    pub fn with_options(mut self, options: AnalysisOptions) -> Self {
        self.options = options;
        self
    }
}