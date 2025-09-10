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
    ConfidenceLevel,
    ThreatLevel,
};

/// Configuration for an analysis engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub engine_id: Uuid,
    pub name: String,
    pub version: String,
    pub engine_type: EngineType,
    pub enabled: bool,
    pub priority: u8,
    pub timeout_seconds: u32,
    pub parameters: HashMap<String, serde_json::Value>,
    pub resource_limits: ResourceLimits,
    pub supported_file_types: Vec<String>,
    pub max_file_size: u64,
    pub reputation_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u32,
    pub max_cpu_percent: u8,
    pub max_disk_mb: u32,
    pub max_network_kbps: Option<u32>,
}

/// Analysis request submitted to the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub request_id: Uuid,
    pub submission_id: Uuid,
    pub bounty_id: Option<Uuid>,
    pub file_info: FileSubmission,
    pub priority: u8,
    pub requested_engines: Option<Vec<String>>,
    pub options: AnalysisOptions,
    pub requester: RequesterInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSubmission {
    pub filename: Option<String>,
    pub file_path: String,
    pub file_size: u64,
    pub mime_type: String,
    pub hashes: Option<FileHashes>,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashes {
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    pub deep_static_analysis: bool,
    pub behavioral_analysis: bool,
    pub max_analysis_time: u32,
    pub extract_network_indicators: bool,
    pub yara_analysis: bool,
    pub custom_yara_rules: Vec<String>,
    pub detailed_report: bool,
    pub store_artifacts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequesterInfo {
    pub user_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub api_key_id: Option<Uuid>,
    pub ip_address: String,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineExecution {
    pub engine_id: Uuid,
    pub engine_name: String,
    pub status: ExecutionStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub processing_time_ms: Option<u64>,
    pub result: Option<DetectionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    #[error("File too large: {size} bytes (max: {max_size})")]
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
    #[error("Database error: {error}")]
    DatabaseError { error: String },
    #[error("Storage error: {error}")]
    StorageError { error: String },
    #[error("Network error: {0}")]
    NetworkError(#[from] anyhow::Error),
    #[error("Configuration error: {error}")]
    ConfigurationError { error: String },
}

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

    pub fn supports_file_type(&self, mime_type: &str) -> bool {
        self.supported_file_types.contains(&"*/*".to_string()) ||
        self.supported_file_types.iter().any(|supported| {
            if supported.ends_with("/*") {
                let prefix = &supported[..supported.len() - 2];
                mime_type.starts_with(prefix)
            } else {
                supported == mime_type
            }
        })
    }

    pub fn can_handle_file_size(&self, file_size: u64) -> bool {
        file_size <= self.max_file_size
    }
}

impl AnalysisRequest {
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

    pub fn with_bounty(mut self, bounty_id: Uuid) -> Self {
        self.bounty_id = Some(bounty_id);
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10).max(1);
        self
    }

    pub fn with_engines(mut self, engines: Vec<String>) -> Self {
        self.requested_engines = Some(engines);
        self
    }

    pub fn with_options(mut self, options: AnalysisOptions) -> Self {
        self.options = options;
        self
    }
}