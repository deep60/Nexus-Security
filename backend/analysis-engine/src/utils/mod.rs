pub mod file_handler;

// Re-export commonly used types and functions for convenience
pub use file_handler::{FileHandler, FileMetadata as UtilsFileMetadata, AnalysisStatus};
pub use self::utils::sanitize_filename;

use anyhow::Result;
use std::collections::HashMap;

/// Common error types used across the analysis engine
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("File processing error: {0}")]
    FileError(String),
    
    #[error("Network communication error: {0}")]
    NetworkError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Blockchain interaction error: {0}")]
    BlockchainError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Analysis engine error: {0}")]
    AnalysisError(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Rate limiting error: {0}")]
    RateLimitError(String),
}

pub type EngineResult<T> = Result<T, EngineError>;

pub mod constants {
    pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;
    
    pub const DEFAULT_ANALYSIS_TIMEOUT: u64 = 300;
    
    pub const MAX_CONCURRENT_ANALYSES: usize = 10;
    
    pub const DEFAULT_STAKE_AMOUNT: f64 = 0.01;
    
    pub const MIN_CONFIDENCE_SCORE: f64 = 0.7;
    
    pub const MAX_BLOCKCHAIN_RETRIES: u32 = 3;
    
    pub const ENGINE_VERSION: &str = "1.0.0";
    
    pub const SUPPORTED_EXTENSIONS: &[&str] = &[
        "exe", "dll", "bat", "cmd", "scr", "pif", "com", "vbs", "js", "jar",
        "zip", "rar", "7z", "tar", "gz", "pdf", "doc", "docx", "xls", "xlsx",
        "ppt", "pptx", "rtf", "apk", "ipa", "deb", "rpm", "msi", "dmg", "bin"
    ];
    
    pub mod blockchain {
        pub const MAINNET_CHAIN_ID: u64 = 1;
        pub const GOERLI_CHAIN_ID: u64 = 5;
        pub const SEPOLIA_CHAIN_ID: u64 = 11155111;
        pub const POLYGON_CHAIN_ID: u64 = 137;
        
        pub const GAS_LIMIT_STAKE: u64 = 150_000;
        pub const GAS_LIMIT_VERDICT: u64 = 100_000;
        pub const GAS_LIMIT_WITHDRAW: u64 = 80_000;
    }
    
    pub mod rate_limits {
        pub const REQUESTS_PER_MINUTE: u32 = 60;
        pub const REQUESTS_PER_HOUR: u32 = 1000;
        pub const REQUESTS_PER_DAY: u32 = 10000;
    }
}

pub mod utils {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use uuid::Uuid;
    
    pub fn generate_analysis_id() -> String {
        Uuid::new_v4().to_string()
    }
    
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size_f = size as f64;
        let mut unit_index = 0;
        
        while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
            size_f /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size_f, UNITS[unit_index])
        }
    }
    
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                c if c.is_control() => '_',
                c => c,
            })
            .collect::<String>()
            .trim()
            .to_string()
    }
    
    pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, hex::FromHexError> {
        hex::decode(hex.trim_start_matches("0x"))
    }
    
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        format!("0x{}", hex::encode(bytes))
    }
    
    pub fn calculate_confidence(verdicts: &HashMap<String, bool>, stakes: &HashMap<String, f64>) -> f64 {
        let mut total_stake = 0.0;
        let mut malicious_stake = 0.0;
        
        for (engine, is_malicious) in verdicts {
            let stake = stakes.get(engine).cloned().unwrap_or(constants::DEFAULT_STAKE_AMOUNT);
            total_stake += stake;
            
            if *is_malicious {
                malicious_stake += stake;
            }
        }
        
        if total_stake == 0.0 {
            0.0
        } else {
            malicious_stake / total_stake
        }
    }
    
    pub fn is_valid_eth_address(address: &str) -> bool {
        if !address.starts_with("0x") || address.len() != 42 {
            return false;
        }
        
        address[2..].chars().all(|c| c.is_ascii_hexdigit())
    }
    
    pub fn generate_nonce() -> u64 {
        use rand::Rng;
        rand::thread_rng().gen()
    }
    
    pub async fn retry_with_backoff<F, T, E>(
        mut operation: F,
        max_retries: u32,
        initial_delay: u64,
    ) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Debug,
    {
        let mut delay = initial_delay;
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error);
                    if attempt < max_retries {
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                        delay = (delay * 2).min(30000); // Cap at 30 seconds
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }
}

pub mod macros {
    #[macro_export]
    macro_rules! log_error {
        ($msg:expr) => {
            eprintln!("ERROR: {}", $msg);
            return Err(EngineError::AnalysisError($msg.to_string()));
        };
        ($fmt:expr, $($arg:tt)*) => {
            let msg = format!($fmt, $($arg)*);
            tracing::error!("{}", &msg);
            return Err(EngineError::AnalysisError(msg));
        };
    }
    
    #[macro_export]
    macro_rules! time_function {
        ($func:expr) => {{
            let start = std::time::Instant::now();
            let result = $func;
            let duration = start.elapsed();
            (result, duration)
        }};
    }
    
    #[macro_export]
    macro_rules! require_field {
        ($field:expr, $field_name:expr) => {
            $field.ok_or_else(|| EngineError::ValidationError(
                format!("Required field '{}' is missing", $field_name)
            ))?
        };
    }
}

// Re-export important traits and types from dependencies
pub use anyhow::{Result as AnyhowResult, Error as AnyhowError};
pub use serde::{Deserialize, Serialize};
pub use tokio;
pub use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils::*;
    
    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1048576), "1.00 MB");
        assert_eq!(format_file_size(500), "500 B");
    }
    
    #[test]
    fn test_sanitize_filename() {
        let dirty = "file<n>with|bad*chars?.exe";
        let clean = sanitize_filename(dirty);
        assert_eq!(clean, "file_n_with_bad_chars_.exe");
    }
    
    #[test]
    fn test_eth_address_validation() {
        assert!(is_valid_eth_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44e"));
        assert!(!is_valid_eth_address("invalid_address"));
        assert!(!is_valid_eth_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44")); // too short
    }
    
    #[test]
    fn test_confidence_calculation() {
        let mut verdicts = HashMap::new();
        verdicts.insert("engine1".to_string(), true);  // malicious
        verdicts.insert("engine2".to_string(), false); // benign
        verdicts.insert("engine3".to_string(), true);  // malicious
        
        let mut stakes = HashMap::new();
        stakes.insert("engine1".to_string(), 0.02);
        stakes.insert("engine2".to_string(), 0.01);
        stakes.insert("engine3".to_string(), 0.03);
        
        let confidence = calculate_confidence(&verdicts, &stakes);
        assert!((confidence - 0.833).abs() < 0.01); // 0.05/0.06 ≈ 0.833
    }
    
    #[test]
    fn test_hex_conversion() {
        let bytes = vec![0xde, 0xad, 0xbe, 0xef];
        let hex = bytes_to_hex(&bytes);
        assert_eq!(hex, "0xdeadbeef");
        
        let converted_back = hex_to_bytes(&hex).unwrap();
        assert_eq!(converted_back, bytes);
    }
}