/// Configuration module for the Analysis Engine
///
/// This module provides centralized configuration management with support for:
/// - Environment variable loading
/// - Default values
/// - Configuration validation
/// - Multiple service configurations (database, storage, scanners, etc.)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// Main configuration structure for the Analysis Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub storage: StorageConfig,
    pub scanners: ScannersConfig,
    pub sandbox: SandboxConfig,
    pub analyzers: AnalyzersConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if present

        Ok(Self {
            server: ServerConfig::from_env()?,
            database: DatabaseConfig::from_env()?,
            redis: RedisConfig::from_env()?,
            storage: StorageConfig::from_env()?,
            scanners: ScannersConfig::from_env()?,
            sandbox: SandboxConfig::from_env()?,
            analyzers: AnalyzersConfig::from_env()?,
            security: SecurityConfig::from_env()?,
            logging: LoggingConfig::from_env()?,
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        self.server.validate()?;
        self.database.validate()?;
        self.redis.validate()?;
        self.storage.validate()?;
        self.scanners.validate()?;
        self.sandbox.validate()?;
        self.analyzers.validate()?;
        self.security.validate()?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            redis: RedisConfig::default(),
            storage: StorageConfig::default(),
            scanners: ScannersConfig::default(),
            sandbox: SandboxConfig::default(),
            analyzers: AnalyzersConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_connections: usize,
    pub request_timeout_seconds: u64,
    pub enable_cors: bool,
    pub cors_allowed_origins: Vec<String>,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8002".to_string())
                .parse()
                .context("Invalid SERVER_PORT")?,
            workers: env::var("SERVER_WORKERS")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .context("Invalid SERVER_WORKERS")?,
            max_connections: env::var("SERVER_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .context("Invalid SERVER_MAX_CONNECTIONS")?,
            request_timeout_seconds: env::var("SERVER_REQUEST_TIMEOUT")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .context("Invalid SERVER_REQUEST_TIMEOUT")?,
            enable_cors: env::var("ENABLE_CORS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "*".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.port == 0 {
            anyhow::bail!("Server port cannot be 0");
        }
        if self.workers == 0 {
            anyhow::bail!("Server workers must be at least 1");
        }
        Ok(())
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8002,
            workers: 4,
            max_connections: 1000,
            request_timeout_seconds: 300,
            enable_cors: true,
            cors_allowed_origins: vec!["*".to_string()],
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_seconds: u64,
    pub acquire_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub enable_migrations: bool,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self> {
        let url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nexus:nexus_password@localhost:5432/nexus_analysis".to_string()
        });

        Ok(Self {
            url,
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .context("Invalid DB_MAX_CONNECTIONS")?,
            min_connections: env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("Invalid DB_MIN_CONNECTIONS")?,
            connection_timeout_seconds: env::var("DB_CONNECTION_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("Invalid DB_CONNECTION_TIMEOUT")?,
            acquire_timeout_seconds: env::var("DB_ACQUIRE_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("Invalid DB_ACQUIRE_TIMEOUT")?,
            idle_timeout_seconds: env::var("DB_IDLE_TIMEOUT")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .context("Invalid DB_IDLE_TIMEOUT")?,
            max_lifetime_seconds: env::var("DB_MAX_LIFETIME")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .context("Invalid DB_MAX_LIFETIME")?,
            enable_migrations: env::var("DB_ENABLE_MIGRATIONS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            anyhow::bail!("Database URL cannot be empty");
        }
        if self.max_connections == 0 {
            anyhow::bail!("Max connections must be at least 1");
        }
        if self.min_connections > self.max_connections {
            anyhow::bail!("Min connections cannot exceed max connections");
        }
        Ok(())
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://nexus:nexus_password@localhost:5432/nexus_analysis".to_string(),
            max_connections: 100,
            min_connections: 10,
            connection_timeout_seconds: 30,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: 600,
            max_lifetime_seconds: 3600,
            enable_migrations: true,
        }
    }
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
    pub response_timeout_seconds: u64,
    pub enable_cluster: bool,
}

impl RedisConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            max_connections: env::var("REDIS_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .context("Invalid REDIS_MAX_CONNECTIONS")?,
            connection_timeout_seconds: env::var("REDIS_CONNECTION_TIMEOUT")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("Invalid REDIS_CONNECTION_TIMEOUT")?,
            response_timeout_seconds: env::var("REDIS_RESPONSE_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("Invalid REDIS_RESPONSE_TIMEOUT")?,
            enable_cluster: env::var("REDIS_ENABLE_CLUSTER")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            anyhow::bail!("Redis URL cannot be empty");
        }
        Ok(())
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: 50,
            connection_timeout_seconds: 10,
            response_timeout_seconds: 30,
            enable_cluster: false,
        }
    }
}

/// Storage configuration (S3 + File System)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_use_ssl: bool,
    pub s3_path_style: bool,
    pub upload_dir: PathBuf,
    pub max_file_size_mb: u64,
    pub enable_cache: bool,
    pub cache_ttl_seconds: u64,
    pub cache_max_size_mb: usize,
}

impl StorageConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            s3_endpoint: env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            s3_region: env::var("S3_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            s3_bucket: env::var("S3_BUCKET")
                .unwrap_or_else(|_| "nexus-security-artifacts".to_string()),
            s3_access_key: env::var("S3_ACCESS_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            s3_secret_key: env::var("S3_SECRET_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            s3_use_ssl: env::var("S3_USE_SSL")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            s3_path_style: env::var("S3_PATH_STYLE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            upload_dir: PathBuf::from(
                env::var("UPLOAD_DIR").unwrap_or_else(|_| "./temp/nexus-uploads".to_string()),
            ),
            max_file_size_mb: env::var("MAX_FILE_SIZE_MB")
                .unwrap_or_else(|_| "500".to_string())
                .parse()
                .context("Invalid MAX_FILE_SIZE_MB")?,
            enable_cache: env::var("ENABLE_STORAGE_CACHE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            cache_ttl_seconds: env::var("CACHE_TTL_SECONDS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .context("Invalid CACHE_TTL_SECONDS")?,
            cache_max_size_mb: env::var("CACHE_MAX_SIZE_MB")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .context("Invalid CACHE_MAX_SIZE_MB")?,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.s3_bucket.is_empty() {
            anyhow::bail!("S3 bucket name cannot be empty");
        }
        if self.max_file_size_mb == 0 {
            anyhow::bail!("Max file size must be greater than 0");
        }
        Ok(())
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            s3_endpoint: "http://localhost:9000".to_string(),
            s3_region: "us-east-1".to_string(),
            s3_bucket: "nexus-security-artifacts".to_string(),
            s3_access_key: "minioadmin".to_string(),
            s3_secret_key: "minioadmin".to_string(),
            s3_use_ssl: false,
            s3_path_style: true,
            upload_dir: PathBuf::from("./temp/nexus-uploads"),
            max_file_size_mb: 500,
            enable_cache: true,
            cache_ttl_seconds: 300,
            cache_max_size_mb: 100,
        }
    }
}

/// Scanners configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannersConfig {
    pub enable_file_scanner: bool,
    pub enable_url_scanner: bool,
    pub enable_email_scanner: bool,
    pub enable_archive_scanner: bool,
    pub max_entropy_threshold: f64,
    pub min_suspicious_strings: usize,
    pub phishing_check_timeout_seconds: u64,
    pub max_archive_depth: usize,
    pub zip_bomb_ratio_threshold: f64,
}

impl ScannersConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            enable_file_scanner: env::var("ENABLE_FILE_SCANNER")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_url_scanner: env::var("ENABLE_URL_SCANNER")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_email_scanner: env::var("ENABLE_EMAIL_SCANNER")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_archive_scanner: env::var("ENABLE_ARCHIVE_SCANNER")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            max_entropy_threshold: env::var("MAX_ENTROPY_THRESHOLD")
                .unwrap_or_else(|_| "7.5".to_string())
                .parse()
                .context("Invalid MAX_ENTROPY_THRESHOLD")?,
            min_suspicious_strings: env::var("MIN_SUSPICIOUS_STRINGS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .context("Invalid MIN_SUSPICIOUS_STRINGS")?,
            phishing_check_timeout_seconds: env::var("PHISHING_CHECK_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("Invalid PHISHING_CHECK_TIMEOUT")?,
            max_archive_depth: env::var("MAX_ARCHIVE_DEPTH")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("Invalid MAX_ARCHIVE_DEPTH")?,
            zip_bomb_ratio_threshold: env::var("ZIP_BOMB_RATIO_THRESHOLD")
                .unwrap_or_else(|_| "100.0".to_string())
                .parse()
                .context("Invalid ZIP_BOMB_RATIO_THRESHOLD")?,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.max_entropy_threshold < 0.0 || self.max_entropy_threshold > 8.0 {
            anyhow::bail!("Entropy threshold must be between 0.0 and 8.0");
        }
        if self.max_archive_depth == 0 {
            anyhow::bail!("Max archive depth must be at least 1");
        }
        Ok(())
    }
}

impl Default for ScannersConfig {
    fn default() -> Self {
        Self {
            enable_file_scanner: true,
            enable_url_scanner: true,
            enable_email_scanner: true,
            enable_archive_scanner: true,
            max_entropy_threshold: 7.5,
            min_suspicious_strings: 3,
            phishing_check_timeout_seconds: 30,
            max_archive_depth: 5,
            zip_bomb_ratio_threshold: 100.0,
        }
    }
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enable_sandbox: bool,
    pub docker_available: bool,
    pub base_image: String,
    pub timeout_seconds: u64,
    pub memory_limit_mb: u64,
    pub cpu_limit_cores: f64,
    pub network_isolation: bool,
    pub enable_screenshots: bool,
    pub max_concurrent_sandboxes: usize,
}

impl SandboxConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            enable_sandbox: env::var("ENABLE_SANDBOX")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            docker_available: env::var("DOCKER_AVAILABLE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            base_image: env::var("SANDBOX_BASE_IMAGE")
                .unwrap_or_else(|_| "ubuntu:22.04".to_string()),
            timeout_seconds: env::var("SANDBOX_TIMEOUT")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .context("Invalid SANDBOX_TIMEOUT")?,
            memory_limit_mb: env::var("SANDBOX_MEMORY_LIMIT_MB")
                .unwrap_or_else(|_| "2048".to_string())
                .parse()
                .context("Invalid SANDBOX_MEMORY_LIMIT_MB")?,
            cpu_limit_cores: env::var("SANDBOX_CPU_LIMIT_CORES")
                .unwrap_or_else(|_| "2.0".to_string())
                .parse()
                .context("Invalid SANDBOX_CPU_LIMIT_CORES")?,
            network_isolation: env::var("SANDBOX_NETWORK_ISOLATION")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_screenshots: env::var("SANDBOX_ENABLE_SCREENSHOTS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            max_concurrent_sandboxes: env::var("MAX_CONCURRENT_SANDBOXES")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("Invalid MAX_CONCURRENT_SANDBOXES")?,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.timeout_seconds == 0 {
            anyhow::bail!("Sandbox timeout must be greater than 0");
        }
        if self.memory_limit_mb < 512 {
            anyhow::bail!("Sandbox memory limit must be at least 512 MB");
        }
        if self.cpu_limit_cores <= 0.0 {
            anyhow::bail!("CPU limit must be greater than 0");
        }
        Ok(())
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enable_sandbox: true,
            docker_available: true,
            base_image: "ubuntu:22.04".to_string(),
            timeout_seconds: 300,
            memory_limit_mb: 2048,
            cpu_limit_cores: 2.0,
            network_isolation: true,
            enable_screenshots: true,
            max_concurrent_sandboxes: 5,
        }
    }
}

/// Analyzers configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzersConfig {
    pub enable_static_analyzer: bool,
    pub enable_dynamic_analyzer: bool,
    pub enable_hash_analyzer: bool,
    pub enable_yara_engine: bool,
    pub enable_ml_analyzer: bool,
    pub yara_rules_directory: PathBuf,
    pub ml_model_path: PathBuf,
    pub analysis_timeout_seconds: u64,
    pub max_concurrent_analyses: usize,
    pub enable_parallel_analysis: bool,
}

impl AnalyzersConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            enable_static_analyzer: env::var("ENABLE_STATIC_ANALYZER")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_dynamic_analyzer: env::var("ENABLE_DYNAMIC_ANALYZER")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_hash_analyzer: env::var("ENABLE_HASH_ANALYZER")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_yara_engine: env::var("ENABLE_YARA_ENGINE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_ml_analyzer: env::var("ENABLE_ML_ANALYZER")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            yara_rules_directory: PathBuf::from(
                env::var("YARA_RULES_DIR").unwrap_or_else(|_| "./rules".to_string()),
            ),
            ml_model_path: PathBuf::from(
                env::var("ML_MODEL_PATH").unwrap_or_else(|_| "./models/malware_detector.onnx".to_string()),
            ),
            analysis_timeout_seconds: env::var("ANALYSIS_TIMEOUT")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .context("Invalid ANALYSIS_TIMEOUT")?,
            max_concurrent_analyses: env::var("MAX_CONCURRENT_ANALYSES")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("Invalid MAX_CONCURRENT_ANALYSES")?,
            enable_parallel_analysis: env::var("ENABLE_PARALLEL_ANALYSIS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.analysis_timeout_seconds == 0 {
            anyhow::bail!("Analysis timeout must be greater than 0");
        }
        if self.max_concurrent_analyses == 0 {
            anyhow::bail!("Max concurrent analyses must be at least 1");
        }
        Ok(())
    }
}

impl Default for AnalyzersConfig {
    fn default() -> Self {
        Self {
            enable_static_analyzer: true,
            enable_dynamic_analyzer: true,
            enable_hash_analyzer: true,
            enable_yara_engine: true,
            enable_ml_analyzer: false,
            yara_rules_directory: PathBuf::from("./rules"),
            ml_model_path: PathBuf::from("./models/malware_detector.onnx"),
            analysis_timeout_seconds: 300,
            max_concurrent_analyses: 10,
            enable_parallel_analysis: true,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_authentication: bool,
    pub enable_rate_limiting: bool,
    pub rate_limit_requests_per_minute: u32,
    pub rate_limit_requests_per_hour: u32,
    pub enable_api_key_auth: bool,
    pub api_keys: Vec<String>,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub max_request_size_mb: u64,
    pub allowed_file_extensions: Vec<String>,
}

impl SecurityConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            enable_authentication: env::var("ENABLE_AUTHENTICATION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            enable_rate_limiting: env::var("ENABLE_RATE_LIMITING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            rate_limit_requests_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .context("Invalid RATE_LIMIT_PER_MINUTE")?,
            rate_limit_requests_per_hour: env::var("RATE_LIMIT_PER_HOUR")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .context("Invalid RATE_LIMIT_PER_HOUR")?,
            enable_api_key_auth: env::var("ENABLE_API_KEY_AUTH")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            api_keys: env::var("API_KEYS")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect(),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me-in-production".to_string()),
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .context("Invalid JWT_EXPIRY_HOURS")?,
            max_request_size_mb: env::var("MAX_REQUEST_SIZE_MB")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .context("Invalid MAX_REQUEST_SIZE_MB")?,
            allowed_file_extensions: env::var("ALLOWED_FILE_EXTENSIONS")
                .unwrap_or_else(|_| "exe,dll,bat,cmd,scr,pif,com,vbs,js,jar,zip,rar,7z,tar,gz,pdf,doc,docx,xls,xlsx,ppt,pptx,rtf,apk,ipa,deb,rpm,msi,dmg,bin".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.enable_authentication && self.jwt_secret == "change-me-in-production" {
            tracing::warn!("Using default JWT secret - please change in production!");
        }
        if self.max_request_size_mb == 0 {
            anyhow::bail!("Max request size must be greater than 0");
        }
        Ok(())
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_authentication: false,
            enable_rate_limiting: true,
            rate_limit_requests_per_minute: 60,
            rate_limit_requests_per_hour: 1000,
            enable_api_key_auth: false,
            api_keys: vec![],
            jwt_secret: "change-me-in-production".to_string(),
            jwt_expiry_hours: 24,
            max_request_size_mb: 100,
            allowed_file_extensions: vec![
                "exe", "dll", "bat", "cmd", "scr", "pif", "com", "vbs", "js", "jar",
                "zip", "rar", "7z", "tar", "gz", "pdf", "doc", "docx", "xls", "xlsx",
                "ppt", "pptx", "rtf", "apk", "ipa", "deb", "rpm", "msi", "dmg", "bin",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub enable_file_logging: bool,
    pub log_file_path: PathBuf,
    pub max_log_file_size_mb: u64,
    pub log_rotation_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

impl LoggingConfig {
    pub fn from_env() -> Result<Self> {
        let format = match env::var("LOG_FORMAT")
            .unwrap_or_else(|_| "pretty".to_string())
            .to_lowercase()
            .as_str()
        {
            "json" => LogFormat::Json,
            "compact" => LogFormat::Compact,
            _ => LogFormat::Pretty,
        };

        Ok(Self {
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            format,
            enable_file_logging: env::var("ENABLE_FILE_LOGGING")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            log_file_path: PathBuf::from(
                env::var("LOG_FILE_PATH").unwrap_or_else(|_| "./logs/analysis-engine.log".to_string()),
            ),
            max_log_file_size_mb: env::var("MAX_LOG_FILE_SIZE_MB")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .context("Invalid MAX_LOG_FILE_SIZE_MB")?,
            log_rotation_count: env::var("LOG_ROTATION_COUNT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("Invalid LOG_ROTATION_COUNT")?,
        })
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            enable_file_logging: false,
            log_file_path: PathBuf::from("./logs/analysis-engine.log"),
            max_log_file_size_mb: 100,
            log_rotation_count: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_creation() {
        let config = Config::default();
        assert_eq!(config.server.port, 8002);
        assert_eq!(config.server.host, "0.0.0.0");
        assert!(config.scanners.enable_file_scanner);
    }

    #[test]
    fn test_server_config_validation() {
        let mut config = ServerConfig::default();
        assert!(config.validate().is_ok());

        config.port = 0;
        assert!(config.validate().is_err());

        config.port = 8080;
        config.workers = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_database_config_validation() {
        let mut config = DatabaseConfig::default();
        assert!(config.validate().is_ok());

        config.url = String::new();
        assert!(config.validate().is_err());

        config.url = "postgresql://localhost/test".to_string();
        config.min_connections = 100;
        config.max_connections = 50;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_storage_config_validation() {
        let mut config = StorageConfig::default();
        assert!(config.validate().is_ok());

        config.s3_bucket = String::new();
        assert!(config.validate().is_err());

        config.s3_bucket = "test-bucket".to_string();
        config.max_file_size_mb = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_scanners_config_validation() {
        let mut config = ScannersConfig::default();
        assert!(config.validate().is_ok());

        config.max_entropy_threshold = 10.0;
        assert!(config.validate().is_err());

        config.max_entropy_threshold = 7.5;
        config.max_archive_depth = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sandbox_config_validation() {
        let mut config = SandboxConfig::default();
        assert!(config.validate().is_ok());

        config.timeout_seconds = 0;
        assert!(config.validate().is_err());

        config.timeout_seconds = 300;
        config.memory_limit_mb = 256;
        assert!(config.validate().is_err());

        config.memory_limit_mb = 1024;
        config.cpu_limit_cores = 0.0;
        assert!(config.validate().is_err());
    }
}
