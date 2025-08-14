use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents the verdict of a threat analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ThreatVerdict {
    Malicious,
    Benign,
    Suspicious,
    Unknown,
}

// Severity level of detected threats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SeverityLevel {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Type of analysis engine that produced the result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EngineType {
    Static,
    Dynamic,
    Yara,
    Hash,
    Behavioral,
    Sandbox,
    Human,
    Ml,
}

/// Detected threat categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatCategory {
    Malware,
    Ransomware,
    Trojan,
    Virus,
    Worm,
    Adware,
    Spyware,
    Rootkit,
    Phishing,
    Exploit,
    Backdoor,
    Other(String),
}

/// Individual detection result from a specific engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// Unique identifier for this detection
    pub detection_id: Uuid,
    /// Name of the engine that produced this result
    pub engine_name: String,
    /// Version of the engine
    pub engine_version: String,
    /// Type of analysis engine
    pub engine_type: EngineType,
    /// The verdict from this engine
    pub verdict: ThreatVerdict,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Severity assessment
    pub severity: SeverityLevel,
    /// Detected threat categories
    pub categories: Vec<ThreatCategory>,
    /// Engine-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Time when this detection was performed
    pub detected_at: DateTime<Utc>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Any error messages from the engine
    pub error_message: Option<String>,
}

/// YARA rule match information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraMatch {
    /// Name of the matched rule
    pub rule_name: String,
    /// Rule namespace
    pub namespace: Option<String>,
    /// Tags associated with the rule
    pub tags: Vec<String>,
    /// Rule metadata
    pub meta: HashMap<String, String>,
    /// Matched strings and their positions
    pub strings: Vec<YaraString>,
}

/// Information about matched strings in YARA rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraString {
    /// String identifier
    pub identifier: String,
    /// Matched instances
    pub instances: Vec<YaraStringInstance>,
}

/// Individual string match instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraStringInstance {
    /// Offset in the file where match occurred
    pub offset: u64,
    /// Length of the matched string
    pub length: u32,
    /// The actual matched data (first 256 bytes)
    pub matched_data: Option<String>,
}

/// File analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Original filename
    pub filename: Option<String>,
    /// File size in bytes
    pub file_size: u64,
    /// MIME type
    pub mime_type: String,
    /// MD5 hash
    pub md5: String,
    /// SHA1 hash
    pub sha1: String,
    /// SHA256 hash
    pub sha256: String,
    /// SHA512 hash
    pub sha512: Option<String>,
    /// File entropy (randomness measure)
    pub entropy: Option<f64>,
    /// File magic bytes signature
    pub magic_bytes: Option<String>,
    /// PE/ELF/Mach-O specific information
    pub executable_info: Option<ExecutableInfo>,
}

/// Executable file specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableInfo {
    /// Architecture (x86, x64, ARM, etc.)
    pub architecture: String,
    /// Entry point address
    pub entry_point: Option<u64>,
    /// Compilation timestamp
    pub compile_time: Option<DateTime<Utc>>,
    /// Imported libraries/DLLs
    pub imports: Vec<String>,
    /// Exported functions
    pub exports: Vec<String>,
    /// Digital signature information
    pub signature_info: Option<SignatureInfo>,
    /// Sections information
    pub sections: Vec<SectionInfo>,
}

/// Digital signature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    /// Is the signature valid
    pub is_valid: bool,
    /// Signer name
    pub signer: Option<String>,
    /// Certificate issuer
    pub issuer: Option<String>,
    /// Signature algorithm
    pub algorithm: Option<String>,
    /// Signing timestamp
    pub signed_at: Option<DateTime<Utc>>,
}

/// Section information for executables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionInfo {
    /// Section name
    pub name: String,
    /// Virtual address
    pub virtual_address: u64,
    /// Virtual size
    pub virtual_size: u32,
    /// Raw size
    pub raw_size: u32,
    /// Section characteristics/permissions
    pub characteristics: Vec<String>,
    /// Entropy of this section
    pub entropy: Option<f64>,
}

/// Network indicators found during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIndicators {
    /// IP addresses contacted
    pub ip_addresses: Vec<String>,
    /// Domain names contacted
    pub domains: Vec<String>,
    /// URLs accessed
    pub urls: Vec<String>,
    /// Email addresses found
    pub email_addresses: Vec<String>,
    /// Network protocols used
    pub protocols: Vec<String>,
}

/// Behavioral analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralAnalysis {
    /// Files created/modified
    pub file_operations: Vec<FileOperation>,
    /// Registry keys accessed (Windows)
    pub registry_operations: Vec<RegistryOperation>,
    /// Processes spawned
    pub process_operations: Vec<ProcessOperation>,
    /// Network connections made
    pub network_operations: Vec<NetworkOperation>,
    /// System calls made
    pub system_calls: Vec<String>,
}

/// File operation during behavioral analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub operation_type: String, // create, modify, delete, read
    pub file_path: String,
    pub timestamp: DateTime<Utc>,
}

/// Registry operation (Windows specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryOperation {
    pub operation_type: String, // create, modify, delete, query
    pub key_path: String,
    pub value_name: Option<String>,
    pub value_data: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Process operation during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessOperation {
    pub operation_type: String, // create, terminate, inject
    pub process_name: String,
    pub process_id: Option<u32>,
    pub command_line: Option<String>,
    pub parent_process: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Network operation during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOperation {
    pub operation_type: String, // connect, listen, dns_query
    pub protocol: String,
    pub local_address: Option<String>,
    pub remote_address: Option<String>,
    pub port: Option<u16>,
    pub data_size: Option<u64>,
    pub timestamp: DateTime<Utc>,
}

/// Main analysis result containing all information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Unique identifier for this analysis
    pub analysis_id: Uuid,
    /// Identifier of the original submission
    pub submission_id: Uuid,
    /// Bounty ID if this analysis is part of a bounty
    pub bounty_id: Option<Uuid>,
    /// File metadata
    pub file_metadata: FileMetadata,
    /// Overall consensus verdict
    pub consensus_verdict: ThreatVerdict,
    /// Overall confidence score (0.0 to 1.0)
    pub consensus_confidence: f32,
    /// Overall severity assessment
    pub consensus_severity: SeverityLevel,
    /// Individual detection results from different engines
    pub detections: Vec<DetectionResult>,
    /// YARA rule matches
    pub yara_matches: Vec<YaraMatch>,
    /// Network indicators
    pub network_indicators: Option<NetworkIndicators>,
    /// Behavioral analysis results
    pub behavioral_analysis: Option<BehavioralAnalysis>,
    /// Analysis tags
    pub tags: Vec<String>,
    /// Additional context or notes
    pub notes: Option<String>,
    /// When the analysis started
    pub started_at: DateTime<Utc>,
    /// When the analysis completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Total processing time in milliseconds
    pub total_processing_time_ms: Option<u64>,
    /// Analysis status
    pub status: AnalysisStatus,
    /// Error message if analysis failed
    pub error_message: Option<String>,
    /// Cost of this analysis in platform tokens
    pub analysis_cost: Option<u64>,
    /// Reputation scores of engines that participated
    pub engine_reputations: HashMap<String, f32>,
}

/// Status of the analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

impl AnalysisResult {
    /// Create a new analysis result with minimal required fields
    pub fn new(
        submission_id: Uuid,
        file_metadata: FileMetadata,
    ) -> Self {
        Self {
            analysis_id: Uuid::new_v4(),
            submission_id,
            bounty_id: None,
            file_metadata,
            consensus_verdict: ThreatVerdict::Unknown,
            consensus_confidence: 0.0,
            consensus_severity: SeverityLevel::Info,
            detections: Vec::new(),
            yara_matches: Vec::new(),
            network_indicators: None,
            behavioral_analysis: None,
            tags: Vec::new(),
            notes: None,
            started_at: Utc::now(),
            completed_at: None,
            total_processing_time_ms: None,
            status: AnalysisStatus::Pending,
            error_message: None,
            analysis_cost: None,
            engine_reputations: HashMap::new(),
        }
    }

    /// Add a detection result from an engine
    pub fn add_detection(&mut self, detection: DetectionResult) {
        self.detections.push(detection);
        self.update_consensus();
    }

    /// Calculate consensus verdict based on all detections
    fn update_consensus(&mut self) {
        if self.detections.is_empty() {
            return;
        }

        let mut malicious_count = 0;
        let mut benign_count = 0;
        let mut suspicious_count = 0;
        let mut total_confidence = 0.0;
        let mut max_severity = SeverityLevel::Info;

        for detection in &self.detections {
            total_confidence += detection.confidence;
            
            match detection.verdict {
                ThreatVerdict::Malicious => malicious_count += 1,
                ThreatVerdict::Benign => benign_count += 1,
                ThreatVerdict::Suspicious => suspicious_count += 1,
                ThreatVerdict::Unknown => {}
            }

            // Update maximum severity
            max_severity = match (&max_severity, &detection.severity) {
                (_, SeverityLevel::Critical) => SeverityLevel::Critical,
                (SeverityLevel::Critical, _) => SeverityLevel::Critical,
                (_, SeverityLevel::High) => SeverityLevel::High,
                (SeverityLevel::High, _) => SeverityLevel::High,
                (_, SeverityLevel::Medium) => SeverityLevel::Medium,
                (SeverityLevel::Medium, _) => SeverityLevel::Medium,
                (_, SeverityLevel::Low) => SeverityLevel::Low,
                _ => max_severity,
            };
        }

        self.consensus_confidence = total_confidence / self.detections.len() as f32;
        self.consensus_severity = max_severity;

        // Simple majority voting for consensus
        if malicious_count > benign_count && malicious_count > suspicious_count {
            self.consensus_verdict = ThreatVerdict::Malicious;
        } else if benign_count > malicious_count && benign_count > suspicious_count {
            self.consensus_verdict = ThreatVerdict::Benign;
        } else if suspicious_count > 0 {
            self.consensus_verdict = ThreatVerdict::Suspicious;
        } else {
            self.consensus_verdict = ThreatVerdict::Unknown;
        }
    }

    /// Mark analysis as completed
    pub fn mark_completed(&mut self) {
        self.completed_at = Some(Utc::now());
        self.status = AnalysisStatus::Completed;
        
        if let Some(completed) = self.completed_at {
            self.total_processing_time_ms = Some(
                (completed - self.started_at).num_milliseconds() as u64
            );
        }
    }

    /// Mark analysis as failed
    pub fn mark_failed(&mut self, error: String) {
        self.status = AnalysisStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
    }

    /// Check if analysis has high confidence malicious verdict
    pub fn is_high_confidence_malicious(&self) -> bool {
        matches!(self.consensus_verdict, ThreatVerdict::Malicious) 
            && self.consensus_confidence >= 0.8
    }

    /// Get unique threat categories across all detections
    pub fn get_all_threat_categories(&self) -> Vec<ThreatCategory> {
        let mut categories = Vec::new();
        for detection in &self.detections {
            for category in &detection.categories {
                if !categories.iter().any(|c| std::mem::discriminant(c) == std::mem::discriminant(category)) {
                    categories.push(category.clone());
                }
            }
        }
        categories
    }

    /// Get engines that detected this as malicious
    pub fn get_malicious_engines(&self) -> Vec<&String> {
        self.detections
            .iter()
            .filter(|d| matches!(d.verdict, ThreatVerdict::Malicious))
            .map(|d| &d.engine_name)
            .collect()
    }
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self::new(
            Uuid::new_v4(),
            FileMetadata {
                filename: None,
                file_size: 0,
                mime_type: "application/octet-stream".to_string(),
                md5: String::new(),
                sha1: String::new(),
                sha256: String::new(),
                sha512: None,
                entropy: None,
                magic_bytes: None,
                executable_info: None,
            }
        )
    }
}