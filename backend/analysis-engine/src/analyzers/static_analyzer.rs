use std::collections::HashMap;
use std::path::Path;
use std::io::{Read, Seek, SeekFrom};
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use regex::Regex;
use lazy_static::lazy_static;

use crate::models::analysis_result::{AnalysisResult, ThreatVerdict, ConfidenceLevel};

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
    pub async fn analyze(&self, file_data: &[u8], filename: Option<&str>) -> Result<AnalysisResult> {
        info!("Starting static analysis for file: {:?}", filename.unwrap_or("unknown"));

        if file_data.len() > self.config.max_file_size {
            return Err(anyhow!("File size {} exceeds maximum allowed size {}", 
                              file_data.len(), self.config.max_file_size));
        }

        let mut threat_indicators = Vec::new();
        let mut metadata = HashMap::new();
        let mut confidence_score = 0.0;

        // Determine file type
        let file_type = self.detect_file_type(file_data);
        metadata.insert("file_type".to_string(), format!("{:?}", file_type));
        metadata.insert("file_size".to_string(), file_data.len().to_string());

        // PE-specific analysis
        if self.config.enable_pe_analysis && file_type == FileType::PE {
            match self.analyze_pe(file_data).await {
                Ok(pe_analysis) => {
                    let (pe_threats, pe_confidence) = self.evaluate_pe_threats(&pe_analysis);
                    threat_indicators.extend(pe_threats);
                    confidence_score += pe_confidence;
                    
                    metadata.insert("pe_machine_type".to_string(), pe_analysis.machine_type);
                    metadata.insert("pe_sections".to_string(), pe_analysis.sections.len().to_string());
                    metadata.insert("pe_imports".to_string(), pe_analysis.imports.len().to_string());
                    metadata.insert("pe_is_packed".to_string(), pe_analysis.is_packed.to_string());
                    metadata.insert("pe_entropy".to_string(), format!("{:.2}", pe_analysis.entropy));
                }
                Err(e) => warn!("PE analysis failed: {}", e),
            }
        }

        // String analysis
        if self.config.enable_string_analysis {
            let string_analysis = self.analyze_strings(file_data);
            let (string_threats, string_confidence) = self.evaluate_string_threats(&string_analysis);
            threat_indicators.extend(string_threats);
            confidence_score += string_confidence;

            metadata.insert("suspicious_strings".to_string(), 
                          string_analysis.suspicious_strings.len().to_string());
            metadata.insert("urls_found".to_string(), string_analysis.urls.len().to_string());
            metadata.insert("ips_found".to_string(), string_analysis.ips.len().to_string());
        }

        // Entropy analysis
        if self.config.enable_entropy_analysis {
            let entropy_analysis = self.analyze_entropy(file_data);
            let (entropy_threats, entropy_confidence) = self.evaluate_entropy_threats(&entropy_analysis);
            threat_indicators.extend(entropy_threats);
            confidence_score += entropy_confidence;

            metadata.insert("overall_entropy".to_string(), 
                          format!("{:.2}", entropy_analysis.overall_entropy));
            metadata.insert("high_entropy_regions".to_string(), 
                          entropy_analysis.high_entropy_regions.len().to_string());
        }

        // Determine final verdict
        let (verdict, confidence) = self.determine_verdict(confidence_score, &threat_indicators);
        
        let details = if threat_indicators.is_empty() {
            "No significant threats detected through static analysis".to_string()
        } else {
            format!("Static analysis detected {} potential threat indicators", 
                   threat_indicators.len())
        };

        // Add threat indicators to metadata
        if !threat_indicators.is_empty() {
            metadata.insert("threat_indicators".to_string(), 
                          threat_indicators.join("; "));
        }

        Ok(AnalysisResult {
            verdict,
            confidence,
            score: confidence_score.min(1.0).max(0.0),
            details,
            metadata,
            timestamp: chrono::Utc::now(),
            analyzer_name: "StaticAnalyzer".to_string(),
            analyzer_version: "1.0.0".to_string(),
        })
    }

    /// Detect file type based on magic bytes and headers
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

        // Mach-O files
        if data.starts_with(&[0xfe, 0xed, 0xfa, 0xce]) ||
           data.starts_with(&[0xce, 0xfa, 0xed, 0xfe]) ||
           data.starts_with(&[0xfe, 0xed, 0xfa, 0xcf]) ||
           data.starts_with(&[0xcf, 0xfa, 0xed, 0xfe]) {
            return FileType::MachO;
        }

        // PDF files
        if data.starts_with(b"%PDF") {
            return FileType::PDF;
        }

        // ZIP files (including Office docs)
        if data.starts_with(b"PK") {
            // Check for Office document signatures
            if data.len() > 30 {
                let data_str = String::from_utf8_lossy(&data[0..100]);
                if data_str.contains("word/") || data_str.contains("xl/") || 
                   data_str.contains("ppt/") || data_str.contains("Content_Types") {
                    return FileType::Office;
                }
            }
            return FileType::Archive;
        }

        // Common image formats
        if data.starts_with(&[0xff, 0xd8, 0xff]) || // JPEG
           data.starts_with(&[0x89, 0x50, 0x4e, 0x47]) || // PNG
           data.starts_with(b"GIF8") { // GIF
            return FileType::Image;
        }

        FileType::Unknown
    }

    /// Analyze PE (Portable Executable) files
    async fn analyze_pe(&self, data: &[u8]) -> Result<PEAnalysis> {
        if data.len() < 64 {
            return Err(anyhow!("File too small to be a valid PE"));
        }

        // Read DOS header
        let e_lfanew = u32::from_le_bytes([data[60], data[61], data[62], data[63]]);
        if e_lfanew as usize >= data.len() - 4 {
            return Err(anyhow!("Invalid PE header offset"));
        }

        // Read PE signature
        let pe_offset = e_lfanew as usize;
        if pe_offset + 24 >= data.len() {
            return Err(anyhow!("PE header extends beyond file"));
        }

        if &data[pe_offset..pe_offset + 4] != b"PE\0\0" {
            return Err(anyhow!("Invalid PE signature"));
        }

        // Read COFF header
        let machine_type = u16::from_le_bytes([data[pe_offset + 4], data[pe_offset + 5]]);
        let timestamp = u32::from_le_bytes([
            data[pe_offset + 8], data[pe_offset + 9],
            data[pe_offset + 10], data[pe_offset + 11]
        ]);

        let machine_str = match machine_type {
            0x014c => "i386",
            0x0200 => "ia64", 
            0x8664 => "amd64",
            0x01c0 => "arm",
            0xaa64 => "arm64",
            _ => "unknown",
        };

        // Read optional header for entry point
        let opt_header_offset = pe_offset + 24;
        let entry_point = if opt_header_offset + 16 < data.len() {
            u32::from_le_bytes([
                data[opt_header_offset + 16],
                data[opt_header_offset + 17],
                data[opt_header_offset + 18],
                data[opt_header_offset + 19],
            ])
        } else {
            0
        };

        // Analyze sections
        let sections = self.parse_pe_sections(data, pe_offset)?;
        
        // Calculate overall entropy
        let entropy = self.calculate_entropy(data);
        
        // Check for packing indicators
        let is_packed = self.detect_packing(&sections, entropy);
        
        // Parse imports and exports (simplified)
        let imports = self.parse_pe_imports(data, pe_offset).unwrap_or_default();
        let exports = self.parse_pe_exports(data, pe_offset).unwrap_or_default();
        
        // Check for digital signature (simplified check)
        let is_signed = self.check_pe_signature(data, pe_offset);

        Ok(PEAnalysis {
            machine_type: machine_str.to_string(),
            timestamp: Some(timestamp),
            entry_point,
            sections,
            imports,
            exports,
            resources: vec![], // TODO: Implement resource parsing
            is_packed,
            is_signed,
            entropy,
        })
    }

    /// Parse PE sections
    fn parse_pe_sections(&self, data: &[u8], pe_offset: usize) -> Result<Vec<PESection>> {
        let mut sections = Vec::new();
        
        if pe_offset + 24 >= data.len() {
            return Ok(sections);
        }

        let num_sections = u16::from_le_bytes([data[pe_offset + 6], data[pe_offset + 7]]);
        let opt_header_size = u16::from_le_bytes([data[pe_offset + 20], data[pe_offset + 21]]);
        
        let section_table_offset = pe_offset + 24 + opt_header_size as usize;
        
        for i in 0..num_sections {
            let section_offset = section_table_offset + (i as usize * 40);
            if section_offset + 40 > data.len() {
                break;
            }

            // Read section name (8 bytes)
            let name_bytes = &data[section_offset..section_offset + 8];
            let name = String::from_utf8_lossy(name_bytes)
                .trim_end_matches('\0')
                .to_string();

            // Read section properties
            let virtual_address = u32::from_le_bytes([
                data[section_offset + 12], data[section_offset + 13],
                data[section_offset + 14], data[section_offset + 15]
            ]);
            
            let size = u32::from_le_bytes([
                data[section_offset + 8], data[section_offset + 9],
                data[section_offset + 10], data[section_offset + 11]
            ]);
            
            let characteristics = u32::from_le_bytes([
                data[section_offset + 36], data[section_offset + 37],
                data[section_offset + 38], data[section_offset + 39]
            ]);

            let raw_offset = u32::from_le_bytes([
                data[section_offset + 20], data[section_offset + 21],
                data[section_offset + 22], data[section_offset + 23]
            ]) as usize;

            let raw_size = u32::from_le_bytes([
                data[section_offset + 16], data[section_offset + 17],
                data[section_offset + 18], data[section_offset + 19]
            ]) as usize;

            // Calculate section entropy
            let entropy = if raw_offset < data.len() && raw_offset + raw_size <= data.len() {
                self.calculate_entropy(&data[raw_offset..raw_offset + raw_size])
            } else {
                0.0
            };

            sections.push(PESection {
                name,
                virtual_address,
                size,
                characteristics,
                entropy,
            });
        }

        Ok(sections)
    }

    /// Parse PE imports (simplified)
    fn parse_pe_imports(&self, _data: &[u8], _pe_offset: usize) -> Result<Vec<String>> {
        // TODO: Implement proper import parsing
        // This is a complex process involving parsing the import table
        Ok(vec![])
    }

    /// Parse PE exports (simplified)
    fn parse_pe_exports(&self, _data: &[u8], _pe_offset: usize) -> Result<Vec<String>> {
        // TODO: Implement proper export parsing
        Ok(vec![])
    }

    /// Check for PE digital signature
    fn check_pe_signature(&self, _data: &[u8], _pe_offset: usize) -> bool {
        // TODO: Implement proper signature verification
        false
    }

    /// Detect if PE is packed
    fn detect_packing(&self, sections: &[PESection], overall_entropy: f64) -> bool {
        // Check for high entropy sections
        let high_entropy_sections = sections.iter()
            .filter(|s| s.entropy > self.config.entropy_threshold)
            .count();

        // Check for common packer section names
        let packer_names = ["upx", "aspack", "pecompact", "fsg", "vmprotect"];
        let has_packer_sections = sections.iter()
            .any(|s| packer_names.iter()
                .any(|packer| s.name.to_lowercase().contains(packer)));

        // Check for unusual section characteristics
        let executable_sections = sections.iter()
            .filter(|s| s.characteristics & 0x20000000 != 0) // IMAGE_SCN_MEM_EXECUTE
            .count();

        overall_entropy > self.config.entropy_threshold || 
        has_packer_sections || 
        high_entropy_sections > executable_sections ||
        sections.len() < 3 // Very few sections can indicate packing
    }

    /// Analyze strings in file data
    fn analyze_strings(&self, data: &[u8]) -> StringAnalysis {
        let data_str = String::from_utf8_lossy(data);
        
        // Extract URLs
        let urls: Vec<String> = URL_REGEX.find_iter(&data_str)
            .map(|m| m.as_str().to_string())
            .collect();

        // Extract IP addresses
        let ips: Vec<String> = IP_REGEX.find_iter(&data_str)
            .map(|m| m.as_str().to_string())
            .collect();

        // Extract email addresses
        let email_addresses: Vec<String> = EMAIL_REGEX.find_iter(&data_str)
            .map(|m| m.as_str().to_string())
            .collect();

        // Extract registry keys
        let registry_keys: Vec<String> = REGISTRY_REGEX.find_iter(&data_str)
            .map(|m| m.as_str().to_string())
            .collect();

        // Extract crypto indicators
        let crypto_indicators: Vec<String> = CRYPTO_REGEX.find_iter(&data_str)
            .map(|m| m.as_str().to_string())
            .collect();

        // Find suspicious strings
        let mut suspicious_strings = Vec::new();
        for (pattern, category) in MALWARE_PATTERNS.iter() {
            for mat in pattern.find_iter(&data_str) {
                suspicious_strings.push(SuspiciousString {
                    content: mat.as_str().to_string(),
                    category: match *category {
                        "malware" => StringCategory::Malware,
                        "attack_technique" => StringCategory::Vulnerability,
                        "exploit" => StringCategory::Vulnerability,
                        "c2_communication" => StringCategory::Network,
                        "anti_analysis" => StringCategory::AntiAnalysis,
                        "injection" => StringCategory::SystemCall,
                        "cryptomining" => StringCategory::Cryptographic,
                        _ => StringCategory::Malware,
                    },
                    confidence: 0.8, // Default confidence
                    offset: mat.start(),
                });
            }
        }

        StringAnalysis {
            suspicious_strings,
            urls,
            ips,
            email_addresses,
            file_paths: vec![], // TODO: Implement file path detection
            registry_keys,
            crypto_indicators,
        }
    }

    /// Analyze file entropy
    fn analyze_entropy(&self, data: &[u8]) -> EntropyAnalysis {
        let overall_entropy = self.calculate_entropy(data);
        
        // Calculate entropy for 1KB chunks
        let mut high_entropy_regions = Vec::new();
        let chunk_size = 1024;
        
        for (i, chunk) in data.chunks(chunk_size).enumerate() {
            let entropy = self.calculate_entropy(chunk);
            if entropy > self.config.entropy_threshold {
                high_entropy_regions.push(EntropyRegion {
                    offset: i * chunk_size,
                    size: chunk.len(),
                    entropy,
                });
            }
        }

        EntropyAnalysis {
            overall_entropy,
            section_entropies: HashMap::new(), // Could be populated from PE analysis
            high_entropy_regions,
        }
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

    /// Evaluate PE-specific threats
    fn evaluate_pe_threats(&self, pe_analysis: &PEAnalysis) -> (Vec<String>, f64) {
        let mut threats = Vec::new();
        let mut confidence = 0.0;

        if pe_analysis.is_packed {
            threats.push("File appears to be packed or obfuscated".to_string());
            confidence += 0.3;
        }

        if pe_analysis.entropy > self.config.entropy_threshold {
            threats.push(format!("High entropy detected: {:.2}", pe_analysis.entropy));
            confidence += 0.2;
        }

        // Check for suspicious imports
        let suspicious_imports = ["CreateRemoteThread", "WriteProcessMemory", "VirtualAllocEx", 
                                "SetWindowsHookEx", "GetProcAddress", "LoadLibrary"];
        
        for import in &pe_analysis.imports {
            if suspicious_imports.iter().any(|&si| import.contains(si)) {
                threats.push(format!("Suspicious import detected: {}", import));
                confidence += 0.1;
            }
        }

        // Check for unusual sections
        for section in &pe_analysis.sections {
            if section.entropy > self.config.entropy_threshold {
                threats.push(format!("High entropy section: {} ({:.2})", 
                                   section.name, section.entropy));
                confidence += 0.1;
            }
        }

        (threats, confidence)
    }

    /// Evaluate string-based threats
    fn evaluate_string_threats(&self, string_analysis: &StringAnalysis) -> (Vec<String>, f64) {
        let mut threats = Vec::new();
        let mut confidence = 0.0;

        for sus_string in &string_analysis.suspicious_strings {
            if sus_string.confidence > self.config.suspicious_string_threshold {
                threats.push(format!("Suspicious string: {} ({:?})", 
                                   sus_string.content, sus_string.category));
                confidence += 0.1;
            }
        }

        if !string_analysis.urls.is_empty() {
            threats.push(format!("Found {} URLs", string_analysis.urls.len()));
            confidence += 0.05;
        }

        if !string_analysis.ips.is_empty() {
            threats.push(format!("Found {} IP addresses", string_analysis.ips.len()));
            confidence += 0.05;
        }

        if !string_analysis.crypto_indicators.is_empty() {
            threats.push(format!("Found {} cryptographic indicators", 
                               string_analysis.crypto_indicators.len()));
            confidence += 0.1;
        }

        (threats, confidence)
    }

    /// Evaluate entropy-based threats
    fn evaluate_entropy_threats(&self, entropy_analysis: &EntropyAnalysis) -> (Vec<String>, f64) {
        let mut threats = Vec::new();
        let mut confidence = 0.0;

        if entropy_analysis.overall_entropy > self.config.entropy_threshold {
            threats.push(format!("High overall entropy: {:.2}", 
                               entropy_analysis.overall_entropy));
            confidence += 0.2;
        }

        if !entropy_analysis.high_entropy_regions.is_empty() {
            threats.push(format!("Found {} high-entropy regions", 
                               entropy_analysis.high_entropy_regions.len()));
            confidence += 0.1;
        }

        (threats, confidence)
    }

    /// Determine final verdict based on analysis results
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

        let confidence = match confidence_score {
            s if s > 0.8 => ConfidenceLevel::High,
            s if s > 0.5 => ConfidenceLevel::Medium,
            _ => ConfidenceLevel::Low,
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
        let pe_header = b"MZ\x90\x00\x03\x00\x00\x00";
        assert_eq!(analyzer.detect_file_type(pe_header), FileType::PE);
        
        // Test ELF detection
        let elf_header = b"\x7fELF\x01\x01\x01\x00";
        assert_eq!(analyzer.detect_file_type(elf_header), FileType::ELF);
        
        // Test PDF detection
        let pdf_header = b"%PDF-1.4";
        assert_eq!(analyzer.detect_file_type(pdf_header), FileType::PDF);
        
        // Test unknown
        let unknown_header = b"UNKNOWN";
        assert_eq!(analyzer.detect_file_type(unknown_header), FileType::Unknown);
    }

    #[test]
    fn test_string_analysis() {
        let analyzer = StaticAnalyzer::new(StaticAnalyzerConfig::default());
        
        let test_data = b"This file contains http://malicious.com and CreateRemoteThread and 192.168.1.1";
        let analysis = analyzer.analyze_strings(test_data);
        
        assert!(!analysis.urls.is_empty());
        assert!(!analysis.ips.is_empty());
        assert!(analysis.urls.contains(&"http://malicious.com".to_string()));
        assert!(analysis.ips.contains(&"192.168.1.1".to_string()));
    }

    #[tokio::test]
    async fn test_pe_analysis() {
        let analyzer = StaticAnalyzer::new(StaticAnalyzerConfig::default());
        
        // Create minimal PE header for testing
        let mut pe_data = vec![0u8; 1024];
        pe_data[0] = b'M';
        pe_data[1] = b'Z';
        pe_data[60..64].copy_from_slice(&200u32.to_le_bytes()); // e_lfanew
        pe_data[200..204].copy_from_slice(b"PE\0\0"); // PE signature
        pe_data[204..206].copy_from_slice(&0x014cu16.to_le_bytes()); // Machine type (i386)
        pe_data[206..208].copy_from_slice(&3u16.to_le_bytes()); // Number of sections
        
        let result = analyzer.analyze_pe(&pe_data).await;
        assert!(result.is_ok());
        
        let pe_analysis = result.unwrap();
        assert_eq!(pe_analysis.machine_type, "i386");
        assert_eq!(pe_analysis.sections.len(), 3);
    }

    #[tokio::test]
    async fn test_full_analysis() {
        let analyzer = StaticAnalyzer::new(StaticAnalyzerConfig::default());
        
        // Test with benign data
        let benign_data = b"Hello, this is a normal file with normal content";
        let result = analyzer.analyze(benign_data, Some("test.txt")).await;
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert_eq!(analysis.verdict, ThreatVerdict::Benign);
    }

    #[test]
    fn test_packing_detection() {
        let analyzer = StaticAnalyzer::new(StaticAnalyzerConfig::default());
        
        // Create sections with high entropy
        let high_entropy_section = PESection {
            name: ".text".to_string(),
            virtual_address: 0x1000,
            size: 0x2000,
            characteristics: 0x20000020,
            entropy: 7.8,
        };
        
        let sections = vec![high_entropy_section];
        let is_packed = analyzer.detect_packing(&sections, 7.5);
        assert!(is_packed);
        
        // Test with packer name
        let upx_section = PESection {
            name: "UPX0".to_string(),
            virtual_address: 0x1000,
            size: 0x1000,
            characteristics: 0x20000020,
            entropy: 6.0,
        };
        
        let sections = vec![upx_section];
        let is_packed = analyzer.detect_packing(&sections, 6.0);
        assert!(is_packed);
    }
}
