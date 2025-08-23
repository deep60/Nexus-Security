use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use md5::{Md5, Digest as Md5Digest};
use sha1::{Sha1, Digest as Sha1Digest};
use tokio::time::timeout;
use reqwest::Client;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};

use crate::models::analysis_result::{AnalysisResult, ThreatVerdict, SeverityLevel, FileMetadata, AnalysisStatus, DetectionResult, EngineType};

/// Supported hash algorithms for analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HashType {
    MD5,
    SHA1,
    SHA256,
}

impl std::fmt::Display for HashType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashType::MD5 => write!(f, "MD5"),
            HashType::SHA1 => write!(f, "SHA1"),
            HashType::SHA256 => write!(f, "SHA256"),
        }
    }
}

/// Hash information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashInfo {
    pub hash_type: HashType,
    pub hash_value: String,
    pub file_size: Option<u64>,
}

/// Reputation data from external sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashReputation {
    pub source: String,
    pub verdict: ThreatVerdict,
    pub confidence: f32,
    pub first_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub detection_names: Vec<String>,
    pub threat_types: Vec<String>,
}

/// VirusTotal API response structure (simplified)
#[derive(Debug, Deserialize)]
struct VirusTotalResponse {
    data: VirusTotalData,
}

#[derive(Debug, Deserialize)]
struct VirusTotalData {
    attributes: VirusTotalAttributes,
}

#[derive(Debug, Deserialize)]
struct VirusTotalAttributes {
    last_analysis_stats: VirusTotalStats,
    last_analysis_results: Option<HashMap<String, VirusTotalEngine>>,
    first_submission_date: Option<i64>,
    last_submission_date: Option<i64>,
    names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct VirusTotalStats {
    malicious: u32,
    suspicious: u32,
    undetected: u32,
    harmless: u32,
}

#[derive(Debug, Deserialize)]
struct VirusTotalEngine {
    category: String,
    engine_name: String,
    engine_version: Option<String>,
    result: Option<String>,
    method: Option<String>,
}

/// Configuration for hash analyzer
#[derive(Debug, Clone)]
pub struct HashAnalyzerConfig {
    pub virustotal_api_key: Option<String>,
    pub malwarebazaar_enabled: bool,
    pub local_cache_enabled: bool,
    pub timeout_seconds: u64,
    pub rate_limit_per_minute: u32,
}

impl Default for HashAnalyzerConfig {
    fn default() -> Self {
        Self {
            virustotal_api_key: None,
            malwarebazaar_enabled: true,
            local_cache_enabled: true,
            timeout_seconds: 30,
            rate_limit_per_minute: 60,
        }
    }
}

/// Hash-based threat analyzer
pub struct HashAnalyzer {
    config: HashAnalyzerConfig,
    http_client: Client,
    local_cache: RwLock<HashMap<String, HashReputation>>,
}

impl HashAnalyzer {
    /// Create a new hash analyzer instance
    pub fn new(config: HashAnalyzerConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent("Nexus-Security/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            local_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Analyze a file by its hash values
    pub async fn analyze_hash(&self, hash_info: &HashInfo, file_data: Option<&[u8]>) -> Result<AnalysisResult> {
        info!("Starting hash analysis for {} hash: {}", 
              hash_info.hash_type, hash_info.hash_value);

        // Validate hash format
        if !self.is_valid_hash(&hash_info.hash_value, &hash_info.hash_type) {
            return Err(anyhow!("Invalid hash format for {:?}", hash_info.hash_type));
        }

        // Check local cache first
        if self.config.local_cache_enabled {
            if let Ok(cache) = self.local_cache.read() {
                if let Some(cached_result) = cache.get(&hash_info.hash_value) {
                    debug!("Found cached result for hash: {}", hash_info.hash_value);
                    return Ok(self.create_analysis_result(hash_info, vec![cached_result.clone()]));
                }
            }
        }

        // Generate additional hashes if file data is provided
        let mut hash_variants = vec![hash_info.clone()];
        if let Some(data) = file_data {
            hash_variants.extend(self.generate_all_hashes(data));
        }

        // Query multiple threat intelligence sources
        let mut reputations = Vec::new();
        
        // Query VirusTotal
        if let Some(ref api_key) = self.config.virustotal_api_key {
            match self.query_virustotal(&hash_info.hash_value, api_key).await {
                Ok(rep) => reputations.push(rep),
                Err(e) => warn!("VirusTotal query failed: {}", e),
            }
        }

        // Query MalwareBazaar
        if self.config.malwarebazaar_enabled {
            match self.query_malwarebazaar(&hash_info.hash_value).await {
                Ok(rep) => reputations.push(rep),
                Err(e) => warn!("MalwareBazaar query failed: {}", e),
            }
        }

        // Check against local threat database
        if let Ok(local_rep) = self.query_local_database(&hash_info.hash_value).await {
            reputations.push(local_rep);
        }

        // Cache the results
        if self.config.local_cache_enabled && !reputations.is_empty() {
            if let Ok(mut cache) = self.local_cache.write() {
                for rep in &reputations {
                    cache.insert(hash_info.hash_value.clone(), rep.clone());
                }
            }
        }

        // Create final analysis result
        Ok(self.create_analysis_result(hash_info, reputations))
    }

    /// Validate hash format based on type
    fn is_valid_hash(&self, hash: &str, hash_type: &HashType) -> bool {
        let expected_len = match hash_type {
            HashType::MD5 => 32,
            HashType::SHA1 => 40,
            HashType::SHA256 => 64,
        };

        hash.len() == expected_len && hash.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Generate all hash types for given file data
    fn generate_all_hashes(&self, data: &[u8]) -> Vec<HashInfo> {
        let mut hashes = Vec::new();

        // MD5
        let mut hasher = Md5::new();
        hasher.update(data);
        let md5_hash = format!("{:x}", hasher.finalize());
        hashes.push(HashInfo {
            hash_type: HashType::MD5,
            hash_value: md5_hash,
            file_size: Some(data.len() as u64),
        });

        // SHA1
        let mut hasher = Sha1::new();
        hasher.update(data);
        let sha1_hash = format!("{:x}", hasher.finalize());
        hashes.push(HashInfo {
            hash_type: HashType::SHA1,
            hash_value: sha1_hash,
            file_size: Some(data.len() as u64),
        });

        // SHA256
        let mut hasher = Sha256::new();
        hasher.update(data);
        let sha256_hash = format!("{:x}", hasher.finalize());
        hashes.push(HashInfo {
            hash_type: HashType::SHA256,
            hash_value: sha256_hash,
            file_size: Some(data.len() as u64),
        });

        hashes
    }

    // Query VirusTotal API for hash reputation
    async fn query_virustotal(&self, hash: &str, api_key: &str) -> Result<HashReputation> {
        let url = format!("https://www.virustotal.com/api/v3/files/{}", hash);
        
        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.http_client
                .get(&url)
                .header("x-apikey", api_key)
                .send()
        ).await??;

        if response.status().is_success() {
            let vt_response: VirusTotalResponse = response.json().await?;
            Ok(self.parse_virustotal_response(vt_response))
        } else if response.status().as_u16() == 404 {
            Ok(HashReputation {
                source: "VirusTotal".to_string(),
                verdict: ThreatVerdict::Unknown,
                confidence: 0.1,
                first_seen: None,
                last_seen: None,
                detection_names: vec![],
                threat_types: vec![],
            })
        } else {
            Err(anyhow!("VirusTotal API error: {}", response.status()))
        }
    }

    /// Parse VirusTotal API response
    fn parse_virustotal_response(&self, response: VirusTotalResponse) -> HashReputation {
        let stats = &response.data.attributes.last_analysis_stats;
        let total_engines = stats.malicious + stats.suspicious + stats.undetected + stats.harmless;
        
        let verdict = if stats.malicious > 0 {
            ThreatVerdict::Malicious
        } else if stats.suspicious > 0 {
            ThreatVerdict::Suspicious
        } else if total_engines > 0 {
            ThreatVerdict::Benign
        } else {
            ThreatVerdict::Unknown
        };

        let confidence = match stats.malicious {
            0 => 0.1,
            1..=3 => 0.6,
            _ => 0.9,
        };

        let detection_names: Vec<String> = response.data.attributes.last_analysis_results
            .unwrap_or_default()
            .values()
            .filter_map(|engine| engine.result.clone())
            .filter(|result| result != "None" && !result.is_empty())
            .collect();

        let first_seen = response.data.attributes.first_submission_date
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0));

        let last_seen = response.data.attributes.last_submission_date
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0));

        HashReputation {
            source: "VirusTotal".to_string(),
            verdict,
            confidence,
            first_seen,
            last_seen,
            detection_names,
            threat_types: vec![], // VirusTotal doesn't provide explicit threat types in this format
        }
    }

    // Query MalwareBazaar for hash reputation
    async fn query_malwarebazaar(&self, hash: &str) -> Result<HashReputation> {
        let url = "https://mb-api.abuse.ch/api/v1/";
        
        let form_data = [
            ("query", "get_info"),
            ("hash", hash),
        ];

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.http_client
                .post(url)
                .form(&form_data)
                .send()
        ).await??;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            
            if json["query_status"] == "ok" {
                // Hash found in MalwareBazaar - it's malicious
                Ok(HashReputation {
                    source: "MalwareBazaar".to_string(),
                    verdict: ThreatVerdict::Malicious,
                    confidence: 0.9,
                    first_seen: json["data"][0]["first_seen"]
                        .as_str()
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    last_seen: None,
                    detection_names: vec![
                        json["data"][0]["signature"]
                            .as_str()
                            .unwrap_or("Unknown")
                            .to_string()
                    ],
                    threat_types: json["data"][0]["tags"]
                        .as_array()
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>())
                        .unwrap_or_default(),
                })
            } else {
                // Hash not found - unknown
                Ok(HashReputation {
                    source: "MalwareBazaar".to_string(),
                    verdict: ThreatVerdict::Unknown,
                    confidence: 0.1,
                    first_seen: None,
                    last_seen: None,
                    detection_names: vec![],
                    threat_types: vec![],
                })
            }
        } else {
            Err(anyhow!("MalwareBazaar API error: {}", response.status()))
        }
    }

    /// Query local threat database
    async fn query_local_database(&self, hash: &str) -> Result<HashReputation> {
        // This would typically query your internal database
        // For now, return a placeholder implementation
        
        debug!("Querying local database for hash: {}", hash);
        
        // TODO: Implement actual database query
        // This is a placeholder that would connect to your PostgreSQL database
        // and check against known malicious hashes
        
        Ok(HashReputation {
            source: "Local Database".to_string(),
            verdict: ThreatVerdict::Unknown,
            confidence: 0.1,
            first_seen: None,
            last_seen: None,
            detection_names: vec![],
            threat_types: vec![],
        })
    }

    /// Create final analysis result from reputation data
    fn create_analysis_result(&self, hash_info: &HashInfo, reputations: Vec<HashReputation>) -> AnalysisResult {
        use uuid::Uuid;
        use chrono::Utc;
        
        // Create file metadata from hash info
        let file_metadata = FileMetadata {
            filename: None,
            file_size: hash_info.file_size.unwrap_or(0),
            mime_type: "application/octet-stream".to_string(),
            md5: if hash_info.hash_type == HashType::MD5 { 
                hash_info.hash_value.clone() 
            } else { 
                String::new() 
            },
            sha1: if hash_info.hash_type == HashType::SHA1 { 
                hash_info.hash_value.clone() 
            } else { 
                String::new() 
            },
            sha256: if hash_info.hash_type == HashType::SHA256 { 
                hash_info.hash_value.clone() 
            } else { 
                String::new() 
            },
            sha512: None,
            entropy: None,
            magic_bytes: None,
            executable_info: None,
        };
        
        let mut result = AnalysisResult::new(Uuid::new_v4(), file_metadata);
        
        if reputations.is_empty() {
            result.status = AnalysisStatus::Completed;
            result.completed_at = Some(Utc::now());
            return result;
        }

        // Aggregate verdicts and confidence levels
        let malicious_count = reputations.iter()
            .filter(|r| r.verdict == ThreatVerdict::Malicious)
            .count();
        
        let suspicious_count = reputations.iter()
            .filter(|r| r.verdict == ThreatVerdict::Suspicious)
            .count();

        let confidence = match malicious_count {
            0 => 0.1,
            1 => 0.6,
            _ => 0.9,
        };
        
        // Create detection results for each reputation source
        for reputation in reputations {
            let detection = DetectionResult {
                detection_id: Uuid::new_v4(),
                engine_name: reputation.source,
                engine_version: "1.0.0".to_string(),
                engine_type: EngineType::Hash,
                verdict: reputation.verdict,
                confidence,
                severity: match reputation.verdict {
                    ThreatVerdict::Malicious => SeverityLevel::High,
                    ThreatVerdict::Suspicious => SeverityLevel::Medium,
                    ThreatVerdict::Benign => SeverityLevel::Low,
                    ThreatVerdict::Unknown => SeverityLevel::Info,
                },
                categories: vec![],
                metadata: std::collections::HashMap::new(),
                detected_at: Utc::now(),
                processing_time_ms: 100, // Placeholder
                error_message: None,
            };
            result.add_detection(detection);
        }
        
        result.status = AnalysisStatus::Completed;
        result.completed_at = Some(Utc::now());
        result
    }

    /// Clear local cache
    pub fn clear_cache(&mut self) {
        if let Ok(mut cache) = self.local_cache.write() {
            cache.clear();
        }
        info!("Hash analyzer cache cleared");
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        if let Ok(cache) = self.local_cache.read() {
            stats.insert("total_cached".to_string(), cache.len());
            
            let malicious_cached = cache.values()
                .filter(|r| r.verdict == ThreatVerdict::Malicious)
                .count();
            stats.insert("malicious_cached".to_string(), malicious_cached);
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hash_validation() {
        let analyzer = HashAnalyzer::new(HashAnalyzerConfig::default());
        
        // Valid hashes
        assert!(analyzer.is_valid_hash("d41d8cd98f00b204e9800998ecf8427e", &HashType::MD5));
        assert!(analyzer.is_valid_hash("da39a3ee5e6b4b0d3255bfef95601890afd80709", &HashType::SHA1));
        assert!(analyzer.is_valid_hash("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", &HashType::SHA256));
        
        // Invalid hashes
        assert!(!analyzer.is_valid_hash("invalid", &HashType::MD5));
        assert!(!analyzer.is_valid_hash("too_short", &HashType::SHA256));
        assert!(!analyzer.is_valid_hash("d41d8cd98f00b204e9800998ecf8427g", &HashType::MD5)); // invalid char
    }

    #[test]
    fn test_hash_generation() {
        let analyzer = HashAnalyzer::new(HashAnalyzerConfig::default());
        let test_data = b"hello world";
        
        let hashes = analyzer.generate_all_hashes(test_data);
        assert_eq!(hashes.len(), 3);
        
        // Check that we got all three hash types
        let hash_types: Vec<HashType> = hashes.iter().map(|h| h.hash_type.clone()).collect();
        assert!(hash_types.contains(&HashType::MD5));
        assert!(hash_types.contains(&HashType::SHA1));
        assert!(hash_types.contains(&HashType::SHA256));
        
        // Verify known hash values
        let md5_hash = hashes.iter().find(|h| h.hash_type == HashType::MD5).unwrap();
        assert_eq!(md5_hash.hash_value, "5d41402abc4b2a76b9719d911017c592");
    }
}