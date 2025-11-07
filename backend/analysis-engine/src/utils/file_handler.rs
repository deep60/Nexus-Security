use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use tokio::fs as async_fs;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use md5;
use crate::utils::constants::MAX_FILE_SIZE;
use chrono::Utc;

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
    pub fn new(base_storage_path: &str) -> Result<Self> {
        let storage_path = PathBuf::from(base_storage_path);
        let quarantine_path = storage_path.join("quarantine");
        let temp_path = storage_path.join("temp");

        fs::create_dir_all(&storage_path)?;
        fs::create_dir_all(&quarantine_path)?;
        fs::create_dir_all(&temp_path)?;

        Ok(FileHandler {
            storage_path,
            quarantine_path,
            temp_path,
        })
    }

    pub async fn store_file(
        &self,
        file_data: &[u8],
        original_name: &str,
        bounty_amount: Option<f64>,
        submitter_address: Option<String>,
    ) -> Result<FileMetadata> {
        if file_data.len() as u64 > MAX_FILE_SIZE {
            return Err(anyhow!("File size exceeds maximum limit of {}MB", MAX_FILE_SIZE / 1024 / 1024));
        }

        let extension = Path::new(original_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if !ALLOWED_EXTENSIONS.contains(&extension.to_lowercase().as_str()) {
            return Err(anyhow!("File type '{}' is not allowed for analysis", extension));
        }

        let file_id = Uuid::new_v4().to_string();

        let sha256_hash = self.calculate_sha256(file_data);
        let md5_hash = self.calculate_md5(file_data);

        if self.file_exists_by_hash(&sha256_hash).await? {
            return Err(anyhow!("File with identical hash already exists"));
        }

        let file_path = self.storage_path.join(&file_id);
        async_fs::write(&file_path, file_data).await?;

        let metadata = FileMetadata {
            file_id: file_id.clone(),
            original_name: original_name.to_string(),
            file_size: file_data.len() as u64,
            file_type: self.detect_file_type(file_data, original_name),
            sha256_hash,
            md5_hash,
            upload_timestamp: Utc::now(),
            analysis_status: AnalysisStatus::Pending,
            bounty_amount,
            submitter_address,
        };

        self.store_metadata(&metadata).await?;

        Ok(metadata)
    }

    pub async fn get_file(&self, file_id: &str) -> Result<Vec<u8>> {
        let file_path = self.storage_path.join(file_id);

        if !file_path.exists() {
            return Err(anyhow!("File not found: {}", file_id));
        }

        let file_data = async_fs::read(file_path).await?;
        Ok(file_data)
    }

    pub async fn get_metadata(&self, file_id: &str) -> Result<FileMetadata> {
        let metadata_path = self.storage_path.join(format!("{}.meta", file_id));

        if !metadata_path.exists() {
            return Err(anyhow!("Metadata not found for file: {}", file_id));
        }

        let metadata_json = async_fs::read_to_string(metadata_path).await?;
        let metadata: FileMetadata = serde_json::from_str(&metadata_json)?;

        Ok(metadata)
    }

    pub async fn update_analysis_status(&self, file_id: &str, status: AnalysisStatus) -> Result<()> {
        let mut metadata = self.get_metadata(file_id).await?;
        metadata.analysis_status = status;
        self.store_metadata(&metadata).await?;

        Ok(())
    }

    pub async fn quarantine_file(&self, file_id: &str, reason: &str) -> Result<()> {
        let source_path = self.storage_path.join(file_id);
        let dest_path = self.quarantine_path.join(file_id);

        async_fs::rename(source_path, dest_path).await?;

        self.update_analysis_status(file_id, AnalysisStatus::Quarantined).await?;
        self.log_action(&format!("Quarantined {}: {}", file_id, reason)).await?;

        Ok(())
    }

    pub async fn clean_old_files(&self, max_age_days: u32) -> Result<usize> {
        let mut entries = async_fs::read_dir(&self.storage_path).await?;
        let mut cleaned = 0;

        let cutoff_duration = std::time::Duration::from_secs(max_age_days as u64 * 24 * 60 * 60);

        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                // Check if file modification time is older than cutoff
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed > cutoff_duration {
                            async_fs::remove_file(entry.path()).await?;
                            cleaned += 1;
                        }
                    }
                }
            }
        }

        Ok(cleaned)
    }

    fn calculate_sha256(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn calculate_md5(&self, data: &[u8]) -> String {
        let digest = md5::compute(data);
        format!("{:x}", digest)
    }

    fn detect_file_type(&self, data: &[u8], filename: &str) -> String {
        if data.len() >= 4 {
            match &data[0..4] {
                [0x4D, 0x5A, _, _] => return "PE Executable".to_string(),
                [0x50, 0x4B, 0x03, 0x04] => return "ZIP Archive".to_string(),
                [0x25, 0x50, 0x44, 0x46] => return "PDF Document".to_string(),
                [0x7F, 0x45, 0x4C, 0x46] => return "ELF Executable".to_string(),
                _ => {}
            }
        }
        
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

    async fn file_exists_by_hash(&self, hash: &str) -> Result<bool> {
        let mut entries = async_fs::read_dir(&self.storage_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let file_name = entry.file_name().to_str().unwrap_or("");
            if file_name.ends_with(".meta") {
                let file_id = file_name.trim_end_matches(".meta");
                if let Ok(metadata) = self.get_metadata(file_id).await {
                    if metadata.sha256_hash == hash {
                        return Ok(true);
                    }
                }
            }
        }
        
        Ok(false)
    }

    async fn store_metadata(&self, metadata: &FileMetadata) -> Result<()> {
        let metadata_path = self.storage_path.join(format!("{}.meta", metadata.file_id));
        let metadata_json = serde_json::to_string_pretty(metadata)?;
        async_fs::write(metadata_path, metadata_json).await?;
        Ok(())
    }

    async fn log_action(&self, log_entry: &str) -> Result<()> {
        let log_path = self.storage_path.join("file_handler.log");
        let mut log_content = if log_path.exists() {
            async_fs::read_to_string(&log_path).await.unwrap_or_default()
        } else {
            String::new()
        };
        
        log_content.push_str(log_entry);
        log_content.push('\n');
        
        async_fs::write(log_path, log_content).await?;
        Ok(())
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
        assert_eq!(retrieved_data, test_data.to_vec());
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
        assert!(matches!(updated_metadata.analysis_status, AnalysisStatus::Quarantined));
    }
}