/// Sandbox module for isolated dynamic analysis
///
/// This module provides secure containerized execution environments for analyzing
/// potentially malicious files. It includes:
/// - Container management (Docker-based isolation)
/// - Real-time monitoring of system activities
/// - Report generation for behavioral analysis

pub mod container;
pub mod monitor;
pub mod report_generator;

pub use container::Container;
pub use monitor::Monitor;
pub use report_generator::ReportGenerator;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Represents the execution environment for a sandbox instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxEnvironment {
    pub sandbox_id: String,
    pub analysis_id: Uuid,
    pub operating_system: OsType,
    pub base_image: String,
    pub created_at: DateTime<Utc>,
    pub status: SandboxStatus,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OsType {
    Linux,
    Windows,
    MacOS,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SandboxStatus {
    Creating,
    Ready,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub disk_read_mb: u64,
    pub disk_write_mb: u64,
    pub network_sent_kb: u64,
    pub network_received_kb: u64,
}

/// Configuration for creating a sandbox instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub timeout_seconds: u64,
    pub max_memory_mb: u64,
    pub max_cpu_percent: u8,
    pub network_enabled: bool,
    pub dns_servers: Vec<String>,
    pub environment_vars: HashMap<String, String>,
    pub mounted_volumes: Vec<VolumeMount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 300,
            max_memory_mb: 1024,
            max_cpu_percent: 50,
            network_enabled: false,
            dns_servers: vec!["8.8.8.8".to_string()],
            environment_vars: HashMap::new(),
            mounted_volumes: Vec::new(),
        }
    }
}

/// Result of sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    pub sandbox_id: String,
    pub analysis_id: Uuid,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub timeout_occurred: bool,
    pub resource_usage: ResourceUsage,
    pub artifacts_collected: Vec<String>,
}
