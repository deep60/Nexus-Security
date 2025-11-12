use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub reputation: ReputationConfig,
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
pub struct ReputationConfig {
    pub base_score: i32,
    pub correct_analysis_points: i32,
    pub incorrect_analysis_penalty: i32,
    pub streak_bonus_multiplier: f64,
    pub decay_rate_per_day: f64,
    pub min_score: i32,
    pub max_score: i32,
    pub consensus_bonus: i32,
    pub early_submission_bonus: i32,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST")
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8086".to_string())
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
            reputation: ReputationConfig {
                base_score: std::env::var("BASE_SCORE")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()?,
                correct_analysis_points: std::env::var("CORRECT_ANALYSIS_POINTS")
                    .unwrap_or_else(|_| "50".to_string())
                    .parse()?,
                incorrect_analysis_penalty: std::env::var("INCORRECT_ANALYSIS_PENALTY")
                    .unwrap_or_else(|_| "-100".to_string())
                    .parse()?,
                streak_bonus_multiplier: std::env::var("STREAK_BONUS_MULTIPLIER")
                    .unwrap_or_else(|_| "1.2".to_string())
                    .parse()?,
                decay_rate_per_day: std::env::var("DECAY_RATE_PER_DAY")
                    .unwrap_or_else(|_| "0.001".to_string())
                    .parse()?,
                min_score: std::env::var("MIN_SCORE")
                    .unwrap_or_else(|_| "0".to_string())
                    .parse()?,
                max_score: std::env::var("MAX_SCORE")
                    .unwrap_or_else(|_| "10000".to_string())
                    .parse()?,
                consensus_bonus: std::env::var("CONSENSUS_BONUS")
                    .unwrap_or_else(|_| "25".to_string())
                    .parse()?,
                early_submission_bonus: std::env::var("EARLY_SUBMISSION_BONUS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
        })
    }
}
