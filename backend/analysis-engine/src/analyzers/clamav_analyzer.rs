use anyhow::{Result, anyhow};
use clamav_client::tokio::{ClamClient, ScanResult};
use tracing::{info, warn, error};
use std::time::Instant;

use crate::models::analysis_result::{
    DetectionResult, ThreatVerdict, SeverityLevel, EngineType, ThreatCategory,
};

/// Configuration for ClamAV analyzer
#[derive(Debug, Clone)]
pub struct ClamAvAnalyzerConfig {
    /// ClamAV daemon host address
    pub host: String,
    /// ClamAV daemon port
    pub port: u16,
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
    /// Enable ClamAV scanning
    pub enabled: bool,
}

impl Default for ClamAvAnalyzerConfig {
    fn default() -> Self {
        Self {
            host: std::env::var("CLAMAV_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            port: 3310,
            timeout_seconds: 30,
            enabled: std::env::var("ENABLE_CLAMAV")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}

/// ClamAV-based malware analyzer
pub struct ClamAvAnalyzer {
    config: ClamAvAnalyzerConfig,
}

impl ClamAvAnalyzer {
    /// Create a new ClamAV analyzer
    pub fn new(config: ClamAvAnalyzerConfig) -> Self {
        info!(
            "Initializing ClamAV analyzer - host: {}:{}, enabled: {}",
            config.host, config.port, config.enabled
        );
        Self { config }
    }

    /// Scan file data for malware using ClamAV
    pub async fn scan_file(&self, file_data: &[u8], filename: &str) -> Result<DetectionResult> {
        if !self.config.enabled {
            warn!("ClamAV analyzer is disabled");
            return Ok(self.create_disabled_result(filename));
        }

        let start_time = Instant::now();
        info!(
            "Starting ClamAV scan for file: {} ({} bytes)",
            filename,
            file_data.len()
        );

        // Connect to ClamAV daemon
        let address = format!("{}:{}", self.config.host, self.config.port);
        let client = match ClamClient::new(&address).await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to ClamAV daemon at {}: {}", address, e);
                return Ok(self.create_error_result(
                    filename,
                    &format!("ClamAV connection failed: {}", e),
                ));
            }
        };

        // Scan the file
        let scan_result = match client.scan_buffer(file_data).await {
            Ok(result) => result,
            Err(e) => {
                error!("ClamAV scan failed for {}: {}", filename, e);
                return Ok(self.create_error_result(
                    filename,
                    &format!("ClamAV scan error: {}", e),
                ));
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Convert ClamAV result to DetectionResult
        let detection = self.convert_scan_result(scan_result, filename, processing_time);

        info!(
            "ClamAV scan completed for {} - verdict: {:?}, time: {}ms",
            filename, detection.verdict, processing_time
        );

        Ok(detection)
    }

    /// Convert ClamAV scan result to DetectionResult
    fn convert_scan_result(
        &self,
        scan_result: ScanResult,
        filename: &str,
        processing_time_ms: u64,
    ) -> DetectionResult {
        match scan_result {
            ScanResult::Clean => {
                info!("ClamAV: File {} is clean", filename);
                DetectionResult {
                    detection_id: uuid::Uuid::new_v4(),
                    engine_name: "ClamAV".to_string(),
                    engine_version: "Latest".to_string(),
                    engine_type: EngineType::Yara,
                    verdict: ThreatVerdict::Benign,
                    confidence: 0.95,
                    severity: SeverityLevel::Info,
                    categories: vec![],
                    metadata: std::collections::HashMap::new(),
                    detected_at: chrono::Utc::now(),
                    processing_time_ms,
                    error_message: None,
                }
            }
            ScanResult::Found(virus_name) => {
                warn!("ClamAV: Malware detected in {} - {}", filename, virus_name);

                let mut metadata = std::collections::HashMap::new();
                metadata.insert(
                    "signature".to_string(),
                    serde_json::Value::String(virus_name.clone()),
                );
                metadata.insert(
                    "filename".to_string(),
                    serde_json::Value::String(filename.to_string()),
                );

                // Determine threat category based on signature name
                let categories = self.categorize_threat(&virus_name);
                let severity = self.determine_severity(&virus_name);

                DetectionResult {
                    detection_id: uuid::Uuid::new_v4(),
                    engine_name: "ClamAV".to_string(),
                    engine_version: "Latest".to_string(),
                    engine_type: EngineType::Yara,
                    verdict: ThreatVerdict::Malicious,
                    confidence: 0.98,
                    severity,
                    categories,
                    metadata,
                    detected_at: chrono::Utc::now(),
                    processing_time_ms,
                    error_message: None,
                }
            }
        }
    }

    /// Categorize threat based on signature name
    fn categorize_threat(&self, signature: &str) -> Vec<ThreatCategory> {
        let sig_lower = signature.to_lowercase();
        let mut categories = vec![];

        if sig_lower.contains("trojan") {
            categories.push(ThreatCategory::Trojan);
        }
        if sig_lower.contains("ransomware") || sig_lower.contains("ransom") {
            categories.push(ThreatCategory::Ransomware);
        }
        if sig_lower.contains("worm") {
            categories.push(ThreatCategory::Worm);
        }
        if sig_lower.contains("rootkit") {
            categories.push(ThreatCategory::Rootkit);
        }
        if sig_lower.contains("backdoor") {
            categories.push(ThreatCategory::Backdoor);
        }
        if sig_lower.contains("spyware") || sig_lower.contains("keylog") {
            categories.push(ThreatCategory::Spyware);
        }
        if sig_lower.contains("adware") {
            categories.push(ThreatCategory::Adware);
        }
        if sig_lower.contains("exploit") {
            categories.push(ThreatCategory::Exploit);
        }

        // Default to generic malware if no specific category
        if categories.is_empty() {
            categories.push(ThreatCategory::Malware);
        }

        categories
    }

    /// Determine severity based on threat type
    fn determine_severity(&self, signature: &str) -> SeverityLevel {
        let sig_lower = signature.to_lowercase();

        if sig_lower.contains("ransomware") || sig_lower.contains("rootkit") {
            SeverityLevel::Critical
        } else if sig_lower.contains("trojan") || sig_lower.contains("backdoor") {
            SeverityLevel::High
        } else if sig_lower.contains("worm") || sig_lower.contains("exploit") {
            SeverityLevel::Medium
        } else {
            SeverityLevel::Low
        }
    }

    /// Create result for disabled state
    fn create_disabled_result(&self, filename: &str) -> DetectionResult {
        DetectionResult {
            detection_id: uuid::Uuid::new_v4(),
            engine_name: "ClamAV".to_string(),
            engine_version: "Disabled".to_string(),
            engine_type: EngineType::Yara,
            verdict: ThreatVerdict::Unknown,
            confidence: 0.0,
            severity: SeverityLevel::Info,
            categories: vec![],
            metadata: std::collections::HashMap::new(),
            detected_at: chrono::Utc::now(),
            processing_time_ms: 0,
            error_message: Some("ClamAV analyzer is disabled".to_string()),
        }
    }

    /// Create result for error state
    fn create_error_result(&self, filename: &str, error_msg: &str) -> DetectionResult {
        warn!("ClamAV error for {}: {}", filename, error_msg);

        DetectionResult {
            detection_id: uuid::Uuid::new_v4(),
            engine_name: "ClamAV".to_string(),
            engine_version: "Error".to_string(),
            engine_type: EngineType::Yara,
            verdict: ThreatVerdict::Unknown,
            confidence: 0.0,
            severity: SeverityLevel::Info,
            categories: vec![],
            metadata: std::collections::HashMap::new(),
            detected_at: chrono::Utc::now(),
            processing_time_ms: 0,
            error_message: Some(error_msg.to_string()),
        }
    }

    /// Get ClamAV version info
    pub async fn get_version(&self) -> Result<String> {
        let address = format!("{}:{}", self.config.host, self.config.port);
        let client = ClamClient::new(&address).await?;

        match client.get_version().await {
            Ok(version) => {
                info!("ClamAV version: {}", version);
                Ok(version)
            }
            Err(e) => {
                error!("Failed to get ClamAV version: {}", e);
                Err(anyhow!("Failed to get ClamAV version: {}", e))
            }
        }
    }

    /// Ping ClamAV daemon to check if it's alive
    pub async fn ping(&self) -> Result<()> {
        let address = format!("{}:{}", self.config.host, self.config.port);
        let client = ClamClient::new(&address).await?;

        match client.ping().await {
            Ok(_) => {
                info!("ClamAV daemon is alive at {}", address);
                Ok(())
            }
            Err(e) => {
                error!("ClamAV ping failed at {}: {}", address, e);
                Err(anyhow!("ClamAV ping failed: {}", e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_eicar_detection() {
        // EICAR test file - standard malware test string
        let eicar = b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";

        let config = ClamAvAnalyzerConfig {
            host: "localhost".to_string(),
            port: 3310,
            timeout_seconds: 30,
            enabled: true,
        };

        let analyzer = ClamAvAnalyzer::new(config);

        // Only run if ClamAV is available
        if analyzer.ping().await.is_ok() {
            let result = analyzer.scan_file(eicar, "eicar.txt").await;
            assert!(result.is_ok());

            let detection = result.unwrap();
            assert_eq!(detection.verdict, ThreatVerdict::Malicious);
            assert!(detection.confidence > 0.9);
        }
    }
}
