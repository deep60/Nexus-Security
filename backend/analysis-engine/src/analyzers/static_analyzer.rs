use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use tracing::{info, warn};
use regex::Regex;
use lazy_static::lazy_static;

use crate::models::analysis_result::{AnalysisResult, ThreatVerdict};

/// File type enumeration based on magic bytes and headers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileType {
    PE,           // Windows Portable Executable
    ELF,          // Linux Executable and Linkable Format
    PDF,          // Portable Document Format
    Unknown,
}

/// Configuration for static analyzer
#[derive(Debug, Clone)]
pub struct StaticAnalyzerConfig {
    pub max_file_size: usize,
    pub entropy_threshold: f64,
    pub enable_string_analysis: bool,
}

impl Default for StaticAnalyzerConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024,       // 100MB
            entropy_threshold: 7.0,
            enable_string_analysis: true,
        }
    }
}

// Simple regex patterns for string analysis
lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"https?://\S+").unwrap();
    static ref IP_REGEX: Regex = Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap();
}

/// Main static analyzer implementation
pub struct StaticAnalyzer {
    config: StaticAnalyzerConfig,
}

impl StaticAnalyzer {
    /// Create a new static analyzer instance
    pub fn new(config: StaticAnalyzerConfig) -> Self {
        Self { config }
    }

    /// Perform static analysis on file data
    pub async fn analyze(&self, file_data: &[u8], filename: Option<&str>) -> Result<AnalysisResult> {
        info!("Starting static analysis for file: {:?}", filename.unwrap_or("unknown"));

        if file_data.len() > self.config.max_file_size {
            return Err(anyhow!("File size {} exceeds maximum allowed size {}", 
                              file_data.len(), self.config.max_file_size));
        }

        let mut metadata = HashMap::new();
        let mut threat_indicators = Vec::new();

        // Determine file type
        let file_type = self.detect_file_type(file_data);
        metadata.insert("file_type".to_string(), format!("{:?}", file_type));
        metadata.insert("file_size".to_string(), file_data.len().to_string());

        // String analysis
        if self.config.enable_string_analysis {
            let (urls, ips) = self.analyze_strings(file_data);
            
            if !urls.is_empty() {
                threat_indicators.push(format!("Found {} URLs", urls.len()));
                metadata.insert("urls_found".to_string(), urls.len().to_string());
            }

            if !ips.is_empty() {
                threat_indicators.push(format!("Found {} IP addresses", ips.len()));
                metadata.insert("ips_found".to_string(), ips.len().to_string());
            }
        }

        // Calculate entropy
        let entropy = self.calculate_entropy(file_data);
        metadata.insert("entropy".to_string(), format!("{:.2}", entropy));

        if entropy > self.config.entropy_threshold {
            threat_indicators.push(format!("High entropy detected: {:.2}", entropy));
        }

        // Determine verdict
        let verdict = if threat_indicators.len() > 2 {
            ThreatVerdict::Suspicious
        } else if threat_indicators.is_empty() {
            ThreatVerdict::Benign
        } else {
            ThreatVerdict::Suspicious
        };

        let details = if threat_indicators.is_empty() {
            "No significant threats detected through static analysis".to_string()
        } else {
            format!("Static analysis detected {} potential indicators", threat_indicators.len())
        };

        // Use the actual AnalysisResult struct fields
        Ok(AnalysisResult {
            analysis_id: uuid::Uuid::new_v4().to_string(),
            submission_id: Some(uuid::Uuid::new_v4().to_string()),
            bounty_id: None,
            file_metadata: None,
            consensus_verdict: verdict,
            consensus_confidence: 0.7,
            consensus_score: 0.0,
            individual_verdicts: HashMap::new(),
            threat_indicators,
            metadata: serde_json::json!(metadata),
            behavioral_analysis: None,
            network_indicators: None,
            yara_matches: Vec::new(),
            hash_reputation: HashMap::new(),
            ml_classification: None,
            created_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            processing_time_ms: 100,
            error_messages: Vec::new(),
            warnings: Vec::new(),
        })
    }

    /// Detect file type based on magic bytes
    fn detect_file_type(&self, data: &[u8]) -> FileType {
        if data.len() < 4 {
            return FileType::Unknown;
        }

        // PE files (MZ header)
        if data.starts_with(b"MZ") {
            return FileType::PE;
        }

        // ELF files
        if data.starts_with(&[0x7f, 0x45, 0x4c, 0x46]) {
            return FileType::ELF;
        }

        // PDF files
        if data.starts_with(b"%PDF") {
            return FileType::PDF;
        }

        FileType::Unknown
    }

    /// Analyze strings in file data
    fn analyze_strings(&self, data: &[u8]) -> (Vec<String>, Vec<String>) {
        let data_str = String::from_utf8_lossy(data);
        
        // Extract URLs
        let urls: Vec<String> = URL_REGEX.find_iter(&data_str)
            .map(|m| m.as_str().to_string())
            .collect();

        // Extract IP addresses  
        let ips: Vec<String> = IP_REGEX.find_iter(&data_str)
            .map(|m| m.as_str().to_string())
            .collect();

        (urls, ips)
    }

    /// Calculate Shannon entropy of data
    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mut frequency = [0u32; 256];
        for &byte in data {
            frequency[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &freq in frequency.iter() {
            if freq > 0 {
                let p = freq as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_calculation() {
        let analyzer = StaticAnalyzer::new(StaticAnalyzerConfig::default());
        
        // Test with uniform data (should have high entropy)
        let uniform_data: Vec<u8> = (0..=255).collect();
        let entropy = analyzer.calculate_entropy(&uniform_data);
        assert!(entropy > 7.9); // Should be close to 8.0
        
        // Test with repeated data (should have low entropy)
        let repeated_data = vec![0u8; 256];
        let entropy = analyzer.calculate_entropy(&repeated_data);
        assert!(entropy < 0.1); // Should be close to 0.0
    }

    #[test]
    fn test_file_type_detection() {
        let analyzer = StaticAnalyzer::new(StaticAnalyzerConfig::default());
        
        // Test PE detection
        let pe_header = b"MZ";
        assert_eq!(analyzer.detect_file_type(pe_header), FileType::PE);
        
        // Test PDF detection  
        let pdf_header = b"%PDF-1.4";
        assert_eq!(analyzer.detect_file_type(pdf_header), FileType::PDF);
        
        // Test unknown
        let unknown_header = b"UNKNOWN";
        assert_eq!(analyzer.detect_file_type(unknown_header), FileType::Unknown);
    }

    #[tokio::test]
    async fn test_basic_analysis() {
        let analyzer = StaticAnalyzer::new(StaticAnalyzerConfig::default());
        
        // Test with benign data
        let benign_data = b"Hello, this is a normal file with normal content";
        let result = analyzer.analyze(benign_data, Some("test.txt")).await;
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert_eq!(analysis.consensus_verdict, ThreatVerdict::Benign);
    }
}
