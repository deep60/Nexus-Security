use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use tokio::fs as async_fs;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use md5;

/// maximum file size allowed (100MB)
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

// Supported file extensions for analysis
const ALLOWED_EXTENSIONS: &[&str] = &["exe", "dll", "bat", "cmd", "scr", "pif", "com", "vbs", "js", "jar", 
    "zip", "rar", "7z", "tar", "gz", "pdf", "doc", "docx", "xls", "xlsx", 
    "ppt", "pptx", "rtf", "apk", "ipa", "deb", "rpm", "msi", "dmg"
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_id: String,
    pub original_name: String,
    pub file_size: u64,
    pub file_type: String,
    pub sha256_hash: String,
    pub md5_hash: String,
    pub upload_timestamp: chrono::DateTime<chrono::Utc>,
    pub analysis_status: AnalysisStatus,
    pub bounty_amount: Option<f64>,
    pub submitter_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Quarantined,
}

#[derive(Debug, Clone)]
pub struct FileHandler {
    storage_path: PathBuf,
    quarantine_path: PathBuf,
    temp_path: PathBuf,
}

impl FileHandler {
    /// Create a new Filehandler instance
    pub fn new(base_storage_path: &str) -> Result<Self> {
        let storage_path = PathBuf::from(base_storage_path);
        let quarantine_path = storage_path.join("quarantine");
        let temp_path = storage_path.join("temp");

        // Create necessary directories
        fs::create_dir_all(&storage_path)?;
        fs::create_dir_all(&quarantine_path)?;
        fs::create_dir_all(&temp_path)?;

        Ok(FileHandler {
            storage_path,
            quarantine_path,
            temp_path,
        })
    }

    // Store an uploaded file and generate metadata
    pub async fn store_file(
        &self,
        file_data: &[u8],
        original_name: &str,
        bounty_amount: Option<f64>,
        submitter_address: Option<String>,
    ) -> Result<FileMetadata> {
        // Validate file size
        if file_data.len() as u64 > MAX_FILE_SIZE {
            return Err(anyhow!("File size exceeds maximum limit of {}MB", MAX_FILE_SIZE / 1024 / 1024));
        }

        // validate file extension
        let extension = Path::new(original_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if !ALLOWED_EXTENSIONS.contains(&extension.to_lowercase().as_str()) {
            return Err(anyhow!("File type '{}' is not allowed for analysis", extension));
        }

        // Generate unique file ID
        let file_id = Uuid::new_v4().to_string();

        // calculate hashes
        let sha256_hash = self.calculate_sha256(file_data);
        let md5_hash = self.calculate_md5(file_data);

        // Check for duplicates files
        if self.file_exists_by_hash(&sha256_hash).await? {
            return Err(anyhow!("File with identical hash already exists"));
        }

        // store file
        let file_path = self.storage_path.join(&file_id);
        async_fs::write(&file_path, file_data).await?;

        // create metadata
        let metadata = FileMetadata {
            file_id: file_id.clone(),
            original_name: original_name.to_string(),
            file_size: file_data.len() as u64,
            file_type: self.detect_file_type(file_data, original_name),
            sha256_hash,
            md5_hash,
            upload_timestamp: chrono::Utc::now(),
            analysis_status: AnalysisStatus::Pending,
            bounty_amount,
            submitter_address,
        };

        // Store metadata
        self.store_metadata(&metadata).await?;

        Ok(metadata)
    }

    // Retrieve file data by file ID
    pub async fn get_file(&self, file_id: &str) -> Result<Vec<u8>> {
        let file_path = self.storage_path.join(file_id);

        if !file_path.exists() {
            return Err(anyhow!("File not found: {}", file_id));
        }

        let file_data = async_fs::read(file_path).await?;
        Ok(file_data)
    }

    // Get file metadata by file ID
    pub async fn get_metadata(&self, file_id: &str) -> Result<FileMetadata> {
        let metadata_path = self.storage_path.join(format!("{}.meta", file_id));

        if !metadata_path.exists() {
            return Err(anyhow!("Metadata not found for file: {}", file_id));
        }

        let metadata_json = async_fs::read_to_string(metadata_path).await?;
        let metadata: FileMetadata = serde_json::from_str(&metadata_json)?;

        Ok(metadata)
    }

    // Update file analysis status
    pub async fn update_analysis_status(&self, file_id: &str, status: AnalysisStatus) -> Result<()> {
        let mut metadata = self.get_metadata(file_id).await?;
        metadata.analysis_status = status;
        self.store_metadata(&metadata).await?;

        Ok(())
    }

    /// Move file to quarantine if deemed malicious
    pub async fn quarantine_file(&self, file_id: &str, reason: &str) -> Result<()> {
        let source_path = self.storage_path.join(file_id);
        let quarantine_path = self.quarantine_path.join(file_id);
        
        if source_path.exists() {
            async_fs::rename(&source_path, &quarantine_path).await?;
            
            // Update status
            self.update_analysis_status(file_id, AnalysisStatus::Quarantined).await?;
            
            // Log quarantine action
            let log_entry = format!(
                "[{}] File {} quarantined: {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                file_id,
                reason
            );
            self.log_action(&log_entry).await?;
        }
        
        Ok(())
    }

    /// Delete file and its metadata
    pub async fn delete_file(&self, file_id: &str) -> Result<()> {
        let file_path = self.storage_path.join(file_id);
        let metadata_path = self.storage_path.join(format!("{}.meta", file_id));
        let quarantine_path = self.quarantine_path.join(file_id);
        
        // Remove from all possible locations
        if file_path.exists() {
            async_fs::remove_file(file_path).await?;
        }
        if quarantine_path.exists() {
            async_fs::remove_file(quarantine_path).await?;
        }
        if metadata_path.exists() {
            async_fs::remove_file(metadata_path).await?;
        }
        
        Ok(())
    }

    /// Get files by analysis status
    pub async fn get_files_by_status(&self, status: AnalysisStatus) -> Result<Vec<FileMetadata>> {
        let mut files = Vec::new();
        let mut entries = async_fs::read_dir(&self.storage_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".meta") {
                    let file_id = file_name.trim_end_matches(".meta");
                    if let Ok(metadata) = self.get_metadata(file_id).await {
                        // matches!
                        if std::mem::discriminant(&metadata.analysis_status) == std::mem::discriminant(&status) {
                            files.push(metadata);
                        }
                    }
                }
            }
        }
        
        Ok(files)
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> Result<HashMap<String, u64>> {
        let mut stats = HashMap::new();
        
        // Count files by status
        let statuses = vec![
            AnalysisStatus::Pending,
            AnalysisStatus::InProgress,
            AnalysisStatus::Completed,
            AnalysisStatus::Failed,
            AnalysisStatus::Quarantined,
        ];
        
        for status in statuses {
            let files = self.get_files_by_status(status.clone()).await?;
            let status_name = format!("{:?}", status).to_lowercase();
            stats.insert(status_name, files.len() as u64);
        }
        
        // Calculate total storage usage
        let total_size = self.calculate_directory_size(&self.storage_path).await?;
        stats.insert("total_storage_bytes".to_string(), total_size);
        
        Ok(stats)
    }

    /// Clean up temporary files older than specified hours
    pub async fn cleanup_temp_files(&self, hours_old: u64) -> Result<u32> {
        let mut cleaned = 0;
        let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(hours_old as i64);
        let mut entries = async_fs::read_dir(&self.temp_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                if let Ok(created) = metadata.created() {
                    let created_time = chrono::DateTime::<chrono::Utc>::from(created);
                    if created_time < cutoff_time {
                        if async_fs::remove_file(entry.path()).await.is_ok() {
                            cleaned += 1;
                        }
                    }
                }
            }
        }
        
        Ok(cleaned)
    }

    // Private helper methods

    /// Calculate SHA256 hash of file data
    fn calculate_sha256(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Calculate MD5 hash of file data
    fn calculate_md5(&self, data: &[u8]) -> String {
        let digest = md5::compute(data);
        format!("{:x}", digest)
    }

    /// Detect file type based on magic bytes and extension
    fn detect_file_type(&self, data: &[u8], filename: &str) -> String {
        // Check magic bytes for common file types
        if data.len() >= 4 {
            match &data[0..4] {
                [0x4D, 0x5A, _, _] => return "PE Executable".to_string(),
                [0x50, 0x4B, 0x03, 0x04] => return "ZIP Archive".to_string(),
                [0x25, 0x50, 0x44, 0x46] => return "PDF Document".to_string(),
                [0x7F, 0x45, 0x4C, 0x46] => return "ELF Executable".to_string(),
                _ => {}
            }
        }
        
        // Fallback to extension-based detection
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown");
            
        match extension.to_lowercase().as_str() {
            "exe" | "dll" | "sys" => "Windows Executable".to_string(),
            "pdf" => "PDF Document".to_string(),
            "zip" | "rar" | "7z" => "Archive".to_string(),
            "doc" | "docx" => "Word Document".to_string(),
            "apk" => "Android Package".to_string(),
            _ => format!("Unknown ({})", extension),
        }
    }

    /// Check if file exists by hash
    async fn file_exists_by_hash(&self, hash: &str) -> Result<bool> {
        let mut entries = async_fs::read_dir(&self.storage_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".meta") {
                    let file_id = file_name.trim_end_matches(".meta");
                    if let Ok(metadata) = self.get_metadata(file_id).await {
                        if metadata.sha256_hash == hash {
                            return Ok(true);
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }

    /// Store metadata to disk
    async fn store_metadata(&self, metadata: &FileMetadata) -> Result<()> {
        let metadata_path = self.storage_path.join(format!("{}.meta", metadata.file_id));
        let metadata_json = serde_json::to_string_pretty(metadata)?;
        async_fs::write(metadata_path, metadata_json).await?;
        Ok(())
    }

    /// Log action to file
    async fn log_action(&self, log_entry: &str) -> Result<()> {
        let log_path = self.storage_path.join("file_handler.log");
        let mut log_content = if log_path.exists() {
            async_fs::read_to_string(&log_path).await?
        } else {
            String::new()
        };
        
        log_content.push_str(log_entry);
        log_content.push('\n');
        
        async_fs::write(log_path, log_content).await?;
        Ok(())
    }

    /// Calculate total size of directory
    async fn calculate_directory_size(&self, dir: &Path) -> Result<u64> {
        let mut total_size = 0;
        let mut entries = async_fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                total_size += metadata.len();
            }
        }
        
        Ok(total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_storage_and_retrieval() {
        let temp_dir = tempdir().unwrap();
        let handler = FileHandler::new(temp_dir.path().to_str().unwrap()).unwrap();
        
        let test_data = b"This is test malware content";
        let metadata = handler
            .store_file(test_data, "test.exe", Some(0.1), Some("0x123".to_string()))
            .await
            .unwrap();
        
        assert_eq!(metadata.original_name, "test.exe");
        assert_eq!(metadata.file_size, test_data.len() as u64);
        
        let retrieved_data = handler.get_file(&metadata.file_id).await.unwrap();
        assert_eq!(retrieved_data, test_data);
    }

    #[tokio::test]
    async fn test_file_quarantine() {
        let temp_dir = tempdir().unwrap();
        let handler = FileHandler::new(temp_dir.path().to_str().unwrap()).unwrap();
        
        let test_data = b"Malicious content";
        let metadata = handler
            .store_file(test_data, "malware.exe", None, None)
            .await
            .unwrap();
        
        handler
            .quarantine_file(&metadata.file_id, "Detected as malware")
            .await
            .unwrap();
        
        let updated_metadata = handler.get_metadata(&metadata.file_id).await.unwrap();
        matches!(updated_metadata.analysis_status, AnalysisStatus::Quarantined);
    }
}