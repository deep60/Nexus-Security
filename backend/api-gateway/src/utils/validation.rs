use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;
use url::Url;
use std::net::IpAddr;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    #[error("Invalid file hash: {0}")]
    InvalidFileHash(String),
    #[error("Invalid file size: {size} bytes (max: {max_size} bytes)")]
    InvalidFileSize { size: u64, max_size: u64 },
    #[error("Invalid file type: {file_type} (allowed: {allowed:?})")]
    InvalidFileType { file_type: String, allowed: Vec<String> },
    #[error("Invalid ethereum address: {0}")]
    InvalidEthereumAddress(String),
    #[error("Invalid bounty amount: {0}")]
    InvalidBountyAmount(String),
    #[error("Invalid reputation score: {0}")]
    InvalidReputationScore(String),
    #[error("Field too short: minimum length {min}, got {actual}")]
    TooShort { min: usize, actual: usize },
    #[error("Field too long: maximum length {max}, got {actual}")]
    TooLong { max: usize, actual: usize },
    #[error("Invalid characters in field")]
    InvalidCharacters,
    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),
    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("Invalid UUID format: {0}")]
    InvalidUuid(String),
    #[error("Value out of range: {value} (min: {min}, max: {max})")]
    ValueOutOfRange { value: f64, min: f64, max: f64 },
}

pub type ValidationResult<T> = Result<T, ValidationError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileValidationRules {
    pub max_size_mb: u64,
    pub allowed_extensions: Vec<String>,
    pub allowed_mime_types: Vec<String>,
    pub require_hash_verification: bool,
}

impl Default for FileValidationRules {
    fn default() -> Self {
        Self {
            max_size_mb: 100, // 100MB default
            allowed_extensions: vec![
                "exe".to_string(), "dll".to_string(), "pdf".to_string(),
                "doc".to_string(), "docx".to_string(), "zip".to_string(),
                "rar".to_string(), "7z".to_string(), "tar".to_string(),
                "gz".to_string(), "bin".to_string(), "apk".to_string(),
                "ipa".to_string(), "msi".to_string(), "dmg".to_string(),
            ],
            allowed_mime_types: vec![
                "application/octet-stream".to_string(),
                "application/x-msdownload".to_string(),
                "application/pdf".to_string(),
                "application/zip".to_string(),
                "application/x-rar-compressed".to_string(),
                "application/x-7z-compressed".to_string(),
                "application/vnd.android.package-archive".to_string(),
            ],
            require_hash_verification: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyValidationRules {
    pub min_amount_wei: u64,
    pub max_amount_wei: u64,
    pub max_description_length: usize,
    pub max_title_length: usize,
    pub allowed_analysis_types: Vec<String>,
}

impl Default for BountyValidationRules {
    fn default() -> Self {
        Self {
            min_amount_wei: 1_000_000_000_000_000, // 0.001 ETH minimum
            max_amount_wei: 1_000_000_000_000_000_000_000, // 1000 ETH maximum
            max_description_length: 5000,
            max_title_length: 200,
            allowed_analysis_types: vec![
                "malware_detection".to_string(),
                "vulnerability_assessment".to_string(),
                "behavioral_analysis".to_string(),
                "static_analysis".to_string(),
                "dynamic_analysis".to_string(),
                "reputation_check".to_string(),
            ],
        }
    }
}

/// Email validation utilities
pub struct EmailValidator;

impl EmailValidator {
    /// Validate email format using regex
    pub fn validate(email: &str) -> ValidationResult<()> {
        let email_regex = Regex::new(
            r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
        ).unwrap();
        
        if email_regex.is_match(email) {
            Ok(())
        } else {
            Err(ValidationError::InvalidEmail(email.to_string()))
        }
    }

    /// Check if email domain is in allowed list
    pub fn validate_domain(email: &str, allowed_domains: &[String]) -> ValidationResult<()> {
        Self::validate(email)?;
        
        let domain = email.split('@').nth(1).unwrap_or("");
        if allowed_domains.is_empty() || allowed_domains.contains(&domain.to_string()) {
            Ok(())
        } else {
            Err(ValidationError::InvalidEmail(format!(
                "Domain {} not in allowed list", domain
            )))
        }
    }
}

/// URL validation utilities
pub struct UrlValidator;

impl UrlValidator {
    /// Validate URL format
    pub fn validate(url_str: &str) -> ValidationResult<Url> {
        match Url::parse(url_str) {
            Ok(url) => {
                // Only allow HTTP and HTTPS schemes
                if url.scheme() == "http" || url.scheme() == "https" {
                    Ok(url)
                } else {
                    Err(ValidationError::InvalidUrl(format!(
                        "Only HTTP and HTTPS schemes allowed, got: {}", url.scheme()
                    )))
                }
            }
            Err(_) => Err(ValidationError::InvalidUrl(url_str.to_string())),
        }
    }

    /// Validate if URL is not pointing to localhost/private networks
    pub fn validate_public_url(url_str: &str) -> ValidationResult<Url> {
        let url = Self::validate(url_str)?;
        
        if let Some(host) = url.host_str() {
            // Check for localhost
            if host == "localhost" || host == "127.0.0.1" || host.starts_with("192.168.") 
                || host.starts_with("10.") || host.starts_with("172.") {
                return Err(ValidationError::InvalidUrl(
                    "Private/localhost URLs not allowed".to_string()
                ));
            }
        }
        
        Ok(url)
    }
}

/// File validation utilities
pub struct FileValidator;

impl FileValidator {
    /// Validate file size
    pub fn validate_size(size_bytes: u64, rules: &FileValidationRules) -> ValidationResult<()> {
        let max_bytes = rules.max_size_mb * 1024 * 1024;
        if size_bytes <= max_bytes {
            Ok(())
        } else {
            Err(ValidationError::InvalidFileSize {
                size: size_bytes,
                max_size: max_bytes,
            })
        }
    }

    /// Validate file extension
    pub fn validate_extension(filename: &str, rules: &FileValidationRules) -> ValidationResult<()> {
        let extension = filename.split('.').last().unwrap_or("").to_lowercase();
        
        if rules.allowed_extensions.is_empty() || 
           rules.allowed_extensions.contains(&extension) {
            Ok(())
        } else {
            Err(ValidationError::InvalidFileType {
                file_type: extension,
                allowed: rules.allowed_extensions.clone(),
            })
        }
    }

    /// Validate MIME type
    pub fn validate_mime_type(mime_type: &str, rules: &FileValidationRules) -> ValidationResult<()> {
        if rules.allowed_mime_types.is_empty() || 
           rules.allowed_mime_types.contains(&mime_type.to_string()) {
            Ok(())
        } else {
            Err(ValidationError::InvalidFileType {
                file_type: mime_type.to_string(),
                allowed: rules.allowed_mime_types.clone(),
            })
        }
    }

    /// Validate complete file metadata
    pub fn validate_file(
        filename: &str,
        size_bytes: u64,
        mime_type: &str,
        rules: &FileValidationRules,
    ) -> ValidationResult<()> {
        Self::validate_size(size_bytes, rules)?;
        Self::validate_extension(filename, rules)?;
        Self::validate_mime_type(mime_type, rules)?;
        Ok(())
    }
}

/// Hash validation utilities
pub struct HashValidator;

impl HashValidator {
    /// Validate SHA-256 hash format
    pub fn validate_sha256(hash: &str) -> ValidationResult<()> {
        if hash.len() != 64 {
            return Err(ValidationError::InvalidFileHash(
                "SHA-256 hash must be 64 characters".to_string()
            ));
        }
        
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidFileHash(
                "Hash contains invalid characters".to_string()
            ));
        }
        
        Ok(())
    }

    /// Validate MD5 hash format
    pub fn validate_md5(hash: &str) -> ValidationResult<()> {
        if hash.len() != 32 {
            return Err(ValidationError::InvalidFileHash(
                "MD5 hash must be 32 characters".to_string()
            ));
        }
        
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidFileHash(
                "Hash contains invalid characters".to_string()
            ));
        }
        
        Ok(())
    }

    /// Validate SHA-1 hash format
    pub fn validate_sha1(hash: &str) -> ValidationResult<()> {
        if hash.len() != 40 {
            return Err(ValidationError::InvalidFileHash(
                "SHA-1 hash must be 40 characters".to_string()
            ));
        }
        
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidFileHash(
                "Hash contains invalid characters".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Blockchain-specific validation utilities
pub struct BlockchainValidator;

impl BlockchainValidator {
    /// Validate Ethereum address format
    pub fn validate_ethereum_address(address: &str) -> ValidationResult<()> {
        if !address.starts_with("0x") || address.len() != 42 {
            return Err(ValidationError::InvalidEthereumAddress(address.to_string()));
        }
        
        let addr_part = &address[2..];
        if !addr_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidEthereumAddress(address.to_string()));
        }
        
        Ok(())
    }

    /// Validate bounty amount in wei
    pub fn validate_bounty_amount(amount_wei: u64, rules: &BountyValidationRules) -> ValidationResult<()> {
        if amount_wei < rules.min_amount_wei {
            return Err(ValidationError::InvalidBountyAmount(format!(
                "Amount {} wei is below minimum {}", amount_wei, rules.min_amount_wei
            )));
        }
        
        if amount_wei > rules.max_amount_wei {
            return Err(ValidationError::InvalidBountyAmount(format!(
                "Amount {} wei exceeds maximum {}", amount_wei, rules.max_amount_wei
            )));
        }
        
        Ok(())
    }

    /// Validate transaction hash
    pub fn validate_transaction_hash(tx_hash: &str) -> ValidationResult<()> {
        if !tx_hash.starts_with("0x") || tx_hash.len() != 66 {
            return Err(ValidationError::InvalidFileHash(
                "Transaction hash must be 66 characters starting with 0x".to_string()
            ));
        }
        
        let hash_part = &tx_hash[2..];
        if !hash_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidFileHash(
                "Transaction hash contains invalid characters".to_string()
            ));
        }
        
        Ok(())
    }
}

/// String validation utilities
pub struct StringValidator;

impl StringValidator {
    /// Validate string length
    pub fn validate_length(s: &str, min: usize, max: usize) -> ValidationResult<()> {
        let len = s.len();
        if len < min {
            Err(ValidationError::TooShort { min, actual: len })
        } else if len > max {
            Err(ValidationError::TooLong { max, actual: len })
        } else {
            Ok(())
        }
    }

    /// Validate that string contains only alphanumeric characters and allowed symbols
    pub fn validate_alphanumeric_with_symbols(s: &str, allowed_symbols: &str) -> ValidationResult<()> {
        let is_valid = s.chars().all(|c| {
            c.is_alphanumeric() || allowed_symbols.contains(c)
        });
        
        if is_valid {
            Ok(())
        } else {
            Err(ValidationError::InvalidCharacters)
        }
    }

    /// Validate username format
    pub fn validate_username(username: &str) -> ValidationResult<()> {
        Self::validate_length(username, 3, 50)?;
        Self::validate_alphanumeric_with_symbols(username, "_-.")?;
        
        // Username cannot start or end with symbols
        if username.starts_with(|c: char| !c.is_alphanumeric()) ||
           username.ends_with(|c: char| !c.is_alphanumeric()) {
            return Err(ValidationError::InvalidCharacters);
        }
        
        Ok(())
    }

    /// Validate that string doesn't contain profanity or malicious content
    pub fn validate_safe_content(content: &str, blocked_words: &HashSet<String>) -> ValidationResult<()> {
        let content_lower = content.to_lowercase();
        for word in blocked_words {
            if content_lower.contains(word) {
                return Err(ValidationError::InvalidCharacters);
            }
        }
        Ok(())
    }
}

/// Numeric validation utilities
pub struct NumericValidator;

impl NumericValidator {
    /// Validate that a number is within a specific range
    pub fn validate_range(value: f64, min: f64, max: f64) -> ValidationResult<()> {
        if value < min || value > max {
            Err(ValidationError::ValueOutOfRange { value, min, max })
        } else {
            Ok(())
        }
    }

    /// Validate reputation score (0-1000)
    pub fn validate_reputation_score(score: u32) -> ValidationResult<()> {
        if score <= 1000 {
            Ok(())
        } else {
            Err(ValidationError::InvalidReputationScore(format!(
                "Reputation score {} exceeds maximum 1000", score
            )))
        }
    }

    /// Validate confidence score (0.0-1.0)
    pub fn validate_confidence_score(score: f64) -> ValidationResult<()> {
        Self::validate_range(score, 0.0, 1.0)?;
        Ok(())
    }

    /// Validate timestamp (Unix timestamp)
    pub fn validate_timestamp(timestamp: i64) -> ValidationResult<()> {
        let current_time = chrono::Utc::now().timestamp();
        let one_year_ago = current_time - (365 * 24 * 60 * 60);
        let one_year_future = current_time + (365 * 24 * 60 * 60);
        
        if timestamp < one_year_ago || timestamp > one_year_future {
            Err(ValidationError::InvalidTimestamp(format!(
                "Timestamp {} is outside reasonable range", timestamp
            )))
        } else {
            Ok(())
        }
    }
}

/// IP address validation utilities
pub struct IpValidator;

impl IpValidator {
    /// Validate IP address format
    pub fn validate_ip_address(ip_str: &str) -> ValidationResult<IpAddr> {
        match ip_str.parse::<IpAddr>() {
            Ok(ip) => Ok(ip),
            Err(_) => Err(ValidationError::InvalidIpAddress(ip_str.to_string())),
        }
    }

    /// Check if IP is in private range
    pub fn is_private_ip(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_private() || ipv4.is_loopback() || ipv4.is_link_local()
            },
            IpAddr::V6(ipv6) => {
                ipv6.is_loopback() || ipv6.is_multicast()
            }
        }
    }

    /// Validate public IP address
    pub fn validate_public_ip(ip_str: &str) -> ValidationResult<IpAddr> {
        let ip = Self::validate_ip_address(ip_str)?;
        if Self::is_private_ip(&ip) {
            Err(ValidationError::InvalidIpAddress(
                "Private IP addresses not allowed".to_string()
            ))
        } else {
            Ok(ip)
        }
    }
}

/// UUID validation utilities
pub struct UuidValidator;

impl UuidValidator {
    /// Validate UUID format
    pub fn validate_uuid(uuid_str: &str) -> ValidationResult<uuid::Uuid> {
        match uuid::Uuid::parse_str(uuid_str) {
            Ok(uuid) => Ok(uuid),
            Err(_) => Err(ValidationError::InvalidUuid(uuid_str.to_string())),
        }
    }

    /// Validate UUID v4 specifically
    pub fn validate_uuid_v4(uuid_str: &str) -> ValidationResult<uuid::Uuid> {
        let uuid = Self::validate_uuid(uuid_str)?;
        if uuid.get_version() == Some(uuid::Version::Random) {
            Ok(uuid)
        } else {
            Err(ValidationError::InvalidUuid(format!(
                "Expected UUID v4, got {:?}", uuid.get_version()
            )))
        }
    }
}

/// Bounty-specific validation utilities
pub struct BountyValidator;

impl BountyValidator {
    /// Validate bounty creation data
    pub fn validate_bounty_creation(
        title: &str,
        description: &str,
        amount_wei: u64,
        analysis_type: &str,
        rules: &BountyValidationRules,
    ) -> ValidationResult<()> {
        // Validate title
        StringValidator::validate_length(title, 5, rules.max_title_length)?;
        
        // Validate description
        StringValidator::validate_length(description, 10, rules.max_description_length)?;
        
        // Validate amount
        BlockchainValidator::validate_bounty_amount(amount_wei, rules)?;
        
        // Validate analysis type
        if !rules.allowed_analysis_types.contains(&analysis_type.to_string()) {
            return Err(ValidationError::InvalidFileType {
                file_type: analysis_type.to_string(),
                allowed: rules.allowed_analysis_types.clone(),
            });
        }
        
        Ok(())
    }

    /// Validate submission data
    pub fn validate_submission(
        bounty_id: &str,
        analysis_result: &str,
        confidence_score: f64,
        is_malicious: bool,
    ) -> ValidationResult<()> {
        // Validate bounty ID format (UUID)
        UuidValidator::validate_uuid_v4(bounty_id)?;
        
        // Validate analysis result length
        StringValidator::validate_length(analysis_result, 10, 10000)?;
        
        // Validate confidence score
        NumericValidator::validate_confidence_score(confidence_score)?;
        
        // Additional logic validation
        if confidence_score > 0.9 && analysis_result.len() < 100 {
            return Err(ValidationError::InvalidCharacters); // High confidence needs detailed analysis
        }
        
        Ok(())
    }
}

/// API request validation utilities
pub struct ApiValidator;

impl ApiValidator {
    /// Validate API key format
    pub fn validate_api_key(api_key: &str) -> ValidationResult<()> {
        if api_key.len() != 64 {
            return Err(ValidationError::InvalidCharacters);
        }
        
        if !api_key.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidCharacters);
        }
        
        Ok(())
    }

    /// Validate pagination parameters
    pub fn validate_pagination(page: u32, page_size: u32) -> ValidationResult<()> {
        if page < 1 {
            return Err(ValidationError::ValueOutOfRange {
                value: page as f64,
                min: 1.0,
                max: f64::MAX,
            });
        }
        
        if page_size < 1 || page_size > 100 {
            return Err(ValidationError::ValueOutOfRange {
                value: page_size as f64,
                min: 1.0,
                max: 100.0,
            });
        }
        
        Ok(())
    }

    /// Validate sorting parameters
    pub fn validate_sort_params(sort_by: &str, sort_order: &str, allowed_fields: &[&str]) -> ValidationResult<()> {
        if !allowed_fields.contains(&sort_by) {
            return Err(ValidationError::InvalidCharacters);
        }
        
        if sort_order != "asc" && sort_order != "desc" {
            return Err(ValidationError::InvalidCharacters);
        }
        
        Ok(())
    }
}

/// Complete validation suite for common operations
pub struct ValidationSuite;

impl ValidationSuite {
    /// Validate file upload request
    pub fn validate_file_upload(
        filename: &str,
        file_size: u64,
        mime_type: &str,
        uploader_address: &str,
        file_hash: Option<&str>,
    ) -> ValidationResult<()> {
        let file_rules = FileValidationRules::default();
        
        // Validate file metadata
        FileValidator::validate_file(filename, file_size, mime_type, &file_rules)?;
        
        // Validate uploader address
        BlockchainValidator::validate_ethereum_address(uploader_address)?;
        
        // Validate hash if provided
        if let Some(hash) = file_hash {
            HashValidator::validate_sha256(hash)?;
        }
        
        Ok(())
    }

    /// Validate user registration
    pub fn validate_user_registration(
        username: &str,
        email: &str,
        ethereum_address: &str,
    ) -> ValidationResult<()> {
        StringValidator::validate_username(username)?;
        EmailValidator::validate(email)?;
        BlockchainValidator::validate_ethereum_address(ethereum_address)?;
        Ok(())
    }

    /// Validate analysis result submission
    pub fn validate_analysis_submission(
        bounty_id: &str,
        analyst_address: &str,
        result: &str,
        confidence: f64,
        stake_amount: u64,
    ) -> ValidationResult<()> {
        UuidValidator::validate_uuid_v4(bounty_id)?;
        BlockchainValidator::validate_ethereum_address(analyst_address)?;
        StringValidator::validate_length(result, 50, 5000)?;
        NumericValidator::validate_confidence_score(confidence)?;
        
        // Validate minimum stake amount
        if stake_amount < 1000000000000000 { // 0.001 ETH minimum
            return Err(ValidationError::InvalidBountyAmount(
                "Stake amount too low".to_string()
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(EmailValidator::validate("test@example.com").is_ok());
        assert!(EmailValidator::validate("invalid.email").is_err());
        assert!(EmailValidator::validate("user@domain").is_err());
    }

    #[test]
    fn test_url_validation() {
        assert!(UrlValidator::validate("https://example.com").is_ok());
        assert!(UrlValidator::validate("http://example.com/path").is_ok());
        assert!(UrlValidator::validate("ftp://example.com").is_err());
        assert!(UrlValidator::validate("invalid-url").is_err());
    }

    #[test]
    fn test_ethereum_address_validation() {
        assert!(BlockchainValidator::validate_ethereum_address("0x742d35Cc6435C2cb62fb0CF4cE385FA4d8457B5a").is_ok());
        assert!(BlockchainValidator::validate_ethereum_address("0x123").is_err());
        assert!(BlockchainValidator::validate_ethereum_address("invalid_address").is_err());
    }

    #[test]
    fn test_hash_validation() {
        let valid_sha256 = "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3";
        assert!(HashValidator::validate_sha256(valid_sha256).is_ok());
        assert!(HashValidator::validate_sha256("invalid_hash").is_err());
        assert!(HashValidator::validate_sha256("too_short").is_err());
    }

    #[test]
    fn test_file_size_validation() {
        let rules = FileValidationRules::default();
        assert!(FileValidator::validate_size(1024 * 1024, &rules).is_ok()); // 1MB
        assert!(FileValidator::validate_size(200 * 1024 * 1024, &rules).is_err()); // 200MB
    }

    #[test]
    fn test_username_validation() {
        assert!(StringValidator::validate_username("valid_user123").is_ok());
        assert!(StringValidator::validate_username("ab").is_err()); // Too short
        assert!(StringValidator::validate_username("_invalid").is_err()); // Starts with symbol
        assert!(StringValidator::validate_username("invalid@").is_err()); // Invalid character
    }

    #[test]
    fn test_confidence_score_validation() {
        assert!(NumericValidator::validate_confidence_score(0.5).is_ok());
        assert!(NumericValidator::validate_confidence_score(1.0).is_ok());
        assert!(NumericValidator::validate_confidence_score(1.1).is_err());
        assert!(NumericValidator::validate_confidence_score(-0.1).is_err());
    }

    #[test]
    fn test_bounty_amount_validation() {
        let rules = BountyValidationRules::default();
        assert!(BlockchainValidator::validate_bounty_amount(1_000_000_000_000_000, &rules).is_ok());
        assert!(BlockchainValidator::validate_bounty_amount(100, &rules).is_err()); // Too small
    }

    #[test]
    fn test_ip_validation() {
        assert!(IpValidator::validate_ip_address("192.168.1.1").is_ok());
        assert!(IpValidator::validate_ip_address("invalid_ip").is_err());
        assert!(IpValidator::validate_public_ip("8.8.8.8").is_ok());
        assert!(IpValidator::validate_public_ip("192.168.1.1").is_err()); // Private IP
    }

    #[test]
    fn test_validation_suite() {
        assert!(ValidationSuite::validate_file_upload(
            "test.exe",
            1024 * 1024, // 1MB
            "application/octet-stream",
            "0x742d35Cc6435C2cb62fb0CF4cE385FA4d8457B5a",
            Some("a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3")
        ).is_ok());

        assert!(ValidationSuite::validate_user_registration(
            "test_user",
            "test@example.com",
            "0x742d35Cc6435C2cb62fb0CF4cE385FA4d8457B5a"
        ).is_ok());
    }
}