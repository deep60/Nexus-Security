use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use anyhow::{Result, Context};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::models::{AnalysisResult, ThreatIndicator, ScanJob};

// ONNX Runtime bindings (you'll need to add ort crate to Cargo.toml)
use ort::{Environment, ExecutionProvider, Session, SessionBuilder, Value};

/// ML Analysis configuration
#[derive(Debug, Clone, Deserialize)]
pub struct MlAnalyzerConfig {
    pub threat_classifier_model_path: String,
    pub anomaly_detector_model_path: String,
    pub feature_extractor_config: FeatureExtractorConfig,
    pub confidence_threshold: f32,
    pub batch_size: usize,
    pub max_file_size: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeatureExtractorConfig {
    pub pe_features: bool,
    pub entropy_features: bool,
    pub ngram_features: bool,
    pub network_features: bool,
    pub api_call_features: bool,
    pub static_analysis_features: bool,
}

/// Feature vector for ML models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub file_features: FileFeatures,
    pub static_features: StaticFeatures,
    pub behavioral_features: Option<BehavioralFeatures>,
    pub network_features: Option<NetworkFeatures>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFeatures {
    pub file_size: f32,
    pub entropy: f32,
    pub file_type: u32,
    pub sections_count: f32,
    pub imports_count: f32,
    pub exports_count: f32,
    pub pe_characteristics: Vec<f32>,
    pub string_features: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticFeatures {
    pub suspicious_strings: f32,
    pub api_calls: Vec<f32>,
    pub code_complexity: f32,
    pub packer_entropy: f32,
    pub section_permissions: Vec<f32>,
    pub unusual_imports: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralFeatures {
    pub file_operations: f32,
    pub registry_operations: f32,
    pub network_connections: f32,
    pub process_creation: f32,
    pub memory_allocation: f32,
    pub mutex_creation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkFeatures {
    pub dns_requests: f32,
    pub http_requests: f32,
    pub suspicious_domains: f32,
    pub port_scanning: f32,
    pub data_exfiltration: f32,
}

/// ML Model wrapper for threat classification
#[derive(Debug)]
pub struct ThreatClassifier {
    session: Session,
    input_name: String,
    output_names: Vec<String>,
    labels: Vec<String>,
}

/// ML Model wrapper for anomaly detection
#[derive(Debug)]
pub struct AnomalyDetector {
    session: Session,
    input_name: String,
    output_name: String,
    threshold: f32,
}

/// ML-based threat analyzer
pub struct MlAnalyzer {
    config: MlAnalyzerConfig,
    threat_classifier: Arc<ThreatClassifier>,
    anomaly_detector: Arc<AnomalyDetector>,
    feature_extractor: FeatureExtractor,
    environment: Arc<Environment>,
}

/// Feature extraction engine
pub struct FeatureExtractor {
    config: FeatureExtractorConfig,
}

#[async_trait]
pub trait Analyzer {
    async fn analyze(&self, job: &ScanJob) -> Result<AnalysisResult>;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
}

impl MlAnalyzer {
    /// Create new ML analyzer instance
    pub async fn new(config: MlAnalyzerConfig) -> Result<Self> {
        info!("Initializing ML Analyzer with models...");
        
        // Initialize ONNX Runtime environment
        let environment = Arc::new(
            Environment::builder()
                .with_name("nexus-ml-analyzer")
                .build()?
        );

        // Load threat classification model
        let threat_classifier = Self::load_threat_classifier(
            &environment,
            &config.threat_classifier_model_path
        ).await?;

        // Load anomaly detection model
        let anomaly_detector = Self::load_anomaly_detector(
            &environment,
            &config.anomaly_detector_model_path,
            config.confidence_threshold
        ).await?;

        let feature_extractor = FeatureExtractor::new(config.feature_extractor_config.clone());

        Ok(Self {
            config,
            threat_classifier: Arc::new(threat_classifier),
            anomaly_detector: Arc::new(anomaly_detector),
            feature_extractor,
            environment,
        })
    }

    /// Load threat classification model
    async fn load_threat_classifier(
        environment: &Environment,
        model_path: &str,
    ) -> Result<ThreatClassifier> {
        info!("Loading threat classifier model from: {}", model_path);
        
        let session = SessionBuilder::new(environment)?
            .with_execution_providers([ExecutionProvider::CPU(Default::default())])?
            .with_model_from_file(model_path)?;

        // Get input/output metadata
        let input_name = session.inputs[0].name.clone();
        let output_names: Vec<String> = session.outputs.iter()
            .map(|output| output.name.clone())
            .collect();

        // Load class labels (you might want to load this from a separate file)
        let labels = vec![
            "benign".to_string(),
            "malware".to_string(),
            "trojan".to_string(),
            "ransomware".to_string(),
            "adware".to_string(),
            "spyware".to_string(),
            "rootkit".to_string(),
            "backdoor".to_string(),
            "worm".to_string(),
            "virus".to_string(),
        ];

        Ok(ThreatClassifier {
            session,
            input_name,
            output_names,
            labels,
        })
    }

    /// Load anomaly detection model
    async fn load_anomaly_detector(
        environment: &Environment,
        model_path: &str,
        threshold: f32,
    ) -> Result<AnomalyDetector> {
        info!("Loading anomaly detector model from: {}", model_path);
        
        let session = SessionBuilder::new(environment)?
            .with_execution_providers([ExecutionProvider::CPU(Default::default())])?
            .with_model_from_file(model_path)?;

        let input_name = session.inputs[0].name.clone();
        let output_name = session.outputs[0].name.clone();

        Ok(AnomalyDetector {
            session,
            input_name,
            output_name,
            threshold,
        })
    }

    /// Extract features from file
    async fn extract_features(&self, file_path: &Path) -> Result<FeatureVector> {
        debug!("Extracting features from: {:?}", file_path);
        
        let file_content = fs::read(file_path).await
            .context("Failed to read file for feature extraction")?;

        self.feature_extractor.extract_features(&file_content).await
    }

    /// Run threat classification
    async fn classify_threat(&self, features: &FeatureVector) -> Result<ClassificationResult> {
        let input_tensor = self.prepare_classification_input(features)?;
        
        let outputs = self.threat_classifier.session.run(vec![
            Value::from_array(self.threat_classifier.session.allocator(), &input_tensor)?
        ])?;

        let probabilities: Vec<f32> = outputs[0].extract_tensor()?
            .view()
            .iter()
            .copied()
            .collect();

        // Find the class with highest probability
        let (predicted_class_idx, confidence) = probabilities
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();

        let predicted_class = self.threat_classifier.labels[predicted_class_idx].clone();
        
        Ok(ClassificationResult {
            predicted_class,
            confidence: *confidence,
            probabilities: probabilities.into_iter()
                .zip(self.threat_classifier.labels.iter())
                .map(|(prob, label)| (label.clone(), prob))
                .collect(),
        })
    }

    /// Run anomaly detection
    async fn detect_anomaly(&self, features: &FeatureVector) -> Result<AnomalyResult> {
        let input_tensor = self.prepare_anomaly_input(features)?;
        
        let outputs = self.anomaly_detector.session.run(vec![
            Value::from_array(self.anomaly_detector.session.allocator(), &input_tensor)?
        ])?;

        let anomaly_score: f32 = outputs[0].extract_tensor()?
            .view()
            .iter()
            .next()
            .copied()
            .unwrap_or(0.0);

        let is_anomaly = anomaly_score > self.anomaly_detector.threshold;

        Ok(AnomalyResult {
            anomaly_score,
            is_anomaly,
            threshold: self.anomaly_detector.threshold,
        })
    }

    /// Prepare input tensor for classification model
    fn prepare_classification_input(&self, features: &FeatureVector) -> Result<ndarray::Array2<f32>> {
        // Convert features to flat array (adjust based on your model's expected input)
        let mut feature_vec = Vec::new();
        
        // File features
        feature_vec.push(features.file_features.file_size);
        feature_vec.push(features.file_features.entropy);
        feature_vec.push(features.file_features.file_type as f32);
        feature_vec.push(features.file_features.sections_count);
        feature_vec.push(features.file_features.imports_count);
        feature_vec.push(features.file_features.exports_count);
        feature_vec.extend(&features.file_features.pe_characteristics);
        feature_vec.extend(&features.file_features.string_features);

        // Static features
        feature_vec.push(features.static_features.suspicious_strings);
        feature_vec.extend(&features.static_features.api_calls);
        feature_vec.push(features.static_features.code_complexity);
        feature_vec.push(features.static_features.packer_entropy);
        feature_vec.extend(&features.static_features.section_permissions);
        feature_vec.push(features.static_features.unusual_imports);

        // Pad or truncate to expected input size
        const EXPECTED_FEATURE_SIZE: usize = 512; // Adjust based on your model
        feature_vec.resize(EXPECTED_FEATURE_SIZE, 0.0);

        Ok(ndarray::Array2::from_shape_vec((1, EXPECTED_FEATURE_SIZE), feature_vec)?)
    }

    /// Prepare input tensor for anomaly detection model
    fn prepare_anomaly_input(&self, features: &FeatureVector) -> Result<ndarray::Array2<f32>> {
        // Similar to classification but might use different feature subset
        self.prepare_classification_input(features)
    }
}

#[async_trait]
impl Analyzer for MlAnalyzer {
    async fn analyze(&self, job: &ScanJob) -> Result<AnalysisResult> {
        info!("Starting ML analysis for job: {}", job.id);

        let file_path = Path::new(&job.file_path);
        
        // Check file size limits
        let metadata = fs::metadata(file_path).await?;
        if metadata.len() as usize > self.config.max_file_size {
            return Ok(AnalysisResult {
                id: Uuid::new_v4(),
                job_id: job.id,
                analyzer_name: self.name().to_string(),
                analyzer_version: self.version().to_string(),
                is_malicious: false,
                confidence_score: 0.0,
                threat_indicators: vec![],
                metadata: HashMap::from([
                    ("error".to_string(), "File too large for ML analysis".to_string())
                ]),
                analysis_duration_ms: 0,
                created_at: chrono::Utc::now(),
            });
        }

        let start_time = std::time::Instant::now();

        // Extract features
        let features = match self.extract_features(file_path).await {
            Ok(features) => features,
            Err(e) => {
                error!("Feature extraction failed: {}", e);
                return Ok(AnalysisResult {
                    id: Uuid::new_v4(),
                    job_id: job.id,
                    analyzer_name: self.name().to_string(),
                    analyzer_version: self.version().to_string(),
                    is_malicious: false,
                    confidence_score: 0.0,
                    threat_indicators: vec![],
                    metadata: HashMap::from([
                        ("error".to_string(), format!("Feature extraction failed: {}", e))
                    ]),
                    analysis_duration_ms: start_time.elapsed().as_millis() as u64,
                    created_at: chrono::Utc::now(),
                });
            }
        };

        // Run classification
        let classification_result = self.classify_threat(&features).await?;
        
        // Run anomaly detection
        let anomaly_result = self.detect_anomaly(&features).await?;

        // Combine results
        let is_malicious = classification_result.predicted_class != "benign" || anomaly_result.is_anomaly;
        let confidence_score = if anomaly_result.is_anomaly {
            (classification_result.confidence + anomaly_result.anomaly_score) / 2.0
        } else {
            classification_result.confidence
        };

        // Generate threat indicators
        let mut threat_indicators = Vec::new();
        
        if is_malicious {
            if classification_result.predicted_class != "benign" {
                threat_indicators.push(ThreatIndicator {
                    id: Uuid::new_v4(),
                    indicator_type: "ml_classification".to_string(),
                    value: classification_result.predicted_class.clone(),
                    description: format!(
                        "ML model classified as {} with {:.2}% confidence",
                        classification_result.predicted_class,
                        classification_result.confidence * 100.0
                    ),
                    severity: Self::map_severity(&classification_result.predicted_class),
                    confidence: classification_result.confidence,
                    source: "ml_classifier".to_string(),
                    created_at: chrono::Utc::now(),
                });
            }

            if anomaly_result.is_anomaly {
                threat_indicators.push(ThreatIndicator {
                    id: Uuid::new_v4(),
                    indicator_type: "anomaly_detection".to_string(),
                    value: format!("anomaly_score_{:.3}", anomaly_result.anomaly_score),
                    description: format!(
                        "Anomaly detected with score {:.3} (threshold: {:.3})",
                        anomaly_result.anomaly_score,
                        anomaly_result.threshold
                    ),
                    severity: if anomaly_result.anomaly_score > 0.8 { "high".to_string() } 
                             else if anomaly_result.anomaly_score > 0.6 { "medium".to_string() }
                             else { "low".to_string() },
                    confidence: anomaly_result.anomaly_score,
                    source: "anomaly_detector".to_string(),
                    created_at: chrono::Utc::now(),
                });
            }
        }

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("classification_result".to_string(), 
                        serde_json::to_string(&classification_result)?);
        metadata.insert("anomaly_result".to_string(), 
                        serde_json::to_string(&anomaly_result)?);
        metadata.insert("feature_count".to_string(), "512".to_string()); // Adjust as needed

        let analysis_duration = start_time.elapsed().as_millis() as u64;
        
        info!("ML analysis completed for job {}: malicious={}, confidence={:.3}, duration={}ms",
              job.id, is_malicious, confidence_score, analysis_duration);

        Ok(AnalysisResult {
            id: Uuid::new_v4(),
            job_id: job.id,
            analyzer_name: self.name().to_string(),
            analyzer_version: self.version().to_string(),
            is_malicious,
            confidence_score,
            threat_indicators,
            metadata,
            analysis_duration_ms: analysis_duration,
            created_at: chrono::Utc::now(),
        })
    }

    fn name(&self) -> &str {
        "ml_analyzer"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
}

impl MlAnalyzer {
    fn map_severity(threat_class: &str) -> String {
        match threat_class {
            "ransomware" | "rootkit" | "backdoor" => "high".to_string(),
            "trojan" | "spyware" | "worm" | "virus" => "medium".to_string(),
            "adware" | "malware" => "low".to_string(),
            _ => "info".to_string(),
        }
    }
}

impl FeatureExtractor {
    pub fn new(config: FeatureExtractorConfig) -> Self {
        Self { config }
    }

    pub async fn extract_features(&self, file_content: &[u8]) -> Result<FeatureVector> {
        debug!("Extracting features from {} bytes", file_content.len());

        let file_features = self.extract_file_features(file_content)?;
        let static_features = self.extract_static_features(file_content)?;
        
        // Behavioral and network features would typically come from dynamic analysis
        // For now, we'll set them as None since this is static analysis
        
        Ok(FeatureVector {
            file_features,
            static_features,
            behavioral_features: None,
            network_features: None,
        })
    }

    fn extract_file_features(&self, content: &[u8]) -> Result<FileFeatures> {
        let file_size = content.len() as f32;
        let entropy = self.calculate_entropy(content);
        let file_type = self.detect_file_type(content);
        
        // PE-specific features (simplified)
        let (sections_count, imports_count, exports_count, pe_characteristics) = 
            if self.is_pe_file(content) {
                self.extract_pe_features(content)?
            } else {
                (0.0, 0.0, 0.0, vec![0.0; 16]) // Pad with zeros for non-PE files
            };

        let string_features = self.extract_string_features(content)?;

        Ok(FileFeatures {
            file_size,
            entropy,
            file_type,
            sections_count,
            imports_count,
            exports_count,
            pe_characteristics,
            string_features,
        })
    }

    fn extract_static_features(&self, content: &[u8]) -> Result<StaticFeatures> {
        let suspicious_strings = self.count_suspicious_strings(content);
        let api_calls = self.extract_api_calls(content);
        let code_complexity = self.calculate_code_complexity(content);
        let packer_entropy = self.calculate_packer_entropy(content);
        let section_permissions = self.analyze_section_permissions(content);
        let unusual_imports = self.count_unusual_imports(content);

        Ok(StaticFeatures {
            suspicious_strings,
            api_calls,
            code_complexity,
            packer_entropy,
            section_permissions,
            unusual_imports,
        })
    }

    fn calculate_entropy(&self, content: &[u8]) -> f32 {
        let mut counts = [0u32; 256];
        for &byte in content {
            counts[byte as usize] += 1;
        }

        let len = content.len() as f32;
        let mut entropy = 0.0f32;

        for &count in &counts {
            if count > 0 {
                let p = count as f32 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    fn detect_file_type(&self, content: &[u8]) -> u32 {
        if content.len() < 4 {
            return 0; // Unknown
        }

        match &content[0..4] {
            b"MZ\x90\x00" | [0x4D, 0x5A, _, _] => 1, // PE
            b"\x7fELF" => 2, // ELF
            [0xCA, 0xFE, 0xBA, 0xBE] => 3, // Mach-O
            b"PK\x03\x04" => 4, // ZIP
            b"\x50\x4B\x03\x04" => 5, // JAR
            _ => 0, // Unknown
        }
    }

    fn is_pe_file(&self, content: &[u8]) -> bool {
        content.len() >= 2 && content[0] == 0x4D && content[1] == 0x5A
    }

    fn extract_pe_features(&self, content: &[u8]) -> Result<(f32, f32, f32, Vec<f32>)> {
        // Simplified PE parsing - in reality, you'd want to use a proper PE parser
        // This is just a placeholder implementation
        
        let sections_count = 5.0; // Placeholder
        let imports_count = 20.0; // Placeholder
        let exports_count = 2.0; // Placeholder
        let pe_characteristics = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 
                                     0.9, 1.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6]; // 16 features

        Ok((sections_count, imports_count, exports_count, pe_characteristics))
    }

    fn extract_string_features(&self, content: &[u8]) -> Result<Vec<f32>> {
        // Extract printable strings and analyze them
        let mut string_features = vec![0.0; 32]; // 32 string-based features
        
        // Count different types of strings
        let printable_strings = self.extract_printable_strings(content);
        string_features[0] = printable_strings.len() as f32;
        
        // Average string length
        if !printable_strings.is_empty() {
            string_features[1] = printable_strings.iter()
                .map(|s| s.len())
                .sum::<usize>() as f32 / printable_strings.len() as f32;
        }

        // Count suspicious keywords
        let suspicious_keywords = [
            "CreateProcess", "WriteFile", "RegSetValue", "GetProcAddress",
            "VirtualAlloc", "CreateThread", "URLDownloadToFile", "WinExec",
        ];
        
        for (i, keyword) in suspicious_keywords.iter().enumerate() {
            if i < 30 { // Leave room for other features
                string_features[i + 2] = printable_strings.iter()
                    .filter(|s| s.contains(keyword))
                    .count() as f32;
            }
        }

        Ok(string_features)
    }

    fn extract_printable_strings(&self, content: &[u8]) -> Vec<String> {
        let mut strings = Vec::new();
        let mut current_string = Vec::new();
        
        for &byte in content {
            if byte.is_ascii_graphic() || byte == b' ' {
                current_string.push(byte);
            } else {
                if current_string.len() >= 4 { // Minimum string length
                    if let Ok(s) = String::from_utf8(current_string.clone()) {
                        strings.push(s);
                    }
                }
                current_string.clear();
            }
        }
        
        // Don't forget the last string
        if current_string.len() >= 4 {
            if let Ok(s) = String::from_utf8(current_string) {
                strings.push(s);
            }
        }
        
        strings
    }

    fn count_suspicious_strings(&self, content: &[u8]) -> f32 {
        let strings = self.extract_printable_strings(content);
        let suspicious_patterns = [
            "cmd.exe", "powershell", "regedit", "taskkill", "net user",
            "bitcoin", "wallet", "ransom", "encrypt", "decrypt",
        ];
        
        strings.iter()
            .filter(|s| {
                suspicious_patterns.iter()
                    .any(|pattern| s.to_lowercase().contains(&pattern.to_lowercase()))
            })
            .count() as f32
    }

    fn extract_api_calls(&self, _content: &[u8]) -> Vec<f32> {
        // Placeholder for API call extraction
        // In reality, this would parse the import table
        vec![0.0; 64] // 64 common API calls
    }

    fn calculate_code_complexity(&self, content: &[u8]) -> f32 {
        // Simple complexity metric based on instruction diversity
        if content.is_empty() {
            return 0.0;
        }
        
        let mut unique_bytes = std::collections::HashSet::new();
        for &byte in content {
            unique_bytes.insert(byte);
        }
        
        unique_bytes.len() as f32 / 256.0
    }

    fn calculate_packer_entropy(&self, content: &[u8]) -> f32 {
        // Calculate entropy of first 1024 bytes (typical for packed files)
        let sample_size = std::cmp::min(1024, content.len());
        self.calculate_entropy(&content[..sample_size])
    }

    fn analyze_section_permissions(&self, _content: &[u8]) -> Vec<f32> {
        // Placeholder for section permission analysis
        vec![0.0; 8] // 8 different permission combinations
    }

    fn count_unusual_imports(&self, _content: &[u8]) -> f32 {
        // Placeholder for unusual import analysis
        0.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ClassificationResult {
    predicted_class: String,
    confidence: f32,
    probabilities: HashMap<String, f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnomalyResult {
    anomaly_score: f32,
    is_anomaly: bool,
    threshold: f32,
}