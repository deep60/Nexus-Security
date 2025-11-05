use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn, debug};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::analyzers::{
    static_analyzer::StaticAnalyzer,
    dynamic_analyzer::DynamicAnalyzer,
    hash_analyzer::HashAnalyzer,
    yara_engine::YaraEngine,
    ml_analyzer::MLAnalyzer,
    signature_matcher::SignatureMatcher,
    heuristic_engine::HeuristicEngine,
};
use crate::scanners::{
    file_scanner::FileScanner,
    url_scanner::URLScanner,
};
use crate::models::{
    analysis_result::AnalysisResult,
    threat_indicator::ThreatIndicator,
    scan_job::{ScanJob, ScanJobStatus, ScanType},
};
use crate::storage::{
    s3_client::S3Client,
    database::Database,
};

use shared::messaging::kafka_client::KafkaClient;
use shared::observability::metrics::MetricsCollector;
use shared::types::errors::AnalysisError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    pub worker_id: String,
    pub max_concurrent_jobs: usize,
    pub job_timeout_secs: u64,
    pub retry_attempts: u32,
    pub heartbeat_interval_secs: u64,
    pub enable_dynamic_analysis: bool,
    pub enable_ml_analysis: bool,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            worker_id: Uuid::new_v4().to_string(),
            max_concurrent_jobs: 10,
            job_timeout_secs: 300,
            retry_attempts: 3,
            heartbeat_interval_secs: 30,
            enable_dynamic_analysis: true,
            enable_ml_analysis: true,
        }
    }
}

pub struct Worker {
    config: WorkerConfig,
    kafka_client: Arc<KafkaClient>,
    s3_client: Arc<S3Client>,
    database: Arc<Database>,
    metrics: Arc<MetricsCollector>,
    
    // Analyzers
    static_analyzer: Arc<StaticAnalyzer>,
    dynamic_analyzer: Option<Arc<DynamicAnalyzer>>,
    hash_analyzer: Arc<HashAnalyzer>,
    yara_engine: Arc<YaraEngine>,
    ml_analyzer: Option<Arc<MLAnalyzer>>,
    signature_matcher: Arc<SignatureMatcher>,
    heuristic_engine: Arc<HeuristicEngine>,
    
    // Scanners
    file_scanner: Arc<FileScanner>,
    url_scanner: Arc<URLScanner>,
    
    // Channels
    shutdown_tx: Option<mpsc::Sender<()>>,
    shutdown_rx: Option<mpsc::Receiver<()>>,
}

impl Worker {
    pub async fn new(
        config: WorkerConfig,
        kafka_client: Arc<KafkaClient>,
        s3_client: Arc<S3Client>,
        database: Arc<Database>,
        metrics: Arc<MetricsCollector>,
    ) -> Result<Self, AnalysisError> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        
        // Initialize analyzers
        let static_analyzer = Arc::new(StaticAnalyzer::new().await?);
        let dynamic_analyzer = if config.enable_dynamic_analysis {
            Some(Arc::new(DynamicAnalyzer::new().await?))
        } else {
            None
        };
        let hash_analyzer = Arc::new(HashAnalyzer::new().await?);
        let yara_engine = Arc::new(YaraEngine::new("./rules/yara").await?);
        let ml_analyzer = if config.enable_ml_analysis {
            Some(Arc::new(MLAnalyzer::new("./ml_models").await?))
        } else {
            None
        };
        let signature_matcher = Arc::new(SignatureMatcher::new().await?);
        let heuristic_engine = Arc::new(HeuristicEngine::new().await?);
        
        // Initialize scanners
        let file_scanner = Arc::new(FileScanner::new());
        let url_scanner = Arc::new(URLScanner::new().await?);
        
        info!(
            worker_id = %config.worker_id,
            "Worker initialized successfully"
        );
        
        Ok(Self {
            config,
            kafka_client,
            s3_client,
            database,
            metrics,
            static_analyzer,
            dynamic_analyzer,
            hash_analyzer,
            yara_engine,
            ml_analyzer,
            signature_matcher,
            heuristic_engine,
            file_scanner,
            url_scanner,
            shutdown_tx: Some(shutdown_tx),
            shutdown_rx: Some(shutdown_rx),
        })
    }
    
    /// Start the worker and begin processing jobs
    pub async fn start(&mut self) -> Result<(), AnalysisError> {
        info!(
            worker_id = %self.config.worker_id,
            "Starting worker"
        );
        
        let mut shutdown_rx = self.shutdown_rx.take()
            .ok_or_else(|| AnalysisError::Internal("Shutdown receiver already taken".to_string()))?;
        
        // Start heartbeat task
        let heartbeat_handle = self.start_heartbeat();
        
        // Create semaphore for concurrent job limiting
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_concurrent_jobs));
        
        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("Received shutdown signal, stopping worker");
                    heartbeat_handle.abort();
                    break;
                }
                
                // Poll for new jobs
                job_result = self.poll_job() => {
                    match job_result {
                        Ok(Some(job)) => {
                            let permit = semaphore.clone().acquire_owned().await
                                .map_err(|e| AnalysisError::Internal(e.to_string()))?;
                            
                            let worker = self.clone_for_processing();
                            
                            tokio::spawn(async move {
                                if let Err(e) = worker.process_job(job).await {
                                    error!(error = %e, "Failed to process job");
                                }
                                drop(permit);
                            });
                        }
                        Ok(None) => {
                            // No jobs available, sleep briefly
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                        Err(e) => {
                            error!(error = %e, "Error polling for jobs");
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Poll for a new job from the queue
    async fn poll_job(&self) -> Result<Option<ScanJob>, AnalysisError> {
        // Try to claim a job from the database
        match self.database.claim_pending_job(&self.config.worker_id).await {
            Ok(job) => {
                if let Some(ref j) = job {
                    info!(
                        job_id = %j.id,
                        scan_type = ?j.scan_type,
                        "Claimed job"
                    );
                    self.metrics.increment_counter("jobs_claimed", &[]);
                }
                Ok(job)
            }
            Err(e) => {
                warn!(error = %e, "Error claiming job");
                Err(e)
            }
        }
    }
    
    /// Process a scan job
    async fn process_job(&self, mut job: ScanJob) -> Result<(), AnalysisError> {
        let start_time = std::time::Instant::now();
        
        info!(
            job_id = %job.id,
            scan_type = ?job.scan_type,
            "Processing job"
        );
        
        // Update job status to processing
        job.status = ScanJobStatus::Processing;
        job.started_at = Some(chrono::Utc::now());
        self.database.update_job_status(&job).await?;
        
        // Process with timeout
        let timeout_duration = Duration::from_secs(self.config.job_timeout_secs);
        let result = tokio::time::timeout(
            timeout_duration,
            self.analyze_job(&job)
        ).await;
        
        match result {
            Ok(Ok(analysis_result)) => {
                // Job completed successfully
                job.status = ScanJobStatus::Completed;
                job.completed_at = Some(chrono::Utc::now());
                job.result = Some(analysis_result.clone());
                
                self.database.update_job_status(&job).await?;
                self.database.save_analysis_result(&analysis_result).await?;
                
                // Publish result to Kafka
                self.kafka_client.publish_analysis_result(&analysis_result).await?;
                
                let duration = start_time.elapsed();
                info!(
                    job_id = %job.id,
                    duration_ms = duration.as_millis(),
                    threat_score = analysis_result.threat_score,
                    "Job completed successfully"
                );
                
                self.metrics.record_histogram(
                    "job_duration_seconds",
                    duration.as_secs_f64(),
                    &[("status", "success")]
                );
                self.metrics.increment_counter("jobs_completed", &[]);
            }
            Ok(Err(e)) => {
                // Job failed
                error!(job_id = %job.id, error = %e, "Job failed");
                
                job.retry_count += 1;
                if job.retry_count >= self.config.retry_attempts {
                    job.status = ScanJobStatus::Failed;
                    job.error_message = Some(e.to_string());
                } else {
                    job.status = ScanJobStatus::Pending;
                }
                
                self.database.update_job_status(&job).await?;
                self.metrics.increment_counter("jobs_failed", &[]);
            }
            Err(_) => {
                // Timeout
                warn!(job_id = %job.id, "Job timeout");
                
                job.retry_count += 1;
                if job.retry_count >= self.config.retry_attempts {
                    job.status = ScanJobStatus::Failed;
                    job.error_message = Some("Job timeout".to_string());
                } else {
                    job.status = ScanJobStatus::Pending;
                }
                
                self.database.update_job_status(&job).await?;
                self.metrics.increment_counter("jobs_timeout", &[]);
            }
        }
        
        Ok(())
    }
    
    /// Analyze a job using all available analyzers
    async fn analyze_job(&self, job: &ScanJob) -> Result<AnalysisResult, AnalysisError> {
        let mut analysis_result = AnalysisResult::new(job.id.clone());
        
        match job.scan_type {
            ScanType::File => {
                // Download file from S3
                let file_data = self.s3_client.download_file(&job.target).await?;
                
                // Scan file type
                let file_info = self.file_scanner.scan(&file_data).await?;
                analysis_result.file_info = Some(file_info);
                
                // Run hash analysis
                debug!(job_id = %job.id, "Running hash analysis");
                let hash_result = self.hash_analyzer.analyze(&file_data).await?;
                analysis_result.add_indicators(hash_result.indicators);
                
                // Run YARA rules
                debug!(job_id = %job.id, "Running YARA analysis");
                let yara_result = self.yara_engine.scan(&file_data).await?;
                analysis_result.add_indicators(yara_result.indicators);
                
                // Run static analysis
                debug!(job_id = %job.id, "Running static analysis");
                let static_result = self.static_analyzer.analyze(&file_data).await?;
                analysis_result.add_indicators(static_result.indicators);
                
                // Run signature matching
                debug!(job_id = %job.id, "Running signature matching");
                let signature_result = self.signature_matcher.match_signatures(&file_data).await?;
                analysis_result.add_indicators(signature_result.indicators);
                
                // Run ML analysis if enabled
                if let Some(ref ml_analyzer) = self.ml_analyzer {
                    debug!(job_id = %job.id, "Running ML analysis");
                    let ml_result = ml_analyzer.analyze(&file_data).await?;
                    analysis_result.add_indicators(ml_result.indicators);
                }
                
                // Run dynamic analysis if enabled
                if let Some(ref dynamic_analyzer) = self.dynamic_analyzer {
                    debug!(job_id = %job.id, "Running dynamic analysis");
                    let dynamic_result = dynamic_analyzer.analyze(&file_data).await?;
                    analysis_result.add_indicators(dynamic_result.indicators);
                }
                
                // Run heuristic analysis
                debug!(job_id = %job.id, "Running heuristic analysis");
                let heuristic_result = self.heuristic_engine.analyze(&analysis_result).await?;
                analysis_result.add_indicators(heuristic_result.indicators);
            }
            
            ScanType::URL => {
                // Scan URL
                debug!(job_id = %job.id, "Running URL analysis");
                let url_result = self.url_scanner.scan(&job.target).await?;
                analysis_result.add_indicators(url_result.indicators);
                
                // Run hash analysis on URL
                let url_hash_result = self.hash_analyzer.analyze_url(&job.target).await?;
                analysis_result.add_indicators(url_hash_result.indicators);
            }
            
            ScanType::Hash => {
                // Analyze hash only
                debug!(job_id = %job.id, "Running hash lookup");
                let hash_result = self.hash_analyzer.lookup_hash(&job.target).await?;
                analysis_result.add_indicators(hash_result.indicators);
            }
        }
        
        // Calculate final threat score
        analysis_result.calculate_threat_score();
        analysis_result.determine_verdict();
        
        Ok(analysis_result)
    }
    
    /// Start heartbeat task
    fn start_heartbeat(&self) -> JoinHandle<()> {
        let worker_id = self.config.worker_id.clone();
        let database = self.database.clone();
        let interval_secs = self.config.heartbeat_interval_secs;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = database.update_worker_heartbeat(&worker_id).await {
                    warn!(
                        worker_id = %worker_id,
                        error = %e,
                        "Failed to update heartbeat"
                    );
                }
            }
        })
    }
    
    /// Gracefully shutdown the worker
    pub async fn shutdown(&mut self) -> Result<(), AnalysisError> {
        info!(worker_id = %self.config.worker_id, "Shutting down worker");
        
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        
        // Mark worker as offline
        self.database.mark_worker_offline(&self.config.worker_id).await?;
        
        Ok(())
    }
    
    /// Clone worker data for spawned tasks
    fn clone_for_processing(&self) -> Self {
        Self {
            config: self.config.clone(),
            kafka_client: self.kafka_client.clone(),
            s3_client: self.s3_client.clone(),
            database: self.database.clone(),
            metrics: self.metrics.clone(),
            static_analyzer: self.static_analyzer.clone(),
            dynamic_analyzer: self.dynamic_analyzer.clone(),
            hash_analyzer: self.hash_analyzer.clone(),
            yara_engine: self.yara_engine.clone(),
            ml_analyzer: self.ml_analyzer.clone(),
            signature_matcher: self.signature_matcher.clone(),
            heuristic_engine: self.heuristic_engine.clone(),
            file_scanner: self.file_scanner.clone(),
            url_scanner: self.url_scanner.clone(),
            shutdown_tx: None,
            shutdown_rx: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_worker_initialization() {
        // Test worker initialization
    }
    
    #[tokio::test]
    async fn test_job_processing() {
        // Test job processing
    }
}