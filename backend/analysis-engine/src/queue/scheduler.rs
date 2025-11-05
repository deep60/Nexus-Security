use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::models::scan_job::{ScanJob, ScanJobStatus, ScanPriority};
use crate::queue::worker::WorkerPool;
use shared::database::redis::RedisClient;
use shared::messaging::kafka_client::KafkaProducer;
use shared::messaging::event_types::AnalysisEvent;
use shared::types::errors::AppError;

const SCHEDULER_TICK_INTERVAL: Duration = Duration::from_secs(5);
const JOB_TIMEOUT_SECONDS: u64 = 3600; // 1 hour
const MAX_RETRIES: u8 = 3;
const PRIORITY_WEIGHT_HIGH: u64 = 100;
const PRIORITY_WEIGHT_MEDIUM: u64 = 50;
const PRIORITY_WEIGHT_LOW: u64 = 10;

/// Job scheduling strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulingStrategy {
    /// First-In-First-Out
    FIFO,
    /// Priority-based scheduling
    Priority,
    /// Shortest Job First
    SJF,
    /// Fair scheduling across submitters
    FairShare,
}

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub strategy: SchedulingStrategy,
    pub max_concurrent_jobs: usize,
    pub job_timeout_seconds: u64,
    pub retry_delay_seconds: u64,
    pub enable_job_batching: bool,
    pub batch_size: usize,
    pub health_check_interval_seconds: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            strategy: SchedulingStrategy::Priority,
            max_concurrent_jobs: 100,
            job_timeout_seconds: JOB_TIMEOUT_SECONDS,
            retry_delay_seconds: 300, // 5 minutes
            enable_job_batching: true,
            batch_size: 10,
            health_check_interval_seconds: 30,
        }
    }
}

/// Job queue statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueStats {
    pub pending_jobs: usize,
    pub running_jobs: usize,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub total_processing_time_ms: u64,
    pub average_processing_time_ms: f64,
    pub queue_throughput_per_minute: f64,
    pub last_updated: Option<DateTime<Utc>>,
}

/// Job scheduler for managing analysis jobs
pub struct JobScheduler {
    config: SchedulerConfig,
    redis: Arc<RedisClient>,
    kafka: Arc<KafkaProducer>,
    worker_pool: Arc<WorkerPool>,
    
    // Job queues by priority
    high_priority_queue: Arc<Mutex<Vec<ScanJob>>>,
    medium_priority_queue: Arc<Mutex<Vec<ScanJob>>>,
    low_priority_queue: Arc<Mutex<Vec<ScanJob>>>,
    
    // Running jobs tracking
    running_jobs: Arc<RwLock<HashMap<Uuid, JobExecution>>>,
    
    // Statistics
    stats: Arc<RwLock<QueueStats>>,
    
    // Fair share tracking (user_id -> job count)
    user_job_counts: Arc<RwLock<HashMap<String, usize>>>,
    
    // Shutdown signal
    shutdown: Arc<Mutex<bool>>,
}

/// Job execution context
#[derive(Debug, Clone)]
struct JobExecution {
    job: ScanJob,
    started_at: SystemTime,
    retry_count: u8,
    worker_id: Option<String>,
}

impl JobScheduler {
    /// Create a new job scheduler
    pub fn new(
        config: SchedulerConfig,
        redis: Arc<RedisClient>,
        kafka: Arc<KafkaProducer>,
        worker_pool: Arc<WorkerPool>,
    ) -> Self {
        Self {
            config,
            redis,
            kafka,
            worker_pool,
            high_priority_queue: Arc::new(Mutex::new(Vec::new())),
            medium_priority_queue: Arc::new(Mutex::new(Vec::new())),
            low_priority_queue: Arc::new(Mutex::new(Vec::new())),
            running_jobs: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(QueueStats::default())),
            user_job_counts: Arc::new(RwLock::new(HashMap::new())),
            shutdown: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the scheduler
    pub async fn start(self: Arc<Self>) -> Result<(), AppError> {
        info!("Starting job scheduler with strategy: {:?}", self.config.strategy);

        // Start scheduler loop
        let scheduler = Arc::clone(&self);
        tokio::spawn(async move {
            scheduler.run_scheduler_loop().await;
        });

        // Start timeout monitor
        let timeout_monitor = Arc::clone(&self);
        tokio::spawn(async move {
            timeout_monitor.run_timeout_monitor().await;
        });

        // Start health check
        let health_check = Arc::clone(&self);
        tokio::spawn(async move {
            health_check.run_health_check().await;
        });

        // Load pending jobs from Redis
        self.load_pending_jobs().await?;

        Ok(())
    }

    /// Main scheduler loop
    async fn run_scheduler_loop(&self) {
        let mut ticker = interval(SCHEDULER_TICK_INTERVAL);

        loop {
            ticker.tick().await;

            // Check for shutdown signal
            if *self.shutdown.lock().await {
                info!("Scheduler shutting down");
                break;
            }

            // Schedule next batch of jobs
            if let Err(e) = self.schedule_next_batch().await {
                error!("Error scheduling jobs: {}", e);
            }

            // Update statistics
            self.update_stats().await;
        }
    }

    /// Schedule the next batch of jobs
    async fn schedule_next_batch(&self) -> Result<(), AppError> {
        let running_count = self.running_jobs.read().await.len();
        let available_slots = self.config.max_concurrent_jobs.saturating_sub(running_count);

        if available_slots == 0 {
            debug!("No available worker slots");
            return Ok(());
        }

        let batch_size = if self.config.enable_job_batching {
            available_slots.min(self.config.batch_size)
        } else {
            available_slots
        };

        let jobs = self.select_jobs_to_schedule(batch_size).await?;

        for job in jobs {
            if let Err(e) = self.dispatch_job(job).await {
                error!("Failed to dispatch job: {}", e);
            }
        }

        Ok(())
    }

    /// Select jobs to schedule based on strategy
    async fn select_jobs_to_schedule(&self, count: usize) -> Result<Vec<ScanJob>, AppError> {
        match self.config.strategy {
            SchedulingStrategy::FIFO => self.select_fifo(count).await,
            SchedulingStrategy::Priority => self.select_priority(count).await,
            SchedulingStrategy::SJF => self.select_shortest_first(count).await,
            SchedulingStrategy::FairShare => self.select_fair_share(count).await,
        }
    }

    /// FIFO scheduling
    async fn select_fifo(&self, count: usize) -> Result<Vec<ScanJob>, AppError> {
        let mut selected = Vec::new();

        // Check each queue in order
        let mut high_queue = self.high_priority_queue.lock().await;
        while selected.len() < count && !high_queue.is_empty() {
            if let Some(job) = high_queue.first() {
                selected.push(job.clone());
                high_queue.remove(0);
            }
        }
        drop(high_queue);

        if selected.len() < count {
            let mut medium_queue = self.medium_priority_queue.lock().await;
            while selected.len() < count && !medium_queue.is_empty() {
                if let Some(job) = medium_queue.first() {
                    selected.push(job.clone());
                    medium_queue.remove(0);
                }
            }
            drop(medium_queue);
        }

        if selected.len() < count {
            let mut low_queue = self.low_priority_queue.lock().await;
            while selected.len() < count && !low_queue.is_empty() {
                if let Some(job) = low_queue.first() {
                    selected.push(job.clone());
                    low_queue.remove(0);
                }
            }
        }

        Ok(selected)
    }

    /// Priority-based scheduling
    async fn select_priority(&self, count: usize) -> Result<Vec<ScanJob>, AppError> {
        let mut selected = Vec::new();

        // Weighted selection based on priority
        let total_weight = PRIORITY_WEIGHT_HIGH + PRIORITY_WEIGHT_MEDIUM + PRIORITY_WEIGHT_LOW;
        let high_slots = (count * PRIORITY_WEIGHT_HIGH as usize) / total_weight as usize;
        let medium_slots = (count * PRIORITY_WEIGHT_MEDIUM as usize) / total_weight as usize;
        let low_slots = count.saturating_sub(high_slots + medium_slots);

        // Select from high priority
        let mut high_queue = self.high_priority_queue.lock().await;
        let high_take = high_slots.min(high_queue.len());
        selected.extend(high_queue.drain(..high_take));
        drop(high_queue);

        // Select from medium priority
        let mut medium_queue = self.medium_priority_queue.lock().await;
        let medium_take = medium_slots.min(medium_queue.len());
        selected.extend(medium_queue.drain(..medium_take));
        drop(medium_queue);

        // Select from low priority
        let mut low_queue = self.low_priority_queue.lock().await;
        let low_take = low_slots.min(low_queue.len());
        selected.extend(low_queue.drain(..low_take));

        // Fill remaining slots from any queue
        let remaining = count.saturating_sub(selected.len());
        if remaining > 0 {
            let additional_take = remaining.min(low_queue.len());
            selected.extend(low_queue.drain(..additional_take));
        }

        Ok(selected)
    }

    /// Shortest job first scheduling
    async fn select_shortest_first(&self, count: usize) -> Result<Vec<ScanJob>, AppError> {
        let mut all_jobs = Vec::new();

        // Collect all jobs
        let mut high_queue = self.high_priority_queue.lock().await;
        all_jobs.extend(high_queue.drain(..));
        drop(high_queue);

        let mut medium_queue = self.medium_priority_queue.lock().await;
        all_jobs.extend(medium_queue.drain(..));
        drop(medium_queue);

        let mut low_queue = self.low_priority_queue.lock().await;
        all_jobs.extend(low_queue.drain(..));
        drop(low_queue);

        // Sort by estimated processing time
        all_jobs.sort_by_key(|job| job.estimated_duration_ms.unwrap_or(u64::MAX));

        // Select shortest jobs
        let selected: Vec<_> = all_jobs.drain(..count.min(all_jobs.len())).collect();

        // Return unselected jobs to queues
        for job in all_jobs {
            self.enqueue_job_internal(job).await?;
        }

        Ok(selected)
    }

    /// Fair share scheduling
    async fn select_fair_share(&self, count: usize) -> Result<Vec<ScanJob>, AppError> {
        let user_counts = self.user_job_counts.read().await.clone();
        let mut all_jobs = Vec::new();

        // Collect all jobs with user info
        let mut high_queue = self.high_priority_queue.lock().await;
        all_jobs.extend(high_queue.drain(..));
        drop(high_queue);

        let mut medium_queue = self.medium_priority_queue.lock().await;
        all_jobs.extend(medium_queue.drain(..));
        drop(medium_queue);

        let mut low_queue = self.low_priority_queue.lock().await;
        all_jobs.extend(low_queue.drain(..));
        drop(low_queue);

        // Sort by user job count (favor users with fewer running jobs)
        all_jobs.sort_by_key(|job| {
            let user_count = user_counts.get(&job.submitted_by).copied().unwrap_or(0);
            (user_count, job.created_at)
        });

        // Select jobs
        let selected: Vec<_> = all_jobs.drain(..count.min(all_jobs.len())).collect();

        // Return unselected jobs to queues
        for job in all_jobs {
            self.enqueue_job_internal(job).await?;
        }

        Ok(selected)
    }

    /// Dispatch a job to a worker
    async fn dispatch_job(&self, mut job: ScanJob) -> Result<(), AppError> {
        info!("Dispatching job: {} (priority: {:?})", job.id, job.priority);

        // Update job status
        job.status = ScanJobStatus::Running;
        job.started_at = Some(Utc::now());

        // Track running job
        let execution = JobExecution {
            job: job.clone(),
            started_at: SystemTime::now(),
            retry_count: job.retry_count,
            worker_id: None,
        };

        self.running_jobs.write().await.insert(job.id, execution);

        // Update user job count
        let mut user_counts = self.user_job_counts.write().await;
        *user_counts.entry(job.submitted_by.clone()).or_insert(0) += 1;
        drop(user_counts);

        // Persist to Redis
        self.save_job_state(&job).await?;

        // Submit to worker pool
        match self.worker_pool.submit_job(job.clone()).await {
            Ok(_) => {
                // Publish event
                let event = AnalysisEvent::JobStarted {
                    job_id: job.id,
                    priority: job.priority,
                    timestamp: Utc::now(),
                };

                if let Err(e) = self.kafka.publish("analysis.events", &event).await {
                    warn!("Failed to publish job started event: {}", e);
                }

                Ok(())
            }
            Err(e) => {
                // Remove from running jobs on failure
                self.running_jobs.write().await.remove(&job.id);
                
                let mut user_counts = self.user_job_counts.write().await;
                if let Some(count) = user_counts.get_mut(&job.submitted_by) {
                    *count = count.saturating_sub(1);
                }

                error!("Failed to submit job to worker pool: {}", e);
                Err(e)
            }
        }
    }

    /// Enqueue a new job
    pub async fn enqueue_job(&self, job: ScanJob) -> Result<(), AppError> {
        info!("Enqueueing job: {} with priority: {:?}", job.id, job.priority);

        self.enqueue_job_internal(job.clone()).await?;
        self.save_job_state(&job).await?;

        // Publish event
        let event = AnalysisEvent::JobQueued {
            job_id: job.id,
            priority: job.priority,
            timestamp: Utc::now(),
        };

        self.kafka.publish("analysis.events", &event).await?;

        Ok(())
    }

    /// Internal job enqueuing
    async fn enqueue_job_internal(&self, job: ScanJob) -> Result<(), AppError> {
        match job.priority {
            ScanPriority::High => {
                self.high_priority_queue.lock().await.push(job);
            }
            ScanPriority::Medium => {
                self.medium_priority_queue.lock().await.push(job);
            }
            ScanPriority::Low => {
                self.low_priority_queue.lock().await.push(job);
            }
        }

        Ok(())
    }

    /// Mark job as completed
    pub async fn complete_job(&self, job_id: Uuid, success: bool) -> Result<(), AppError> {
        let mut running_jobs = self.running_jobs.write().await;
        
        if let Some(execution) = running_jobs.remove(&job_id) {
            let duration = execution.started_at.elapsed().unwrap_or_default();
            
            // Update user job count
            let mut user_counts = self.user_job_counts.write().await;
            if let Some(count) = user_counts.get_mut(&execution.job.submitted_by) {
                *count = count.saturating_sub(1);
            }
            drop(user_counts);

            // Update stats
            let mut stats = self.stats.write().await;
            if success {
                stats.completed_jobs += 1;
            } else {
                stats.failed_jobs += 1;
            }
            stats.total_processing_time_ms += duration.as_millis() as u64;
            stats.average_processing_time_ms = 
                stats.total_processing_time_ms as f64 / (stats.completed_jobs + stats.failed_jobs) as f64;
            drop(stats);

            info!(
                "Job {} completed in {}ms (success: {})",
                job_id,
                duration.as_millis(),
                success
            );

            // Clean up Redis state
            self.delete_job_state(job_id).await?;
        }

        Ok(())
    }

    /// Timeout monitor
    async fn run_timeout_monitor(&self) {
        let mut ticker = interval(Duration::from_secs(30));

        loop {
            ticker.tick().await;

            if *self.shutdown.lock().await {
                break;
            }

            if let Err(e) = self.check_timeouts().await {
                error!("Error checking timeouts: {}", e);
            }
        }
    }

    /// Check for timed out jobs
    async fn check_timeouts(&self) -> Result<(), AppError> {
        let timeout_duration = Duration::from_secs(self.config.job_timeout_seconds);
        let now = SystemTime::now();
        let mut timed_out_jobs = Vec::new();

        // Find timed out jobs
        let running_jobs = self.running_jobs.read().await;
        for (job_id, execution) in running_jobs.iter() {
            if let Ok(elapsed) = now.duration_since(execution.started_at) {
                if elapsed > timeout_duration {
                    timed_out_jobs.push((*job_id, execution.clone()));
                }
            }
        }
        drop(running_jobs);

        // Handle timeouts
        for (job_id, execution) in timed_out_jobs {
            warn!("Job {} timed out after {:?}", job_id, timeout_duration);
            
            if execution.retry_count < MAX_RETRIES {
                // Retry the job
                let mut job = execution.job.clone();
                job.retry_count += 1;
                job.status = ScanJobStatus::Pending;
                
                self.running_jobs.write().await.remove(&job_id);
                self.enqueue_job(job).await?;
            } else {
                // Mark as failed
                error!("Job {} exceeded max retries", job_id);
                self.complete_job(job_id, false).await?;
            }
        }

        Ok(())
    }

    /// Health check routine
    async fn run_health_check(&self) {
        let mut ticker = interval(Duration::from_secs(self.config.health_check_interval_seconds));

        loop {
            ticker.tick().await;

            if *self.shutdown.lock().await {
                break;
            }

            self.log_health_status().await;
        }
    }

    /// Log health status
    async fn log_health_status(&self) {
        let stats = self.stats.read().await;
        let high_queue = self.high_priority_queue.lock().await.len();
        let medium_queue = self.medium_priority_queue.lock().await.len();
        let low_queue = self.low_priority_queue.lock().await.len();
        let running = self.running_jobs.read().await.len();

        info!(
            "Scheduler Health - Queued: H:{} M:{} L:{}, Running: {}, Completed: {}, Failed: {}, Avg Time: {:.2}ms",
            high_queue,
            medium_queue,
            low_queue,
            running,
            stats.completed_jobs,
            stats.failed_jobs,
            stats.average_processing_time_ms
        );
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> QueueStats {
        let mut stats = self.stats.read().await.clone();
        stats.pending_jobs = self.high_priority_queue.lock().await.len()
            + self.medium_priority_queue.lock().await.len()
            + self.low_priority_queue.lock().await.len();
        stats.running_jobs = self.running_jobs.read().await.len();
        stats.last_updated = Some(Utc::now());
        stats
    }

    /// Load pending jobs from Redis
    async fn load_pending_jobs(&self) -> Result<(), AppError> {
        info!("Loading pending jobs from Redis");
        
        // Implementation depends on your Redis schema
        // This is a placeholder
        
        Ok(())
    }

    /// Save job state to Redis
    async fn save_job_state(&self, job: &ScanJob) -> Result<(), AppError> {
        let key = format!("job:{}", job.id);
        let value = serde_json::to_string(job)
            .map_err(|e| AppError::SerializationError(e.to_string()))?;
        
        self.redis.set(&key, &value, Some(Duration::from_secs(86400))).await?;
        Ok(())
    }

    /// Delete job state from Redis
    async fn delete_job_state(&self, job_id: Uuid) -> Result<(), AppError> {
        let key = format!("job:{}", job_id);
        self.redis.delete(&key).await?;
        Ok(())
    }

    /// Update statistics
    async fn update_stats(&self) {
        let mut stats = self.stats.write().await;
        
        // Calculate throughput (jobs per minute)
        if let Some(last_updated) = stats.last_updated {
            let elapsed = Utc::now().signed_duration_since(last_updated);
            if elapsed.num_seconds() > 0 {
                let jobs_processed = stats.completed_jobs + stats.failed_jobs;
                stats.queue_throughput_per_minute = 
                    (jobs_processed as f64 / elapsed.num_seconds() as f64) * 60.0;
            }
        }
        
        stats.last_updated = Some(Utc::now());
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<(), AppError> {
        info!("Initiating scheduler shutdown");
        *self.shutdown.lock().await = true;

        // Wait for running jobs to complete (with timeout)
        let mut attempts = 0;
        while self.running_jobs.read().await.len() > 0 && attempts < 60 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            attempts += 1;
        }

        info!("Scheduler shutdown complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_priority_scheduling() {
        // Test priority-based job selection
    }

    #[tokio::test]
    async fn test_fair_share() {
        // Test fair share scheduling
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        // Test job timeout detection
    }
}