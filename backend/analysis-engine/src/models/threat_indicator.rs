use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

/// Represents the severity level of a threat
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "threat_severity", rename_all = "lowercase")]
pub enum ThreatSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl ThreatSeverity {
    pub fn score(&self) -> u8 {
        match self {
            Self::Critical => 100,
            Self::High => 75,
            Self::Medium => 50,
            Self::Low => 25,
            Self::Info => 0,
        }
    }
}

/// Types of indicators of compromise
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "ioc_type", rename_all = "lowercase")]
pub enum IocType {
    FileHash,
    IpAddress,
    Domain,
    Url,
    Email,
    Registry,
    Mutex,
    Process,
    FilePattern,
    NetworkSignature,
    BehaviorPattern,
}

/// Confidence level of the threat detection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "confidence_level", rename_all = "lowercase")]
pub enum ConfidenceLevel {
    Confirmed,
    High,
    Medium,
    Low,
    Tentative,
}

impl ConfidenceLevel {
    pub fn percentage(&self) -> u8 {
        match self {
            Self::Confirmed => 100,
            Self::High => 80,
            Self::Medium => 60,
            Self::Low => 40,
            Self::Tentative => 20,
        }
    }
}

/// Categories of malware/threat types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "threat_category", rename_all = "lowercase")]
pub enum ThreatCategory {
    Malware,
    Ransomware,
    Trojan,
    Worm,
    Virus,
    Rootkit,
    Spyware,
    Adware,
    Backdoor,
    Exploit,
    Phishing,
    C2,
    Cryptominer,
    Apt,
    PotentiallyUnwanted,
    Suspicious,
    Unknown,
}

/// Attack techniques based on MITRE ATT&CK framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitreAttack {
    pub tactic: String,
    pub technique_id: String,
    pub technique_name: String,
    pub subtechnique: Option<String>,
    pub description: Option<String>,
}

/// Behavioral indicators detected during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorIndicator {
    pub behavior_type: String,
    pub description: String,
    pub severity: ThreatSeverity,
    pub observed_at: DateTime<Utc>,
    pub process_name: Option<String>,
    pub command_line: Option<String>,
    pub network_connections: Vec<NetworkConnection>,
    pub file_operations: Vec<FileOperation>,
    pub registry_operations: Vec<RegistryOperation>,
}

/// Network connection details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub protocol: String,
    pub source_ip: String,
    pub source_port: u16,
    pub destination_ip: String,
    pub destination_port: u16,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub timestamp: DateTime<Utc>,
}

/// File operation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub operation: String, // read, write, delete, create, modify
    pub file_path: String,
    pub file_hash: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
}

/// Registry operation details (Windows)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryOperation {
    pub operation: String, // create, modify, delete, query
    pub key_path: String,
    pub value_name: Option<String>,
    pub value_data: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// YARA rule match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraMatch {
    pub rule_name: String,
    pub namespace: Option<String>,
    pub tags: Vec<String>,
    pub meta: HashMap<String, String>,
    pub strings_matched: Vec<StringMatch>,
}

/// String match details from YARA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringMatch {
    pub identifier: String,
    pub data: String,
    pub offset: u64,
}

/// Machine learning model prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlPrediction {
    pub model_name: String,
    pub model_version: String,
    pub prediction: String,
    pub confidence: f64,
    pub features_used: Vec<String>,
    pub probabilities: HashMap<String, f64>,
}

/// Main threat indicator structure
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ThreatIndicator {
    pub id: Uuid,
    pub scan_job_id: Uuid,
    
    // Core indicator information
    pub ioc_type: IocType,
    pub ioc_value: String,
    pub threat_category: ThreatCategory,
    pub severity: ThreatSeverity,
    pub confidence: ConfidenceLevel,
    
    // Threat details
    pub threat_name: Option<String>,
    pub description: String,
    pub family: Option<String>,
    pub variant: Option<String>,
    
    // Detection information
    pub detection_engine: String,
    pub engine_version: String,
    pub signature_id: Option<String>,
    pub signature_name: Option<String>,
    
    // Context
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub occurrence_count: i32,
    
    // Additional data (stored as JSONB in database)
    #[sqlx(default)]
    pub mitre_attacks: Option<sqlx::types::Json<Vec<MitreAttack>>>,
    #[sqlx(default)]
    pub behavior_indicators: Option<sqlx::types::Json<Vec<BehaviorIndicator>>>,
    #[sqlx(default)]
    pub yara_matches: Option<sqlx::types::Json<Vec<YaraMatch>>>,
    #[sqlx(default)]
    pub ml_predictions: Option<sqlx::types::Json<Vec<MlPrediction>>>,
    #[sqlx(default)]
    pub related_iocs: Option<sqlx::types::Json<Vec<String>>>,
    #[sqlx(default)]
    pub tags: Option<sqlx::types::Json<Vec<String>>>,
    #[sqlx(default)]
    pub metadata: Option<sqlx::types::Json<HashMap<String, serde_json::Value>>>,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Builder pattern for creating threat indicators
#[derive(Debug, Default)]
pub struct ThreatIndicatorBuilder {
    scan_job_id: Option<Uuid>,
    ioc_type: Option<IocType>,
    ioc_value: Option<String>,
    threat_category: Option<ThreatCategory>,
    severity: Option<ThreatSeverity>,
    confidence: Option<ConfidenceLevel>,
    threat_name: Option<String>,
    description: Option<String>,
    family: Option<String>,
    variant: Option<String>,
    detection_engine: Option<String>,
    engine_version: Option<String>,
    signature_id: Option<String>,
    signature_name: Option<String>,
    occurrence_count: Option<i32>,
    mitre_attacks: Option<Vec<MitreAttack>>,
    behavior_indicators: Option<Vec<BehaviorIndicator>>,
    yara_matches: Option<Vec<YaraMatch>>,
    ml_predictions: Option<Vec<MlPrediction>>,
    related_iocs: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    metadata: Option<HashMap<String, serde_json::Value>>,
}

impl ThreatIndicatorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scan_job_id(mut self, id: Uuid) -> Self {
        self.scan_job_id = Some(id);
        self
    }

    pub fn ioc_type(mut self, ioc_type: IocType) -> Self {
        self.ioc_type = Some(ioc_type);
        self
    }

    pub fn ioc_value(mut self, value: impl Into<String>) -> Self {
        self.ioc_value = Some(value.into());
        self
    }

    pub fn threat_category(mut self, category: ThreatCategory) -> Self {
        self.threat_category = Some(category);
        self
    }

    pub fn severity(mut self, severity: ThreatSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    pub fn confidence(mut self, confidence: ConfidenceLevel) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn threat_name(mut self, name: impl Into<String>) -> Self {
        self.threat_name = Some(name.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn family(mut self, family: impl Into<String>) -> Self {
        self.family = Some(family.into());
        self
    }

    pub fn detection_engine(mut self, engine: impl Into<String>) -> Self {
        self.detection_engine = Some(engine.into());
        self
    }

    pub fn engine_version(mut self, version: impl Into<String>) -> Self {
        self.engine_version = Some(version.into());
        self
    }

    pub fn signature_id(mut self, id: impl Into<String>) -> Self {
        self.signature_id = Some(id.into());
        self
    }

    pub fn mitre_attacks(mut self, attacks: Vec<MitreAttack>) -> Self {
        self.mitre_attacks = Some(attacks);
        self
    }

    pub fn behavior_indicators(mut self, behaviors: Vec<BehaviorIndicator>) -> Self {
        self.behavior_indicators = Some(behaviors);
        self
    }

    pub fn yara_matches(mut self, matches: Vec<YaraMatch>) -> Self {
        self.yara_matches = Some(matches);
        self
    }

    pub fn ml_predictions(mut self, predictions: Vec<MlPrediction>) -> Self {
        self.ml_predictions = Some(predictions);
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn build(self) -> Result<ThreatIndicator, String> {
        let now = Utc::now();
        
        Ok(ThreatIndicator {
            id: Uuid::new_v4(),
            scan_job_id: self.scan_job_id.ok_or("scan_job_id is required")?,
            ioc_type: self.ioc_type.ok_or("ioc_type is required")?,
            ioc_value: self.ioc_value.ok_or("ioc_value is required")?,
            threat_category: self.threat_category.ok_or("threat_category is required")?,
            severity: self.severity.ok_or("severity is required")?,
            confidence: self.confidence.ok_or("confidence is required")?,
            threat_name: self.threat_name,
            description: self.description.ok_or("description is required")?,
            family: self.family,
            variant: self.variant,
            detection_engine: self.detection_engine.ok_or("detection_engine is required")?,
            engine_version: self.engine_version.ok_or("engine_version is required")?,
            signature_id: self.signature_id,
            signature_name: self.signature_name,
            first_seen: now,
            last_seen: now,
            occurrence_count: self.occurrence_count.unwrap_or(1),
            mitre_attacks: self.mitre_attacks.map(sqlx::types::Json),
            behavior_indicators: self.behavior_indicators.map(sqlx::types::Json),
            yara_matches: self.yara_matches.map(sqlx::types::Json),
            ml_predictions: self.ml_predictions.map(sqlx::types::Json),
            related_iocs: self.related_iocs.map(sqlx::types::Json),
            tags: self.tags.map(sqlx::types::Json),
            metadata: self.metadata.map(sqlx::types::Json),
            created_at: now,
            updated_at: now,
        })
    }
}

impl ThreatIndicator {
    /// Calculate a composite risk score based on severity and confidence
    pub fn risk_score(&self) -> f64 {
        let severity_score = self.severity.score() as f64;
        let confidence_factor = self.confidence.percentage() as f64 / 100.0;
        severity_score * confidence_factor
    }

    /// Check if this is a high-priority threat
    pub fn is_high_priority(&self) -> bool {
        matches!(self.severity, ThreatSeverity::Critical | ThreatSeverity::High)
            && matches!(self.confidence, ConfidenceLevel::Confirmed | ConfidenceLevel::High)
    }

    /// Get a summary of the threat for display
    pub fn summary(&self) -> String {
        format!(
            "{} - {} (Severity: {:?}, Confidence: {:?})",
            self.threat_name.as_deref().unwrap_or("Unknown Threat"),
            self.threat_category.as_ref(),
            self.severity,
            self.confidence
        )
    }

    /// Update the last seen timestamp and increment occurrence count
    pub fn record_occurrence(&mut self) {
        self.last_seen = Utc::now();
        self.occurrence_count += 1;
        self.updated_at = Utc::now();
    }
}

impl AsRef<ThreatCategory> for ThreatCategory {
    fn as_ref(&self) -> &ThreatCategory {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_severity_score() {
        assert_eq!(ThreatSeverity::Critical.score(), 100);
        assert_eq!(ThreatSeverity::High.score(), 75);
        assert_eq!(ThreatSeverity::Medium.score(), 50);
    }

    #[test]
    fn test_confidence_percentage() {
        assert_eq!(ConfidenceLevel::Confirmed.percentage(), 100);
        assert_eq!(ConfidenceLevel::High.percentage(), 80);
    }

    #[test]
    fn test_risk_score_calculation() {
        let indicator = ThreatIndicatorBuilder::new()
            .scan_job_id(Uuid::new_v4())
            .ioc_type(IocType::FileHash)
            .ioc_value("abc123")
            .threat_category(ThreatCategory::Malware)
            .severity(ThreatSeverity::Critical)
            .confidence(ConfidenceLevel::High)
            .description("Test threat")
            .detection_engine("test-engine")
            .engine_version("1.0")
            .build()
            .unwrap();

        assert_eq!(indicator.risk_score(), 80.0);
    }

    #[test]
    fn test_high_priority_detection() {
        let indicator = ThreatIndicatorBuilder::new()
            .scan_job_id(Uuid::new_v4())
            .ioc_type(IocType::FileHash)
            .ioc_value("abc123")
            .threat_category(ThreatCategory::Ransomware)
            .severity(ThreatSeverity::Critical)
            .confidence(ConfidenceLevel::Confirmed)
            .description("Ransomware detected")
            .detection_engine("test-engine")
            .engine_version("1.0")
            .build()
            .unwrap();

        assert!(indicator.is_high_priority());
    }
}