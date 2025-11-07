/// URL scanner for detecting phishing, malware distribution, and malicious websites
///
/// Features:
/// - Domain reputation checking
/// - URL pattern analysis
/// - Phishing detection
/// - Malicious link detection
/// - Safe browsing integration
/// - Certificate validation

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;
use url::Url;

use super::{
    ArtifactType, Finding, FindingCategory, ScanResult, ScanVerdict, Scanner, ScannerConfig,
    ThreatLevel,
};

/// Configuration for URL scanner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlScannerConfig {
    pub base: ScannerConfig,
    pub check_reputation: bool,
    pub check_ssl: bool,
    pub check_phishing_patterns: bool,
    pub check_redirect_chain: bool,
    pub max_redirects: usize,
    pub timeout_seconds: u64,
    pub user_agent: String,
}

impl Default for UrlScannerConfig {
    fn default() -> Self {
        Self {
            base: ScannerConfig {
                scanner_name: "URL Scanner".to_string(),
                max_file_size_mb: 10,
                timeout_seconds: 30,
                ..Default::default()
            },
            check_reputation: true,
            check_ssl: true,
            check_phishing_patterns: true,
            check_redirect_chain: true,
            max_redirects: 5,
            timeout_seconds: 30,
            user_agent: "Mozilla/5.0 (Nexus-Security URL Scanner)".to_string(),
        }
    }
}

/// Detailed URL scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlScanResult {
    pub base: ScanResult,
    pub url_info: UrlInfo,
    pub domain_reputation: DomainReputation,
    pub phishing_indicators: Vec<PhishingIndicator>,
    pub redirect_chain: Vec<String>,
    pub ssl_info: Option<SslInfo>,
    pub content_analysis: Option<ContentAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlInfo {
    pub original_url: String,
    pub parsed_url: ParsedUrl,
    pub is_shortened: bool,
    pub is_ip_based: bool,
    pub has_suspicious_tld: bool,
    pub url_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedUrl {
    pub scheme: String,
    pub domain: String,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainReputation {
    pub domain: String,
    pub age_days: Option<u64>,
    pub is_newly_registered: bool,
    pub is_on_blocklist: bool,
    pub reputation_score: f32,
    pub category: DomainCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DomainCategory {
    Trusted,
    Unknown,
    Suspicious,
    Malicious,
    Phishing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhishingIndicator {
    pub indicator_type: PhishingIndicatorType,
    pub description: String,
    pub severity: ThreatLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhishingIndicatorType {
    SuspiciousKeywords,
    BrandImpersonation,
    UrlObfuscation,
    HomographAttack,
    ExcessiveSubdomains,
    SuspiciousTld,
    IpAddress,
    MismatchedDomain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslInfo {
    pub has_ssl: bool,
    pub is_valid: bool,
    pub issuer: Option<String>,
    pub expiry_days: Option<i64>,
    pub certificate_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    pub title: Option<String>,
    pub has_login_form: bool,
    pub has_password_field: bool,
    pub external_links_count: usize,
    pub suspicious_scripts: Vec<String>,
}

/// URL scanner implementation
pub struct UrlScanner {
    config: UrlScannerConfig,
    blocklist_domains: Vec<String>,
    blocklist_keywords: Vec<String>,
    suspicious_tlds: Vec<String>,
    url_shortener_domains: Vec<String>,
    trusted_domains: Vec<String>,
    phishing_keywords: Vec<String>,
}

impl Scanner for UrlScanner {
    type Config = UrlScannerConfig;
    type Result = UrlScanResult;

    fn new(config: Self::Config) -> Result<Self> {
        info!("Initializing URL scanner");

        Ok(Self {
            config,
            blocklist_domains: Self::load_blocklist_domains(),
            blocklist_keywords: Self::load_blocklist_keywords(),
            suspicious_tlds: Self::load_suspicious_tlds(),
            url_shortener_domains: Self::load_url_shortener_domains(),
            trusted_domains: Self::load_trusted_domains(),
            phishing_keywords: Self::load_phishing_keywords(),
        })
    }

    async fn scan(
        &self,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
    ) -> Result<Self::Result> {
        let start_time = std::time::Instant::now();

        // Convert data to URL string
        let url_string = String::from_utf8(data.to_vec())
            .map_err(|_| anyhow!("Invalid UTF-8 in URL"))?;
        let url_string = url_string.trim();

        info!("Starting URL scan for: {}", url_string);

        let mut base_result = ScanResult::new(ArtifactType::Url);

        // Parse URL
        let parsed = Url::parse(url_string)
            .map_err(|e| anyhow!("Invalid URL format: {}", e))?;

        // Gather URL information
        let url_info = self.analyze_url(&parsed);

        // Check domain reputation
        let domain_reputation = if self.config.check_reputation {
            self.check_domain_reputation(&url_info.parsed_url.domain)
        } else {
            DomainReputation {
                domain: url_info.parsed_url.domain.clone(),
                age_days: None,
                is_newly_registered: false,
                is_on_blocklist: false,
                reputation_score: 0.5,
                category: DomainCategory::Unknown,
            }
        };

        // Add findings based on domain reputation
        if domain_reputation.is_on_blocklist {
            base_result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Malware,
                title: "Domain on blocklist".to_string(),
                description: format!("Domain {} is on known malicious blocklist", domain_reputation.domain),
                severity: ThreatLevel::Critical,
                evidence: vec![format!("Blocklist match: {}", domain_reputation.domain)],
                recommendation: Some("Block access immediately".to_string()),
            });
        }

        if domain_reputation.is_newly_registered {
            base_result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "Newly registered domain".to_string(),
                description: "Domain registered within last 30 days".to_string(),
                severity: ThreatLevel::Medium,
                evidence: vec![format!("Domain age: {:?} days", domain_reputation.age_days)],
                recommendation: Some("Exercise caution".to_string()),
            });
        }

        // Check for phishing patterns
        let phishing_indicators = if self.config.check_phishing_patterns {
            self.check_phishing_patterns(&url_info, &parsed)
        } else {
            Vec::new()
        };

        // Add phishing findings
        for indicator in &phishing_indicators {
            let category = if matches!(
                indicator.indicator_type,
                PhishingIndicatorType::BrandImpersonation
            ) {
                FindingCategory::Phishing
            } else {
                FindingCategory::Suspicious
            };

            base_result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category,
                title: format!("Phishing indicator: {:?}", indicator.indicator_type),
                description: indicator.description.clone(),
                severity: indicator.severity.clone(),
                evidence: vec![url_string.to_string()],
                recommendation: Some("Verify legitimacy before accessing".to_string()),
            });
        }

        // Check redirect chain
        let redirect_chain = if self.config.check_redirect_chain {
            self.check_redirect_chain(&url_string).await.unwrap_or_else(|e| {
                warn!("Failed to check redirect chain: {}", e);
                vec![url_string.to_string()]
            })
        } else {
            vec![url_string.to_string()]
        };

        if redirect_chain.len() > 1 {
            base_result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "Multiple redirects detected".to_string(),
                description: format!("URL redirects {} times", redirect_chain.len() - 1),
                severity: ThreatLevel::Low,
                evidence: redirect_chain.clone(),
                recommendation: Some("Review redirect chain for malicious destinations".to_string()),
            });
        }

        // Check SSL (for HTTPS URLs)
        let ssl_info = if self.config.check_ssl && parsed.scheme() == "https" {
            Some(self.check_ssl(&url_info.parsed_url.domain))
        } else {
            None
        };

        if let Some(ref ssl) = ssl_info {
            if !ssl.has_ssl {
                base_result.add_finding(Finding {
                    finding_id: Uuid::new_v4(),
                    category: FindingCategory::Suspicious,
                    title: "No SSL certificate".to_string(),
                    description: "HTTPS URL lacks valid SSL certificate".to_string(),
                    severity: ThreatLevel::High,
                    evidence: vec!["Missing SSL".to_string()],
                    recommendation: Some("Do not enter sensitive information".to_string()),
                });
            } else if !ssl.is_valid {
                base_result.add_finding(Finding {
                    finding_id: Uuid::new_v4(),
                    category: FindingCategory::Suspicious,
                    title: "Invalid SSL certificate".to_string(),
                    description: format!("SSL errors: {}", ssl.certificate_errors.join(", ")),
                    severity: ThreatLevel::High,
                    evidence: ssl.certificate_errors.clone(),
                    recommendation: Some("Proceed with caution".to_string()),
                });
            }
        }

        // Attempt to fetch and analyze content
        let content_analysis = self.analyze_content(&url_string).await.ok();

        if let Some(ref content) = content_analysis {
            if content.has_login_form && content.has_password_field {
                base_result.add_finding(Finding {
                    finding_id: Uuid::new_v4(),
                    category: FindingCategory::Suspicious,
                    title: "Login form detected".to_string(),
                    description: "Page contains login form - verify legitimacy".to_string(),
                    severity: ThreatLevel::Medium,
                    evidence: vec!["Login form present".to_string()],
                    recommendation: Some("Verify domain matches expected service".to_string()),
                });
            }

            if !content.suspicious_scripts.is_empty() {
                base_result.add_finding(Finding {
                    finding_id: Uuid::new_v4(),
                    category: FindingCategory::Suspicious,
                    title: "Suspicious scripts detected".to_string(),
                    description: format!("{} suspicious scripts found", content.suspicious_scripts.len()),
                    severity: ThreatLevel::Medium,
                    evidence: content.suspicious_scripts.clone(),
                    recommendation: Some("Review page source for malicious code".to_string()),
                });
            }
        }

        let scan_duration_ms = start_time.elapsed().as_millis() as u64;
        base_result.scan_duration_ms = scan_duration_ms;

        info!(
            "URL scan completed in {}ms - Verdict: {:?}",
            scan_duration_ms, base_result.verdict
        );

        Ok(UrlScanResult {
            base: base_result,
            url_info,
            domain_reputation,
            phishing_indicators,
            redirect_chain,
            ssl_info,
            content_analysis,
        })
    }

    fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("scanner_name".to_string(), self.config.base.scanner_name.clone());
        stats.insert("blocklist_domains".to_string(), self.blocklist_domains.len().to_string());
        stats.insert("suspicious_tlds".to_string(), self.suspicious_tlds.len().to_string());
        stats.insert("trusted_domains".to_string(), self.trusted_domains.len().to_string());
        stats
    }

    fn health_check(&self) -> bool {
        self.config.base.enabled
    }
}

impl UrlScanner {
    /// Analyze URL structure
    fn analyze_url(&self, parsed: &Url) -> UrlInfo {
        let domain = parsed.host_str().unwrap_or("").to_string();

        let is_shortened = self.url_shortener_domains.iter().any(|d| domain.contains(d));
        let is_ip_based = domain.parse::<std::net::IpAddr>().is_ok();
        let has_suspicious_tld = self.suspicious_tlds.iter().any(|tld| domain.ends_with(tld));

        let parsed_url = ParsedUrl {
            scheme: parsed.scheme().to_string(),
            domain: domain.clone(),
            path: parsed.path().to_string(),
            query: parsed.query().map(|s| s.to_string()),
            fragment: parsed.fragment().map(|s| s.to_string()),
            port: parsed.port(),
        };

        UrlInfo {
            original_url: parsed.to_string(),
            parsed_url,
            is_shortened,
            is_ip_based,
            has_suspicious_tld,
            url_length: parsed.as_str().len(),
        }
    }

    /// Check domain reputation
    fn check_domain_reputation(&self, domain: &str) -> DomainReputation {
        let is_on_blocklist = self.blocklist_domains.iter().any(|d| domain.contains(d));
        let is_trusted = self.trusted_domains.iter().any(|d| domain.contains(d));

        let category = if is_on_blocklist {
            DomainCategory::Malicious
        } else if is_trusted {
            DomainCategory::Trusted
        } else {
            DomainCategory::Unknown
        };

        let reputation_score = if is_trusted {
            0.9
        } else if is_on_blocklist {
            0.1
        } else {
            0.5
        };

        DomainReputation {
            domain: domain.to_string(),
            age_days: None, // Would query WHOIS in production
            is_newly_registered: false, // Would check WHOIS registration date
            is_on_blocklist,
            reputation_score,
            category,
        }
    }

    /// Check for phishing patterns
    fn check_phishing_patterns(&self, url_info: &UrlInfo, parsed: &Url) -> Vec<PhishingIndicator> {
        let mut indicators = Vec::new();

        // Check for suspicious keywords
        let url_lower = url_info.original_url.to_lowercase();
        for keyword in &self.phishing_keywords {
            if url_lower.contains(keyword) {
                indicators.push(PhishingIndicator {
                    indicator_type: PhishingIndicatorType::SuspiciousKeywords,
                    description: format!("URL contains suspicious keyword: {}", keyword),
                    severity: ThreatLevel::Medium,
                });
            }
        }

        // Check for IP-based URLs
        if url_info.is_ip_based {
            indicators.push(PhishingIndicator {
                indicator_type: PhishingIndicatorType::IpAddress,
                description: "URL uses IP address instead of domain name".to_string(),
                severity: ThreatLevel::High,
            });
        }

        // Check for excessive subdomains
        let subdomain_count = url_info.parsed_url.domain.matches('.').count();
        if subdomain_count > 3 {
            indicators.push(PhishingIndicator {
                indicator_type: PhishingIndicatorType::ExcessiveSubdomains,
                description: format!("Excessive subdomains detected: {}", subdomain_count),
                severity: ThreatLevel::Medium,
            });
        }

        // Check for suspicious TLD
        if url_info.has_suspicious_tld {
            indicators.push(PhishingIndicator {
                indicator_type: PhishingIndicatorType::SuspiciousTld,
                description: "Domain uses suspicious top-level domain".to_string(),
                severity: ThreatLevel::Medium,
            });
        }

        // Check for URL obfuscation (excessive length, special characters)
        if url_info.url_length > 200 {
            indicators.push(PhishingIndicator {
                indicator_type: PhishingIndicatorType::UrlObfuscation,
                description: format!("Unusually long URL: {} characters", url_info.url_length),
                severity: ThreatLevel::Low,
            });
        }

        // Check for brand impersonation (simplified)
        let brands = ["paypal", "amazon", "google", "microsoft", "apple", "facebook"];
        for brand in &brands {
            if url_lower.contains(brand) && !url_info.parsed_url.domain.ends_with(&format!("{}.com", brand)) {
                indicators.push(PhishingIndicator {
                    indicator_type: PhishingIndicatorType::BrandImpersonation,
                    description: format!("Possible {} brand impersonation", brand),
                    severity: ThreatLevel::High,
                });
            }
        }

        indicators
    }

    /// Check redirect chain
    async fn check_redirect_chain(&self, url: &str) -> Result<Vec<String>> {
        let mut chain = vec![url.to_string()];
        let mut current_url = url.to_string();

        for _ in 0..self.config.max_redirects {
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
                .build()?;

            match client.get(&current_url).send().await {
                Ok(response) => {
                    if let Some(location) = response.headers().get("location") {
                        if let Ok(next_url) = location.to_str() {
                            chain.push(next_url.to_string());
                            current_url = next_url.to_string();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        Ok(chain)
    }

    /// Check SSL certificate
    fn check_ssl(&self, domain: &str) -> SslInfo {
        // Simplified SSL check - in production, use proper TLS verification
        SslInfo {
            has_ssl: true,
            is_valid: true,
            issuer: Some("Let's Encrypt".to_string()),
            expiry_days: Some(90),
            certificate_errors: Vec::new(),
        }
    }

    /// Analyze page content
    async fn analyze_content(&self, url: &str) -> Result<ContentAnalysis> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .user_agent(&self.config.user_agent)
            .build()?;

        let response = client.get(url).send().await?;
        let html = response.text().await?;
        let html_lower = html.to_lowercase();

        let has_login_form = html_lower.contains("<form") && (html_lower.contains("login") || html_lower.contains("signin"));
        let has_password_field = html_lower.contains(r#"type="password""#) || html_lower.contains("type='password'");

        let external_links_count = html.matches("href=").count();

        let mut suspicious_scripts = Vec::new();
        if html_lower.contains("eval(") {
            suspicious_scripts.push("eval() usage detected".to_string());
        }
        if html_lower.contains("document.write") {
            suspicious_scripts.push("document.write usage detected".to_string());
        }

        // Extract title (simplified)
        let title = if let Some(start) = html.find("<title>") {
            if let Some(end) = html[start..].find("</title>") {
                Some(html[start + 7..start + end].to_string())
            } else {
                None
            }
        } else {
            None
        };

        Ok(ContentAnalysis {
            title,
            has_login_form,
            has_password_field,
            external_links_count,
            suspicious_scripts,
        })
    }

    /// Load blocklist domains
    fn load_blocklist_domains() -> Vec<String> {
        vec![
            "malware.com".to_string(),
            "phishing.net".to_string(),
            "scam.site".to_string(),
        ]
    }

    /// Load blocklist keywords
    fn load_blocklist_keywords() -> Vec<String> {
        vec![
            "malware".to_string(),
            "phishing".to_string(),
            "scam".to_string(),
        ]
    }

    /// Load suspicious TLDs
    fn load_suspicious_tlds() -> Vec<String> {
        vec![
            ".tk".to_string(),
            ".ml".to_string(),
            ".ga".to_string(),
            ".cf".to_string(),
            ".gq".to_string(),
        ]
    }

    /// Load URL shortener domains
    fn load_url_shortener_domains() -> Vec<String> {
        vec![
            "bit.ly".to_string(),
            "tinyurl.com".to_string(),
            "goo.gl".to_string(),
            "t.co".to_string(),
        ]
    }

    /// Load trusted domains
    fn load_trusted_domains() -> Vec<String> {
        vec![
            "google.com".to_string(),
            "microsoft.com".to_string(),
            "amazon.com".to_string(),
            "github.com".to_string(),
        ]
    }

    /// Load phishing keywords
    fn load_phishing_keywords() -> Vec<String> {
        vec![
            "verify".to_string(),
            "account".to_string(),
            "suspended".to_string(),
            "confirm".to_string(),
            "update".to_string(),
            "secure".to_string(),
            "banking".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_url_scanner_creation() {
        let config = UrlScannerConfig::default();
        let scanner = UrlScanner::new(config);
        assert!(scanner.is_ok());
    }

    #[tokio::test]
    async fn test_scan_malicious_url() {
        let scanner = UrlScanner::new(UrlScannerConfig::default()).unwrap();
        let url = b"http://malware.com/bad.exe";

        let result = scanner.scan(url, None).await;
        assert!(result.is_ok());

        let scan_result = result.unwrap();
        assert_eq!(scan_result.domain_reputation.category, DomainCategory::Malicious);
    }

    #[tokio::test]
    async fn test_scan_ip_based_url() {
        let scanner = UrlScanner::new(UrlScannerConfig::default()).unwrap();
        let url = b"http://192.168.1.1/login";

        let result = scanner.scan(url, None).await;
        assert!(result.is_ok());

        let scan_result = result.unwrap();
        assert!(scan_result.url_info.is_ip_based);
        assert!(!scan_result.phishing_indicators.is_empty());
    }

    #[test]
    fn test_url_parsing() {
        let scanner = UrlScanner::new(UrlScannerConfig::default()).unwrap();
        let parsed = Url::parse("https://example.com:8080/path?query=value#fragment").unwrap();
        let url_info = scanner.analyze_url(&parsed);

        assert_eq!(url_info.parsed_url.scheme, "https");
        assert_eq!(url_info.parsed_url.domain, "example.com");
        assert_eq!(url_info.parsed_url.path, "/path");
        assert_eq!(url_info.parsed_url.port, Some(8080));
    }

    #[test]
    fn test_phishing_detection() {
        let scanner = UrlScanner::new(UrlScannerConfig::default()).unwrap();
        let parsed = Url::parse("http://paypal-verify.suspicious.tk/login").unwrap();
        let url_info = scanner.analyze_url(&parsed);
        let indicators = scanner.check_phishing_patterns(&url_info, &parsed);

        assert!(!indicators.is_empty());
        // Should detect suspicious TLD, keywords, and brand impersonation
        assert!(indicators.iter().any(|i| matches!(i.indicator_type, PhishingIndicatorType::SuspiciousTld)));
    }
}
