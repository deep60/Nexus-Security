/// Container management for sandbox isolation
///
/// This module handles Docker container lifecycle management for secure execution
/// of potentially malicious samples in isolated environments.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::analyzers::dynamic_analyzer::{
    DynamicAnalyzerConfig, NetworkConfig, ResourceLimits,
};

use super::{SandboxConfig, SandboxEnvironment, SandboxStatus, OsType, ResourceUsage};

/// Manages Docker containers for sandbox execution
pub struct Container {
    docker_available: bool,
    base_image: String,
    work_dir: PathBuf,
    active_containers: HashMap<String, ContainerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContainerInfo {
    container_id: String,
    analysis_id: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    image: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SandboxContainerConfig {
    pub image: String,
    pub memory_limit: String,
    pub cpu_quota: String,
    pub network_mode: String,
    pub dns_servers: Vec<String>,
    pub environment: HashMap<String, String>,
    pub volumes: Vec<String>,
    pub working_dir: String,
    pub user: String,
    pub security_opts: Vec<String>,
    pub cap_drop: Vec<String>,
    pub read_only_rootfs: bool,
}

impl Container {
    /// Create a new container manager
    pub fn new(config: &DynamicAnalyzerConfig) -> Result<Self> {
        let work_dir = PathBuf::from("/tmp/nexus-sandbox");
        std::fs::create_dir_all(&work_dir)?;

        let docker_available = Self::check_docker_available();
        if !docker_available {
            warn!("Docker is not available. Sandbox functionality will be limited.");
        }

        Ok(Self {
            docker_available,
            base_image: "nexus-security/sandbox:latest".to_string(),
            work_dir,
            active_containers: HashMap::new(),
        })
    }

    /// Check if Docker is available on the system
    fn check_docker_available() -> bool {
        std::process::Command::new("docker")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Create sandbox configuration
    pub fn create_sandbox_config(
        &self,
        config: &DynamicAnalyzerConfig,
    ) -> Result<SandboxContainerConfig> {
        let memory_limit = format!("{}m", config.resource_limits.max_memory_mb);
        let cpu_quota = format!("{}", config.resource_limits.max_cpu_percent * 1000);

        let network_mode = if config.network_config.internet_access {
            "bridge".to_string()
        } else {
            "none".to_string()
        };

        let mut environment = HashMap::new();
        environment.insert("LANG".to_string(), "en_US.UTF-8".to_string());
        environment.insert("TZ".to_string(), "UTC".to_string());

        Ok(SandboxContainerConfig {
            image: self.base_image.clone(),
            memory_limit,
            cpu_quota,
            network_mode,
            dns_servers: config.network_config.dns_servers.clone(),
            environment,
            volumes: vec![],
            working_dir: "/workspace".to_string(),
            user: "sandbox".to_string(),
            security_opts: vec![
                "no-new-privileges".to_string(),
                "seccomp=unconfined".to_string(),
            ],
            cap_drop: vec!["ALL".to_string()],
            read_only_rootfs: false,
        })
    }

    /// Create a new container instance
    pub async fn create_container(&mut self, config: SandboxContainerConfig) -> Result<String> {
        if !self.docker_available {
            return Err(anyhow!("Docker is not available"));
        }

        info!("Creating sandbox container with image: {}", config.image);

        // Pull image if not exists
        self.pull_image_if_needed(&config.image).await?;

        // Generate unique container ID
        let container_id = format!("nexus-sandbox-{}", Uuid::new_v4());

        // Build docker run command
        let mut cmd = Command::new("docker");
        cmd.arg("create")
            .arg("--name")
            .arg(&container_id)
            .arg("--memory")
            .arg(&config.memory_limit)
            .arg("--cpus")
            .arg(config.cpu_quota)
            .arg("--network")
            .arg(&config.network_mode)
            .arg("--workdir")
            .arg(&config.working_dir);

        // Add DNS servers
        for dns in &config.dns_servers {
            cmd.arg("--dns").arg(dns);
        }

        // Add environment variables
        for (key, value) in &config.environment {
            cmd.arg("--env").arg(format!("{}={}", key, value));
        }

        // Add security options
        for opt in &config.security_opts {
            cmd.arg("--security-opt").arg(opt);
        }

        // Drop capabilities
        for cap in &config.cap_drop {
            cmd.arg("--cap-drop").arg(cap);
        }

        // Add volumes
        for volume in &config.volumes {
            cmd.arg("--volume").arg(volume);
        }

        // Set user
        cmd.arg("--user").arg(&config.user);

        // Read-only root filesystem
        if config.read_only_rootfs {
            cmd.arg("--read-only");
        }

        // Add the image and command
        cmd.arg(&config.image)
            .arg("sleep")
            .arg("infinity"); // Keep container running

        debug!("Docker create command: {:?}", cmd);

        let output = cmd.output().await.context("Failed to create container")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("Failed to create container: {}", error);
            return Err(anyhow!("Docker create failed: {}", error));
        }

        let actual_container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Start the container
        self.start_container(&actual_container_id).await?;

        // Store container info
        let info = ContainerInfo {
            container_id: actual_container_id.clone(),
            analysis_id: Uuid::new_v4(),
            created_at: chrono::Utc::now(),
            image: config.image.clone(),
            status: "running".to_string(),
        };

        self.active_containers.insert(actual_container_id.clone(), info);

        info!("Container created and started: {}", actual_container_id);

        Ok(actual_container_id)
    }

    /// Pull Docker image if it doesn't exist locally
    async fn pull_image_if_needed(&self, image: &str) -> Result<()> {
        debug!("Checking if image exists: {}", image);

        // Check if image exists
        let check = Command::new("docker")
            .args(["image", "inspect", image])
            .output()
            .await?;

        if check.status.success() {
            debug!("Image already exists: {}", image);
            return Ok(());
        }

        // Try to build the image if it's our custom image
        if image.starts_with("nexus-security/") {
            info!("Building custom sandbox image: {}", image);
            return self.build_sandbox_image().await;
        }

        // Pull the image
        info!("Pulling Docker image: {}", image);
        let output = Command::new("docker")
            .args(["pull", image])
            .output()
            .await
            .context("Failed to pull Docker image")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to pull image: {}", error));
        }

        Ok(())
    }

    /// Build the custom sandbox image
    async fn build_sandbox_image(&self) -> Result<()> {
        // Create a temporary Dockerfile
        let dockerfile_content = r#"
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    wine \
    wine64 \
    nodejs \
    npm \
    default-jre \
    strace \
    ltrace \
    tcpdump \
    net-tools \
    procps \
    && rm -rf /var/lib/apt/lists/*

# Create sandbox user
RUN useradd -m -u 1000 -s /bin/bash sandbox

# Create workspace directory
RUN mkdir -p /workspace && chown sandbox:sandbox /workspace

WORKDIR /workspace
USER sandbox

CMD ["/bin/bash"]
"#;

        let dockerfile_path = self.work_dir.join("Dockerfile");
        tokio::fs::write(&dockerfile_path, dockerfile_content).await?;

        info!("Building sandbox Docker image...");

        let output = Command::new("docker")
            .args([
                "build",
                "-t",
                &self.base_image,
                "-f",
                dockerfile_path.to_str().unwrap(),
                self.work_dir.to_str().unwrap(),
            ])
            .output()
            .await
            .context("Failed to build sandbox image")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to build sandbox image: {}", error));
        }

        info!("Sandbox image built successfully");
        Ok(())
    }

    /// Start a stopped container
    async fn start_container(&self, container_id: &str) -> Result<()> {
        debug!("Starting container: {}", container_id);

        let output = Command::new("docker")
            .args(["start", container_id])
            .output()
            .await
            .context("Failed to start container")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to start container: {}", error));
        }

        Ok(())
    }

    /// Apply resource limits to a running container
    pub async fn apply_resource_limits(
        &self,
        container_id: &str,
        limits: &ResourceLimits,
    ) -> Result<()> {
        debug!("Applying resource limits to container: {}", container_id);

        // Update memory limit
        let output = Command::new("docker")
            .args([
                "update",
                "--memory",
                &format!("{}m", limits.max_memory_mb),
                "--cpus",
                &format!("{}", limits.max_cpu_percent as f32 / 100.0),
                container_id,
            ])
            .output()
            .await
            .context("Failed to update container limits")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to apply resource limits: {}", error);
        }

        Ok(())
    }

    /// Configure network settings for the container
    pub async fn configure_network(
        &self,
        container_id: &str,
        network_config: &NetworkConfig,
    ) -> Result<()> {
        debug!("Configuring network for container: {}", container_id);

        // Network configuration is applied during container creation
        // This method can be used for additional runtime network configuration

        if !network_config.internet_access {
            debug!("Container has no internet access");
        }

        // TODO: Implement iptables rules for network filtering if needed

        Ok(())
    }

    /// Copy a file into the sandbox container
    pub async fn copy_file_to_sandbox(
        &self,
        container_id: &str,
        file_path: &Path,
    ) -> Result<PathBuf> {
        let filename = file_path
            .file_name()
            .ok_or_else(|| anyhow!("Invalid file path"))?
            .to_string_lossy();

        let container_path = format!("/workspace/{}", filename);

        info!(
            "Copying file to container: {} -> {}",
            file_path.display(),
            container_path
        );

        let output = Command::new("docker")
            .args([
                "cp",
                file_path.to_str().unwrap(),
                &format!("{}:{}", container_id, container_path),
            ])
            .output()
            .await
            .context("Failed to copy file to container")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to copy file: {}", error));
        }

        Ok(PathBuf::from(container_path))
    }

    /// Execute a command in the sandbox container
    pub async fn execute_command(&self, container_id: &str, command: &str) -> Result<String> {
        debug!("Executing command in container {}: {}", container_id, command);

        let output = Command::new("docker")
            .args(["exec", container_id, "bash", "-c", command])
            .output()
            .await
            .context("Failed to execute command in container")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            warn!("Command execution had non-zero exit: {}", stderr);
        }

        Ok(format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr))
    }

    /// Get resource usage statistics for a container
    pub async fn get_resource_usage(&self, container_id: &str) -> Result<ResourceUsage> {
        let output = Command::new("docker")
            .args(["stats", "--no-stream", "--format", "{{json .}}", container_id])
            .output()
            .await
            .context("Failed to get container stats")?;

        if !output.status.success() {
            return Ok(ResourceUsage::default());
        }

        let stats_json = String::from_utf8_lossy(&output.stdout);
        let stats: serde_json::Value = serde_json::from_str(&stats_json)
            .unwrap_or_else(|_| serde_json::json!({}));

        Ok(ResourceUsage {
            cpu_percent: stats["CPUPerc"]
                .as_str()
                .and_then(|s| s.trim_end_matches('%').parse().ok())
                .unwrap_or(0.0),
            memory_mb: stats["MemUsage"]
                .as_str()
                .and_then(|s| {
                    s.split('/')
                        .next()
                        .and_then(|mem| mem.trim().trim_end_matches("MiB").parse().ok())
                })
                .unwrap_or(0),
            disk_read_mb: 0,
            disk_write_mb: 0,
            network_sent_kb: 0,
            network_received_kb: 0,
        })
    }

    /// Stop a running container
    pub async fn stop_container(&self, container_id: &str) -> Result<()> {
        info!("Stopping container: {}", container_id);

        let output = Command::new("docker")
            .args(["stop", "-t", "5", container_id])
            .output()
            .await
            .context("Failed to stop container")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to stop container gracefully: {}", error);
        }

        Ok(())
    }

    /// Remove a container
    pub async fn remove_container(&mut self, container_id: &str) -> Result<()> {
        info!("Removing container: {}", container_id);

        // Stop first if running
        let _ = self.stop_container(container_id).await;

        let output = Command::new("docker")
            .args(["rm", "-f", container_id])
            .output()
            .await
            .context("Failed to remove container")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to remove container: {}", error);
        }

        // Remove from active containers
        self.active_containers.remove(container_id);

        Ok(())
    }

    /// Cleanup all active containers
    pub async fn cleanup_all(&mut self) -> Result<()> {
        info!("Cleaning up all active containers");

        let container_ids: Vec<String> = self.active_containers.keys().cloned().collect();

        for container_id in container_ids {
            if let Err(e) = self.remove_container(&container_id).await {
                error!("Failed to remove container {}: {}", container_id, e);
            }
        }

        Ok(())
    }

    /// Check if Docker is available
    pub fn is_docker_available(&self) -> bool {
        self.docker_available
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        // Attempt to cleanup containers on drop
        // This is best-effort and may not complete
        if !self.active_containers.is_empty() {
            warn!("Container manager dropped with {} active containers", self.active_containers.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_docker_available() {
        let available = Container::check_docker_available();
        println!("Docker available: {}", available);
    }

    #[tokio::test]
    async fn test_sandbox_config_creation() {
        let config = DynamicAnalyzerConfig::default();
        let container = Container::new(&config).unwrap();
        let sandbox_config = container.create_sandbox_config(&config).unwrap();

        assert_eq!(sandbox_config.memory_limit, "1024m");
        assert_eq!(sandbox_config.network_mode, "none");
    }
}
