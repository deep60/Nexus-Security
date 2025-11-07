/// Real-time monitoring of sandbox execution
///
/// This module monitors various system activities during sandbox execution including:
/// - File system operations
/// - Network connections and traffic
/// - Process creation and termination
/// - Registry modifications (Windows)
/// - System calls

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::analyzers::dynamic_analyzer::{
    DynamicBehavior, FileOperation, FileOperationType, MonitoringConfig, NetworkOperation,
    ProcessOperation, ProcessOperationType, RegistryOperation, RegistryOperationType,
    Screenshot, SystemCall, ConnectionState, NetworkCapture,
};

/// Monitor handle for tracking active monitoring sessions
pub struct MonitorHandle {
    pub monitor_id: Uuid,
    pub sandbox_id: String,
    pub started_at: DateTime<Utc>,
    monitoring_task: Option<JoinHandle<Result<DynamicBehavior>>>,
}

/// Main monitoring engine
pub struct Monitor {
    config: MonitoringConfig,
    active_monitors: Arc<Mutex<HashMap<Uuid, MonitoringSession>>>,
}

#[derive(Debug, Clone)]
struct MonitoringSession {
    monitor_id: Uuid,
    sandbox_id: String,
    analysis_id: Uuid,
    started_at: DateTime<Utc>,
    behavior_data: Arc<Mutex<BehaviorCollector>>,
}

/// Collector for behavioral data during monitoring
#[derive(Debug, Clone, Default)]
struct BehaviorCollector {
    file_operations: Vec<FileOperation>,
    network_operations: Vec<NetworkOperation>,
    process_operations: Vec<ProcessOperation>,
    registry_operations: Vec<RegistryOperation>,
    system_calls: Vec<SystemCall>,
    screenshots: Vec<Screenshot>,
    network_capture: Option<NetworkCapture>,
}

impl Monitor {
    /// Create a new monitor instance
    pub fn new(config: &MonitoringConfig) -> Result<Self> {
        info!("Initializing sandbox monitor");

        Ok(Self {
            config: config.clone(),
            active_monitors: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Start monitoring a sandbox instance
    pub async fn start_monitoring(
        &self,
        sandbox_id: &str,
        analysis_id: &Uuid,
    ) -> Result<MonitorHandle> {
        let monitor_id = Uuid::new_v4();
        info!(
            "Starting monitoring session {} for sandbox {}",
            monitor_id, sandbox_id
        );

        let session = MonitoringSession {
            monitor_id,
            sandbox_id: sandbox_id.to_string(),
            analysis_id: *analysis_id,
            started_at: Utc::now(),
            behavior_data: Arc::new(Mutex::new(BehaviorCollector::default())),
        };

        // Store the session
        {
            let mut monitors = self.active_monitors.lock().await;
            monitors.insert(monitor_id, session.clone());
        }

        // Start monitoring tasks
        let monitoring_task = self.spawn_monitoring_tasks(session.clone()).await?;

        Ok(MonitorHandle {
            monitor_id,
            sandbox_id: sandbox_id.to_string(),
            started_at: Utc::now(),
            monitoring_task: Some(monitoring_task),
        })
    }

    /// Spawn all monitoring tasks
    async fn spawn_monitoring_tasks(
        &self,
        session: MonitoringSession,
    ) -> Result<JoinHandle<Result<DynamicBehavior>>> {
        let config = self.config.clone();
        let behavior_data = session.behavior_data.clone();
        let sandbox_id = session.sandbox_id.clone();

        let handle = tokio::spawn(async move {
            let mut tasks = Vec::new();

            // File system monitoring
            if config.monitor_file_system {
                let fs_task = Self::monitor_file_system(
                    sandbox_id.clone(),
                    behavior_data.clone(),
                );
                tasks.push(tokio::spawn(fs_task));
            }

            // Network monitoring
            if config.monitor_network {
                let net_task = Self::monitor_network(
                    sandbox_id.clone(),
                    behavior_data.clone(),
                    config.capture_pcap,
                );
                tasks.push(tokio::spawn(net_task));
            }

            // Process monitoring
            if config.monitor_processes {
                let proc_task = Self::monitor_processes(
                    sandbox_id.clone(),
                    behavior_data.clone(),
                );
                tasks.push(tokio::spawn(proc_task));
            }

            // Registry monitoring (Windows only)
            if config.monitor_registry {
                let reg_task = Self::monitor_registry(
                    sandbox_id.clone(),
                    behavior_data.clone(),
                );
                tasks.push(tokio::spawn(reg_task));
            }

            // Screenshot capture
            if config.capture_screenshots {
                let screenshot_task = Self::capture_screenshots(
                    sandbox_id.clone(),
                    behavior_data.clone(),
                );
                tasks.push(tokio::spawn(screenshot_task));
            }

            // Wait for all tasks to complete
            for task in tasks {
                if let Err(e) = task.await {
                    error!("Monitoring task failed: {}", e);
                }
            }

            // Collect final behavior data
            let collector = behavior_data.lock().await;
            Ok(DynamicBehavior {
                file_operations: collector.file_operations.clone(),
                network_operations: collector.network_operations.clone(),
                process_operations: collector.process_operations.clone(),
                registry_operations: collector.registry_operations.clone(),
                system_calls: collector.system_calls.clone(),
                screenshots: collector.screenshots.clone(),
                network_capture: collector.network_capture.clone(),
            })
        });

        Ok(handle)
    }

    /// Monitor file system operations
    async fn monitor_file_system(
        sandbox_id: String,
        behavior_data: Arc<Mutex<BehaviorCollector>>,
    ) -> Result<()> {
        debug!("Starting file system monitoring for {}", sandbox_id);

        // Use inotify-like monitoring in the container
        let command = r#"
            strace -e trace=open,openat,creat,unlink,unlinkat,rename,renameat,write,read -f -p 1 2>&1 | \
            while IFS= read -r line; do
                echo "$line"
            done
        "#;

        let output = Command::new("docker")
            .args(["exec", &sandbox_id, "bash", "-c", command])
            .output()
            .await;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Self::parse_file_operations(&stdout, &behavior_data).await;
            }
            Err(e) => {
                warn!("File system monitoring failed: {}", e);
            }
        }

        Ok(())
    }

    /// Parse file operations from strace output
    async fn parse_file_operations(
        strace_output: &str,
        behavior_data: &Arc<Mutex<BehaviorCollector>>,
    ) {
        let mut collector = behavior_data.lock().await;

        for line in strace_output.lines() {
            if line.contains("open") || line.contains("openat") {
                // Parse file open operations
                if let Some(path) = Self::extract_file_path(line) {
                    collector.file_operations.push(FileOperation {
                        operation_type: FileOperationType::Read,
                        source_path: PathBuf::from(path),
                        target_path: None,
                        timestamp: Utc::now(),
                        success: !line.contains("ENOENT") && !line.contains("EACCES"),
                    });
                }
            } else if line.contains("creat") {
                if let Some(path) = Self::extract_file_path(line) {
                    collector.file_operations.push(FileOperation {
                        operation_type: FileOperationType::Create,
                        source_path: PathBuf::from(path),
                        target_path: None,
                        timestamp: Utc::now(),
                        success: true,
                    });
                }
            } else if line.contains("unlink") {
                if let Some(path) = Self::extract_file_path(line) {
                    collector.file_operations.push(FileOperation {
                        operation_type: FileOperationType::Delete,
                        source_path: PathBuf::from(path),
                        target_path: None,
                        timestamp: Utc::now(),
                        success: true,
                    });
                }
            } else if line.contains("rename") {
                // Parse rename operations (source and target)
                if let Some((src, dst)) = Self::extract_rename_paths(line) {
                    collector.file_operations.push(FileOperation {
                        operation_type: FileOperationType::Move,
                        source_path: PathBuf::from(src),
                        target_path: Some(PathBuf::from(dst)),
                        timestamp: Utc::now(),
                        success: true,
                    });
                }
            }
        }
    }

    /// Monitor network operations
    async fn monitor_network(
        sandbox_id: String,
        behavior_data: Arc<Mutex<BehaviorCollector>>,
        capture_pcap: bool,
    ) -> Result<()> {
        debug!("Starting network monitoring for {}", sandbox_id);

        // Monitor network connections using netstat
        let netstat_output = Command::new("docker")
            .args([
                "exec",
                &sandbox_id,
                "netstat",
                "-tunaep",
            ])
            .output()
            .await;

        if let Ok(output) = netstat_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Self::parse_network_connections(&stdout, &behavior_data).await;
        }

        // Capture network traffic if enabled
        if capture_pcap {
            Self::capture_network_traffic(&sandbox_id, &behavior_data).await?;
        }

        Ok(())
    }

    /// Parse network connections from netstat output
    async fn parse_network_connections(
        netstat_output: &str,
        behavior_data: &Arc<Mutex<BehaviorCollector>>,
    ) {
        let mut collector = behavior_data.lock().await;

        for line in netstat_output.lines().skip(2) {
            // Skip header lines
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 6 {
                continue;
            }

            let protocol = parts[0].to_string();
            let local_addr = parts[3];
            let foreign_addr = parts[4];
            let state = parts[5];

            // Parse addresses
            if let Some((src_ip, src_port)) = Self::parse_address(local_addr) {
                if let Some((dst_ip, dst_port)) = Self::parse_address(foreign_addr) {
                    let connection_state = match state {
                        "ESTABLISHED" => ConnectionState::Established,
                        "SYN_SENT" => ConnectionState::Connecting,
                        "CLOSE_WAIT" | "TIME_WAIT" | "CLOSED" => ConnectionState::Closed,
                        _ => ConnectionState::Failed,
                    };

                    collector.network_operations.push(NetworkOperation {
                        protocol,
                        source_ip: src_ip.to_string(),
                        source_port: src_port,
                        destination_ip: dst_ip.to_string(),
                        destination_port: dst_port,
                        bytes_sent: 0,
                        bytes_received: 0,
                        timestamp: Utc::now(),
                        connection_state,
                    });
                }
            }
        }
    }

    /// Capture network traffic using tcpdump
    async fn capture_network_traffic(
        sandbox_id: &str,
        behavior_data: &Arc<Mutex<BehaviorCollector>>,
    ) -> Result<()> {
        debug!("Capturing network traffic for {}", sandbox_id);

        // Run tcpdump for a limited time
        let output = Command::new("docker")
            .args([
                "exec",
                sandbox_id,
                "timeout",
                "30",
                "tcpdump",
                "-i",
                "any",
                "-w",
                "/tmp/capture.pcap",
            ])
            .output()
            .await;

        if let Ok(_) = output {
            // Copy pcap file from container
            let pcap_output = Command::new("docker")
                .args(["cp", &format!("{}:/tmp/capture.pcap", sandbox_id), "/tmp/"])
                .output()
                .await;

            if let Ok(_) = pcap_output {
                if let Ok(pcap_data) = tokio::fs::read("/tmp/capture.pcap").await {
                    let mut collector = behavior_data.lock().await;
                    collector.network_capture = Some(NetworkCapture {
                        pcap_data,
                        start_time: Utc::now(),
                        end_time: Utc::now(),
                        packet_count: 0,
                    });
                }
            }
        }

        Ok(())
    }

    /// Monitor process operations
    async fn monitor_processes(
        sandbox_id: String,
        behavior_data: Arc<Mutex<BehaviorCollector>>,
    ) -> Result<()> {
        debug!("Starting process monitoring for {}", sandbox_id);

        // Monitor processes using ps
        let output = Command::new("docker")
            .args(["exec", &sandbox_id, "ps", "aux"])
            .output()
            .await;

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            Self::parse_process_list(&stdout, &behavior_data).await;
        }

        Ok(())
    }

    /// Parse process list from ps output
    async fn parse_process_list(
        ps_output: &str,
        behavior_data: &Arc<Mutex<BehaviorCollector>>,
    ) {
        let mut collector = behavior_data.lock().await;

        for line in ps_output.lines().skip(1) {
            // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 11 {
                continue;
            }

            let pid: u32 = parts[1].parse().unwrap_or(0);
            let command = parts[10..].join(" ");

            // Detect suspicious process creation
            if command.contains("bash") || command.contains("sh") || command.contains("python") {
                collector.process_operations.push(ProcessOperation {
                    operation_type: ProcessOperationType::Create,
                    process_name: parts[10].to_string(),
                    process_id: pid,
                    parent_process_id: None,
                    command_line: command,
                    timestamp: Utc::now(),
                });
            }
        }
    }

    /// Monitor registry operations (Windows-specific, simulated for Linux)
    async fn monitor_registry(
        sandbox_id: String,
        behavior_data: Arc<Mutex<BehaviorCollector>>,
    ) -> Result<()> {
        debug!("Starting registry monitoring for {}", sandbox_id);

        // For Wine environments, monitor Wine registry
        let command = "find ~/.wine -name '*.reg' -mmin -5 2>/dev/null";

        let output = Command::new("docker")
            .args(["exec", &sandbox_id, "bash", "-c", command])
            .output()
            .await;

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);

            let mut collector = behavior_data.lock().await;
            for line in stdout.lines() {
                if !line.is_empty() {
                    collector.registry_operations.push(RegistryOperation {
                        operation_type: RegistryOperationType::SetValue,
                        key_path: line.to_string(),
                        value_name: None,
                        value_data: None,
                        timestamp: Utc::now(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Capture screenshots during execution
    async fn capture_screenshots(
        sandbox_id: String,
        behavior_data: Arc<Mutex<BehaviorCollector>>,
    ) -> Result<()> {
        debug!("Starting screenshot capture for {}", sandbox_id);

        // Capture screenshots every 5 seconds
        for i in 0..6 {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            // Use xwd or similar to capture screenshots (if X server is available)
            let screenshot_path = format!("/tmp/screenshot_{}.png", i);
            let command = format!("scrot {}", screenshot_path);

            let output = Command::new("docker")
                .args(["exec", &sandbox_id, "bash", "-c", &command])
                .output()
                .await;

            if let Ok(_) = output {
                // Copy screenshot from container
                let copy_cmd = format!("{}:{}", sandbox_id, screenshot_path);
                if let Ok(_) = Command::new("docker")
                    .args(["cp", &copy_cmd, "/tmp/"])
                    .output()
                    .await
                {
                    if let Ok(image_data) = tokio::fs::read(&screenshot_path).await {
                        let mut collector = behavior_data.lock().await;
                        collector.screenshots.push(Screenshot {
                            timestamp: Utc::now(),
                            image_data,
                            image_format: "png".to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Stop monitoring and collect results
    pub async fn stop_monitoring_and_collect(
        &self,
        mut handle: MonitorHandle,
    ) -> Result<DynamicBehavior> {
        info!("Stopping monitoring session {}", handle.monitor_id);

        // Wait for monitoring task to complete
        let behavior = if let Some(task) = handle.monitoring_task.take() {
            match tokio::time::timeout(tokio::time::Duration::from_secs(10), task).await {
                Ok(Ok(Ok(behavior))) => behavior,
                Ok(Ok(Err(e))) => {
                    error!("Monitoring task failed: {}", e);
                    DynamicBehavior {
                        file_operations: vec![],
                        network_operations: vec![],
                        process_operations: vec![],
                        registry_operations: vec![],
                        system_calls: vec![],
                        screenshots: vec![],
                        network_capture: None,
                    }
                }
                Ok(Err(e)) => {
                    error!("Monitoring task panicked: {}", e);
                    return Err(anyhow!("Monitoring task panicked"));
                }
                Err(_) => {
                    warn!("Monitoring task timed out");
                    return Err(anyhow!("Monitoring timeout"));
                }
            }
        } else {
            return Err(anyhow!("No monitoring task"));
        };

        // Remove from active monitors
        {
            let mut monitors = self.active_monitors.lock().await;
            monitors.remove(&handle.monitor_id);
        }

        info!(
            "Monitoring completed: {} file ops, {} network ops, {} process ops",
            behavior.file_operations.len(),
            behavior.network_operations.len(),
            behavior.process_operations.len()
        );

        Ok(behavior)
    }

    /// Helper: Extract file path from strace line
    fn extract_file_path(line: &str) -> Option<String> {
        // Look for quoted strings in strace output
        let start = line.find('"')?;
        let end = line[start + 1..].find('"')? + start + 1;
        Some(line[start + 1..end].to_string())
    }

    /// Helper: Extract source and destination paths from rename operation
    fn extract_rename_paths(line: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = line.split('"').collect();
        if parts.len() >= 4 {
            Some((parts[1].to_string(), parts[3].to_string()))
        } else {
            None
        }
    }

    /// Helper: Parse IP:port address
    fn parse_address(addr: &str) -> Option<(&str, u16)> {
        let parts: Vec<&str> = addr.rsplitn(2, ':').collect();
        if parts.len() == 2 {
            let port = parts[0].parse().ok()?;
            Some((parts[1], port))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_path() {
        let line = r#"openat(AT_FDCWD, "/tmp/test.txt", O_RDONLY) = 3"#;
        let path = Monitor::extract_file_path(line);
        assert_eq!(path, Some("/tmp/test.txt".to_string()));
    }

    #[test]
    fn test_parse_address() {
        let addr = "192.168.1.100:8080";
        let (ip, port) = Monitor::parse_address(addr).unwrap();
        assert_eq!(ip, "192.168.1.100");
        assert_eq!(port, 8080);
    }

    #[tokio::test]
    async fn test_monitor_creation() {
        let config = MonitoringConfig {
            monitor_file_system: true,
            monitor_network: true,
            monitor_registry: false,
            monitor_processes: true,
            capture_screenshots: false,
            capture_pcap: false,
        };

        let monitor = Monitor::new(&config);
        assert!(monitor.is_ok());
    }
}
