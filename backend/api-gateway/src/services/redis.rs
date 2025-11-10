use anyhow::{Context, Result};
use redis::{AsyncCommands, Client, RedisResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct RedisService {
    client: Client,
    pub(crate) connection_pool: redis::aio::MultiplexedConnection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisCache {
    pub file_hash: String,
    pub analysis_result: String,
    pub confidence_score: f64,
    pub threat_level: String,
    pub engines_analyzed: Vec<String>,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BountyCache {
    pub bounty_id: Uuid,
    pub status: String,
    pub current_reward: String, // Store as string to avoid precision issues with crypto amounts
    pub submissions_count: u32,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: Uuid,
    pub wallet_address: String,
    pub reputation_score: i32,
    pub session_token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl RedisService {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)
            .context("Failed to create Redis client")?;
        
        let connection_pool = client.get_multiplexed_async_connection()
            .await
            .context("Failed to establish Redis connection pool")?;

        info!("Redis service initialized successfully");
        
        Ok(RedisService {
            client,
            connection_pool,
        })
    }

    // Analysis result caching
    pub async fn cache_analysis(&mut self, file_hash: &str, analysis: &AnalysisCache, ttl_seconds: u64) -> Result<()> {
        let key = format!("analysis:{}", file_hash);
        let serialized = serde_json::to_string(analysis)
            .context("Failed to serialize analysis cache")?;

        let _: () = self.connection_pool
            .set_ex(&key, serialized, ttl_seconds)
            .await
            .context("Failed to cache analysis result")?;

        info!("Cached analysis result for file hash: {}", file_hash);
        Ok(())
    }

    pub async fn get_cached_analysis(&mut self, file_hash: &str) -> Result<Option<AnalysisCache>> {
        let key = format!("analysis:{}", file_hash);
        
        let cached: Option<String> = self.connection_pool
            .get(&key)
            .await
            .context("Failed to retrieve cached analysis")?;

        match cached {
            Some(data) => {
                let analysis: AnalysisCache = serde_json::from_str(&data)
                    .context("Failed to deserialize cached analysis")?;
                Ok(Some(analysis))
            }
            None => Ok(None)
        }
    }

    // Bounty caching
    pub async fn cache_bounty(&mut self, bounty: &BountyCache, ttl_seconds: u64) -> Result<()> {
        let key = format!("bounty:{}", bounty.bounty_id);
        let serialized = serde_json::to_string(bounty)
            .context("Failed to serialize bounty cache")?;

        let _: () = self.connection_pool
            .set_ex(&key, serialized, ttl_seconds)
            .await
            .context("Failed to cache bounty")?;

        info!("Cached bounty: {}", bounty.bounty_id);
        Ok(())
    }

    pub async fn get_cached_bounty(&mut self, bounty_id: &Uuid) -> Result<Option<BountyCache>> {
        let key = format!("bounty:{}", bounty_id);
        
        let cached: Option<String> = self.connection_pool
            .get(&key)
            .await
            .context("Failed to retrieve cached bounty")?;

        match cached {
            Some(data) => {
                let bounty: BountyCache = serde_json::from_str(&data)
                    .context("Failed to deserialize cached bounty")?;
                Ok(Some(bounty))
            }
            None => Ok(None)
        }
    }

    pub async fn invalidate_bounty_cache(&mut self, bounty_id: &Uuid) -> Result<()> {
        let key = format!("bounty:{}", bounty_id);
        
        let _: () = self.connection_pool
            .del(&key)
            .await
            .context("Failed to invalidate bounty cache")?;

        info!("Invalidated cache for bounty: {}", bounty_id);
        Ok(())
    }

    // User session management
    pub async fn create_session(&mut self, session: &UserSession, ttl_seconds: u64) -> Result<()> {
        let key = format!("session:{}", session.session_token);
        let user_key = format!("user_session:{}", session.user_id);
        
        let serialized = serde_json::to_string(session)
            .context("Failed to serialize user session")?;

        // Store session data with expiration
        let _: () = self.connection_pool
            .set_ex(&key, &serialized, ttl_seconds)
            .await
            .context("Failed to create session")?;

        // Store user -> session mapping
        let _: () = self.connection_pool
            .set_ex(&user_key, &session.session_token, ttl_seconds)
            .await
            .context("Failed to create user session mapping")?;

        info!("Created session for user: {}", session.user_id);
        Ok(())
    }

    pub async fn get_session(&mut self, session_token: &str) -> Result<Option<UserSession>> {
        let key = format!("session:{}", session_token);

        let cached: Option<String> = self.connection_pool
            .get(&key)
            .await
            .context("Failed to retrieve session")?;

        match cached {
            Some(data) => {
                let session: UserSession = serde_json::from_str(&data)
                    .context("Failed to deserialize session")?;

                // Check if session is expired
                if session.expires_at < chrono::Utc::now() {
                    // Delete expired session directly without recursion
                    let _: () = self.connection_pool.del(&key).await.context("Failed to delete session")?;
                    let user_key = format!("user_session:{}", session.user_id);
                    let _: () = self.connection_pool.del(&user_key).await.context("Failed to remove user session")?;
                    Ok(None)
                } else {
                    Ok(Some(session))
                }
            }
            None => Ok(None)
        }
    }

    pub async fn invalidate_session(&mut self, session_token: &str) -> Result<()> {
        // Get session data directly without calling get_session to avoid recursion
        let key = format!("session:{}", session_token);
        let cached: Option<String> = self.connection_pool.get(&key).await.context("Failed to retrieve session")?;

        if let Some(data) = cached {
            if let Ok(session) = serde_json::from_str::<UserSession>(&data) {
                let user_key = format!("user_session:{}", session.user_id);
                let _: () = self.connection_pool
                    .del(&user_key)
                    .await
                    .context("Failed to remove user session mapping")?;
            }
        }

        let key = format!("session:{}", session_token);
        let _: () = self.connection_pool
            .del(&key)
            .await
            .context("Failed to invalidate session")?;

        info!("Invalidated session: {}", session_token);
        Ok(())
    }

    // Rate limiting
    pub async fn check_rate_limit(&mut self, identifier: &str, max_requests: u32, window_seconds: u64) -> Result<bool> {
        let key = format!("rate_limit:{}", identifier);
        
        // Use Redis sliding window rate limiting
        let current_count: u32 = self.connection_pool
            .incr(&key, 1)
            .await
            .context("Failed to increment rate limit counter")?;

        if current_count == 1 {
            // Set expiration for new key
            let _: () = self.connection_pool
                .expire(&key, window_seconds as i64)
                .await
                .context("Failed to set rate limit expiration")?;
        }

        let allowed = current_count <= max_requests;
        
        if !allowed {
            warn!("Rate limit exceeded for identifier: {} ({}/{})", identifier, current_count, max_requests);
        }

        Ok(allowed)
    }

    // Real-time notifications and pub/sub
    pub async fn publish_analysis_complete(&mut self, file_hash: &str, analysis_id: &Uuid) -> Result<()> {
        let channel = "analysis_complete";
        let message = serde_json::json!({
            "file_hash": file_hash,
            "analysis_id": analysis_id,
            "timestamp": chrono::Utc::now()
        });

        let _: () = self.connection_pool
            .publish(channel, message.to_string())
            .await
            .context("Failed to publish analysis complete notification")?;

        info!("Published analysis complete notification for file: {}", file_hash);
        Ok(())
    }

    pub async fn publish_bounty_update(&mut self, bounty_id: &Uuid, update_type: &str, data: serde_json::Value) -> Result<()> {
        let channel = format!("bounty_updates:{}", bounty_id);
        let message = serde_json::json!({
            "bounty_id": bounty_id,
            "update_type": update_type,
            "data": data,
            "timestamp": chrono::Utc::now()
        });

        let _: () = self.connection_pool
            .publish(&channel, message.to_string())
            .await
            .context("Failed to publish bounty update")?;

        info!("Published bounty update for bounty: {} ({})", bounty_id, update_type);
        Ok(())
    }

    // Engine reputation caching
    pub async fn cache_engine_reputation(&mut self, engine_id: &str, reputation_score: f64, ttl_seconds: u64) -> Result<()> {
        let key = format!("engine_reputation:{}", engine_id);
        
        let _: () = self.connection_pool
            .set_ex(&key, reputation_score, ttl_seconds)
            .await
            .context("Failed to cache engine reputation")?;

        info!("Cached reputation for engine: {} (score: {})", engine_id, reputation_score);
        Ok(())
    }

    pub async fn get_engine_reputation(&mut self, engine_id: &str) -> Result<Option<f64>> {
        let key = format!("engine_reputation:{}", engine_id);
        
        let reputation: Option<f64> = self.connection_pool
            .get(&key)
            .await
            .context("Failed to retrieve engine reputation")?;

        Ok(reputation)
    }

    // Leaderboard functionality
    pub async fn update_leaderboard(&mut self, leaderboard_name: &str, member: &str, score: f64) -> Result<()> {
        let key = format!("leaderboard:{}", leaderboard_name);
        
        let _: () = self.connection_pool
            .zadd(&key, member, score)
            .await
            .context("Failed to update leaderboard")?;

        Ok(())
    }

    pub async fn get_leaderboard(&mut self, leaderboard_name: &str, limit: isize) -> Result<Vec<(String, f64)>> {
        let key = format!("leaderboard:{}", leaderboard_name);
        
        let results: Vec<(String, f64)> = self.connection_pool
            .zrevrange_withscores(&key, 0, limit - 1)
            .await
            .context("Failed to retrieve leaderboard")?;

        Ok(results)
    }

    // Health check
    pub async fn health_check(&mut self) -> Result<bool> {
        let result: RedisResult<String> = redis::cmd("PING")
            .query_async(&mut self.connection_pool)
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Redis health check failed: {}", e);
                Ok(false)
            }
        }
    }

    // Cache statistics
    pub async fn get_cache_stats(&mut self) -> Result<serde_json::Value> {
        let info: String = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut self.connection_pool)
            .await
            .context("Failed to get Redis memory info")?;

        // Parse basic stats from Redis INFO command
        let mut stats = serde_json::Map::new();
        
        for line in info.lines() {
            if line.contains(':') {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "used_memory" | "used_memory_human" | "used_memory_peak" | 
                        "used_memory_peak_human" | "maxmemory" | "maxmemory_human" => {
                            stats.insert(parts[0].to_string(), serde_json::Value::String(parts[1].to_string()));
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(serde_json::Value::Object(stats))
    }
}