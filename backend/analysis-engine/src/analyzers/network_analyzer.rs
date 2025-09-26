use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use url::Url;
use regex::Regex;
use tracing::{debug, error, info, warn};
use anyhow::{anyhow, Result};

use crate::models::{AnalysisResult, ThreatIndicator};

/// Network behavior analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAnalysis {
    pub connections: Vec<NetworkConnection>,
    pub dns_queries: Vec<DnsQuery>,
    pub http_requests: Vec<HttpRequest>,
    pub suspicious_domains: Vec<SuspiciousDomain>,
    pub malicious_ips: Vec<MaliciousIp>,
    pub network_indicators: Vec<NetworkIndicator>,
    pub risk_score: f64,
    pub verdict: NetworkVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub local_addr: String,
    pub remote_addr: String,
    pub port: u16,
    pub protocol: String,
    pub direction: String, // inbound/outbound
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub duration: Duration,
    pub established: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsQuery {
    pub domain: String,
    pub query_type: String,
    pub response_code: u16,
    pub resolved_ips: Vec<String>,
    pub timestamp: u64,
    pub is_suspicious: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub user_agent: String,
    pub status_code: u16,
    pub response_size: u64,
    pub timing: Duration,
    pub is_encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousDomain {
    pub domain: String,
    pub reason: String,
    pub reputation_score: f64,
    pub category: String,
    pub first_seen: u64,
    pub last_seen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaliciousIp {
    pub ip: String,
    pub reputation: IpReputation,
    pub country: String,
    pub asn: String,
    pub threat_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpReputation {
    pub score: f64,
    pub sources: Vec<String>,
    pub last_updated: u64,
    pub is_malicious: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIndicator {
    pub indicator_type: String,
    pub value: String,
    pub confidence: f64,
    pub description: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkVerdict {
    Benign,
    Suspicious,
    Malicious,
    Unknown,
}

/// Configuration for network analysis
#[derive(Debug, Clone, Deserialize)]
pub struct NetworkAnalyzerConfig {
    pub timeout_seconds: u64,
    pub max_connections: usize,
    pub dns_timeout_ms: u64,
    pub threat_intel_sources: Vec<String>,
    pub suspicious_ports: Vec<u16>,
    pub blocked_domains: Vec<String>,
    pub whitelist_domains: Vec<String>,
    pub enable_deep_packet_inspection: bool,
}

impl Default for NetworkAnalyzerConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_connections: 100,
            dns_timeout_ms: 5000,
            threat_intel_sources: vec![
                "virustotal".to_string(),
                "urlvoid".to_string(),
                "abuseipdb".to_string(),
            ],
            suspicious_ports: vec![22, 23, 135, 139, 445, 1433, 3389, 5900],
            blocked_domains: vec![],
            whitelist_domains: vec![
                "google.com".to_string(),
                "microsoft.com".to_string(),
                "amazon.com".to_string(),
            ],
            enable_deep_packet_inspection: false,
        }
    }
}

/// Main network analyzer implementation
pub struct NetworkAnalyzer {
    config: NetworkAnalyzerConfig,
    threat_intel_cache: HashMap<String, IpReputation>,
    domain_cache: HashMap<String, DomainReputation>,
    suspicious_patterns: Vec<Regex>,
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DomainReputation {
    score: f64,
    category: String,
    is_malicious: bool,
    last_updated: u64,
}

impl NetworkAnalyzer {
    /// Create a new network analyzer instance
    pub fn new(config: NetworkAnalyzerConfig) -> Result<Self> {
        let suspicious_patterns = Self::compile_suspicious_patterns()?;
        
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent("Nexus-Security-Analyzer/1.0")
            .build()?;

        Ok(Self {
            config,
            threat_intel_cache: HashMap::new(),
            domain_cache: HashMap::new(),
            suspicious_patterns,
            http_client,
        })
    }

    /// Analyze network behavior of a sample
    pub async fn analyze_network_behavior(
        &mut self,
        network_data: &[u8],
        metadata: Option<HashMap<String, String>>,
    ) -> Result<NetworkAnalysis> {
        info!("Starting network behavior analysis");

        let mut analysis = NetworkAnalysis {
            connections: Vec::new(),
            dns_queries: Vec::new(),
            http_requests: Vec::new(),
            suspicious_domains: Vec::new(),
            malicious_ips: Vec::new(),
            network_indicators: Vec::new(),
            risk_score: 0.0,
            verdict: NetworkVerdict::Unknown,
        };

        // Parse network traffic data
        let connections = self.parse_network_connections(network_data).await?;
        let dns_queries = self.parse_dns_queries(network_data).await?;
        let http_requests = self.parse_http_requests(network_data).await?;

        analysis.connections = connections;
        analysis.dns_queries = dns_queries;
        analysis.http_requests = http_requests;

        // Analyze for suspicious behavior
        analysis.suspicious_domains = self.detect_suspicious_domains(&analysis.dns_queries).await?;
        analysis.malicious_ips = self.check_ip_reputation(&analysis.connections).await?;
        analysis.network_indicators = self.extract_network_indicators(&analysis).await?;

        // Calculate risk score and verdict
        analysis.risk_score = self.calculate_risk_score(&analysis);
        analysis.verdict = self.determine_verdict(analysis.risk_score);

        info!("Network analysis completed with verdict: {:?}", analysis.verdict);
        Ok(analysis)
    }

    /// Analyze a specific URL for malicious behavior
    pub async fn analyze_url(&mut self, url: &str) -> Result<NetworkAnalysis> {
        info!("Analyzing URL: {}", url);

        let mut analysis = NetworkAnalysis {
            connections: Vec::new(),
            dns_queries: Vec::new(),
            http_requests: Vec::new(),
            suspicious_domains: Vec::new(),
            malicious_ips: Vec::new(),
            network_indicators: Vec::new(),
            risk_score: 0.0,
            verdict: NetworkVerdict::Unknown,
        };

        // Parse and validate URL
        let parsed_url = Url::parse(url).map_err(|e| anyhow!("Invalid URL: {}", e))?;
        
        // Check URL against blacklists and patterns
        if self.is_url_suspicious(&parsed_url) {
            analysis.network_indicators.push(NetworkIndicator {
                indicator_type: "suspicious_url".to_string(),
                value: url.to_string(),
                confidence: 0.8,
                description: "URL matches suspicious patterns".to_string(),
                severity: "medium".to_string(),
            });
        }

        // Check domain reputation
        if let Some(host) = parsed_url.host_str() {
            if let Ok(domain_rep) = self.check_domain_reputation(host).await {
                if domain_rep.is_malicious {
                    analysis.suspicious_domains.push(SuspiciousDomain {
                        domain: host.to_string(),
                        reason: "Known malicious domain".to_string(),
                        reputation_score: domain_rep.score,
                        category: domain_rep.category,
                        first_seen: domain_rep.last_updated,
                        last_seen: domain_rep.last_updated,
                    });
                }
            }
        }

        // Perform HTTP request analysis
        match self.perform_safe_http_request(url).await {
            Ok(http_req) => analysis.http_requests.push(http_req),
            Err(e) => warn!("Failed to perform HTTP request: {}", e),
        }

        // Calculate final scores
        analysis.risk_score = self.calculate_risk_score(&analysis);
        analysis.verdict = self.determine_verdict(analysis.risk_score);

        Ok(analysis)
    }

    /// Parse network connections from raw data
    async fn parse_network_connections(&self, data: &[u8]) -> Result<Vec<NetworkConnection>> {
        // In a real implementation, this would parse actual network traffic
        // For now, we'll simulate parsing from a structured format
        debug!("Parsing network connections from {} bytes", data.len());
        
        let mut connections = Vec::new();
        
        // Simulate parsing logic - in reality, you'd use libraries like:
        // - libpcap for packet capture analysis
        // - netstat parsing for active connections
        // - Process monitoring APIs
        
        if data.len() > 100 {
            connections.push(NetworkConnection {
                local_addr: "192.168.1.100".to_string(),
                remote_addr: "8.8.8.8".to_string(),
                port: 53,
                protocol: "UDP".to_string(),
                direction: "outbound".to_string(),
                bytes_sent: 64,
                bytes_received: 128,
                duration: Duration::from_millis(50),
                established: true,
            });
        }
        
        Ok(connections)
    }

    /// Parse DNS queries from network data
    async fn parse_dns_queries(&self, data: &[u8]) -> Result<Vec<DnsQuery>> {
        debug!("Parsing DNS queries from network data");
        
        let mut queries = Vec::new();
        
        // Simulate DNS query parsing
        if data.len() > 50 {
            queries.push(DnsQuery {
                domain: "example.com".to_string(),
                query_type: "A".to_string(),
                response_code: 0,
                resolved_ips: vec!["93.184.216.34".to_string()],
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                is_suspicious: false,
            });
        }
        
        Ok(queries)
    }

    /// Parse HTTP requests from network data
    async fn parse_http_requests(&self, data: &[u8]) -> Result<Vec<HttpRequest>> {
        debug!("Parsing HTTP requests from network data");
        
        let mut requests = Vec::new();
        
        // Simulate HTTP request parsing
        if data.len() > 200 {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "text/html".to_string());
            
            requests.push(HttpRequest {
                method: "GET".to_string(),
                url: "https://example.com".to_string(),
                headers,
                user_agent: "Mozilla/5.0".to_string(),
                status_code: 200,
                response_size: 1024,
                timing: Duration::from_millis(150),
                is_encrypted: true,
            });
        }
        
        Ok(requests)
    }

    /// Detect suspicious domains from DNS queries
    async fn detect_suspicious_domains(&mut self, dns_queries: &[DnsQuery]) -> Result<Vec<SuspiciousDomain>> {
        let mut suspicious = Vec::new();
        
        for query in dns_queries {
            if self.is_domain_suspicious(&query.domain) {
                // Check against threat intelligence
                if let Ok(reputation) = self.check_domain_reputation(&query.domain).await {
                    if reputation.is_malicious || reputation.score < 0.5 {
                        suspicious.push(SuspiciousDomain {
                            domain: query.domain.clone(),
                            reason: "Low reputation score".to_string(),
                            reputation_score: reputation.score,
                            category: reputation.category,
                            first_seen: query.timestamp,
                            last_seen: query.timestamp,
                        });
                    }
                }
            }
        }
        
        Ok(suspicious)
    }

    /// Check IP reputation against threat intelligence sources
    async fn check_ip_reputation(&mut self, connections: &[NetworkConnection]) -> Result<Vec<MaliciousIp>> {
        let mut malicious_ips = Vec::new();
        let unique_ips: HashSet<_> = connections
            .iter()
            .map(|c| &c.remote_addr)
            .collect();

        for ip in unique_ips {
            if let Ok(parsed_ip) = ip.parse::<IpAddr>() {
                if !self.is_private_ip(&parsed_ip) {
                    if let Ok(reputation) = self.get_ip_reputation(ip).await {
                        if reputation.is_malicious {
                            malicious_ips.push(MaliciousIp {
                                ip: ip.clone(),
                                reputation,
                                country: "Unknown".to_string(),
                                asn: "Unknown".to_string(),
                                threat_types: vec!["malware".to_string()],
                            });
                        }
                    }
                }
            }
        }

        Ok(malicious_ips)
    }

    /// Extract network-based threat indicators
    async fn extract_network_indicators(&self, analysis: &NetworkAnalysis) -> Result<Vec<NetworkIndicator>> {
        let mut indicators = Vec::new();

        // Check for suspicious ports
        for connection in &analysis.connections {
            if self.config.suspicious_ports.contains(&connection.port) {
                indicators.push(NetworkIndicator {
                    indicator_type: "suspicious_port".to_string(),
                    value: connection.port.to_string(),
                    confidence: 0.6,
                    description: format!("Connection to suspicious port {}", connection.port),
                    severity: "medium".to_string(),
                });
            }
        }

        // Check for large data transfers
        for connection in &analysis.connections {
            if connection.bytes_sent > 10_000_000 || connection.bytes_received > 10_000_000 {
                indicators.push(NetworkIndicator {
                    indicator_type: "large_data_transfer".to_string(),
                    value: format!("{}:{}", connection.remote_addr, connection.port),
                    confidence: 0.5,
                    description: "Large data transfer detected".to_string(),
                    severity: "low".to_string(),
                });
            }
        }

        // Check for suspicious HTTP patterns
        for request in &analysis.http_requests {
            if self.has_suspicious_http_patterns(request) {
                indicators.push(NetworkIndicator {
                    indicator_type: "suspicious_http".to_string(),
                    value: request.url.clone(),
                    confidence: 0.7,
                    description: "Suspicious HTTP request patterns".to_string(),
                    severity: "medium".to_string(),
                });
            }
        }

        Ok(indicators)
    }

    /// Get IP reputation from threat intelligence sources
    async fn get_ip_reputation(&mut self, ip: &str) -> Result<IpReputation> {
        // Check cache first
        if let Some(cached) = self.threat_intel_cache.get(ip) {
            if SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() - cached.last_updated < 3600 // 1 hour cache
            {
                return Ok(cached.clone());
            }
        }

        // Query threat intelligence sources
        let reputation = self.query_threat_intel_for_ip(ip).await?;
        
        // Cache the result
        self.threat_intel_cache.insert(ip.to_string(), reputation.clone());
        
        Ok(reputation)
    }

    /// Query threat intelligence sources for IP information
    async fn query_threat_intel_for_ip(&self, ip: &str) -> Result<IpReputation> {
        // Simulate threat intelligence query
        // In a real implementation, this would query actual services like:
        // - VirusTotal API
        // - AbuseIPDB
        // - IBM X-Force
        // - OTX AlienVault
        
        debug!("Querying threat intel for IP: {}", ip);
        
        // Simulate response based on IP patterns
        let is_malicious = self.simulate_ip_maliciousness(ip);
        let score = if is_malicious { 0.1 } else { 0.9 };

        Ok(IpReputation {
            score,
            sources: vec!["simulated".to_string()],
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_malicious,
        })
    }

    /// Check domain reputation
    async fn check_domain_reputation(&mut self, domain: &str) -> Result<DomainReputation> {
        // Check cache first
        if let Some(cached) = self.domain_cache.get(domain) {
            if SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() - cached.last_updated < 3600
            {
                return Ok(cached.clone());
            }
        }

        // Query domain reputation
        let reputation = self.query_domain_reputation(domain).await?;
        
        // Cache result
        self.domain_cache.insert(domain.to_string(), reputation.clone());
        
        Ok(reputation)
    }

    /// Query domain reputation from threat intelligence
    async fn query_domain_reputation(&self, domain: &str) -> Result<DomainReputation> {
        debug!("Querying domain reputation for: {}", domain);
        
        // Simulate domain reputation check
        let is_malicious = self.config.blocked_domains.contains(&domain.to_string()) ||
                          domain.contains("malware") ||
                          domain.contains("phishing");
        
        let score = if is_malicious { 0.1 } else { 0.8 };
        
        Ok(DomainReputation {
            score,
            category: if is_malicious { "malware".to_string() } else { "benign".to_string() },
            is_malicious,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Perform safe HTTP request for URL analysis
    async fn perform_safe_http_request(&self, url: &str) -> Result<HttpRequest> {
        debug!("Performing safe HTTP request to: {}", url);
        
        let start_time = std::time::Instant::now();
        
        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.http_client.head(url)
        ).await??;

        let timing = start_time.elapsed();
        let mut headers = HashMap::new();
        
        for (name, value) in response.headers() {
            headers.insert(
                name.to_string(),
                value.to_str().unwrap_or("").to_string()
            );
        }

        Ok(HttpRequest {
            method: "HEAD".to_string(),
            url: url.to_string(),
            headers,
            user_agent: "Nexus-Security-Analyzer/1.0".to_string(),
            status_code: response.status().as_u16(),
            response_size: 0,
            timing,
            is_encrypted: url.starts_with("https://"),
        })
    }

    /// Calculate overall risk score for network analysis
    fn calculate_risk_score(&self, analysis: &NetworkAnalysis) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Factor in suspicious domains
        if !analysis.suspicious_domains.is_empty() {
            score += analysis.suspicious_domains.len() as f64 * 0.3;
            factors += 1;
        }

        // Factor in malicious IPs
        if !analysis.malicious_ips.is_empty() {
            score += analysis.malicious_ips.len() as f64 * 0.4;
            factors += 1;
        }

        // Factor in network indicators
        for indicator in &analysis.network_indicators {
            let indicator_score = match indicator.severity.as_str() {
                "critical" => 0.9,
                "high" => 0.7,
                "medium" => 0.5,
                "low" => 0.3,
                _ => 0.1,
            };
            score += indicator_score * indicator.confidence;
            factors += 1;
        }

        // Factor in suspicious connections
        let suspicious_connections = analysis.connections
            .iter()
            .filter(|c| self.config.suspicious_ports.contains(&c.port))
            .count();
        
        if suspicious_connections > 0 {
            score += suspicious_connections as f64 * 0.2;
            factors += 1;
        }

        // Normalize score
        if factors > 0 {
            (score / factors as f64).min(1.0)
        } else {
            0.0
        }
    }

    /// Determine verdict based on risk score
    fn determine_verdict(&self, risk_score: f64) -> NetworkVerdict {
        match risk_score {
            score if score >= 0.8 => NetworkVerdict::Malicious,
            score if score >= 0.5 => NetworkVerdict::Suspicious,
            score if score >= 0.1 => NetworkVerdict::Benign,
            _ => NetworkVerdict::Unknown,
        }
    }

    /// Check if domain is suspicious based on patterns
    fn is_domain_suspicious(&self, domain: &str) -> bool {
        // Check against whitelist first
        if self.config.whitelist_domains.iter().any(|w| domain.contains(w)) {
            return false;
        }

        // Check suspicious patterns
        self.suspicious_patterns.iter().any(|pattern| pattern.is_match(domain))
    }

    /// Check if URL is suspicious
    fn is_url_suspicious(&self, url: &Url) -> bool {
        // Check for suspicious URL patterns
        let url_str = url.as_str();
        
        // Check for known malicious patterns
        let suspicious_patterns = [
            r"\.tk/",
            r"\.ml/",
            r"bit\.ly",
            r"tinyurl",
            r"[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}", // IP addresses
        ];

        suspicious_patterns.iter().any(|pattern| {
            Regex::new(pattern).map(|re| re.is_match(url_str)).unwrap_or(false)
        })
    }

    /// Check if IP is private/local
    fn is_private_ip(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_private() || ipv4.is_loopback() || ipv4.is_link_local()
            }
            IpAddr::V6(ipv6) => {
                ipv6.is_loopback() || ipv6.is_multicast()
            }
        }
    }

    /// Check for suspicious HTTP request patterns
    fn has_suspicious_http_patterns(&self, request: &HttpRequest) -> bool {
        // Check User-Agent
        if request.user_agent.is_empty() || 
           request.user_agent.len() < 10 ||
           request.user_agent.contains("wget") ||
           request.user_agent.contains("curl") {
            return true;
        }

        // Check for suspicious headers
        if request.headers.contains_key("X-Forwarded-For") &&
           request.headers.len() < 3 {
            return true;
        }

        // Check response codes
        if request.status_code >= 400 {
            return true;
        }

        false
    }

    /// Simulate IP maliciousness for testing
    fn simulate_ip_maliciousness(&self, ip: &str) -> bool {
        // Simple simulation - mark certain IP patterns as malicious
        ip.starts_with("192.0.2.") || // RFC 5737 test network
        ip.starts_with("198.51.100.") ||
        ip.contains("666") ||
        ip.ends_with(".666")
    }

    /// Compile suspicious domain patterns
    fn compile_suspicious_patterns() -> Result<Vec<Regex>> {
        let patterns = vec![
            r"[0-9]{10,}", // Long numeric sequences
            r"[a-z]{20,}", // Long random strings
            r"\.tk$|\.ml$|\.ga$|\.cf$", // Suspicious TLDs
            r"(download|click|here|free)", // Suspicious keywords
            r"[0-9a-f]{32}", // Hex strings (possible hashes)
        ];

        patterns.into_iter()
            .map(|p| Regex::new(p).map_err(|e| anyhow!("Failed to compile regex {}: {}", p, e)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_analyzer_creation() {
        let config = NetworkAnalyzerConfig::default();
        let analyzer = NetworkAnalyzer::new(config);
        assert!(analyzer.is_ok());
    }

    #[tokio::test]
    async fn test_url_analysis() {
        let config = NetworkAnalyzerConfig::default();
        let mut analyzer = NetworkAnalyzer::new(config).unwrap();
        
        let result = analyzer.analyze_url("https://example.com").await;
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert_eq!(analysis.verdict, NetworkVerdict::Benign);
    }

    #[test]
    fn test_private_ip_detection() {
        let config = NetworkAnalyzerConfig::default();
        let analyzer = NetworkAnalyzer::new(config).unwrap();
        
        let private_ip: IpAddr = "192.168.1.1".parse().unwrap();
        let public_ip: IpAddr = "8.8.8.8".parse().unwrap();
        
        assert!(analyzer.is_private_ip(&private_ip));
        assert!(!analyzer.is_private_ip(&public_ip));
    }

    #[test]
    fn test_risk_score_calculation() {
        let config = NetworkAnalyzerConfig::default();
        let analyzer = NetworkAnalyzer::new(config).unwrap();
        
        let analysis = NetworkAnalysis {
            connections: vec![],
            dns_queries: vec![],
            http_requests: vec![],
            suspicious_domains: vec![],
            malicious_ips: vec![],
            network_indicators: vec![
                NetworkIndicator {
                    indicator_type: "test".to_string(),
                    value: "test".to_string(),
                    confidence: 0.8,
                    description: "test".to_string(),
                    severity: "high".to_string(),
                }
            ],
            risk_score: 0.0,
            verdict: NetworkVerdict::Unknown,
        };
        
        let score = analyzer.calculate_risk_score(&analysis);
        assert!(score > 0.0);
    }
}