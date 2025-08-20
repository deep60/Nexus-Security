use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::analysis_result::{AnalysisResult, ThreatLevel, MatchDetails};

#[derive(Error, Debug)]
pub enum YaraEngineError {
    #[error("YARA compilation failed: {0}")]
    CompilationError(String),
    #[error("YARA scan failed: {0}")]
    ScanError(String),
    #[error("Rule loading failed: {0}")]
    RuleLoadError(String),
    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Timeout error: scan took too long")]
    TimeoutError,
    #[error("Invalid rule format: {0}")]
    InvalidRuleFormat(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraMatch {
    pub rule_name: String,
    pub namespace: String,
    pub tags: Vec<String>,
    pub meta: HashMap<String, String>,
    pub strings: Vec<YaraStringMatch>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraStringMatch {
    pub identifier: String,
    pub offset: u64,
    pub length: usize,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraRule {
    pub name: String,
    pub namespace: String,
    pub content: String,
    pub tags: Vec<String>,
    pub meta: HashMap<String, String>,
    pub enabled: bool,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaraEngineConfig {
    pub rules_directory: PathBuf,
    pub max_scan_time: Duration,
    pub max_file_size: u64,
    pub enable_fast_mode: bool,
    pub max_matches_per_rule: usize,
    pub timeout_seconds: u64,
}

impl Default for YaraEngineConfig {
    fn default() ->  Self {
        Self {
            rules_directory: PathBuf::from("./rules"),
            max_scan_time: Duration::from_secs(30),
            max_file_size: 1024 * 1024 * 1024,    // 100MB
            enable_fast_mode: false,
            max_matches_per_rule: 100,
            timeout_seconds: 30,
        }
    }
}

pub struct YaraEngine {
    config: YaraEngineConfig,
    loaded_rules: Vec<YaraRule>,
    compiled_rules: Option<String>, // Placeholder - would be yara::Rules
    rules_hash: String,
}

impl YaraEngine {
    pub fn new(config: YaraEngineConfig) -> Result<Self, YaraEngineError> {
        let mut engine = Self {
            config,
            loaded_rules: Vec::new(),
            compiled_rules: None,
            rules_hash: String::new(),
        };

        engine.load_rules()?;
        engine.compile_rules()?;

        Ok(engine)
    }

    pub fn load_rules(&mut self) -> Result<(), YaraEngineError> {
        info!("Loading YARA rules from directory: {:?}", self.config.rules_directory);

        if !self.config.rules_directory.exists() {
            return Err(YaraEngineError::RuleLoadError(
                format!("Rules directory does not exists: {:?}", self.config.rules_directory)
            ));
        }

        let mut rules = Vec::new();
        let rule_files = self.discover_rule_files(&self.config.rules_directory)?;

        for rule_file in rule_files {
            match self.parse_rule_file(&rule_file) {
                Ok(mut file_rules) => {
                    rules.append(&mut file_rules);
                }
                Err(e) => {
                    warn!("Failed to purse rule file {:?}: {}", rule_file, e);
                    continue;
                }
            }
        }

        self.loaded_rules = rules.into_iter().filter(|rule| rule.enabled).collect();
        info!("Loaded {} active YARA rules", self.loaded_rules.len());

        // Generate hash of all rules for cache invalidation
        self.rules_hash = self.generate_rules_hash();

        Ok(())
    }

    fn discover_rule_files(&self, dir: &Path) -> Result<Vec<PathBuf>, YaraEngineError> {
        let mut rule_files = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "yara" || ext == "yar") {
                rule_files.push(path);
            } else if path.is_dir() {
                // Recursively search subdirectories
                let mut sub_files = self.discover_rule_files(&path)?;
                rule_files.append(&mut sub_files);
            }
        }

        Ok(rule_files)
    }

    fn parse_rule_file(&self, path: &Path) -> Result<Vec<YaraRule>, YaraEngineError> {
        let content = fs::read_to_string(path)?;
        let mut rules = Vec::new();

        // Simple YARA rule parser - in production, you'd want a more robust parser
        let rule_blocks = self.split_rules(&content);

        for (i, block) in rule_blocks.iter().enumerate() {
            if let Ok(rule) = self.parse_rule_block(block, path, i) {
                rules.push(rule);
            }
        }

        Ok(rules)
    }

    fn split_rules(&self, content: &str) -> Vec<String> {
        // Split YARA file into individual rule blocks
        let mut rules = Vec::new();
        let mut current_rule = String::new();
        let mut brace_count = 0;
        let mut in_rule = false;
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("rule ") {
                if !current_rule.is_empty() {
                    rules.push(current_rule.clone());
                }
                current_rule = String::new();
                in_rule = true;
                brace_count = 0;
            }
            
            if in_rule {
                current_rule.push_str(line);
                current_rule.push('\n');
                
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;
                
                if brace_count == 0 && current_rule.contains('{') {
                    rules.push(current_rule.clone());
                    current_rule = String::new();
                    in_rule = false;
                }
            }
        }
        
        if !current_rule.is_empty() {
            rules.push(current_rule);
        }
        
        rules
    }

    fn parse_rule_block(&self, block: &str, file_path: &Path, index: usize) -> Result<YaraRule, YaraEngineError> {
        let lines: Vec<&str> = block.lines().collect();
        
        // Extract rule name
        let rule_line = lines.iter()
            .find(|line| line.trim().starts_with("rule "))
            .ok_or_else(|| YaraEngineError::InvalidRuleFormat("No rule declaration found".to_string()))?;
        
        let rule_name = rule_line
            .split_whitespace()
            .nth(1)
            .and_then(|name| name.split(':').next())
            .and_then(|name| name.split('{').next())
            .map(|name| name.trim().to_string())
            .ok_or_else(|| YaraEngineError::InvalidRuleFormat("Could not extract rule name".to_string()))?;

        // Extract namespace (from file path or rule)
        let namespace = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("default")
            .to_string();

        // Extract tags and metadata
        let mut tags = Vec::new();
        let mut meta = HashMap::new();
        
        // Simple tag extraction
        if let Some(tag_line) = lines.iter().find(|line| line.contains("tags:")) {
            // Extract tags from the rule (simplified)
            tags = vec!["malware".to_string()]; // Placeholder
        }
        
        // Extract metadata
        for line in lines.iter() {
            if line.trim().starts_with("author") {
                if let Some(author) = line.split('=').nth(1) {
                    meta.insert("author".to_string(), author.trim_matches(|c| c == '"' || c == ' ').to_string());
                }
            }
            if line.trim().starts_with("description") {
                if let Some(desc) = line.split('=').nth(1) {
                    meta.insert("description".to_string(), desc.trim_matches(|c| c == '"' || c == ' ').to_string());
                }
            }
        }

        Ok(YaraRule {
            name: rule_name,
            namespace,
            content: block.to_string(),
            tags,
            meta,
            enabled: true,
            priority: 1,
        })
    }

    fn generate_rules_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        for rule in &self.loaded_rules {
            rule.content.hash(&mut hasher);
        }
        format!("{:x}", hasher.finish())
    }

    fn compile_rules(&mut self) -> Result<(), YaraEngineError> {
        info!("Compiling {} YARA rules", self.loaded_rules.len());
        
        // In a real implementation, you'd use the yara crate to compile rules
        // For now, we'll simulate the compilation process
        
        if self.loaded_rules.is_empty() {
            warn!("No rules to compile");
            return Ok(());
        }

        // Simulate compilation - in reality you'd do:
        // let mut compiler = yara::Compiler::new()?;
        // for rule in &self.loaded_rules {
        //     compiler.add_rules_str(&rule.content)?;
        // }
        // self.compiled_rules = Some(compiler.compile_rules()?);

        debug!("Successfully compiled {} YARA rules", self.loaded_rules.len());
        Ok(())
    }

    pub async fn analyze_file(&self, file_path: &Path) -> Result<AnalysisResult, YaraEngineError> {
        info!("Starting YARA analysis of file: {:?}", file_path);

        // Check file size
        let metadata = fs::metadata(file_path)?;
        if metadata.len() > self.config.max_file_size {
            return Err(YaraEngineError::ScanError(
                format!("File too large: {} bytes", metadata.len())
            ));
        }

        // Perform scan with timeout
        let scan_result = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.scan_file_internal(file_path)
        ).await
        .map_err(|_| YaraEngineError::TimeoutError)??;

        Ok(scan_result)
    }

    pub async fn analyze_bytes(&self, data: &[u8], filename: &str) -> Result<AnalysisResult, YaraEngineError> {
        info!("Starting YARA analysis of {} bytes for file: {}", data.len(), filename);

        if data.len() as u64 > self.config.max_file_size {
            return Err(YaraEngineError::ScanError(
                format!("Data too large: {} bytes", data.len())
            ));
        }

        let scan_result = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.scan_bytes_internal(data, filename)
        ).await
        .map_err(|_| YaraEngineError::TimeoutError)??;

        Ok(scan_result)
    }

    async fn scan_file_internal(&self, file_path: &Path) -> Result<AnalysisResult, YaraEngineError> {
        let file_data = fs::read(file_path)?;
        let filename = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
            
        self.scan_bytes_internal(&file_data, filename).await
    }

    async fn scan_bytes_internal(&self, data: &[u8], filename: &str) -> Result<AnalysisResult, YaraEngineError> {
        let mut matches = Vec::new();
        // Simulate YARA scanning - in reality you'd use:
        // let scan_results = self.compiled_rules
        //     .as_ref()
        //     .ok_or_else(|| YaraEngineError::ScanError("No compiled rules".to_string()))?
        //     .scan_mem(data, self.config.timeout_seconds as i32)?;

        // For demonstration, we'll create some mock matches based on simple heuristics
        matches.extend(self.perform_heuristic_analysis(data, filename));

        let threat_level = self.calculate_threat_level(&matches);
        let confidence = self.calculate_confidence(&matches);

        use crate::models::analysis_result::{FileMetadata, ThreatVerdict, SeverityLevel, DetectionResult, EngineType};
        use uuid::Uuid;
        use chrono::Utc;
        
        // Create file metadata
        let file_metadata = FileMetadata {
            filename: Some(filename.to_string()),
            file_size: data.len() as u64,
            mime_type: "application/octet-stream".to_string(),
            md5: String::new(),
            sha1: String::new(),
            sha256: self.calculate_hash(data),
            sha512: None,
            entropy: None,
            magic_bytes: None,
            executable_info: None,
        };
        
        let mut result = AnalysisResult::new(Uuid::new_v4(), file_metadata);
        
        // Add YARA matches to the result
        for m in matches {
            let detection = DetectionResult {
                detection_id: Uuid::new_v4(),
                engine_name: "YARA".to_string(),
                engine_version: "1.0.0".to_string(),
                engine_type: EngineType::Yara,
                verdict: if m.confidence > 0.7 { ThreatVerdict::Malicious } else { ThreatVerdict::Suspicious },
                confidence: m.confidence,
                severity: if m.tags.contains(&"critical".to_string()) { SeverityLevel::Critical } else { SeverityLevel::Medium },
                categories: vec![],
                metadata: m.meta.into_iter().map(|(k, v)| (k, serde_json::json!(v))).collect(),
                detected_at: Utc::now(),
                processing_time_ms: 100,
                error_message: None,
            };
            result.add_detection(detection);
        }
        
        result.mark_completed();
        Ok(result)
    }

    fn perform_heuristic_analysis(&self, data: &[u8], filename: &str) -> Vec<YaraMatch> {
        let mut matches = Vec::new();

        // Simple heuristic checks (in reality, YARA rules would do this)
        
        // Check for PE header
        if data.len() > 2 && &data[0..2] == b"MZ" {
            matches.push(YaraMatch {
                rule_name: "pe_file_detected".to_string(),
                namespace: "heuristic".to_string(),
                tags: vec!["executable".to_string()],
                meta: HashMap::from([
                    ("description".to_string(), "Portable Executable file detected".to_string()),
                ]),
                strings: vec![YaraStringMatch {
                    identifier: "mz_header".to_string(),
                    offset: 0,
                    length: 2,
                    content: "MZ".to_string(),
                }],
                confidence: 0.8,
            });
        }

        // Check for suspicious strings
        let suspicious_strings = [
            b"cmd.exe", b"powershell", b"CreateProcess", b"VirtualAlloc",
            b"GetProcAddress", b"LoadLibrary", b"RegCreateKey"
        ];

        for (i, window) in data.windows(10).enumerate() {
            for suspicious in &suspicious_strings {
                if window.starts_with(suspicious) {
                    matches.push(YaraMatch {
                        rule_name: format!("suspicious_string_{}", String::from_utf8_lossy(suspicious)),
                        namespace: "heuristic".to_string(),
                        tags: vec!["suspicious".to_string()],
                        meta: HashMap::from([
                            ("description".to_string(), format!("Suspicious string found: {}", String::from_utf8_lossy(suspicious))),
                        ]),
                        strings: vec![YaraStringMatch {
                            identifier: "suspicious_string".to_string(),
                            offset: i as u64,
                            length: suspicious.len(),
                            content: String::from_utf8_lossy(suspicious).to_string(),
                        }],
                        confidence: 0.6,
                    });
                    break;
                }
            }
        }

        // Check file extension
        if filename.ends_with(".exe") || filename.ends_with(".dll") || filename.ends_with(".scr") {
            matches.push(YaraMatch {
                rule_name: "executable_extension".to_string(),
                namespace: "heuristic".to_string(),
                tags: vec!["executable".to_string()],
                meta: HashMap::from([
                    ("description".to_string(), "Executable file extension detected".to_string()),
                ]),
                strings: vec![],
                confidence: 0.7,
            });
        }

        matches
    }

    fn calculate_threat_level(&self, matches: &[YaraMatch]) -> ThreatLevel {
        if matches.is_empty() {
            return ThreatLevel::Clean;
        }

        let max_confidence = matches.iter()
            .map(|m| m.confidence)
            .fold(0.0f32, |acc, x| acc.max(x));

        let has_critical = matches.iter().any(|m| m.tags.contains(&"critical".to_string()));
        let suspicious_count = matches.len();

        if has_critical || max_confidence > 0.8 {
            ThreatLevel::Critical
        } else if max_confidence > 0.6 || suspicious_count > 3 {
            ThreatLevel::High
        } else if max_confidence > 0.4 || suspicious_count > 1 {
            ThreatLevel::Medium
        } else {
            ThreatLevel::Low
        }
    }

    fn calculate_confidence(&self, matches: &[YaraMatch]) -> f32 {
        if matches.is_empty() {
            return 0.95; // High confidence in clean files
        }

        let avg_confidence: f32 = matches.iter()
            .map(|m| m.confidence)
            .sum::<f32>() / matches.len() as f32;

        // Adjust confidence based on number of matches
        let match_bonus = (matches.len() as f32 * 0.1).min(0.3);
        
        (avg_confidence + match_bonus).min(1.0)
    }

    fn calculate_hash(&self, data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    pub fn reload_rules(&mut self) -> Result<(), YaraEngineError> {
        info!("Reloading YARA rules");
        self.load_rules()?;
        self.compile_rules()?;
        info!("Successfully reloaded {} rules", self.loaded_rules.len());
        Ok(())
    }

    pub fn get_loaded_rules(&self) -> &[YaraRule] {
        &self.loaded_rules
    }

    pub fn get_rules_hash(&self) -> &str {
        &self.rules_hash
    }

    pub fn get_stats(&self) -> HashMap<String, String> {
        HashMap::from([
            ("rules_loaded".to_string(), self.loaded_rules.len().to_string()),
            ("rules_hash".to_string(), self.rules_hash.clone()),
            ("rules_directory".to_string(), self.config.rules_directory.display().to_string()),
            ("max_file_size".to_string(), self.config.max_file_size.to_string()),
            ("timeout_seconds".to_string(), self.config.timeout_seconds.to_string()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_rule(name: &str, content: &str) -> String {
        format!(r#" rule {}
{{
    meta:
        author = "test"
        description = "test rule"
    
    strings:
        $test = "{}"
    
    condition:
        $test
}}
        "#, name, content)
    }
    
    #[tokio::test]
    async fn test_yara_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().to_path_buf();
        
        // Create a test rule file
        let rule_content = create_test_rule("TestRule", "malware");
        std::fs::write(rules_dir.join("test.yara"), rule_content).unwrap();
        
        let config = YaraEngineConfig {
            rules_directory: rules_dir,
            ..Default::default()
        };
        
        let engine = YaraEngine::new(config);
        assert!(engine.is_ok());
        
        let engine = engine.unwrap();
        assert_eq!(engine.get_loaded_rules().len(), 1);
    }
    
    #[tokio::test]
    async fn test_file_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().to_path_buf();
        
        let rule_content = create_test_rule("TestRule", "test");
        std::fs::write(rules_dir.join("test.yara"), rule_content).unwrap();
        
        let config = YaraEngineConfig {
            rules_directory: rules_dir,
            ..Default::default()
        };
        
        let engine = YaraEngine::new(config).unwrap();
        
        // Create test file
        let test_file = temp_dir.path().join("test.exe");
        std::fs::write(&test_file, b"MZ test content").unwrap();
        
        let result = engine.analyze_file(&test_file).await;
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert!(!analysis.detections.is_empty());
    }
}