use std::collections::HashMap;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};

// Re-export all analyzer modules
pub mod hash_analyzer;
pub mod static_analyzer;
pub mod yara_engine;

// Re-export commonly used types
pub use hash_analyzer::{HashAnalyzer, HashAnalyzerConfig, HashInfo, HashType, HashReputation};
pub use static_analyzer::{StaticAnalyzer, StaticAnalyzerConfig, FileType, PEAnalysis, StringAnalysis, EntropyAnalysis};
pub use yara_engine::{YaraEngine, YaraEngineConfig, YaraMatch, YaraRule, YaraEngineError};

use crate::models::analysis_result::{AnalysisResult, ThreatVerdict, ConfidenceLevel};

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

/// Combined analysis result from all analyzers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedAnalysisResult {
    pub overall_verdict: ThreatVerdict,
    pub overall_confidence: ConfidenceLevel,
    pub overall_score: f64,
    pub hash_analysis: Option<AnalysisResult>,
    pub static_analysis: Option<AnalysisResult>,
    pub yara_analysis: Option<AnalysisResult>,
    pub analysis_summary: String,
    pub metadata: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub total_analysis_time_ms: u64,
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
    pub async fn analyze_file(&mut self, request: FileAnalysisRequest) -> Result<CombinedAnalysisResult> {
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

        Ok(CombinedAnalysisResult {
            total_analysis_time_ms: total_time,
            ..analysis_result
        })
    }

    async fn perform_analysis(&mut self, request: &FileAnalysisRequest) -> Result<CombinedAnalysisResult> {
        let mut hash_result = None;
        let mut static_result = None;
        let mut yara_result = None;
        let mut analysis_errors = Vec::new();

        if self.config.enable_parallel_analysis {
            // Run analyzers in parallel
            let (hash_res, static_res, yara_res) = tokio::join!(
                self.run_hash_analysis(request),
                self.run_static_analysis(request),
                self.run_yara_analysis(request)
            );

            // Collect results and errors
            match hash_res {
                Ok(result) => hash_result = Some(result),
                Err(e) => {
                    warn!("Hash analysis failed: {}", e);
                    analysis_errors.push(format!("Hash: {}", e));
                }
            }

            match static_res {
                Ok(result) => static_result = Some(result),
                Err(e) => {
                    warn!("Static analysis failed: {}", e);
                    analysis_errors.push(format!("Static: {}", e));
                }
            }

            match yara_res {
                Ok(result) => yara_result = Some(result),
                Err(e) => {
                    warn!("YARA analysis failed: {}", e);
                    analysis_errors.push(format!("YARA: {}", e));
                }
            }
        } else {
            // Run analyzers sequentially
            if request.analysis_options.enable_hash_analysis {
                match self.run_hash_analysis(request).await {
                    Ok(result) => hash_result = Some(result),
                    Err(e) => {
                        warn!("Hash analysis failed: {}", e);
                        analysis_errors.push(format!("Hash: {}", e));
                    }
                }
            }

            if request.analysis_options.enable_static_analysis {
                match self.run_static_analysis(request).await {
                    Ok(result) => static_result = Some(result),
                    Err(e) => {
                        warn!("Static analysis failed: {}", e);
                        analysis_errors.push(format!("Static: {}", e));
                    }
                }
            }

            if request.analysis_options.enable_yara_analysis {
                match self.run_yara_analysis(request).await {
                    Ok(result) => yara_result = Some(result),
                    Err(e) => {
                        warn!("YARA analysis failed: {}", e);
                        analysis_errors.push(format!("YARA: {}", e));
                    }
                }
            }
        }

        // Check if we have at least some results
        if hash_result.is_none() && static_result.is_none() && yara_result.is_none() {
            if self.config.require_all_analyzers {
                return Err(anyhow!("All analyzers failed: {}", analysis_errors.join("; ")));
            } else {
                warn!("All analyzers failed, but continuing with empty result");
            }
        }

        // Combine and correlate results
        let combined_result = self.combine_results(
            hash_result,
            static_result,
            yara_result,
            &request.filename,
            analysis_errors
        );

        Ok(combined_result)
    }

    async fn run_hash_analysis(&mut self, request: &FileAnalysisRequest) -> Result<AnalysisResult> {
        if !request.analysis_options.enable_hash_analysis {
            return Err(anyhow!("Hash analysis disabled"));
        }

        debug!("Running hash analysis");

        // Generate hashes if not provided
        let hash_info = if let Some(hashes) = &request.file_hashes {
            if let Some(sha256_hash) = hashes.get(&HashType::SHA256) {
                HashInfo {
                    hash_type: HashType::SHA256,
                    hash_value: sha256_hash.clone(),
                    file_size: Some(request.file_data.len() as u64),
                }
            } else if let Some(sha1_hash) = hashes.get(&HashType::SHA1) {
                HashInfo {
                    hash_type: HashType::SHA1,
                    hash_value: sha1_hash.clone(),
                    file_size: Some(request.file_data.len() as u64),
                }
            } else if let Some(md5_hash) = hashes.get(&HashType::MD5) {
                HashInfo {
                    hash_type: HashType::MD5,
                    hash_value: md5_hash.clone(),
                    file_size: Some(request.file_data.len() as u64),
                }
            } else {
                return Err(anyhow!("No supported hash types provided"));
            }
        } else {
            // Generate SHA256 hash
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&request.file_data);
            let hash_result = hasher.finalize();
            
            HashInfo {
                hash_type: HashType::SHA256,
                hash_value: format!("{:x}", hash_result),
                file_size: Some(request.file_data.len() as u64),
            }
        };

        self.hash_analyzer.analyze_hash(&hash_info, Some(&request.file_data)).await
    }

    async fn run_static_analysis(&self, request: &FileAnalysisRequest) -> Result<AnalysisResult> {
        if !request.analysis_options.enable_static_analysis {
            return Err(anyhow!("Static analysis disabled"));
        }

        debug!("Running static analysis");
        self.static_analyzer.analyze(&request.file_data, Some(&request.filename)).await
    }

    async fn run_yara_analysis(&self, request: &FileAnalysisRequest) -> Result<AnalysisResult> {
        if !request.analysis_options.enable_yara_analysis {
            return Err(anyhow!("YARA analysis disabled"));
        }

        debug!("Running YARA analysis");
        
        // Convert YaraEngineError to anyhow::Error
        self.yara_engine.analyze_bytes(&request.file_data, &request.filename)
            .await
            .map_err(|e| anyhow!("YARA analysis error: {}", e))?
            .try_into() // Convert the YARA AnalysisResult to our common AnalysisResult
            .map_err(|e| anyhow!("Failed to convert YARA result: {:?}", e))
    }

    fn combine_results(
        &self,
        hash_result: Option<AnalysisResult>,
        static_result: Option<AnalysisResult>,
        yara_result: Option<AnalysisResult>,
        filename: &str,
        errors: Vec<String>
    ) -> CombinedAnalysisResult {
        debug!("Combining analysis results");

        let results = vec![&hash_result, &static_result, &yara_result]
            .into_iter()
            .filter_map(|r| r.as_ref())
            .collect::<Vec<_>>();

        if results.is_empty() {
            return CombinedAnalysisResult {
                overall_verdict: ThreatVerdict::Unknown,
                overall_confidence: ConfidenceLevel::Low,
                overall_score: 0.0,
                hash_analysis: hash_result,
                static_analysis: static_result,
                yara_analysis: yara_result,
                analysis_summary: format!("Analysis failed for {}: {}", filename, errors.join("; ")),
                metadata: HashMap::from([
                    ("errors".to_string(), errors.join("; ")),
                ]),
                timestamp: chrono::Utc::now(),
                total_analysis_time_ms: 0,
            };
        }

        // Aggregate verdicts using weighted scoring
        let hash_weight = 0.4;
        let static_weight = 0.3;
        let yara_weight = 0.3;

        let mut weighted_score = 0.0;
        let mut total_weight = 0.0;

        if let Some(ref result) = hash_result {
            weighted_score += result.score * hash_weight;
            total_weight += hash_weight;
        }

        if let Some(ref result) = static_result {
            weighted_score += result.score * static_weight;
            total_weight += static_weight;
        }

        if let Some(ref result) = yara_result {
            weighted_score += result.score * yara_weight;
            total_weight += yara_weight;
        }

        let overall_score = if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            0.0
        };

        // Determine overall verdict
        let malicious_count = results.iter().filter(|r| r.verdict == ThreatVerdict::Malicious).count();
        let suspicious_count = results.iter().filter(|r| r.verdict == ThreatVerdict::Suspicious).count();

        let overall_verdict = if malicious_count > 0 {
            ThreatVerdict::Malicious
        } else if suspicious_count > 0 {
            ThreatVerdict::Suspicious
        } else if results.iter().any(|r| r.verdict == ThreatVerdict::Benign) {
            ThreatVerdict::Benign
        } else {
            ThreatVerdict::Unknown
        };

        // Determine overall confidence
        let avg_confidence = results.len() as f64 / 3.0; // Percentage of successful analyzers
        let overall_confidence = match avg_confidence {
            c if c >= 0.8 => ConfidenceLevel::High,
            c if c >= 0.5 => ConfidenceLevel::Medium,
            _ => ConfidenceLevel::Low,
        };

        // Generate summary
        let analysis_summary = self.generate_analysis_summary(
            &overall_verdict,
            &results,
            filename,
            &errors
        );

        // Combine metadata
        let mut combined_metadata = HashMap::new();
        combined_metadata.insert("analyzers_run".to_string(), results.len().to_string());
        combined_metadata.insert("overall_score".to_string(), format!("{:.3}", overall_score));
        
        if !errors.is_empty() {
            combined_metadata.insert("analyzer_errors".to_string(), errors.join("; "));
        }

        // Add individual analyzer metadata
        if let Some(ref result) = hash_result {
            for (key, value) in &result.metadata {
                combined_metadata.insert(format!("hash_{}", key), value.clone());
            }
        }

        if let Some(ref result) = static_result {
            for (key, value) in &result.metadata {
                combined_metadata.insert(format!("static_{}", key), value.clone());
            }
        }

        if let Some(ref result) = yara_result {
            for (key, value) in &result.metadata {
                combined_metadata.insert(format!("yara_{}", key), value.clone());
            }
        }

        CombinedAnalysisResult {
            overall_verdict,
            overall_confidence,
            overall_score,
            hash_analysis: hash_result,
            static_analysis: static_result,
            yara_analysis: yara_result,
            analysis_summary,
            metadata: combined_metadata,
            timestamp: chrono::Utc::now(),
            total_analysis_time_ms: 0, // Will be set by caller
        }
    }

    fn generate_analysis_summary(
        &self,
        verdict: &ThreatVerdict,
        results: &[&AnalysisResult],
        filename: &str,
        errors: &[String]
    ) -> String {
        let mut summary = format!("Analysis of '{}': ", filename);

        match verdict {
            ThreatVerdict::Malicious => {
                summary.push_str("THREAT DETECTED - File identified as malicious by ");
            },
            ThreatVerdict::Suspicious => {
                summary.push_str("SUSPICIOUS - File shows potentially malicious characteristics detected by ");
            },
            ThreatVerdict::Benign => {
                summary.push_str("CLEAN - File appears benign based on analysis by ");
            },
            ThreatVerdict::Unknown => {
                summary.push_str("UNKNOWN - Unable to determine threat level from analysis by ");
            },
        }

        let analyzer_names: Vec<String> = results.iter()
            .map(|r| r.analyzer_name.clone())
            .collect();
        
        summary.push_str(&analyzer_names.join(", "));

        if !errors.is_empty() {
            summary.push_str(&format!(". {} analyzer(s) failed.", errors.len()));
        }

        summary
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

// Helper trait to convert between different AnalysisResult types
trait TryIntoCommonResult {
    fn try_into(self) -> Result<AnalysisResult>;
}

// Implementation for YARA's AnalysisResult (assuming it has different fields)
impl TryIntoCommonResult for yara_engine::AnalysisResult {
    fn try_into(self) -> Result<AnalysisResult> {
        // Convert YARA's ThreatLevel to our ThreatVerdict
        let verdict = match self.threat_level {
            yara_engine::ThreatLevel::Clean => ThreatVerdict::Benign,
            yara_engine::ThreatLevel::Low => ThreatVerdict::Suspicious,
            yara_engine::ThreatLevel::Medium => ThreatVerdict::Suspicious,
            yara_engine::ThreatLevel::High => ThreatVerdict::Malicious,
            yara_engine::ThreatLevel::Critical => ThreatVerdict::Malicious,
        };

        let confidence = if self.confidence > 0.8 {
            ConfidenceLevel::High
        } else if self.confidence > 0.5 {
            ConfidenceLevel::Medium
        } else {
            ConfidenceLevel::Low
        };

        let mut metadata = HashMap::new();
        metadata.insert("file_hash".to_string(), self.file_hash);
        metadata.insert("scan_time".to_string(), self.scan_time.to_string());
        metadata.insert("matches_count".to_string(), self.matches.len().to_string());
        
        // Add YARA-specific metadata
        for (key, value) in self.metadata {
            metadata.insert(key, value);
        }

        let details = if self.is_malicious {
            format!("YARA detected {} rule matches indicating malicious content", self.matches.len())
        } else {
            "YARA scan completed with no malicious indicators".to_string()
        };

        let score = if self.is_malicious { 
            self.confidence as f64 
        } else { 
            0.0 
        };

        Ok(AnalysisResult {
            verdict,
            confidence,
            score,
            details,
            metadata,
            timestamp: chrono::DateTime::from_timestamp(self.scan_time as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now()),
            analyzer_name: self.engine_name,
            analyzer_version: self.version,
        })
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
        assert!(!analysis.analysis_summary.is_empty());
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