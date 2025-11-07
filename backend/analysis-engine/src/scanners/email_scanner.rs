/// Email scanner for detecting spam, phishing, and malicious attachments
///
/// Features:
/// - Header analysis
/// - SPF/DKIM/DMARC validation
/// - Phishing detection
/// - Malicious attachment scanning
/// - URL extraction and analysis
/// - Content analysis

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::{
    ArtifactType, Finding, FindingCategory, ScanResult, ScanVerdict, Scanner, ScannerConfig,
    ThreatLevel,
};

/// Configuration for email scanner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailScannerConfig {
    pub base: ScannerConfig,
    pub check_spf: bool,
    pub check_dkim: bool,
    pub check_dmarc: bool,
    pub scan_attachments: bool,
    pub extract_urls: bool,
    pub check_headers: bool,
    pub max_attachment_size_mb: u64,
}

impl Default for EmailScannerConfig {
    fn default() -> Self {
        Self {
            base: ScannerConfig {
                scanner_name: "Email Scanner".to_string(),
                max_file_size_mb: 50,
                ..Default::default()
            },
            check_spf: true,
            check_dkim: true,
            check_dmarc: true,
            scan_attachments: true,
            extract_urls: true,
            check_headers: true,
            max_attachment_size_mb: 25,
        }
    }
}

/// Detailed email scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailScanResult {
    pub base: ScanResult,
    pub email_info: EmailInfo,
    pub header_analysis: HeaderAnalysis,
    pub authentication_results: AuthenticationResults,
    pub content_analysis: ContentAnalysis,
    pub attachments: Vec<AttachmentInfo>,
    pub extracted_urls: Vec<String>,
    pub spam_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailInfo {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub date: Option<String>,
    pub message_id: Option<String>,
    pub reply_to: Option<String>,
    pub return_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderAnalysis {
    pub has_suspicious_headers: bool,
    pub suspicious_headers: Vec<String>,
    pub header_count: usize,
    pub received_hops: usize,
    pub is_forged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResults {
    pub spf_result: AuthResult,
    pub dkim_result: AuthResult,
    pub dmarc_result: AuthResult,
    pub is_authenticated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthResult {
    Pass,
    Fail,
    SoftFail,
    Neutral,
    None,
    TempError,
    PermError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    pub body_text: String,
    pub body_html: Option<String>,
    pub has_suspicious_content: bool,
    pub suspicious_patterns: Vec<String>,
    pub language: Option<String>,
    pub urgency_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub filename: String,
    pub size: u64,
    pub mime_type: String,
    pub is_suspicious: bool,
    pub hash: String,
    pub scan_result: Option<String>,
}

/// Email scanner implementation
pub struct EmailScanner {
    config: EmailScannerConfig,
    spam_keywords: Vec<String>,
    phishing_patterns: Vec<String>,
    suspicious_extensions: Vec<String>,
    urgent_keywords: Vec<String>,
}

impl Scanner for EmailScanner {
    type Config = EmailScannerConfig;
    type Result = EmailScanResult;

    fn new(config: Self::Config) -> Result<Self> {
        info!("Initializing email scanner");

        Ok(Self {
            config,
            spam_keywords: Self::load_spam_keywords(),
            phishing_patterns: Self::load_phishing_patterns(),
            suspicious_extensions: Self::load_suspicious_extensions(),
            urgent_keywords: Self::load_urgent_keywords(),
        })
    }

    async fn scan(
        &self,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
    ) -> Result<Self::Result> {
        let start_time = std::time::Instant::now();

        // Parse email content
        let email_text = String::from_utf8(data.to_vec())
            .map_err(|_| anyhow!("Invalid UTF-8 in email"))?;

        info!("Starting email scan ({} bytes)", data.len());

        let mut base_result = ScanResult::new(ArtifactType::Email);

        // Parse email headers and body
        let (headers, body) = self.parse_email(&email_text)?;

        // Extract email information
        let email_info = self.extract_email_info(&headers);

        // Analyze headers
        let header_analysis = self.analyze_headers(&headers, &email_info);

        if header_analysis.has_suspicious_headers {
            base_result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "Suspicious email headers detected".to_string(),
                description: format!(
                    "{} suspicious headers found",
                    header_analysis.suspicious_headers.len()
                ),
                severity: ThreatLevel::Medium,
                evidence: header_analysis.suspicious_headers.clone(),
                recommendation: Some("Verify sender authenticity".to_string()),
            });
        }

        if header_analysis.is_forged {
            base_result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Phishing,
                title: "Forged email headers".to_string(),
                description: "Email headers show signs of forgery".to_string(),
                severity: ThreatLevel::High,
                evidence: vec!["Header forgery detected".to_string()],
                recommendation: Some("Do not trust this email".to_string()),
            });
        }

        // Check email authentication
        let authentication_results = if self.config.check_spf || self.config.check_dkim || self.config.check_dmarc {
            self.check_authentication(&headers)
        } else {
            AuthenticationResults {
                spf_result: AuthResult::None,
                dkim_result: AuthResult::None,
                dmarc_result: AuthResult::None,
                is_authenticated: false,
            }
        };

        if !authentication_results.is_authenticated {
            let severity = if authentication_results.spf_result == AuthResult::Fail
                || authentication_results.dkim_result == AuthResult::Fail
            {
                ThreatLevel::High
            } else {
                ThreatLevel::Medium
            };

            base_result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "Email authentication failed".to_string(),
                description: format!(
                    "SPF: {:?}, DKIM: {:?}, DMARC: {:?}",
                    authentication_results.spf_result,
                    authentication_results.dkim_result,
                    authentication_results.dmarc_result
                ),
                severity,
                evidence: vec!["Failed authentication".to_string()],
                recommendation: Some("Verify sender identity".to_string()),
            });
        }

        // Analyze content
        let content_analysis = self.analyze_content(&body);

        if content_analysis.has_suspicious_content {
            base_result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Phishing,
                title: "Suspicious email content".to_string(),
                description: format!(
                    "{} suspicious patterns detected",
                    content_analysis.suspicious_patterns.len()
                ),
                severity: ThreatLevel::High,
                evidence: content_analysis.suspicious_patterns.clone(),
                recommendation: Some("Do not click links or download attachments".to_string()),
            });
        }

        if !content_analysis.urgency_indicators.is_empty() {
            base_result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Phishing,
                title: "Urgency tactics detected".to_string(),
                description: "Email uses urgency to pressure action".to_string(),
                severity: ThreatLevel::Medium,
                evidence: content_analysis.urgency_indicators.clone(),
                recommendation: Some("Phishing emails often create false urgency".to_string()),
            });
        }

        // Extract and analyze URLs
        let extracted_urls = if self.config.extract_urls {
            self.extract_urls(&body)
        } else {
            Vec::new()
        };

        if !extracted_urls.is_empty() {
            // Check for suspicious URLs
            let suspicious_urls: Vec<String> = extracted_urls
                .iter()
                .filter(|url| self.is_suspicious_url(url))
                .cloned()
                .collect();

            if !suspicious_urls.is_empty() {
                base_result.add_finding(Finding {
                    finding_id: Uuid::new_v4(),
                    category: FindingCategory::Phishing,
                    title: "Suspicious URLs detected".to_string(),
                    description: format!("{} suspicious URLs found", suspicious_urls.len()),
                    severity: ThreatLevel::High,
                    evidence: suspicious_urls,
                    recommendation: Some("Do not click these links".to_string()),
                });
            }
        }

        // Scan attachments
        let attachments = if self.config.scan_attachments {
            self.scan_attachments(&email_text)?
        } else {
            Vec::new()
        };

        for attachment in &attachments {
            if attachment.is_suspicious {
                base_result.add_finding(Finding {
                    finding_id: Uuid::new_v4(),
                    category: FindingCategory::Malware,
                    title: "Suspicious attachment detected".to_string(),
                    description: format!("Attachment: {}", attachment.filename),
                    severity: ThreatLevel::High,
                    evidence: vec![
                        format!("Filename: {}", attachment.filename),
                        format!("Type: {}", attachment.mime_type),
                        format!("Size: {}", attachment.size),
                    ],
                    recommendation: Some("Do not open attachment".to_string()),
                });
            }
        }

        // Calculate spam score
        let spam_score = self.calculate_spam_score(
            &content_analysis,
            &header_analysis,
            &authentication_results,
            &attachments,
        );

        if spam_score > 5.0 {
            base_result.add_finding(Finding {
                finding_id: Uuid::new_v4(),
                category: FindingCategory::Suspicious,
                title: "High spam score".to_string(),
                description: format!("Spam score: {:.1}/10", spam_score),
                severity: ThreatLevel::Medium,
                evidence: vec![format!("Score: {:.1}", spam_score)],
                recommendation: Some("Likely spam email".to_string()),
            });
        }

        let scan_duration_ms = start_time.elapsed().as_millis() as u64;
        base_result.scan_duration_ms = scan_duration_ms;

        info!(
            "Email scan completed in {}ms - Verdict: {:?}, Spam score: {:.1}",
            scan_duration_ms, base_result.verdict, spam_score
        );

        Ok(EmailScanResult {
            base: base_result,
            email_info,
            header_analysis,
            authentication_results,
            content_analysis,
            attachments,
            extracted_urls,
            spam_score,
        })
    }

    fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("scanner_name".to_string(), self.config.base.scanner_name.clone());
        stats.insert("spam_keywords".to_string(), self.spam_keywords.len().to_string());
        stats.insert("phishing_patterns".to_string(), self.phishing_patterns.len().to_string());
        stats
    }

    fn health_check(&self) -> bool {
        self.config.base.enabled
    }
}

impl EmailScanner {
    /// Parse email into headers and body
    fn parse_email(&self, email: &str) -> Result<(HashMap<String, String>, String)> {
        let mut headers = HashMap::new();
        let mut body = String::new();
        let mut in_body = false;

        for line in email.lines() {
            if in_body {
                body.push_str(line);
                body.push('\n');
            } else if line.is_empty() {
                in_body = true;
            } else if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }

        Ok((headers, body))
    }

    /// Extract email information
    fn extract_email_info(&self, headers: &HashMap<String, String>) -> EmailInfo {
        EmailInfo {
            from: headers.get("from").cloned().unwrap_or_default(),
            to: headers
                .get("to")
                .map(|s| vec![s.clone()])
                .unwrap_or_default(),
            subject: headers.get("subject").cloned().unwrap_or_default(),
            date: headers.get("date").cloned(),
            message_id: headers.get("message-id").cloned(),
            reply_to: headers.get("reply-to").cloned(),
            return_path: headers.get("return-path").cloned(),
        }
    }

    /// Analyze email headers
    fn analyze_headers(&self, headers: &HashMap<String, String>, email_info: &EmailInfo) -> HeaderAnalysis {
        let mut suspicious_headers = Vec::new();
        let mut is_forged = false;

        // Check for mismatched From and Reply-To
        if let Some(reply_to) = &email_info.reply_to {
            if !email_info.from.is_empty() && !email_info.from.contains(reply_to) {
                suspicious_headers.push("Mismatched From and Reply-To".to_string());
            }
        }

        // Check for mismatched From and Return-Path
        if let Some(return_path) = &email_info.return_path {
            if !email_info.from.is_empty() && !email_info.from.contains(return_path) {
                suspicious_headers.push("Mismatched From and Return-Path".to_string());
                is_forged = true;
            }
        }

        // Check received hops (simplified)
        let received_hops = headers
            .iter()
            .filter(|(k, _)| k.starts_with("received"))
            .count();

        if received_hops > 10 {
            suspicious_headers.push(format!("Excessive mail hops: {}", received_hops));
        }

        // Check for missing important headers
        if !headers.contains_key("date") {
            suspicious_headers.push("Missing Date header".to_string());
        }
        if !headers.contains_key("message-id") {
            suspicious_headers.push("Missing Message-ID".to_string());
        }

        HeaderAnalysis {
            has_suspicious_headers: !suspicious_headers.is_empty(),
            suspicious_headers,
            header_count: headers.len(),
            received_hops,
            is_forged,
        }
    }

    /// Check email authentication (SPF, DKIM, DMARC)
    fn check_authentication(&self, headers: &HashMap<String, String>) -> AuthenticationResults {
        // Simplified authentication check - in production, validate properly
        let auth_results = headers.get("authentication-results").cloned().unwrap_or_default().to_lowercase();

        let spf_result = if auth_results.contains("spf=pass") {
            AuthResult::Pass
        } else if auth_results.contains("spf=fail") {
            AuthResult::Fail
        } else {
            AuthResult::None
        };

        let dkim_result = if auth_results.contains("dkim=pass") {
            AuthResult::Pass
        } else if auth_results.contains("dkim=fail") {
            AuthResult::Fail
        } else {
            AuthResult::None
        };

        let dmarc_result = if auth_results.contains("dmarc=pass") {
            AuthResult::Pass
        } else if auth_results.contains("dmarc=fail") {
            AuthResult::Fail
        } else {
            AuthResult::None
        };

        let is_authenticated = spf_result == AuthResult::Pass
            || dkim_result == AuthResult::Pass
            || dmarc_result == AuthResult::Pass;

        AuthenticationResults {
            spf_result,
            dkim_result,
            dmarc_result,
            is_authenticated,
        }
    }

    /// Analyze email content
    fn analyze_content(&self, body: &str) -> ContentAnalysis {
        let body_lower = body.to_lowercase();
        let mut suspicious_patterns = Vec::new();
        let mut urgency_indicators = Vec::new();

        // Check for spam keywords
        for keyword in &self.spam_keywords {
            if body_lower.contains(keyword) {
                suspicious_patterns.push(format!("Spam keyword: {}", keyword));
            }
        }

        // Check for phishing patterns
        for pattern in &self.phishing_patterns {
            if body_lower.contains(pattern) {
                suspicious_patterns.push(format!("Phishing pattern: {}", pattern));
            }
        }

        // Check for urgency indicators
        for keyword in &self.urgent_keywords {
            if body_lower.contains(keyword) {
                urgency_indicators.push(keyword.clone());
            }
        }

        ContentAnalysis {
            body_text: body.chars().take(500).collect(), // First 500 chars
            body_html: None,
            has_suspicious_content: !suspicious_patterns.is_empty(),
            suspicious_patterns,
            language: None,
            urgency_indicators,
        }
    }

    /// Extract URLs from email body
    fn extract_urls(&self, body: &str) -> Vec<String> {
        let mut urls = Vec::new();
        let url_patterns = [
            regex::Regex::new(r"https?://[^\s<>\"]+").unwrap(),
            regex::Regex::new(r"www\.[^\s<>\"]+").unwrap(),
        ];

        for pattern in &url_patterns {
            for capture in pattern.find_iter(body) {
                urls.push(capture.as_str().to_string());
            }
        }

        urls
    }

    /// Check if URL is suspicious
    fn is_suspicious_url(&self, url: &str) -> bool {
        let url_lower = url.to_lowercase();

        // Check for suspicious patterns
        url_lower.contains("phishing")
            || url_lower.contains("malware")
            || url_lower.contains("@") // URLs with @ can be deceptive
            || url.matches('.').count() > 5 // Too many dots
            || url.len() > 200 // Excessively long
    }

    /// Scan email attachments
    fn scan_attachments(&self, email: &str) -> Result<Vec<AttachmentInfo>> {
        let mut attachments = Vec::new();

        // Simplified attachment detection - in production, parse MIME properly
        if email.contains("Content-Disposition: attachment") {
            // Extract attachment info (simplified)
            for line in email.lines() {
                if line.contains("filename=") {
                    if let Some(start) = line.find("filename=") {
                        let filename_part = &line[start + 9..];
                        let filename = filename_part
                            .trim_matches(|c| c == '"' || c == '\'' || c == ';')
                            .trim()
                            .to_string();

                        let extension = std::path::Path::new(&filename)
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("");

                        let is_suspicious = self.suspicious_extensions.contains(&extension.to_string());

                        attachments.push(AttachmentInfo {
                            filename: filename.clone(),
                            size: 0, // Would parse from MIME
                            mime_type: "application/octet-stream".to_string(),
                            is_suspicious,
                            hash: String::new(),
                            scan_result: None,
                        });
                    }
                }
            }
        }

        Ok(attachments)
    }

    /// Calculate spam score (0-10)
    fn calculate_spam_score(
        &self,
        content: &ContentAnalysis,
        headers: &HeaderAnalysis,
        auth: &AuthenticationResults,
        attachments: &[AttachmentInfo],
    ) -> f32 {
        let mut score = 0.0;

        // Content-based scoring
        score += content.suspicious_patterns.len() as f32 * 1.0;
        score += content.urgency_indicators.len() as f32 * 0.5;

        // Header-based scoring
        score += headers.suspicious_headers.len() as f32 * 1.5;
        if headers.is_forged {
            score += 3.0;
        }

        // Authentication-based scoring
        if auth.spf_result == AuthResult::Fail {
            score += 2.0;
        }
        if auth.dkim_result == AuthResult::Fail {
            score += 2.0;
        }
        if !auth.is_authenticated {
            score += 1.0;
        }

        // Attachment-based scoring
        for attachment in attachments {
            if attachment.is_suspicious {
                score += 2.0;
            }
        }

        score.min(10.0)
    }

    /// Load spam keywords
    fn load_spam_keywords() -> Vec<String> {
        vec![
            "viagra".to_string(),
            "cialis".to_string(),
            "lottery".to_string(),
            "winner".to_string(),
            "congratulations".to_string(),
            "million dollars".to_string(),
            "nigerian prince".to_string(),
        ]
    }

    /// Load phishing patterns
    fn load_phishing_patterns() -> Vec<String> {
        vec![
            "verify your account".to_string(),
            "suspended account".to_string(),
            "confirm your identity".to_string(),
            "click here immediately".to_string(),
            "unusual activity".to_string(),
            "update your password".to_string(),
        ]
    }

    /// Load suspicious file extensions
    fn load_suspicious_extensions() -> Vec<String> {
        vec![
            "exe".to_string(),
            "scr".to_string(),
            "bat".to_string(),
            "cmd".to_string(),
            "com".to_string(),
            "pif".to_string(),
            "vbs".to_string(),
            "js".to_string(),
        ]
    }

    /// Load urgent keywords
    fn load_urgent_keywords() -> Vec<String> {
        vec![
            "urgent".to_string(),
            "immediate action".to_string(),
            "act now".to_string(),
            "limited time".to_string(),
            "expires".to_string(),
            "24 hours".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_email_scanner_creation() {
        let config = EmailScannerConfig::default();
        let scanner = EmailScanner::new(config);
        assert!(scanner.is_ok());
    }

    #[tokio::test]
    async fn test_scan_phishing_email() {
        let scanner = EmailScanner::new(EmailScannerConfig::default()).unwrap();
        let email = b"From: support@paypal-verify.com\nTo: victim@example.com\nSubject: Urgent: Verify your account\n\nYour account has been suspended. Click here immediately to verify your identity.";

        let result = scanner.scan(email, None).await;
        assert!(result.is_ok());

        let scan_result = result.unwrap();
        assert!(!scan_result.content_analysis.suspicious_patterns.is_empty());
        assert!(!scan_result.content_analysis.urgency_indicators.is_empty());
    }

    #[test]
    fn test_url_extraction() {
        let scanner = EmailScanner::new(EmailScannerConfig::default()).unwrap();
        let body = "Visit https://example.com and www.test.com for more info";
        let urls = scanner.extract_urls(body);

        assert!(urls.contains(&"https://example.com".to_string()));
        assert!(urls.iter().any(|u| u.contains("www.test.com")));
    }

    #[test]
    fn test_spam_score_calculation() {
        let scanner = EmailScanner::new(EmailScannerConfig::default()).unwrap();

        let content = ContentAnalysis {
            body_text: String::new(),
            body_html: None,
            has_suspicious_content: true,
            suspicious_patterns: vec!["spam".to_string(), "phishing".to_string()],
            language: None,
            urgency_indicators: vec!["urgent".to_string()],
        };

        let headers = HeaderAnalysis {
            has_suspicious_headers: false,
            suspicious_headers: vec![],
            header_count: 10,
            received_hops: 2,
            is_forged: false,
        };

        let auth = AuthenticationResults {
            spf_result: AuthResult::Pass,
            dkim_result: AuthResult::Pass,
            dmarc_result: AuthResult::Pass,
            is_authenticated: true,
        };

        let score = scanner.calculate_spam_score(&content, &headers, &auth, &[]);
        assert!(score > 0.0);
    }
}
