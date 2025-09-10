use std::collections::HashMap;
use std::path::Path;
use goblin::elf::Elf;
use goblin::mach::Mach;
use goblin::pe::PE;
use goblin::Object;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use regex::Regex;
use lazy_static::lazy_static;
use chrono::Utc;
use uuid::Uuid;

use crate::models::analysis_result::{DetectionResult, ThreatVerdict, ConfidenceLevel, EngineType, SeverityLevel, ThreatCategory};

/// File type enumeration based on magic bytes and headers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileType {
    PE,           // Windows Portable Executable
    ELF,          // Linux Executable and Linkable Format
    MachO,        // macOS Mach-O
    PDF,          // Portable Document Format
    Office,       // Microsoft Office documents
    Archive,      // ZIP, RAR, etc.
    Script,       // JavaScript, PowerShell, etc.
    Image,        // JPEG, PNG, etc.
    Unknown,
}

/// PE (Portable Executable) specific analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PEAnalysis {
    pub machine_type: String,
    pub timestamp: Option<u32>,
    pub entry_point: u32,
    pub sections: Vec<PESection>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub resources: Vec<String>,
    pub is_packed: bool,
    pub is_signed: bool,
    pub entropy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PESection {
    pub name: String,
    pub virtual_address: u32,
    pub size: u32,
    pub characteristics: u32,
    pub entropy: f64,
}

/// String analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringAnalysis {
    pub suspicious_strings: Vec<SuspiciousString>,
    pub urls: Vec<String>,
    pub ips: Vec<String>,
    pub email_addresses: Vec<String>,
    pub file_paths: Vec<String>,
    pub registry_keys: Vec<String>,
    pub crypto_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousString {
    pub content: String,
    pub category: StringCategory,
    pub confidence: f64,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringCategory {
    Malware,
    Vulnerability,
    Cryptographic,
    Network,
    SystemCall,
    Obfuscation,
    AntiAnalysis,
}

/// Entropy analysis for detecting packed/encrypted content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyAnalysis {
    pub overall_entropy: f64,
    pub section_entropies: HashMap<String, f64>,
    pub high_entropy_regions: Vec<EntropyRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyRegion {
    pub offset: usize,
    pub size: usize,
    pub entropy: f64,
}

/// Configuration for static analyzer
#[derive(Debug, Clone)]
pub struct StaticAnalyzerConfig {
    pub max_file_size: usize,
    pub max_string_length: usize,
    pub entropy_threshold: f64,
    pub enable_pe_analysis: bool,
    pub enable_string_analysis: bool,
    pub enable_entropy_analysis: bool,
    pub suspicious_string_threshold: f64,
}

impl Default for StaticAnalyzerConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024,       // 100MB
            max_string_length: 1000,
            entropy_threshold: 7.0,
            enable_pe_analysis: true,
            enable_string_analysis: true,
            enable_entropy_analysis: true,
            suspicious_string_threshold: 0.7,
        }
    }
}

// Lazy-loaded regex patterns for string analysis
lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"https?://\S+").unwrap();
    static ref IP_REGEX: Regex = Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap();
    static ref EMAIL_REGEX: Regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
    static ref REGISTRY_REGEX: Regex = Regex::new(r"HKEY_[A-Z_]+\\[^\\]+(?:\\[^\\]+)*").unwrap();
    static ref CRYPTO_REGEX: Regex = Regex::new(r"(?i)(aes|des|rsa|md5|sha\d+|base64|encrypt|decrypt|cipher)").unwrap();
    
    // Suspicious string patterns
    static ref MALWARE_PATTERNS: Vec<(Regex, &'static str)> = vec![
        (Regex::new(r"(?i)(backdoor|rootkit|keylog|trojan|virus|worm)").unwrap(), "malware"),
        (Regex::new(r"(?i)(persistence|lateral|privilege|escalation)").unwrap(), "attack_technique"),
        (Regex::new(r"(?i)(shellcode|payload|exploit|vulnerability)").unwrap(), "exploit"),
        (Regex::new(r"(?i)(c2|command.*control|exfiltrat|beacon)").unwrap(), "c2_communication"),
        (Regex::new(r"(?i)(anti.*debug|anti.*vm|sandbox|evasion)").unwrap(), "anti_analysis"),
        (Regex::new(r"(?i)(process.*injection|dll.*injection|hollowing)").unwrap(), "injection"),
        (Regex::new(r"(?i)(crypto.*mining|bitcoin|monero|wallet)").unwrap(), "cryptomining"),
    ];
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

    /// Perform comprehensive static analysis on file data
    pub async fn analyze(&self, file_data: &[u8], filename: Option<&str>) -> Result<DetectionResult> {
        info!("Starting static analysis for file: {:?}", filename.unwrap_or("unknown"));

        if file_data.len() > self.config.max_file_size {
            return Err(anyhow!("File too large for static analysis"));
        }

        let start = std::time::Instant::now();

        let file_type = self.detect_file_type(file_data);

        let (mut confidence_score, mut threats) = (0.0, vec![]);

        let mut metadata = HashMap::new();

        if self.config.enable_entropy_analysis {
            let entropy_analysis = self.analyze_entropy(file_data);
            metadata.insert("entropy_analysis".to_string(), serde_json::to_value(entropy_analysis)?);
            if entropy_analysis.overall_entropy > self.config.entropy_threshold {
                threats.push("High entropy detected, possible packing".to_string());
                confidence_score += 0.3;
            }
        }

        if self.config.enable_string_analysis {
            let string_analysis = self.analyze_strings(file_data);
            metadata.insert("string_analysis".to_string(), serde_json::to_value(string_analysis)?);
            let suspicious_count = string_analysis.suspicious_strings.len();
            if suspicious_count > 5 {
                threats.push(format!("{} suspicious strings detected", suspicious_count));
                confidence_score += 0.2 * suspicious_count as f64;
            }
        }

        if self.config.enable_pe_analysis && matches!(file_type, FileType::PE) {
            let pe_analysis = self.analyze_pe(file_data).await?;
            metadata.insert("pe_analysis".to_string(), serde_json::to_value(pe_analysis)?);
            if pe_analysis.is_packed {
                threats.push("Packed executable detected".to_string());
                confidence_score += 0.4;
            }
        }

        confidence_score = confidence_score.min(1.0);

        let (verdict, confidence_level) = self.determine_verdict(confidence_score, &threats);
        let confidence = match confidence_level {
            ConfidenceLevel::High => 0.9,
            ConfidenceLevel::Medium => 0.6,
            ConfidenceLevel::Low => 0.3,
        };

        let severity = if confidence_score > 0.8 { SeverityLevel::High } else if confidence_score > 0.5 { SeverityLevel::Medium } else { SeverityLevel::Low };

        let categories = threats.iter().map(|t| ThreatCategory::Other(t.clone())).collect();

        let processing_time_ms = start.elapsed().as_millis() as u64;

        Ok(DetectionResult {
            detection_id: Uuid::new_v4(),
            engine_name: "Static Analyzer".to_string(),
            engine_version: "1.0".to_string(),
            engine_type: EngineType::Static,
            verdict,
            confidence,
            severity,
            categories,
            metadata,
            detected_at: Utc::now(),
            processing_time_ms,
            error_message: None,
        })
    }

    fn detect_file_type(&self, data: &[u8]) -> FileType {
        match Object::parse(data) {
            Ok(Object::PE(_)) => FileType::PE,
            Ok(Object::Elf(_)) => FileType::ELF,
            Ok(Object::Mach(_)) => FileType::MachO,
            _ => FileType::Unknown,
        }
    }

    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mut freq = [0u32; 256];
        for &byte in data {
            freq[byte as usize] += 1;
        }

        let len = data.len() as f64;
        freq.iter().filter(|&&f| f > 0).fold(0.0, |ent, &f| {
            let p = f as f64 / len;
            ent - p * p.log2()
        })
    }

    fn analyze_entropy(&self, file_data: &[u8]) -> EntropyAnalysis {
        let overall_entropy = self.calculate_entropy(file_data);

        // Stub for sections and regions
        EntropyAnalysis {
            overall_entropy,
            section_entropies: HashMap::new(),
            high_entropy_regions: vec![],
        }
    }

    fn analyze_strings(&self, data: &[u8]) -> StringAnalysis {
        let str_data = String::from_utf8_lossy(data).to_string();

        let urls = URL_REGEX.find_iter(&str_data).map(|m| m.as_str().to_string()).collect();
        let ips = IP_REGEX.find_iter(&str_data).map(|m| m.as_str().to_string()).collect();

        // Stub for others
        StringAnalysis {
            suspicious_strings: vec![],
            urls,
            ips,
            email_addresses: vec![],
            file_paths: vec![],
            registry_keys: vec![],
            crypto_indicators: vec![],
        }
    }

    async fn analyze_pe(&self, data: &[u8]) -> Result<PEAnalysis> {
        let pe = PE::parse(data).map_err(|e| anyhow!("PE parse error: {}", e))?;

        Ok(PEAnalysis {
            machine_type: format!("{:?}", pe.header.coff_header.machine),
            timestamp: Some(pe.header.coff_header.time_date_stamp),
            entry_point: pe.entry,
            sections: pe.sections.iter().map(|s| PESection {
                name: String::from_utf8_lossy(&s.name).to_string(),
                virtual_address: s.virtual_address,
                size: s.virtual_size,
                characteristics: s.characteristics,
                entropy: 0.0, // Stub
            }).collect(),
            imports: pe.imports.iter().map(|i| i.name.to_string()).collect(),
            exports: pe.exports.iter().map(|e| e.name.unwrap_or("").to_string()).collect(),
            resources: vec![],
            is_packed: false, // Stub
            is_signed: pe.is_signed,
            entropy: 0.0, // Stub
        })
    }

    fn determine_verdict(&self, confidence_score: f64, threats: &[String]) -> (ThreatVerdict, ConfidenceLevel) {
        let verdict = if confidence_score > 0.8 || threats.len() > 10 {
            ThreatVerdict::Malicious
        } else if confidence_score > 0.5 || threats.len() > 5 {
            ThreatVerdict::Suspicious
        } else if confidence_score > 0.2 || !threats.is_empty() {
            ThreatVerdict::Suspicious
        } else {
            ThreatVerdict::Benign
        };

        let confidence = if confidence_score > 0.8 {
            ConfidenceLevel::High
        } else if confidence_score > 0.5 {
            ConfidenceLevel::Medium
        } else {
            ConfidenceLevel::Low
        };

        (verdict, confidence)
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
        assert!(entropy > 7.9); // Close to 8.0
        
        // Test with repeated data (low entropy)
        let repeated_data = vec![0u8; 256];
        let entropy = analyzer.calculate_entropy(&repeated_data);
        assert!(entropy < 0.1); // Close to 0.0
    }

    #[tokio::test]
    async fn test_full_analysis() {
        let analyzer = StaticAnalyzer::new(StaticAnalyzerConfig::default());
        
        // Test with benign data
        let benign_data = b"Hello, this is a normal file with normal content";
        let result = analyzer.analyze(benign_data, Some("test.txt")).await;
        assert!(result.is_ok());
        
        let detection = result.unwrap();
        assert_eq!(detection.verdict, ThreatVerdict::Benign);
    }
}