use std::collections::HashMap;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};
use futures::join;
use chrono::Utc;
use uuid::Uuid;

// Re-export all analyzer modules
pub mod hash_analyzer;
pub mod static_analyzer;
pub mod yara_engine;

// Re-export commonly used types
pub use hash_analyzer::{HashAnalyzer, HashAnalyzerConfig, HashInfo, HashType};
pub use static_analyzer::{StaticAnalyzer, StaticAnalyzerConfig, FileType, PEAnalysis, StringAnalysis, EntropyAnalysis};
pub use yara_engine::{YaraEngine, YaraEngineConfig, YaraMatch, YaraRule, YaraEngineError};

use crate::models::analysis_result::{AnalysisResult, ThreatVerdict, ConfidenceLevel, DetectionResult, FileMetadata, AnalysisStatus, EngineType, SeverityLevel, ThreatCategory};

/// Configuration for the combined analysis engine
#[derive(Debug, Clone)]
pub struct AnalysisEngineConfig {
    pub hash_analyzer: HashAnalyzerConfig,
    pub static_analyzer: StaticAnalyzerConfig,
    pub yara_engine: YaraEngineConfig,
    pub enable_parallel_analysis: bool,
    pub analysis_timeout_seconds: u64,
    pub require_all_analyzers: bool,
}

impl Default for AnalysisEngineConfig {
    fn default() -> Self {
        Self {
            hash_analyzer: HashAnalyzerConfig::default(),
            static_analyzer: StaticAnalyzerConfig::default(),
            yara_engine: YaraEngineConfig::default(),
            enable_parallel_analysis: true,
            analysis_timeout_seconds: 120,
            require_all_analyzers: false,
        }
    }
}

/// File analysis request
#[derive(Debug, Clone)]
pub struct FileAnalysisRequest {
    pub filename: String,
    pub file_data: Vec<u8>,
    pub file_hashes: Option<HashMap<HashType, String>>,
    pub analysis_options: AnalysisOptions,
}

/// Analysis options to control which analyzers to run
#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    pub enable_hash_analysis: bool,
    pub enable_static_analysis: bool,
    pub enable_yara_analysis: bool,
    pub priority: AnalysisPriority,
    pub custom_metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisPriority {
    Low,    // More thorough, slower
    Normal, // Balanced
    High,   // Fast, less thorough
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            enable_hash_analysis: true,
            enable_static_analysis: true,
            enable_yara_analysis: true,
            priority: AnalysisPriority::Normal,
            custom_metadata: HashMap::new(),
        }
    }
}

/// Main analysis engine that coordinates all analyzers
pub struct AnalysisEngine {
    config: AnalysisEngineConfig,
    hash_analyzer: HashAnalyzer,
    static_analyzer: StaticAnalyzer,
    yara_engine: YaraEngine,
}

impl AnalysisEngine {
    /// Create a new analysis engine with the given configuration
    pub fn new(config: AnalysisEngineConfig) -> Result<Self> {
        info!("Initializing analysis engine");

        let hash_analyzer = HashAnalyzer::new(config.hash_analyzer.clone());
        let static_analyzer = StaticAnalyzer::new(config.static_analyzer.clone());
        
        let yara_engine = YaraEngine::new(config.yara_engine.clone())
            .map_err(|e| anyhow!("Failed to initialize YARA engine: {}", e))?;

        Ok(Self {
            config,
            hash_analyzer,
            static_analyzer,
            yara_engine,
        })
    }

    /// Perform comprehensive analysis on a file
    pub async fn analyze_file(&mut self, request: FileAnalysisRequest) -> Result<AnalysisResult> {
        let start_time = std::time::Instant::now();
        
        info!("Starting comprehensive analysis for file: {}", request.filename);
        debug!("File size: {} bytes", request.file_data.len());

        // Run analysis with timeout
        let analysis_result = timeout(
            Duration::from_secs(self.config.analysis_timeout_seconds),
            self.perform_analysis(&request)
        ).await
        .map_err(|_| anyhow!("Analysis timeout after {} seconds", self.config.analysis_timeout_seconds))??;

        let total_time = start_time.elapsed().as_millis() as u64;
        
        info!("Analysis completed in {}ms for file: {}", total_time, request.filename);

        Ok(analysis_result)
    }

    async fn perform_analysis(&mut self, request: &FileAnalysisRequest) -> Result<AnalysisResult> {
        let mut detections = Vec::new();
        let mut analysis_errors = Vec::new();

        // Compute file metadata
        let file_metadata = self.create_file_metadata(request);

        let mut result = AnalysisResult::new(Uuid::new_v4(), file_metadata);
        result.started_at = Utc::now();

        if self.config.enable_parallel_analysis {
            // Run analyzers in parallel
            let hash_future = self.run_hash_analysis(request);
            let static_future = self.run_static_analysis(request);
            let yara_future = self.run_yara_analysis(request);

            let (hash_res, static_res, yara_res) = join!(hash_future, static_future, yara_future);

            // Collect results and errors
            match hash_res {
                Ok(mut dets) => detections.append(&mut dets),
                Err(e) => {
                    warn!("Hash analysis failed: {}", e);
                    analysis_errors.push(format!("Hash: {}", e));
                }
            }
            match static_res {
                Ok(det) => detections.push(det),
                Err(e) => {
                    warn!("Static analysis failed: {}", e);
                    analysis_errors.push(format!("Static: {}", e));
                }
            }
            match yara_res {
                Ok(det) => detections.push(det),
                Err(e) => {
                    warn!("Yara analysis failed: {}", e);
                    analysis_errors.push(format!("Yara: {}", e));
                }
            }
        } else {
            // Run sequentially
            if let Ok(mut dets) = self.run_hash_analysis(request).await {
                detections.append(&mut dets);
            }
            if let Ok(det) = self.run_static_analysis(request).await {
                detections.push(det);
            }
            if let Ok(det) = self.run_yara_analysis(request).await {
                detections.push(det);
            }
        }

        // Add detections to result
        for det in detections {
            result.add_detection(det);
        }

        // Handle errors
        if !analysis_errors.is_empty() && self.config.require_all_analyzers {
            result.mark_failed(analysis_errors.join("; "));
        } else {
            result.mark_completed();
        }

        Ok(result)
    }

    async fn run_hash_analysis(&self, request: &FileAnalysisRequest) -> Result<Vec<DetectionResult>> {
        if request.analysis_options.enable_hash_analysis {
            let hash_info = HashInfo {
                hash_type: HashType::SHA256,
                hash_value: self.hash_analyzer.compute_sha256(&request.file_data),  // Assume method added in hash_analyzer
                file_size: Some(request.file_data.len() as u64),
            };
            self.hash_analyzer.analyze_hash(&hash_info, Some(&request.file_data)).await
        } else {
            Ok(vec![])
        }
    }

    async fn run_static_analysis(&self, request: &FileAnalysisRequest) -> Result<DetectionResult> {
        if request.analysis_options.enable_static_analysis {
            self.static_analyzer.analyze(&request.file_data, Some(&request.filename)).await
        } else {
            Err(anyhow!("Static analysis disabled"))
        }
    }

    async fn run_yara_analysis(&self, request: &FileAnalysisRequest) -> Result<DetectionResult> {
        if request.analysis_options.enable_yara_analysis {
            self.yara_engine.analyze_file_data(&request.file_data, &request.filename).await // Assume method updated
        } else {
            Err(anyhow!("Yara analysis disabled"))
        }
    }

    fn create_file_metadata(&self, request: &FileAnalysisRequest) -> FileMetadata {
        let hashes = request.file_hashes.clone().unwrap_or_default();
        FileMetadata {
            filename: Some(request.filename.clone()),
            file_size: request.file_data.len() as u64,
            mime_type: "application/octet-stream".to_string(),
            md5: hashes.get(&HashType::MD5).cloned().unwrap_or_default(),
            sha1: hashes.get(&HashType::SHA1).cloned().unwrap_or_default(),
            sha256: hashes.get(&HashType::SHA256).cloned().unwrap_or_default(),
            sha512: None,
            entropy: None,
            magic_bytes: None,
            executable_info: None,
        }
    }

    /// Get statistics about the analysis engine
    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        
        // Combine stats from all analyzers
        let hash_stats = self.hash_analyzer.get_cache_stats();
        for (key, value) in hash_stats {
            stats.insert(format!("hash_{}", key), value.to_string());
        }

        let yara_stats = self.yara_engine.get_stats();
        for (key, value) in yara_stats {
            stats.insert(format!("yara_{}", key), value);
        }

        // Add engine-level stats
        stats.insert("parallel_analysis".to_string(), self.config.enable_parallel_analysis.to_string());
        stats.insert("analysis_timeout".to_string(), self.config.analysis_timeout_seconds.to_string());
        stats.insert("require_all_analyzers".to_string(), self.config.require_all_analyzers.to_string());

        stats
    }

    /// Clear all caches
    pub fn clear_caches(&mut self) {
        self.hash_analyzer.clear_cache();
        info!("All analyzer caches cleared");
    }

    /// Reload YARA rules
    pub fn reload_yara_rules(&mut self) -> Result<()> {
        self.yara_engine.reload_rules()
            .map_err(|e| anyhow!("Failed to reload YARA rules: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_analysis_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = AnalysisEngineConfig::default();
        config.yara_engine.rules_directory = temp_dir.path().to_path_buf();
        
        // Create a minimal YARA rule for testing
        std::fs::write(
            temp_dir.path().join("test.yara"),
            "rule TestRule { condition: true }"
        ).unwrap();

        let engine = AnalysisEngine::new(config);
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_file_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = AnalysisEngineConfig::default();
        config.yara_engine.rules_directory = temp_dir.path().to_path_buf();
        config.require_all_analyzers = false; // Allow partial failures for testing
        
        std::fs::write(
            temp_dir.path().join("test.yara"),
            "rule TestRule { condition: true }"
        ).unwrap();

        let mut engine = AnalysisEngine::new(config).unwrap();

        let request = FileAnalysisRequest {
            filename: "test.exe".to_string(),
            file_data: b"MZ\x90\x00test content".to_vec(),
            file_hashes: None,
            analysis_options: AnalysisOptions::default(),
        };

        let result = engine.analyze_file(request).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.status, AnalysisStatus::Completed);
    }

    #[test]
    fn test_analysis_options() {
        let options = AnalysisOptions {
            enable_hash_analysis: false,
            enable_static_analysis: true,
            enable_yara_analysis: true,
            priority: AnalysisPriority::High,
            custom_metadata: HashMap::from([
                ("source".to_string(), "unit_test".to_string()),
            ]),
        };

        assert!(!options.enable_hash_analysis);
        assert_eq!(options.priority, AnalysisPriority::High);
        assert_eq!(options.custom_metadata.get("source").unwrap(), "unit_test");
    }
}