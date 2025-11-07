/// Storage module for persisting analysis results and artifacts
///
/// This module provides:
/// - Database operations (PostgreSQL via SQLx)
/// - S3-compatible object storage for file artifacts
/// - Caching layer for frequently accessed data

pub mod database;
pub mod s3_client;

pub use database::{Database, DatabaseConfig, DatabaseError};
pub use s3_client::{S3Client, S3Config, S3Error};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unified storage manager that coordinates database and object storage
pub struct StorageManager {
    database: Arc<Database>,
    s3_client: Arc<S3Client>,
    cache: Arc<RwLock<StorageCache>>,
}

/// Configuration for storage manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database: DatabaseConfig,
    pub s3: S3Config,
    pub enable_cache: bool,
    pub cache_ttl_seconds: u64,
    pub max_cache_size_mb: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            s3: S3Config::default(),
            enable_cache: true,
            cache_ttl_seconds: 300, // 5 minutes
            max_cache_size_mb: 100,
        }
    }
}

/// Simple in-memory cache for storage operations
#[derive(Debug, Default)]
struct StorageCache {
    entries: std::collections::HashMap<String, CacheEntry>,
    total_size_bytes: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    timestamp: chrono::DateTime<chrono::Utc>,
    size_bytes: usize,
}

impl StorageManager {
    /// Create a new storage manager
    pub async fn new(config: StorageConfig) -> Result<Self> {
        tracing::info!("Initializing storage manager");

        let database = Arc::new(Database::new(config.database.clone()).await?);
        let s3_client = Arc::new(S3Client::new(config.s3.clone())?);
        let cache = Arc::new(RwLock::new(StorageCache::default()));

        Ok(Self {
            database,
            s3_client,
            cache,
        })
    }

    /// Get reference to database
    pub fn database(&self) -> Arc<Database> {
        Arc::clone(&self.database)
    }

    /// Get reference to S3 client
    pub fn s3_client(&self) -> Arc<S3Client> {
        Arc::clone(&self.s3_client)
    }

    /// Store analysis result with associated file
    pub async fn store_analysis_result(
        &self,
        analysis_id: &uuid::Uuid,
        result: &crate::models::analysis_result::AnalysisResult,
        file_data: Option<&[u8]>,
    ) -> Result<()> {
        tracing::debug!("Storing analysis result: {}", analysis_id);

        // Store file in S3 if provided
        if let Some(data) = file_data {
            let s3_key = format!("analyses/{}/artifact", analysis_id);
            self.s3_client.upload_file(&s3_key, data).await?;
        }

        // Store analysis result in database
        self.database.save_analysis_result(result).await?;

        Ok(())
    }

    /// Retrieve analysis result
    pub async fn get_analysis_result(
        &self,
        analysis_id: &uuid::Uuid,
    ) -> Result<Option<crate::models::analysis_result::AnalysisResult>> {
        self.database.get_analysis_result(analysis_id).await
    }

    /// Retrieve file from S3 with caching
    pub async fn get_file(&self, analysis_id: &uuid::Uuid) -> Result<Option<Vec<u8>>> {
        let cache_key = format!("file:{}", analysis_id);

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.entries.get(&cache_key) {
                // Check if cache entry is still valid (within TTL)
                let now = chrono::Utc::now();
                let age = now.signed_duration_since(entry.timestamp);
                if age.num_seconds() < 300 {
                    // 5 minutes TTL
                    tracing::debug!("Cache hit for file: {}", analysis_id);
                    return Ok(Some(entry.data.clone()));
                }
            }
        }

        // Not in cache or expired, fetch from S3
        let s3_key = format!("analyses/{}/artifact", analysis_id);
        if let Some(data) = self.s3_client.download_file(&s3_key).await? {
            // Update cache
            let mut cache = self.cache.write().await;
            cache.entries.insert(
                cache_key,
                CacheEntry {
                    data: data.clone(),
                    timestamp: chrono::Utc::now(),
                    size_bytes: data.len(),
                },
            );
            cache.total_size_bytes += data.len();

            // Evict old entries if cache is too large
            self.evict_cache_if_needed(&mut cache).await;

            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.entries.clear();
        cache.total_size_bytes = 0;
        tracing::info!("Storage cache cleared");
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> StorageStats {
        let cache = self.cache.read().await;
        let db_stats = self.database.get_stats().await;

        StorageStats {
            cache_entries: cache.entries.len(),
            cache_size_mb: cache.total_size_bytes / (1024 * 1024),
            database_connections: db_stats.active_connections,
            total_analyses_stored: db_stats.total_records,
        }
    }

    /// Evict cache entries if size exceeds limit
    async fn evict_cache_if_needed(&self, cache: &mut StorageCache) {
        let max_size_bytes = 100 * 1024 * 1024; // 100 MB

        if cache.total_size_bytes > max_size_bytes {
            tracing::debug!("Cache size exceeded, evicting old entries");

            // Sort by timestamp and remove oldest entries
            let mut entries: Vec<_> = cache.entries.iter().collect();
            entries.sort_by_key(|(_, entry)| entry.timestamp);

            let mut removed_size = 0;
            let target_size = max_size_bytes * 80 / 100; // Reduce to 80% of max

            for (key, entry) in entries {
                if cache.total_size_bytes - removed_size <= target_size {
                    break;
                }
                removed_size += entry.size_bytes;
                cache.entries.remove(key);
            }

            cache.total_size_bytes -= removed_size;
            tracing::debug!("Evicted {} bytes from cache", removed_size);
        }
    }

    /// Health check for storage systems
    pub async fn health_check(&self) -> StorageHealth {
        let db_healthy = self.database.health_check().await;
        let s3_healthy = self.s3_client.health_check().await;

        StorageHealth {
            database_healthy: db_healthy,
            s3_healthy,
            overall_healthy: db_healthy && s3_healthy,
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub cache_entries: usize,
    pub cache_size_mb: usize,
    pub database_connections: usize,
    pub total_analyses_stored: u64,
}

/// Storage health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHealth {
    pub database_healthy: bool,
    pub s3_healthy: bool,
    pub overall_healthy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert!(config.enable_cache);
        assert_eq!(config.cache_ttl_seconds, 300);
        assert_eq!(config.max_cache_size_mb, 100);
    }

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry {
            data: vec![1, 2, 3, 4, 5],
            timestamp: chrono::Utc::now(),
            size_bytes: 5,
        };

        assert_eq!(entry.data.len(), 5);
        assert_eq!(entry.size_bytes, 5);
    }
}
