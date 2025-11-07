/// File scanner for analyzing various file types
///
/// Provides comprehensive file analysis including:
/// - File type detection
/// - Magic byte analysis
/// - Entropy calculation
/// - Embedded resource extraction
/// - Signature-based detection

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::{
    ArtifactType, Finding, FindingCategory, ScanResult, ScanVerdict, Scanner, ScannerConfig,
    ThreatLevel,
};

/// Configuration for file scanner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileScannerConfig {
    pub base: ScannerConfig,
    pub check_magic_bytes: bool,
    pub calculate_entropy: bool,
    pub extract_strings: bool,
    pub max_string_length: usize,
    pub min_string_length: usize,
    pub detect_packers: bool,
    pub scan_embedded_files: bool,
}

impl Default for FileScannerConfig {
    fn default() -> Self {
        Self {
            base: ScannerConfig {
                scanner_name: "File Scanner".to_string(),
                ..Default::default()
            },
            check_magic_bytes: true,
            calculate_entropy: true,
            extract_strings: true,
            max_string_length: 1000,
            min_string_length: 4,
            detect_packers: true,
            scan_embedded_files: true,
        }
    }
}

/// Detailed file scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileScanResult {
    pub base: ScanResult,
    pub file_type: FileType,
    pub file_info: FileInfo,
    pub entropy: f64,
    pub strings_found: Vec<String>,
    pub embedded_files: Vec<EmbeddedFile>,
    pub signature_matches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    Executable,
    Document,
    Archive,
    Script,
    Image,
    Audio,
    Video,
    Text,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub size: u64,
    pub mime_type: String,
    pub magic_bytes: String,
    pub extension: Option<String>,
    pub is_packed: bool,
    pub packer_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedFile {
    pub file_id: Uuid,
    pub name: String,
    pub size: u64,
    pub offset: u64,
    pub file_type: FileType,
    pub is_suspicious: bool,
}

/// File scanner implementation
pub struct FileScanner {
    config: FileScannerConfig,
    known_malicious_patterns: Vec<Vec<u8>>,
    known_packer_signatures: HashMap<String, Vec<u8>>,
    suspicious_strings: Vec<String>,
}

impl Scanner for FileScanner {
    type Config = FileScannerConfig;
    type Result = FileScanResult;

    fn new(config: Self::Config) -> Result<Self> {
        info!("Initializing file scanner");

        let known_malicious_patterns = Self::load_malicious_patterns();
        let known_packer_signatures = Self::load_packer_signatures();
        let suspicious_strings = Self::load_suspicious_strings();

        Ok(Self {
            config,
            known_malicious_patterns,
            known_packer_signatures,
            suspicious_strings,
        })
    }

    async fn scan(
        &self,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
    ) -> Result<Self::Result> {
        let start_time = std::time::Instant::now();
        info!("Starting file scan on {} bytes", data.len());

        // Check file size limit
        if data.len() > (self.config.base.max_file_size_mb as usize * 1024 * 1024) {
            return Err(anyhow!(
                "File size exceeds limit of {} MB",
                self.config.base.max_file_size_mb
            ));
        }

        let mut base_result = ScanResult::new(ArtifactType::File);

        // Detect file type
        let file_type = self.detect_file_type(data);
        debug!("Detected file type: {:?}", file_type);

        // Gather file information
        let file_info = self.gather_file_info(data, &metadata);

        // Calculate entropy
        let entropy = if self.config.calculate_entropy {
            self.calculate_entropy(data)
        } else {
            0.0
        };

        // Check for high entropy (possible encryption/packing)
        if entropy > 7.5 {
            base_result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "High entropy detected".to_string(),
                description: format!("File entropy: {:.2} (possible packing/encryption)", entropy),
                severity: ThreatLevel::Medium,
                evidence: vec![format!("Entropy: {:.2}", entropy)],
                recommendation: Some("Investigate for packed or encrypted content".to_string()),
            });
        }

        // Extract strings
        let strings_found = if self.config.extract_strings {
            self.extract_strings(data)
        } else {
            Vec::new()
        };

        // Check for suspicious strings
        self.check_suspicious_strings(&strings_found, &mut base_result);

        // Detect packers
        if self.config.detect_packers {
            if let Some(packer_name) = self.detect_packer(data) {
                base_result.add_finding(Finding {
                    finding_id: Uuid::new_v4(),
                    category: FindingCategory::Suspicious,
                    title: "Packer detected".to_string(),
                    description: format!("File appears to be packed with: {}", packer_name),
                    severity: ThreatLevel::Medium,
                    evidence: vec![format!("Packer: {}", packer_name)],
                    recommendation: Some("Unpack and analyze contents".to_string()),
                });
            }
        }

        // Check magic bytes
        if self.config.check_magic_bytes {
            self.check_magic_bytes(data, &mut base_result);
        }

        // Scan for embedded files
        let embedded_files = if self.config.scan_embedded_files {
            self.scan_for_embedded_files(data)
        } else {
            Vec::new()
        };

        // Check for malicious patterns
        let signature_matches = self.check_malicious_patterns(data);
        if !signature_matches.is_empty() {
            base_result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Malware,
                title: "Malicious pattern detected".to_string(),
                description: format!("{} malicious patterns found", signature_matches.len()),
                severity: ThreatLevel::High,
                evidence: signature_matches.clone(),
                recommendation: Some("Quarantine file immediately".to_string()),
            });
        }

        // File type-specific checks
        self.perform_type_specific_checks(data, &file_type, &mut base_result);

        let scan_duration_ms = start_time.elapsed().as_millis() as u64;
        base_result.scan_duration_ms = scan_duration_ms;

        info!(
            "File scan completed in {}ms - Verdict: {:?}",
            scan_duration_ms, base_result.verdict
        );

        Ok(FileScanResult {
            base: base_result,
            file_type,
            file_info,
            entropy,
            strings_found: strings_found.into_iter().take(100).collect(), // Limit to 100 strings
            embedded_files,
            signature_matches,
        })
    }

    fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert(
            "scanner_name".to_string(),
            self.config.base.scanner_name.clone(),
        );
        stats.insert(
            "malicious_patterns".to_string(),
            self.known_malicious_patterns.len().to_string(),
        );
        stats.insert(
            "packer_signatures".to_string(),
            self.known_packer_signatures.len().to_string(),
        );
        stats.insert(
            "suspicious_strings".to_string(),
            self.suspicious_strings.len().to_string(),
        );
        stats
    }

    fn health_check(&self) -> bool {
        self.config.base.enabled
    }
}

impl FileScanner {
    /// Detect file type from content
    fn detect_file_type(&self, data: &[u8]) -> FileType {
        if data.len() < 4 {
            return FileType::Unknown;
        }

        // Check magic bytes
        match &data[0..2] {
            b"MZ" => FileType::Executable, // PE/DOS executable
            b"PK" => FileType::Archive,    // ZIP
            [0x7f, 0x45] if data.len() > 4 && &data[1..4] == b"ELF" => FileType::Executable, // ELF
            _ => {}
        }

        // Check for common file signatures
        if data.len() >= 4 {
            match &data[0..4] {
                [0x89, 0x50, 0x4E, 0x47] => return FileType::Image, // PNG
                [0xFF, 0xD8, 0xFF, _] => return FileType::Image,    // JPEG
                [0x25, 0x50, 0x44, 0x46] => return FileType::Document, // PDF
                [0x52, 0x61, 0x72, 0x21] => return FileType::Archive, // RAR
                _ => {}
            }
        }

        // Check for script files
        if data.starts_with(b"#!/") || data.starts_with(b"<?php") || data.starts_with(b"<script") {
            return FileType::Script;
        }

        // Check for text content
        if data.iter().take(512).all(|&b| b.is_ascii() || b == b'\n' || b == b'\r' || b == b'\t')
        {
            return FileType::Text;
        }

        FileType::Unknown
    }

    /// Gather file information
    fn gather_file_info(
        &self,
        data: &[u8],
        metadata: &Option<HashMap<String, String>>,
    ) -> FileInfo {
        let magic_bytes = if data.len() >= 4 {
            format!("{:02X}{:02X}{:02X}{:02X}", data[0], data[1], data[2], data[3])
        } else {
            "Unknown".to_string()
        };

        let extension = metadata
            .as_ref()
            .and_then(|m| m.get("extension"))
            .map(|s| s.to_string());

        let mime_type = self.detect_mime_type(data);

        let (is_packed, packer_name) = if self.config.detect_packers {
            if let Some(packer) = self.detect_packer(data) {
                (true, Some(packer))
            } else {
                (false, None)
            }
        } else {
            (false, None)
        };

        FileInfo {
            size: data.len() as u64,
            mime_type,
            magic_bytes,
            extension,
            is_packed,
            packer_name,
        }
    }

    /// Detect MIME type
    fn detect_mime_type(&self, data: &[u8]) -> String {
        if data.len() < 4 {
            return "application/octet-stream".to_string();
        }

        match &data[0..2] {
            b"MZ" => "application/x-msdownload".to_string(),
            b"PK" => "application/zip".to_string(),
            _ => {}
        }

        if data.len() >= 4 {
            match &data[0..4] {
                [0x89, 0x50, 0x4E, 0x47] => return "image/png".to_string(),
                [0xFF, 0xD8, 0xFF, _] => return "image/jpeg".to_string(),
                [0x25, 0x50, 0x44, 0x46] => return "application/pdf".to_string(),
                _ => {}
            }
        }

        "application/octet-stream".to_string()
    }

    /// Calculate Shannon entropy
    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        let mut byte_counts = [0u64; 256];
        for &byte in data {
            byte_counts[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &count in &byte_counts {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Extract printable strings
    fn extract_strings(&self, data: &[u8]) -> Vec<String> {
        let mut strings = Vec::new();
        let mut current_string = Vec::new();

        for &byte in data {
            if byte.is_ascii_graphic() || byte == b' ' {
                current_string.push(byte);
            } else {
                if current_string.len() >= self.config.min_string_length
                    && current_string.len() <= self.config.max_string_length
                {
                    if let Ok(s) = String::from_utf8(current_string.clone()) {
                        strings.push(s);
                    }
                }
                current_string.clear();
            }
        }

        strings
    }

    /// Check for suspicious strings
    fn check_suspicious_strings(&self, strings: &[String], result: &mut ScanResult) {
        let mut found_suspicious = Vec::new();

        for string in strings {
            let lower = string.to_lowercase();
            for pattern in &self.suspicious_strings {
                if lower.contains(pattern) {
                    found_suspicious.push(string.clone());
                    break;
                }
            }
        }

        if !found_suspicious.is_empty() {
            result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "Suspicious strings found".to_string(),
                description: format!("{} suspicious strings detected", found_suspicious.len()),
                severity: ThreatLevel::Medium,
                evidence: found_suspicious.into_iter().take(10).collect(),
                recommendation: Some("Review strings for malicious intent".to_string()),
            });
        }
    }

    /// Detect packer
    fn detect_packer(&self, data: &[u8]) -> Option<String> {
        for (packer_name, signature) in &self.known_packer_signatures {
            if data.len() >= signature.len() {
                if data[..signature.len()] == signature[..] {
                    return Some(packer_name.clone());
                }
            }
        }
        None
    }

    /// Check magic bytes for mismatches
    fn check_magic_bytes(&self, data: &[u8], result: &mut ScanResult) {
        // This is a simplified check - in production, compare with file extension
        if data.len() < 2 {
            return;
        }

        // Check for common executable disguised as other types
        if data.starts_with(b"MZ") {
            result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "Executable detected".to_string(),
                description: "File contains PE/DOS executable header".to_string(),
                severity: ThreatLevel::Low,
                evidence: vec!["Magic bytes: MZ".to_string()],
                recommendation: None,
            });
        }
    }

    /// Scan for embedded files
    fn scan_for_embedded_files(&self, data: &[u8]) -> Vec<EmbeddedFile> {
        let mut embedded_files = Vec::new();

        // Look for embedded PE files
        for (i, window) in data.windows(2).enumerate() {
            if window == b"MZ" && i > 512 {
                // Likely embedded PE
                embedded_files.push(EmbeddedFile {
                    file_id: Uuid::new_v4(),
                    name: format!("embedded_pe_{}.exe", i),
                    size: 0, // Unknown size
                    offset: i as u64,
                    file_type: FileType::Executable,
                    is_suspicious: true,
                });
            }
        }

        // Look for embedded ZIP archives (PK header)
        for (i, window) in data.windows(2).enumerate() {
            if window == b"PK" && i > 512 {
                embedded_files.push(EmbeddedFile {
                    file_id: Uuid::new_v4(),
                    name: format!("embedded_archive_{}.zip", i),
                    size: 0,
                    offset: i as u64,
                    file_type: FileType::Archive,
                    is_suspicious: true,
                });
            }
        }

        embedded_files
    }

    /// Check for known malicious patterns
    fn check_malicious_patterns(&self, data: &[u8]) -> Vec<String> {
        let mut matches = Vec::new();

        for (i, pattern) in self.known_malicious_patterns.iter().enumerate() {
            if data
                .windows(pattern.len())
                .any(|window| window == pattern.as_slice())
            {
                matches.push(format!("Malicious pattern #{}", i + 1));
            }
        }

        matches
    }

    /// Perform file type-specific checks
    fn perform_type_specific_checks(
        &self,
        data: &[u8],
        file_type: &FileType,
        result: &mut ScanResult,
    ) {
        match file_type {
            FileType::Executable => self.check_executable(data, result),
            FileType::Script => self.check_script(data, result),
            FileType::Document => self.check_document(data, result),
            _ => {}
        }
    }

    /// Check executable-specific threats
    fn check_executable(&self, data: &[u8], result: &mut ScanResult) {
        // Check for suspicious imports/exports
        if let Some(pos) = Self::find_pattern(data, b"GetProcAddress") {
            result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "Dynamic API loading detected".to_string(),
                description: "Executable uses GetProcAddress (common in malware)".to_string(),
                severity: ThreatLevel::Medium,
                evidence: vec![format!("Found at offset: {}", pos)],
                recommendation: None,
            });
        }

        // Check for network-related imports
        if Self::find_pattern(data, b"WinHttpOpen").is_some()
            || Self::find_pattern(data, b"InternetOpen").is_some()
        {
            result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "Network capabilities detected".to_string(),
                description: "Executable has network communication capabilities".to_string(),
                severity: ThreatLevel::Low,
                evidence: vec!["Network API imports".to_string()],
                recommendation: None,
            });
        }
    }

    /// Check script-specific threats
    fn check_script(&self, data: &[u8], result: &mut ScanResult) {
        let content = String::from_utf8_lossy(data).to_lowercase();

        // Check for obfuscation
        if content.contains("eval(") || content.contains("exec(") {
            result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "Code execution detected".to_string(),
                description: "Script contains eval/exec statements".to_string(),
                severity: ThreatLevel::Medium,
                evidence: vec!["eval/exec usage".to_string()],
                recommendation: Some("Review script for malicious code".to_string()),
            });
        }

        // Check for download capabilities
        if content.contains("download") || content.contains("curl") || content.contains("wget") {
            result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "Download capability detected".to_string(),
                description: "Script can download files".to_string(),
                severity: ThreatLevel::Medium,
                evidence: vec!["Download commands".to_string()],
                recommendation: None,
            });
        }
    }

    /// Check document-specific threats
    fn check_document(&self, data: &[u8], result: &mut ScanResult) {
        // Check for embedded executables in documents
        if Self::find_pattern(data, b"MZ").is_some() {
            result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "Embedded executable in document".to_string(),
                description: "Document contains embedded executable".to_string(),
                severity: ThreatLevel::High,
                evidence: vec!["PE header found".to_string()],
                recommendation: Some("Do not open without isolation".to_string()),
            });
        }

        // Check for macros (simplified - look for VBA indicators)
        if Self::find_pattern(data, b"VBA").is_some()
            || Self::find_pattern(data, b"Macro").is_some()
        {
            result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "Macros detected".to_string(),
                description: "Document may contain VBA macros".to_string(),
                severity: ThreatLevel::Medium,
                evidence: vec!["VBA/Macro indicators".to_string()],
                recommendation: Some("Disable macros before opening".to_string()),
            });
        }
    }

    /// Helper: Find pattern in data
    fn find_pattern(data: &[u8], pattern: &[u8]) -> Option<usize> {
        data.windows(pattern.len())
            .position(|window| window == pattern)
    }

    /// Load malicious patterns
    fn load_malicious_patterns() -> Vec<Vec<u8>> {
        // In production, load from signature database
        vec![
            // Example malicious patterns (simplified)
            vec![0xEB, 0xFE], // Infinite loop
        ]
    }

    /// Load packer signatures
    fn load_packer_signatures() -> HashMap<String, Vec<u8>> {
        let mut signatures = HashMap::new();
        // UPX packer signature
        signatures.insert("UPX".to_string(), vec![0x55, 0x50, 0x58, 0x21]);
        signatures
    }

    /// Load suspicious strings
    fn load_suspicious_strings() -> Vec<String> {
        vec![
            "cmd.exe".to_string(),
            "powershell".to_string(),
            "keylogger".to_string(),
            "backdoor".to_string(),
            "trojan".to_string(),
            "ransomware".to_string(),
            "encrypt".to_string(),
            "decrypt".to_string(),
            "payload".to_string(),
            "exploit".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_scanner_creation() {
        let config = FileScannerConfig::default();
        let scanner = FileScanner::new(config);
        assert!(scanner.is_ok());
    }

    #[tokio::test]
    async fn test_scan_pe_file() {
        let scanner = FileScanner::new(FileScannerConfig::default()).unwrap();
        let pe_data = b"MZ\x90\x00\x03\x00\x00\x00test executable content";

        let result = scanner.scan(pe_data, None).await;
        assert!(result.is_ok());

        let scan_result = result.unwrap();
        assert_eq!(scan_result.file_type, FileType::Executable);
    }

    #[tokio::test]
    async fn test_entropy_calculation() {
        let scanner = FileScanner::new(FileScannerConfig::default()).unwrap();

        // Low entropy data (repetitive)
        let low_entropy = vec![0x41; 1000];
        let entropy_low = scanner.calculate_entropy(&low_entropy);
        assert!(entropy_low < 1.0);

        // High entropy data (random)
        let high_entropy: Vec<u8> = (0..=255).cycle().take(1000).collect();
        let entropy_high = scanner.calculate_entropy(&high_entropy);
        assert!(entropy_high > 5.0);
    }

    #[tokio::test]
    async fn test_string_extraction() {
        let scanner = FileScanner::new(FileScannerConfig::default()).unwrap();
        let data = b"Test\x00String\x00Extraction\x00";

        let strings = scanner.extract_strings(data);
        assert!(strings.contains(&"Test".to_string()));
        assert!(strings.contains(&"String".to_string()));
    }

    #[test]
    fn test_file_type_detection() {
        let scanner = FileScanner::new(FileScannerConfig::default()).unwrap();

        // PE file
        let pe_data = b"MZ\x90\x00\x03";
        assert_eq!(scanner.detect_file_type(pe_data), FileType::Executable);

        // ZIP file
        let zip_data = b"PK\x03\x04";
        assert_eq!(scanner.detect_file_type(zip_data), FileType::Archive);

        // PNG image
        let png_data = b"\x89PNG";
        assert_eq!(scanner.detect_file_type(png_data), FileType::Image);
    }
}
