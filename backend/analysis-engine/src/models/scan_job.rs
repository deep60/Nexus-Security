use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Status of a scan job in the analysis pipeline
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "scan_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ScanStatus {
    /// Job is queued and waiting to be processed
    Queued,
    /// Job is currently being processed
    Processing,
    /// Job completed successfully
    Completed,
    /// Job failed due to an error
    Failed,
    /// Job was cancelled by user or system
    Cancelled,
    /// Job timed out during processing
    Timeout,
}

/// Priority level for scan jobs
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, PartialOrd, Ord)]
#[sqlx(type_name = "scan_priority", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ScanPriority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

impl Default for ScanPriority {
    fn default() -> Self {
        ScanPriority::Medium
    }
}

/// Type of artifact being scanned
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "artifact_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ArtifactType {
    File,
    Url,
    Email,
    Hash,
    IpAddress,
    Domain,
    Archive,
}

/// Configuration for different analyzers to run
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalyzerConfig {
    /// Run static code analysis
    pub static_analysis: bool,
    /// Run dynamic analysis in sandbox
    pub dynamic_analysis: bool,
    /// Check hash reputation databases
    pub hash_check: bool,
    /// Run YARA rule matching
    pub yara_scan: bool,
    /// Use ML-based detection
    pub ml_detection: bool,
    /// Analyze network behavior
    pub network_analysis: bool,
    /// Run signature detection
    pub signature_scan: bool,
    /// Apply heuristic analysis
    pub heuristic_analysis: bool,
    /// Maximum execution time for sandbox (seconds)
    pub sandbox_timeout: Option<u64>,
    /// Enable deep scanning (resource intensive)
    pub deep_scan: bool,
}

/// Metadata about the submitted artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// Original filename (if applicable)
    pub filename: Option<String>,
    /// File size in bytes
    pub size: Option<u64>,
    /// MIME type
    pub mime_type: Option<String>,
    /// MD5 hash
    pub md5: Option<String>,
    /// SHA1 hash
    pub sha1: Option<String>,
    /// SHA256 hash
    pub sha256: Option<String>,
    /// Additional custom metadata
    #[serde(default)]
    pub custom: serde_json::Value,
}

/// Progress tracking for multi-stage analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanProgress {
    /// Total number of analysis stages
    pub total_stages: u32,
    /// Number of completed stages
    pub completed_stages: u32,
    /// Current stage being processed
    pub current_stage: Option<String>,
    /// Overall percentage complete (0-100)
    pub percentage: u8,
}

impl ScanProgress {
    pub fn new(total_stages: u32) -> Self {
        Self {
            total_stages,
            completed_stages: 0,
            current_stage: None,
            percentage: 0,
        }
    }

    pub fn update(&mut self, completed: u32, current_stage: Option<String>) {
        self.completed_stages = completed;
        self.current_stage = current_stage;
        self.percentage = if self.total_stages > 0 {
            ((completed as f32 / self.total_stages as f32) * 100.0) as u8
        } else {
            0
        };
    }
}

/// Main scan job structure
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScanJob {
    /// Unique identifier for the scan job
    pub id: Uuid,
    
    /// User ID who submitted the scan
    pub user_id: Uuid,
    
    /// Associated bounty ID (if this scan is part of a bounty)
    pub bounty_id: Option<Uuid>,
    
    /// Type of artifact being scanned
    #[sqlx(try_from = "String")]
    pub artifact_type: ArtifactType,
    
    /// S3 key or URL of the artifact
    pub artifact_location: String,
    
    /// Current status of the scan
    #[sqlx(try_from = "String")]
    pub status: ScanStatus,
    
    /// Priority level
    #[sqlx(try_from = "String")]
    pub priority: ScanPriority,
    
    /// Configuration for analyzers
    #[sqlx(json)]
    pub config: AnalyzerConfig,
    
    /// Metadata about the artifact
    #[sqlx(json)]
    pub metadata: ArtifactMetadata,
    
    /// Current progress (0-100)
    #[sqlx(json)]
    pub progress: ScanProgress,
    
    /// Result ID once analysis is complete
    pub result_id: Option<Uuid>,
    
    /// Error message if scan failed
    pub error_message: Option<String>,
    
    /// Number of retry attempts
    #[sqlx(default)]
    pub retry_count: i32,
    
    /// Maximum allowed retries
    #[sqlx(default)]
    pub max_retries: i32,
    
    /// Timestamp when job was created
    pub created_at: DateTime<Utc>,
    
    /// Timestamp when job was last updated
    pub updated_at: DateTime<Utc>,
    
    /// Timestamp when processing started
    pub started_at: Option<DateTime<Utc>>,
    
    /// Timestamp when processing completed
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Estimated completion time
    pub estimated_completion: Option<DateTime<Utc>>,
    
    /// Worker/engine ID processing this job
    pub assigned_worker: Option<String>,
}

impl ScanJob {
    /// Create a new scan job
    pub fn new(
        user_id: Uuid,
        artifact_type: ArtifactType,
        artifact_location: String,
        metadata: ArtifactMetadata,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            bounty_id: None,
            artifact_type,
            artifact_location,
            status: ScanStatus::Queued,
            priority: ScanPriority::default(),
            config: AnalyzerConfig::default(),
            metadata,
            progress: ScanProgress::default(),
            result_id: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            estimated_completion: None,
            assigned_worker: None,
        }
    }

    /// Update job status
    pub fn update_status(&mut self, status: ScanStatus) {
        self.status = status;
        self.updated_at = Utc::now();

        match status {
            ScanStatus::Processing => {
                if self.started_at.is_none() {
                    self.started_at = Some(Utc::now());
                }
            }
            ScanStatus::Completed | ScanStatus::Failed | ScanStatus::Cancelled | ScanStatus::Timeout => {
                self.completed_at = Some(Utc::now());
            }
            _ => {}
        }
    }

    /// Update progress
    pub fn update_progress(&mut self, progress: ScanProgress) {
        self.progress = progress;
        self.updated_at = Utc::now();
    }

    /// Check if job can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
            && matches!(self.status, ScanStatus::Failed | ScanStatus::Timeout)
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
        self.updated_at = Utc::now();
    }

    /// Mark job as failed
    pub fn mark_failed(&mut self, error: String) {
        self.status = ScanStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark job as completed
    pub fn mark_completed(&mut self, result_id: Uuid) {
        self.status = ScanStatus::Completed;
        self.result_id = Some(result_id);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        self.progress.percentage = 100;
    }

    /// Calculate processing duration if available
    pub fn processing_duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    /// Check if job is terminal (completed, failed, or cancelled)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            ScanStatus::Completed | ScanStatus::Failed | ScanStatus::Cancelled | ScanStatus::Timeout
        )
    }

    /// Check if job is active (queued or processing)
    pub fn is_active(&self) -> bool {
        matches!(self.status, ScanStatus::Queued | ScanStatus::Processing)
    }
}

/// Request to create a new scan job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScanJobRequest {
    pub artifact_type: ArtifactType,
    pub artifact_location: String,
    pub bounty_id: Option<Uuid>,
    pub priority: Option<ScanPriority>,
    pub config: Option<AnalyzerConfig>,
    pub metadata: ArtifactMetadata,
}

/// Response when querying scan job status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanJobResponse {
    pub id: Uuid,
    pub status: ScanStatus,
    pub priority: ScanPriority,
    pub progress: ScanProgress,
    pub result_id: Option<Uuid>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

impl From<ScanJob> for ScanJobResponse {
    fn from(job: ScanJob) -> Self {
        Self {
            id: job.id,
            status: job.status,
            priority: job.priority,
            progress: job.progress,
            result_id: job.result_id,
            error_message: job.error_message,
            created_at: job.created_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
            estimated_completion: job.estimated_completion,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_job_creation() {
        let user_id = Uuid::new_v4();
        let metadata = ArtifactMetadata {
            filename: Some("test.exe".to_string()),
            size: Some(1024),
            mime_type: Some("application/x-executable".to_string()),
            md5: None,
            sha1: None,
            sha256: None,
            custom: serde_json::json!({}),
        };

        let job = ScanJob::new(
            user_id,
            ArtifactType::File,
            "s3://bucket/test.exe".to_string(),
            metadata,
        );

        assert_eq!(job.status, ScanStatus::Queued);
        assert_eq!(job.priority, ScanPriority::Medium);
        assert_eq!(job.retry_count, 0);
    }

    #[test]
    fn test_progress_calculation() {
        let mut progress = ScanProgress::new(5);
        progress.update(2, Some("Static Analysis".to_string()));
        
        assert_eq!(progress.completed_stages, 2);
        assert_eq!(progress.percentage, 40);
    }

    #[test]
    fn test_retry_logic() {
        let mut job = ScanJob::new(
            Uuid::new_v4(),
            ArtifactType::File,
            "test".to_string(),
            ArtifactMetadata {
                filename: None,
                size: None,
                mime_type: None,
                md5: None,
                sha1: None,
                sha256: None,
                custom: serde_json::json!({}),
            },
        );

        job.mark_failed("Test error".to_string());
        assert!(job.can_retry());

        job.max_retries = 0;
        assert!(!job.can_retry());
    }
}