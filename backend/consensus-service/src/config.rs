use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub consensus: ConsensusConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub min_submissions: usize,
    pub max_submissions: usize,
    pub consensus_threshold: f64,
    pub weighted_voting: bool,
    pub reputation_weight: f64,
    pub confidence_weight: f64,
    pub time_weight: f64,
    pub dispute_threshold: f64,
    pub auto_finalize_hours: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST")
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8087".to_string())
                    .parse()?,
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")?,
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
            redis: RedisConfig {
                url: std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            consensus: ConsensusConfig {
                min_submissions: std::env::var("MIN_SUBMISSIONS")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()?,
                max_submissions: std::env::var("MAX_SUBMISSIONS")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
                consensus_threshold: std::env::var("CONSENSUS_THRESHOLD")
                    .unwrap_or_else(|_| "0.66".to_string())
                    .parse()?,
                weighted_voting: std::env::var("WEIGHTED_VOTING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                reputation_weight: std::env::var("REPUTATION_WEIGHT")
                    .unwrap_or_else(|_| "0.5".to_string())
                    .parse()?,
                confidence_weight: std::env::var("CONFIDENCE_WEIGHT")
                    .unwrap_or_else(|_| "0.3".to_string())
                    .parse()?,
                time_weight: std::env::var("TIME_WEIGHT")
                    .unwrap_or_else(|_| "0.2".to_string())
                    .parse()?,
                dispute_threshold: std::env::var("DISPUTE_THRESHOLD")
                    .unwrap_or_else(|_| "0.4".to_string())
                    .parse()?,
                auto_finalize_hours: std::env::var("AUTO_FINALIZE_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()?,
            },
        })
    }
}
