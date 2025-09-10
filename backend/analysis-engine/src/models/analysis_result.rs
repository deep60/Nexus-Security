use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ThreatVerdict {
    Malicious,
    Benign,
    Suspicious,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreatLevel {
    Clean,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchDetails {
    pub rule_name: String,
    pub namespace: Option<String>,
    pub tags: Vec<String>,
    pub meta: HashMap<String, String>,
    pub strings: Vec<YaraString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
pub enum SeverityLevel {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl SeverityLevel {
    pub fn max(self, other: Self) -> Self {
        use SeverityLevel::*;
        match (self, other) {
            (Critical, _) | (_, Critical) => Critical,
            (High, _) | (_, High) => High,
            (Medium, _) | (_, Medium) => Medium,
            (Low, _) | (_, Low) => Low,
            _ => Info,
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub detection_id: Uuid,
    pub engine_name: String,
    pub engine_version: String,
    pub engine_type: EngineType,
    pub verdict: ThreatVerdict,
    pub confidence: f32,
    pub severity: SeverityLevel,
    pub categories: Vec<ThreatCategory>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub detected_at: DateTime<Utc>,
    pub processing_time_ms: u64,
    pub error_message: Option<String>,
}

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
    pub instances: Vec<YaraStringInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraStringInstance {
    pub offset: u64,
    pub length: u32,
    pub matched_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub filename: Option<String>,
    pub file_size: u64,
    pub mime_type: String,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: Option<String>,
    pub entropy: Option<f64>,
    pub magic_bytes: Option<String>,
    pub executable_info: Option<ExecutableInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableInfo {
    pub architecture: String,
    pub entry_point: Option<u64>,
    pub compile_time: Option<DateTime<Utc>>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub signature_info: Option<SignatureInfo>,
    pub sections: Vec<SectionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub is_valid: bool,
    pub signer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionInfo {
    pub name: String,
    pub size: u64,
    pub entropy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIndicators {
    pub urls: Vec<String>,
    pub ips: Vec<String>,
    pub domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralAnalysis {
    pub process_operations: Vec<ProcessOperation>,
    pub file_operations: Vec<FileOperation>,
    pub registry_operations: Vec<RegistryOperation>,
    pub network_operations: Vec<NetworkOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessOperation {
    pub operation: String,
    pub process_id: u32,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub operation: String,
    pub path: String,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryOperation {
    pub operation: String,
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOperation {
    pub operation: String,
    pub destination: String,
    pub protocol: String,
    pub data_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub analysis_id: Uuid,
    pub submission_id: Uuid,
    pub bounty_id: Option<Uuid>,
    pub file_metadata: FileMetadata,
    pub consensus_verdict: ThreatVerdict,
    pub consensus_confidence: f32,
    pub consensus_severity: SeverityLevel,
    pub detections: Vec<DetectionResult>,
    pub yara_matches: Vec<YaraMatch>,
    pub network_indicators: Option<NetworkIndicators>,
    pub behavioral_analysis: Option<BehavioralAnalysis>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub total_processing_time_ms: Option<u64>,
    pub status: AnalysisStatus,
    pub error_message: Option<String>,
    pub analysis_cost: Option<f64>,
    pub engine_reputations: HashMap<String, f32>,
}

impl AnalysisResult {
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

    pub fn add_detection(&mut self, detection: DetectionResult) {
        self.detections.push(detection);
        self.update_consensus();
    }

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

            max_severity = max_severity.max(detection.severity.clone());
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

    pub fn mark_completed(&mut self) {
        self.completed_at = Some(Utc::now());
        self.status = AnalysisStatus::Completed;
        
        if let Some(completed) = self.completed_at {
            self.total_processing_time_ms = Some(
                (completed - self.started_at).num_milliseconds() as u64
            );
        }
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = AnalysisStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
    }

    pub fn is_high_confidence_malicious(&self) -> bool {
        matches!(self.consensus_verdict, ThreatVerdict::Malicious) 
            && self.consensus_confidence >= 0.8
    }

    pub fn get_all_threat_categories(&self) -> Vec<ThreatCategory> {
        let mut categories = Vec::new();
        for detection in &self.detections {
            for category in &detection.categories {
                if !categories.contains(category) {
                    categories.push(category.clone());
                }
            }
        }
        categories
    }

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