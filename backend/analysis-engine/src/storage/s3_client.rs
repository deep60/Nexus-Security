/// S3-compatible object storage client for file artifacts
///
/// This module provides object storage operations using AWS S3 or compatible services
/// (MinIO, DigitalOcean Spaces, etc.) for storing analysis artifacts and files.

use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::SystemTime;
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// S3 error types
#[derive(Debug, Error)]
pub enum S3Error {
    #[error("Upload error: {0}")]
    UploadError(String),

    #[error("Download error: {0}")]
    DownloadError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("S3 error: {0}")]
    Other(#[from] anyhow::Error),
}

/// S3 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub use_ssl: bool,
    pub path_style: bool,
    pub upload_timeout_seconds: u64,
    pub download_timeout_seconds: u64,
    pub max_file_size_mb: u64,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:9000".to_string(), // MinIO default
            region: "us-east-1".to_string(),
            bucket: "nexus-security-artifacts".to_string(),
            access_key_id: "minioadmin".to_string(),
            secret_access_key: "minioadmin".to_string(),
            use_ssl: false,
            path_style: true,
            upload_timeout_seconds: 300,
            download_timeout_seconds: 300,
            max_file_size_mb: 500,
        }
    }
}

/// S3 client for object storage operations
pub struct S3Client {
    config: S3Config,
    http_client: Client,
    base_url: String,
}

impl S3Client {
    /// Create a new S3 client
    pub fn new(config: S3Config) -> Result<Self> {
        info!("Initializing S3 client for bucket: {}", config.bucket);

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.upload_timeout_seconds))
            .build()
            .context("Failed to create HTTP client")?;

        let base_url = if config.path_style {
            format!("{}/{}", config.endpoint, config.bucket)
        } else {
            format!("{}", config.endpoint)
        };

        Ok(Self {
            config,
            http_client,
            base_url,
        })
    }

    /// Upload file to S3
    pub async fn upload_file(&self, key: &str, data: &[u8]) -> Result<String> {
        debug!("Uploading file to S3: {} ({} bytes)", key, data.len());

        // Check file size limit
        let size_mb = data.len() as u64 / (1024 * 1024);
        if size_mb > self.config.max_file_size_mb {
            return Err(anyhow!(
                "File size {} MB exceeds limit of {} MB",
                size_mb,
                self.config.max_file_size_mb
            ));
        }

        let url = format!("{}/{}", self.base_url, key);
        let content_type = self.detect_content_type(data);

        // Calculate content hash for integrity
        let content_hash = self.calculate_sha256(data);

        // Generate authorization headers (simplified - in production use proper AWS SigV4)
        let headers = self.build_headers("PUT", key, &content_type, data.len());

        let response = self
            .http_client
            .put(&url)
            .headers(headers)
            .body(data.to_vec())
            .send()
            .await
            .context("Failed to upload file to S3")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("S3 upload failed: {} - {}", status, error_text);
            return Err(anyhow!("S3 upload failed: {}", status));
        }

        info!("File uploaded successfully: {}", key);
        Ok(content_hash)
    }

    /// Download file from S3
    pub async fn download_file(&self, key: &str) -> Result<Option<Vec<u8>>> {
        debug!("Downloading file from S3: {}", key);

        let url = format!("{}/{}", self.base_url, key);
        let headers = self.build_headers("GET", key, "application/octet-stream", 0);

        let response = self
            .http_client
            .get(&url)
            .headers(headers)
            .timeout(std::time::Duration::from_secs(
                self.config.download_timeout_seconds,
            ))
            .send()
            .await
            .context("Failed to download file from S3")?;

        match response.status() {
            status if status.is_success() => {
                let data = response
                    .bytes()
                    .await
                    .context("Failed to read response body")?
                    .to_vec();

                debug!("File downloaded successfully: {} ({} bytes)", key, data.len());
                Ok(Some(data))
            }
            reqwest::StatusCode::NOT_FOUND => {
                debug!("File not found in S3: {}", key);
                Ok(None)
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                error!("S3 download failed: {} - {}", status, error_text);
                Err(anyhow!("S3 download failed: {}", status))
            }
        }
    }

    /// Delete file from S3
    pub async fn delete_file(&self, key: &str) -> Result<bool> {
        debug!("Deleting file from S3: {}", key);

        let url = format!("{}/{}", self.base_url, key);
        let headers = self.build_headers("DELETE", key, "application/octet-stream", 0);

        let response = self
            .http_client
            .delete(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to delete file from S3")?;

        if response.status().is_success() || response.status() == reqwest::StatusCode::NOT_FOUND {
            info!("File deleted successfully: {}", key);
            Ok(true)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("S3 delete failed: {} - {}", status, error_text);
            Err(anyhow!("S3 delete failed: {}", status))
        }
    }

    /// Check if file exists in S3
    pub async fn file_exists(&self, key: &str) -> Result<bool> {
        debug!("Checking if file exists in S3: {}", key);

        let url = format!("{}/{}", self.base_url, key);
        let headers = self.build_headers("HEAD", key, "application/octet-stream", 0);

        let response = self
            .http_client
            .head(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to check file existence")?;

        Ok(response.status().is_success())
    }

    /// List objects with prefix
    pub async fn list_objects(&self, prefix: &str) -> Result<Vec<String>> {
        debug!("Listing objects with prefix: {}", prefix);

        let url = format!("{}?prefix={}", self.base_url, prefix);
        let headers = self.build_headers("GET", "", "application/octet-stream", 0);

        let response = self
            .http_client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to list objects")?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to list objects: {}", response.status()));
        }

        // Parse XML response (simplified - in production use proper XML parser)
        let body = response.text().await?;
        let keys = self.parse_list_objects_response(&body);

        Ok(keys)
    }

    /// Get object metadata
    pub async fn get_metadata(&self, key: &str) -> Result<ObjectMetadata> {
        debug!("Getting metadata for object: {}", key);

        let url = format!("{}/{}", self.base_url, key);
        let headers = self.build_headers("HEAD", key, "application/octet-stream", 0);

        let response = self
            .http_client
            .head(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to get object metadata")?;

        if !response.status().is_success() {
            return Err(anyhow!("Object not found: {}", key));
        }

        let content_length = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let last_modified = response
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim_matches('"').to_string());

        Ok(ObjectMetadata {
            key: key.to_string(),
            size: content_length,
            content_type,
            last_modified,
            etag,
        })
    }

    /// Generate presigned URL for temporary access (simplified)
    pub fn generate_presigned_url(&self, key: &str, expiry_seconds: u64) -> Result<String> {
        // Simplified presigned URL generation
        // In production, implement proper AWS SigV4 URL signing
        let expires = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + expiry_seconds;

        let url = format!(
            "{}/{}?X-Amz-Expires={}",
            self.base_url, key, expires
        );

        Ok(url)
    }

    /// Health check
    pub async fn health_check(&self) -> bool {
        // Try to list objects in bucket
        let url = format!("{}?max-keys=1", self.base_url);
        let headers = self.build_headers("GET", "", "application/octet-stream", 0);

        self.http_client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Build HTTP headers for S3 request
    fn build_headers(
        &self,
        method: &str,
        key: &str,
        content_type: &str,
        content_length: usize,
    ) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            content_type.parse().unwrap(),
        );

        if content_length > 0 {
            headers.insert(
                reqwest::header::CONTENT_LENGTH,
                content_length.to_string().parse().unwrap(),
            );
        }

        // Add AWS authentication headers (simplified)
        // In production, implement proper AWS SigV4 signing
        headers.insert(
            "x-amz-date",
            chrono::Utc::now()
                .format("%Y%m%dT%H%M%SZ")
                .to_string()
                .parse()
                .unwrap(),
        );

        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!(
                "AWS {}:{}",
                self.config.access_key_id, "simplified-signature"
            )
            .parse()
            .unwrap(),
        );

        headers
    }

    /// Detect content type from file data
    fn detect_content_type(&self, data: &[u8]) -> String {
        if data.len() < 4 {
            return "application/octet-stream".to_string();
        }

        // Check magic bytes
        match &data[0..2] {
            b"PK" => "application/zip".to_string(),
            b"MZ" => "application/x-msdownload".to_string(),
            [0x1f, 0x8b] => "application/gzip".to_string(),
            _ => {
                if data.len() >= 4 {
                    match &data[0..4] {
                        [0x89, 0x50, 0x4E, 0x47] => "image/png".to_string(),
                        [0xFF, 0xD8, 0xFF, _] => "image/jpeg".to_string(),
                        [0x25, 0x50, 0x44, 0x46] => "application/pdf".to_string(),
                        _ => "application/octet-stream".to_string(),
                    }
                } else {
                    "application/octet-stream".to_string()
                }
            }
        }
    }

    /// Calculate SHA256 hash
    fn calculate_sha256(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Parse list objects XML response (simplified)
    fn parse_list_objects_response(&self, xml: &str) -> Vec<String> {
        let mut keys = Vec::new();

        // Very simplified XML parsing - in production use proper XML parser
        for line in xml.lines() {
            if line.contains("<Key>") {
                if let Some(start) = line.find("<Key>") {
                    if let Some(end) = line.find("</Key>") {
                        let key = line[start + 5..end].trim().to_string();
                        keys.push(key);
                    }
                }
            }
        }

        keys
    }

    /// Get S3 statistics
    pub async fn get_stats(&self) -> S3Stats {
        S3Stats {
            bucket: self.config.bucket.clone(),
            endpoint: self.config.endpoint.clone(),
            region: self.config.region.clone(),
            max_file_size_mb: self.config.max_file_size_mb,
        }
    }
}

/// Object metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetadata {
    pub key: String,
    pub size: u64,
    pub content_type: String,
    pub last_modified: Option<String>,
    pub etag: Option<String>,
}

/// S3 statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Stats {
    pub bucket: String,
    pub endpoint: String,
    pub region: String,
    pub max_file_size_mb: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_config_default() {
        let config = S3Config::default();
        assert_eq!(config.bucket, "nexus-security-artifacts");
        assert_eq!(config.region, "us-east-1");
        assert!(config.path_style);
    }

    #[test]
    fn test_content_type_detection() {
        let client = S3Client::new(S3Config::default()).unwrap();

        // ZIP file
        let zip_data = b"PK\x03\x04";
        assert_eq!(client.detect_content_type(zip_data), "application/zip");

        // PNG image
        let png_data = b"\x89PNG";
        assert_eq!(client.detect_content_type(png_data), "image/png");

        // Unknown
        let unknown_data = b"TEST";
        assert_eq!(
            client.detect_content_type(unknown_data),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_sha256_calculation() {
        let client = S3Client::new(S3Config::default()).unwrap();
        let data = b"Hello, World!";
        let hash = client.calculate_sha256(data);

        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_presigned_url_generation() {
        let client = S3Client::new(S3Config::default()).unwrap();
        let url = client.generate_presigned_url("test/file.txt", 3600).unwrap();

        assert!(url.contains("test/file.txt"));
        assert!(url.contains("X-Amz-Expires="));
    }
}
