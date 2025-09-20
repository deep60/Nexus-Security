use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod analysis_result;

// Re-export commonly used types
pub use analysis_result::{
    AnalysisResult, AnalysisStatus, DetectionResult, ThreatVerdict, ConfidenceLevel, EngineType, 
    SeverityLevel, ThreatCategory, FileMetadata, ExecutableInfo, AnalysisError,
};

/// Base configuration for all analysis engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub engine_name: String,
    pub version: String,
    pub enabled: bool,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_interval_ms: u64,
    pub log_level: String,
    pub custom_settings: HashMap<String, String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            engine_name: "Unknown".to_string(),
            version: "1.0".to_string(),
            enabled: true,
            timeout_seconds: 30,
            max_retries: 3,
            retry_interval_ms: 1000,
            log_level: "info".to_string(),
            custom_settings: HashMap::new(),
        }
    }
}

/// Generic analysis request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub request_id: uuid::Uuid,
    pub artifact_type: String, // "file", "url", "hash", etc.
    pub artifact_data: String, // File path, URL, or hash value
    pub priority: u8,         // 0 (low) to 100 (high)
    pub metadata: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AnalysisRequest {
    pub fn new(artifact_type: &str, artifact_data: &str) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4(),
            artifact_type: artifact_type.to_string(),
            artifact_data: artifact_data.to_string(),
            priority: 50,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        assert_eq!(config.engine_name, "Unknown");
        assert_eq!(config.version, "1.0");
        assert!(config.enabled);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_analysis_request_new() {
        let request = AnalysisRequest::new("file", "test.exe");
        assert_eq!(request.artifact_type, "file");
        assert_eq!(request.artifact_data, "test.exe");
        assert_eq!(request.priority, 50);
        assert!(request.metadata.is_empty());
    }
}