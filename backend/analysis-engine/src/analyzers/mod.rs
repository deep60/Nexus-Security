use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::HashMap;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, warn};
use uuid::Uuid;

// Re-export all analyzer modules
pub mod dynamic_analyzer;
pub mod hash_analyzer;
pub mod heuristic_engine;
pub mod signature_matcher;
pub mod static_analyzer;

#[cfg(feature = "clamav")]
pub mod clamav_analyzer;
#[cfg(feature = "ml-engine")]
pub mod ml_analyzer;
#[cfg(feature = "yara-engine")]
pub mod yara_engine;

// Re-export commonly used types
pub use hash_analyzer::{HashAnalyzer, HashAnalyzerConfig, HashInfo, HashType};
pub use heuristic_engine::{HeuristicEngine, HeuristicMatch, HeuristicSeverity};
pub use signature_matcher::{
    SignatureMatch, SignatureMatcher, SignatureMatcherConfig, ThreatSeverity as SignatureSeverity,
};
pub use static_analyzer::{StaticAnalyzer, StaticAnalyzerConfig};

#[cfg(feature = "clamav")]
pub use clamav_analyzer::{ClamAvAnalyzer, ClamAvAnalyzerConfig};
#[cfg(feature = "ml-engine")]
#[allow(unused_imports)]
pub use ml_analyzer::{MlAnalyzer, MlAnalyzerConfig};
#[cfg(feature = "yara-engine")]
#[allow(unused_imports)]
pub use yara_engine::{YaraEngine, YaraEngineConfig, YaraEngineError, YaraMatch, YaraRule};

// ── Stubs when native features are disabled ──────────────────────

#[cfg(not(feature = "yara-engine"))]
pub mod yara_stub {
    use crate::models::analysis_result::DetectionResult;
    use anyhow::Result;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[derive(Debug, Clone)]
    pub struct YaraEngineConfig {
        pub rules_directory: PathBuf,
        pub max_matches: usize,
    }
    impl Default for YaraEngineConfig {
        fn default() -> Self {
            Self {
                rules_directory: PathBuf::from("./rules"),
                max_matches: 100,
            }
        }
    }

    pub struct YaraEngine;
    impl YaraEngine {
        pub fn new(_config: YaraEngineConfig) -> Result<Self> {
            Ok(Self)
        }
        pub async fn analyze_file_data(
            &self,
            _data: &[u8],
            _filename: &str,
        ) -> Result<DetectionResult> {
            Err(anyhow::anyhow!(
                "YARA engine not compiled (enable 'yara-engine' feature)"
            ))
        }
        pub fn get_stats(&self) -> HashMap<String, String> {
            HashMap::new()
        }
        pub fn reload_rules(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct YaraMatch {
        pub rule: String,
    }
    #[derive(Debug, Clone)]
    pub struct YaraRule;
    #[derive(Debug, thiserror::Error)]
    #[error("YARA engine not available")]
    pub struct YaraEngineError;
}
#[cfg(not(feature = "yara-engine"))]
pub use yara_stub::*;

#[cfg(not(feature = "clamav"))]
pub mod clamav_stub {
    use crate::models::analysis_result::DetectionResult;
    use anyhow::Result;

    #[derive(Debug, Clone)]
    pub struct ClamAvAnalyzerConfig {
        pub host: String,
        pub port: u16,
        pub enabled: bool,
    }
    impl Default for ClamAvAnalyzerConfig {
        fn default() -> Self {
            Self {
                host: "localhost".into(),
                port: 3310,
                enabled: false,
            }
        }
    }

    pub struct ClamAvAnalyzer {
        _config: ClamAvAnalyzerConfig,
    }
    impl ClamAvAnalyzer {
        pub fn new(config: ClamAvAnalyzerConfig) -> Self {
            Self { _config: config }
        }
        pub async fn scan_file(&self, _data: &[u8], _filename: &str) -> Result<DetectionResult> {
            Err(anyhow::anyhow!(
                "ClamAV not compiled (enable 'clamav' feature)"
            ))
        }
    }
}
#[cfg(not(feature = "clamav"))]
pub use clamav_stub::*;

#[cfg(not(feature = "ml-engine"))]
pub mod ml_stub {
    use crate::models::analysis_result::DetectionResult;
    use anyhow::Result;

    #[derive(Debug, Clone)]
    pub struct MlAnalyzerConfig {
        pub enabled: bool,
        pub feature_size: usize,
    }
    impl Default for MlAnalyzerConfig {
        fn default() -> Self {
            Self {
                enabled: false,
                feature_size: 256,
            }
        }
    }

    pub struct MlAnalyzer {
        _config: MlAnalyzerConfig,
    }
    impl MlAnalyzer {
        pub fn new(config: MlAnalyzerConfig) -> Self {
            Self { _config: config }
        }
        pub fn is_available(&self) -> bool {
            false
        }
        pub async fn analyze_file_data(
            &self,
            _data: &[u8],
            _filename: &str,
        ) -> Result<DetectionResult> {
            Err(anyhow::anyhow!(
                "ML engine not compiled (enable 'ml-engine' feature)"
            ))
        }
    }
}
#[cfg(not(feature = "ml-engine"))]
pub use ml_stub::*;

use crate::models::analysis_result::{
    AnalysisResult, DetectionResult, EngineType, FileMetadata, SeverityLevel, ThreatCategory,
    ThreatVerdict,
};

/// Configuration for the combined analysis engine
#[derive(Debug, Clone)]
pub struct AnalysisEngineConfig {
    pub hash_analyzer: HashAnalyzerConfig,
    pub static_analyzer: StaticAnalyzerConfig,
    pub yara_engine: YaraEngineConfig,
    pub clamav_analyzer: ClamAvAnalyzerConfig,
    pub ml_analyzer: MlAnalyzerConfig,
    pub signature_matcher: SignatureMatcherConfig,
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
            clamav_analyzer: ClamAvAnalyzerConfig::default(),
            ml_analyzer: MlAnalyzerConfig::default(),
            signature_matcher: SignatureMatcherConfig::default(),
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
    pub enable_clamav_analysis: bool,
    pub enable_ml_analysis: bool,
    pub enable_heuristic_analysis: bool,
    pub enable_signature_analysis: bool,
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
            enable_yara_analysis: cfg!(feature = "yara-engine"),
            enable_clamav_analysis: cfg!(feature = "clamav"),
            enable_ml_analysis: cfg!(feature = "ml-engine"),
            enable_heuristic_analysis: true,
            enable_signature_analysis: true,
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
    clamav_analyzer: ClamAvAnalyzer,
    ml_analyzer: MlAnalyzer,
    heuristic_engine: HeuristicEngine,
    signature_matcher: SignatureMatcher,
}

impl AnalysisEngine {
    /// Create a new analysis engine with the given configuration
    pub async fn new(config: AnalysisEngineConfig) -> Result<Self> {
        info!("Initializing analysis engine");

        let hash_analyzer = HashAnalyzer::new(config.hash_analyzer.clone())
            .map_err(|e| anyhow!("Failed to initialize hash analyzer: {e}"))?;
        let static_analyzer = StaticAnalyzer::new(config.static_analyzer.clone());

        let yara_engine = YaraEngine::new(config.yara_engine.clone())
            .map_err(|e| anyhow!("Failed to initialize YARA engine: {e}"))?;

        let clamav_analyzer = ClamAvAnalyzer::new(config.clamav_analyzer.clone());
        let ml_analyzer = MlAnalyzer::new(config.ml_analyzer.clone());
        let heuristic_engine = HeuristicEngine::new();
        let signature_matcher = SignatureMatcher::new(config.signature_matcher.clone())
            .await
            .map_err(|e| anyhow!("Failed to initialize signature matcher: {e}"))?;

        Ok(Self {
            config,
            hash_analyzer,
            static_analyzer,
            yara_engine,
            clamav_analyzer,
            ml_analyzer,
            heuristic_engine,
            signature_matcher,
        })
    }

    /// Perform comprehensive analysis on a file
    pub async fn analyze_file(&mut self, request: FileAnalysisRequest) -> Result<AnalysisResult> {
        let start_time = std::time::Instant::now();

        info!(
            "Starting comprehensive analysis for file: {}",
            request.filename
        );
        debug!("File size: {} bytes", request.file_data.len());

        // Run analysis with timeout
        let analysis_result = timeout(
            Duration::from_secs(self.config.analysis_timeout_seconds),
            self.perform_analysis(&request),
        )
        .await
        .map_err(|_| {
            anyhow!(
                "Analysis timeout after {} seconds",
                self.config.analysis_timeout_seconds
            )
        })??;

        let total_time = start_time.elapsed().as_millis() as u64;

        info!(
            "Analysis completed in {}ms for file: {}",
            total_time, request.filename
        );

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
            let clamav_future = self.run_clamav_analysis(request);
            let ml_future = self.run_ml_analysis(request);
            let heuristic_future = self.run_heuristic_analysis(request);
            let signature_future = self.run_signature_analysis(request);

            let (hash_res, static_res, yara_res, clamav_res, ml_res, heuristic_res, signature_res) = tokio::join!(
                hash_future,
                static_future,
                yara_future,
                clamav_future,
                ml_future,
                heuristic_future,
                signature_future
            );

            // Collect results and errors
            match hash_res {
                Ok(mut dets) => detections.append(&mut dets),
                Err(e) => {
                    warn!("Hash analysis failed: {}", e);
                    analysis_errors.push(format!("Hash: {e}"));
                }
            }
            match static_res {
                Ok(det) => detections.push(det),
                Err(e) => {
                    warn!("Static analysis failed: {}", e);
                    analysis_errors.push(format!("Static: {e}"));
                }
            }
            match yara_res {
                Ok(det) => detections.push(det),
                Err(e) => {
                    warn!("Yara analysis failed: {}", e);
                    analysis_errors.push(format!("Yara: {e}"));
                }
            }
            match clamav_res {
                Ok(det) => detections.push(det),
                Err(e) => {
                    warn!("ClamAV analysis failed: {}", e);
                    analysis_errors.push(format!("ClamAV: {e}"));
                }
            }
            match ml_res {
                Ok(det) => detections.push(det),
                Err(e) => {
                    warn!("ML analysis failed: {}", e);
                    analysis_errors.push(format!("ML: {e}"));
                }
            }
            match heuristic_res {
                Ok(det) => detections.push(det),
                Err(e) => {
                    warn!("Heuristic analysis failed: {}", e);
                    analysis_errors.push(format!("Heuristic: {e}"));
                }
            }
            match signature_res {
                Ok(det) => detections.push(det),
                Err(e) => {
                    warn!("Signature matching failed: {}", e);
                    analysis_errors.push(format!("Signature: {e}"));
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
            if let Ok(det) = self.run_clamav_analysis(request).await {
                detections.push(det);
            }
            if let Ok(det) = self.run_ml_analysis(request).await {
                detections.push(det);
            }
            if let Ok(det) = self.run_heuristic_analysis(request).await {
                detections.push(det);
            }
            if let Ok(det) = self.run_signature_analysis(request).await {
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

    async fn run_hash_analysis(
        &self,
        request: &FileAnalysisRequest,
    ) -> Result<Vec<DetectionResult>> {
        if request.analysis_options.enable_hash_analysis {
            let sha256_hash = {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&request.file_data);
                format!("{:x}", hasher.finalize())
            };
            let hash_info = HashInfo {
                hash_type: HashType::SHA256,
                hash_value: sha256_hash,
                file_size: Some(request.file_data.len() as u64),
                computed_at: chrono::Utc::now(),
            };
            let analysis_result = self
                .hash_analyzer
                .analyze_hash(&hash_info, Some(&request.file_data))
                .await
                .map_err(|e| anyhow!("Hash analysis error: {e}"))?;
            Ok(analysis_result.detections)
        } else {
            Ok(vec![])
        }
    }

    async fn run_static_analysis(&self, request: &FileAnalysisRequest) -> Result<DetectionResult> {
        if request.analysis_options.enable_static_analysis {
            self.static_analyzer
                .analyze(&request.file_data, Some(&request.filename))
                .await
        } else {
            Err(anyhow!("Static analysis disabled"))
        }
    }

    async fn run_yara_analysis(&self, request: &FileAnalysisRequest) -> Result<DetectionResult> {
        if request.analysis_options.enable_yara_analysis {
            self.yara_engine
                .analyze_file_data(&request.file_data, &request.filename)
                .await
        } else {
            Err(anyhow!("Yara analysis disabled"))
        }
    }

    async fn run_clamav_analysis(&self, request: &FileAnalysisRequest) -> Result<DetectionResult> {
        if request.analysis_options.enable_clamav_analysis {
            self.clamav_analyzer
                .scan_file(&request.file_data, &request.filename)
                .await
        } else {
            Err(anyhow!("ClamAV analysis disabled"))
        }
    }

    async fn run_ml_analysis(&self, request: &FileAnalysisRequest) -> Result<DetectionResult> {
        if request.analysis_options.enable_ml_analysis {
            self.ml_analyzer
                .analyze_file_data(&request.file_data, &request.filename)
                .await
        } else {
            Err(anyhow!("ML analysis disabled"))
        }
    }

    async fn run_heuristic_analysis(
        &self,
        request: &FileAnalysisRequest,
    ) -> Result<DetectionResult> {
        if !request.analysis_options.enable_heuristic_analysis {
            return Err(anyhow!("Heuristic analysis disabled"));
        }

        let start = std::time::Instant::now();
        let content = String::from_utf8_lossy(&request.file_data);
        let matches = self
            .heuristic_engine
            .analyze_content(&content, std::path::Path::new(&request.filename))
            .await
            .map_err(|e| anyhow!("Heuristic analysis error: {e}"))?;
        let risk_score = self.heuristic_engine.calculate_risk_score(&matches);

        Ok(DetectionResult {
            detection_id: Uuid::new_v4(),
            engine_name: "Verdyx Heuristic Engine".to_string(),
            engine_version: self.heuristic_engine.version().to_string(),
            engine_type: EngineType::Static,
            verdict: if risk_score >= 50.0 {
                ThreatVerdict::Malicious
            } else if risk_score >= 15.0 {
                ThreatVerdict::Suspicious
            } else {
                ThreatVerdict::Benign
            },
            confidence: (risk_score / 100.0).clamp(0.0, 1.0),
            severity: Self::heuristic_severity(&matches),
            categories: vec![ThreatCategory::Malware],
            metadata: HashMap::from([
                ("match_count".to_string(), serde_json::json!(matches.len())),
                ("risk_score".to_string(), serde_json::json!(risk_score)),
                ("matches".to_string(), serde_json::to_value(&matches)?),
            ]),
            detected_at: Utc::now(),
            processing_time_ms: start.elapsed().as_millis() as u64,
            error_message: None,
        })
    }

    async fn run_signature_analysis(
        &self,
        request: &FileAnalysisRequest,
    ) -> Result<DetectionResult> {
        if !request.analysis_options.enable_signature_analysis {
            return Err(anyhow!("Signature analysis disabled"));
        }

        let start = std::time::Instant::now();
        let matches = self
            .signature_matcher
            .match_bytes(&request.file_data)
            .await
            .map_err(|e| anyhow!("Signature matching error: {e}"))?;
        let confidence = matches.iter().map(|m| m.confidence).fold(0.0_f32, f32::max);

        Ok(DetectionResult {
            detection_id: Uuid::new_v4(),
            engine_name: "Verdyx Signature Matcher".to_string(),
            engine_version: "1.0.0".to_string(),
            engine_type: EngineType::Static,
            verdict: if matches.is_empty() {
                ThreatVerdict::Benign
            } else if confidence >= 0.7 {
                ThreatVerdict::Malicious
            } else {
                ThreatVerdict::Suspicious
            },
            confidence,
            severity: Self::signature_severity(&matches),
            categories: vec![ThreatCategory::Malware],
            metadata: HashMap::from([
                ("match_count".to_string(), serde_json::json!(matches.len())),
                ("matches".to_string(), serde_json::to_value(&matches)?),
            ]),
            detected_at: Utc::now(),
            processing_time_ms: start.elapsed().as_millis() as u64,
            error_message: None,
        })
    }

    fn heuristic_severity(matches: &[HeuristicMatch]) -> SeverityLevel {
        matches
            .iter()
            .map(|m| match m.severity {
                HeuristicSeverity::Critical => SeverityLevel::Critical,
                HeuristicSeverity::High => SeverityLevel::High,
                HeuristicSeverity::Medium => SeverityLevel::Medium,
                HeuristicSeverity::Low => SeverityLevel::Low,
                HeuristicSeverity::Info => SeverityLevel::Info,
            })
            .max()
            .unwrap_or(SeverityLevel::Info)
    }

    fn signature_severity(matches: &[SignatureMatch]) -> SeverityLevel {
        matches
            .iter()
            .map(|m| match m.severity {
                SignatureSeverity::Critical => SeverityLevel::Critical,
                SignatureSeverity::High => SeverityLevel::High,
                SignatureSeverity::Medium => SeverityLevel::Medium,
                SignatureSeverity::Low => SeverityLevel::Low,
                SignatureSeverity::Info => SeverityLevel::Info,
            })
            .max()
            .unwrap_or(SeverityLevel::Info)
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
    pub async fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();

        // Combine stats from all analyzers
        let hash_stats = self.hash_analyzer.get_cache_stats().await;
        for (key, value) in hash_stats {
            stats.insert(format!("hash_{key}"), value.to_string());
        }

        let yara_stats = self.yara_engine.get_stats();
        for (key, value) in yara_stats {
            stats.insert(format!("yara_{key}"), value);
        }

        // Add engine-level stats
        stats.insert(
            "parallel_analysis".to_string(),
            self.config.enable_parallel_analysis.to_string(),
        );
        stats.insert(
            "analysis_timeout".to_string(),
            self.config.analysis_timeout_seconds.to_string(),
        );
        stats.insert(
            "require_all_analyzers".to_string(),
            self.config.require_all_analyzers.to_string(),
        );

        stats
    }

    /// Clear all caches
    pub async fn clear_caches(&mut self) {
        self.hash_analyzer.clear_cache().await;
        info!("All analyzer caches cleared");
    }

    /// Reload YARA rules
    pub fn reload_yara_rules(&mut self) -> Result<()> {
        self.yara_engine
            .reload_rules()
            .map_err(|e| anyhow!("Failed to reload YARA rules: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_analysis_options() {
        let options = AnalysisOptions {
            enable_hash_analysis: false,
            enable_static_analysis: true,
            enable_yara_analysis: true,
            enable_clamav_analysis: false,
            enable_ml_analysis: false,
            enable_heuristic_analysis: true,
            enable_signature_analysis: true,
            priority: AnalysisPriority::High,
            custom_metadata: HashMap::from([("source".to_string(), "unit_test".to_string())]),
        };

        assert!(!options.enable_hash_analysis);
        assert_eq!(options.priority, AnalysisPriority::High);
        assert_eq!(options.custom_metadata.get("source").unwrap(), "unit_test");
    }
}
