/// Report generation for sandbox analysis results
///
/// This module generates comprehensive analysis reports from behavioral data
/// collected during sandbox execution, including threat assessments and IOCs.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

use crate::analyzers::dynamic_analyzer::{
    DynamicBehavior, DynamicThreatIndicators, FileOperation, FileOperationType,
    NetworkOperation, ProcessOperation, RegistryOperation,
};

/// Comprehensive dynamic analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicAnalysisReport {
    pub report_id: String,
    pub generated_at: DateTime<Utc>,
    pub executive_summary: ExecutiveSummary,
    pub behavioral_analysis: BehavioralAnalysisSection,
    pub threat_assessment: ThreatAssessment,
    pub indicators_of_compromise: Vec<IndicatorOfCompromise>,
    pub network_analysis: NetworkAnalysisSection,
    pub file_activity: FileActivitySection,
    pub process_activity: ProcessActivitySection,
    pub recommendations: Vec<String>,
    pub metadata: ReportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub threat_level: ThreatLevel,
    pub risk_score: f32,
    pub malware_family: Option<String>,
    pub key_findings: Vec<String>,
    pub affected_systems: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreatLevel {
    Critical,
    High,
    Medium,
    Low,
    Clean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralAnalysisSection {
    pub total_operations: u32,
    pub suspicious_behaviors: Vec<SuspiciousBehavior>,
    pub evasion_techniques: Vec<String>,
    pub persistence_mechanisms: Vec<String>,
    pub data_theft_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousBehavior {
    pub behavior_type: String,
    pub description: String,
    pub severity: String,
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAssessment {
    pub is_malicious: bool,
    pub confidence: f32,
    pub threat_categories: Vec<String>,
    pub attack_techniques: Vec<AttackTechnique>,
    pub capability_assessment: CapabilityAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackTechnique {
    pub mitre_id: Option<String>,
    pub technique_name: String,
    pub description: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAssessment {
    pub can_persist: bool,
    pub can_exfiltrate: bool,
    pub can_propagate: bool,
    pub can_evade_detection: bool,
    pub can_modify_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorOfCompromise {
    pub ioc_type: IocType,
    pub value: String,
    pub confidence: f32,
    pub context: String,
    pub first_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IocType {
    IpAddress,
    Domain,
    Url,
    FileHash,
    FilePath,
    RegistryKey,
    Mutex,
    Process,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAnalysisSection {
    pub total_connections: u32,
    pub unique_destinations: u32,
    pub protocols_used: Vec<String>,
    pub suspicious_connections: Vec<SuspiciousConnection>,
    pub dns_queries: Vec<DnsQuery>,
    pub data_transfer_summary: DataTransferSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousConnection {
    pub destination: String,
    pub port: u16,
    pub protocol: String,
    pub reason: String,
    pub bytes_transferred: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsQuery {
    pub query: String,
    pub query_type: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTransferSummary {
    pub total_sent_bytes: u64,
    pub total_received_bytes: u64,
    pub outbound_connections: u32,
    pub inbound_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileActivitySection {
    pub files_created: Vec<FileActivityDetail>,
    pub files_modified: Vec<FileActivityDetail>,
    pub files_deleted: Vec<FileActivityDetail>,
    pub files_accessed: Vec<FileActivityDetail>,
    pub suspicious_file_operations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileActivityDetail {
    pub path: String,
    pub operation: String,
    pub timestamp: DateTime<Utc>,
    pub suspicious: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessActivitySection {
    pub processes_created: Vec<ProcessDetail>,
    pub process_injections: Vec<String>,
    pub process_tree: Vec<ProcessTreeNode>,
    pub suspicious_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDetail {
    pub name: String,
    pub pid: u32,
    pub command_line: String,
    pub timestamp: DateTime<Utc>,
    pub parent_pid: Option<u32>,
    pub suspicious: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTreeNode {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub children: Vec<ProcessTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub analysis_duration_ms: u64,
    pub sandbox_os: String,
    pub analysis_tools: Vec<String>,
    pub data_sources: Vec<String>,
}

/// Report generator
pub struct ReportGenerator {
    threat_intel_db: ThreatIntelligence,
}

struct ThreatIntelligence {
    known_malicious_ips: HashSet<String>,
    known_malicious_domains: HashSet<String>,
    suspicious_file_paths: HashSet<String>,
}

impl ReportGenerator {
    /// Create a new report generator
    pub fn new() -> Result<Self> {
        info!("Initializing report generator");

        // Initialize threat intelligence database
        let threat_intel_db = ThreatIntelligence {
            known_malicious_ips: Self::load_malicious_ips(),
            known_malicious_domains: Self::load_malicious_domains(),
            suspicious_file_paths: Self::load_suspicious_paths(),
        };

        Ok(Self { threat_intel_db })
    }

    /// Generate comprehensive dynamic analysis report
    pub async fn generate_dynamic_report(
        &self,
        behavior: &DynamicBehavior,
        threat_indicators: &DynamicThreatIndicators,
    ) -> Result<DynamicAnalysisReport> {
        info!("Generating dynamic analysis report");

        let report_id = uuid::Uuid::new_v4().to_string();
        let generated_at = Utc::now();

        // Generate executive summary
        let executive_summary = self.generate_executive_summary(behavior, threat_indicators);

        // Analyze behaviors
        let behavioral_analysis = self.analyze_behaviors(behavior, threat_indicators);

        // Assess threats
        let threat_assessment = self.assess_threats(behavior, threat_indicators);

        // Extract IOCs
        let indicators_of_compromise = self.extract_iocs(behavior, threat_indicators);

        // Analyze network activity
        let network_analysis = self.analyze_network_activity(&behavior.network_operations);

        // Analyze file activity
        let file_activity = self.analyze_file_activity(&behavior.file_operations);

        // Analyze process activity
        let process_activity = self.analyze_process_activity(&behavior.process_operations);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&threat_assessment);

        // Create metadata
        let metadata = ReportMetadata {
            analysis_duration_ms: 0, // This should be passed in
            sandbox_os: "Linux".to_string(),
            analysis_tools: vec![
                "strace".to_string(),
                "netstat".to_string(),
                "tcpdump".to_string(),
            ],
            data_sources: vec![
                "system_calls".to_string(),
                "network_traffic".to_string(),
                "file_operations".to_string(),
            ],
        };

        Ok(DynamicAnalysisReport {
            report_id,
            generated_at,
            executive_summary,
            behavioral_analysis,
            threat_assessment,
            indicators_of_compromise,
            network_analysis,
            file_activity,
            process_activity,
            recommendations,
            metadata,
        })
    }

    /// Generate executive summary
    fn generate_executive_summary(
        &self,
        behavior: &DynamicBehavior,
        threat_indicators: &DynamicThreatIndicators,
    ) -> ExecutiveSummary {
        let total_indicators = threat_indicators.malicious_network_connections.len()
            + threat_indicators.suspicious_file_operations.len()
            + threat_indicators.malicious_processes.len()
            + threat_indicators.persistence_mechanisms.len();

        let threat_level = match total_indicators {
            0 => ThreatLevel::Clean,
            1..=2 => ThreatLevel::Low,
            3..=5 => ThreatLevel::Medium,
            6..=10 => ThreatLevel::High,
            _ => ThreatLevel::Critical,
        };

        let risk_score = (total_indicators as f32 / 20.0).min(1.0) * 100.0;

        let mut key_findings = Vec::new();

        if !threat_indicators.malicious_network_connections.is_empty() {
            key_findings.push(format!(
                "{} suspicious network connections detected",
                threat_indicators.malicious_network_connections.len()
            ));
        }

        if !threat_indicators.persistence_mechanisms.is_empty() {
            key_findings.push("Persistence mechanisms detected".to_string());
        }

        if !threat_indicators.evasion_techniques.is_empty() {
            key_findings.push("Evasion techniques observed".to_string());
        }

        if !threat_indicators.data_exfiltration_attempts.is_empty() {
            key_findings.push("Potential data exfiltration detected".to_string());
        }

        ExecutiveSummary {
            threat_level,
            risk_score,
            malware_family: None,
            key_findings,
            affected_systems: vec!["Sandbox Environment".to_string()],
        }
    }

    /// Analyze behaviors for suspicious patterns
    fn analyze_behaviors(
        &self,
        behavior: &DynamicBehavior,
        threat_indicators: &DynamicThreatIndicators,
    ) -> BehavioralAnalysisSection {
        let total_operations = (behavior.file_operations.len()
            + behavior.network_operations.len()
            + behavior.process_operations.len()
            + behavior.registry_operations.len()) as u32;

        let mut suspicious_behaviors = Vec::new();

        // Analyze file operations for suspicious behavior
        for file_op in &behavior.file_operations {
            if self.is_sensitive_path(&file_op.source_path.to_string_lossy()) {
                suspicious_behaviors.push(SuspiciousBehavior {
                    behavior_type: "File Operation".to_string(),
                    description: format!(
                        "{:?} operation on sensitive path",
                        file_op.operation_type
                    ),
                    severity: "Medium".to_string(),
                    timestamp: file_op.timestamp,
                    details: {
                        let mut map = HashMap::new();
                        map.insert("path".to_string(), file_op.source_path.display().to_string());
                        map.insert("operation".to_string(), format!("{:?}", file_op.operation_type));
                        map
                    },
                });
            }
        }

        // Analyze network operations
        for net_op in &behavior.network_operations {
            if self.is_malicious_ip(&net_op.destination_ip) {
                suspicious_behaviors.push(SuspiciousBehavior {
                    behavior_type: "Network Connection".to_string(),
                    description: "Connection to known malicious IP".to_string(),
                    severity: "High".to_string(),
                    timestamp: net_op.timestamp,
                    details: {
                        let mut map = HashMap::new();
                        map.insert("destination".to_string(), net_op.destination_ip.clone());
                        map.insert("port".to_string(), net_op.destination_port.to_string());
                        map
                    },
                });
            }
        }

        BehavioralAnalysisSection {
            total_operations,
            suspicious_behaviors,
            evasion_techniques: threat_indicators.evasion_techniques.clone(),
            persistence_mechanisms: threat_indicators.persistence_mechanisms.clone(),
            data_theft_indicators: threat_indicators.data_exfiltration_attempts.clone(),
        }
    }

    /// Assess overall threat level and capabilities
    fn assess_threats(
        &self,
        behavior: &DynamicBehavior,
        threat_indicators: &DynamicThreatIndicators,
    ) -> ThreatAssessment {
        let total_indicators = threat_indicators.malicious_network_connections.len()
            + threat_indicators.suspicious_file_operations.len()
            + threat_indicators.malicious_processes.len();

        let is_malicious = total_indicators >= 3;
        let confidence = (total_indicators as f32 / 10.0).min(1.0);

        let mut threat_categories = Vec::new();
        if !threat_indicators.malicious_network_connections.is_empty() {
            threat_categories.push("Network Threat".to_string());
        }
        if !threat_indicators.persistence_mechanisms.is_empty() {
            threat_categories.push("Persistence".to_string());
        }
        if !threat_indicators.data_exfiltration_attempts.is_empty() {
            threat_categories.push("Data Exfiltration".to_string());
        }

        let attack_techniques = self.map_to_mitre_techniques(threat_indicators);

        let capability_assessment = CapabilityAssessment {
            can_persist: !threat_indicators.persistence_mechanisms.is_empty(),
            can_exfiltrate: !threat_indicators.data_exfiltration_attempts.is_empty(),
            can_propagate: self.detect_propagation_capability(behavior),
            can_evade_detection: !threat_indicators.evasion_techniques.is_empty(),
            can_modify_system: !threat_indicators.registry_modifications.is_empty(),
        };

        ThreatAssessment {
            is_malicious,
            confidence,
            threat_categories,
            attack_techniques,
            capability_assessment,
        }
    }

    /// Map threat indicators to MITRE ATT&CK techniques
    fn map_to_mitre_techniques(
        &self,
        threat_indicators: &DynamicThreatIndicators,
    ) -> Vec<AttackTechnique> {
        let mut techniques = Vec::new();

        if !threat_indicators.persistence_mechanisms.is_empty() {
            techniques.push(AttackTechnique {
                mitre_id: Some("T1547".to_string()),
                technique_name: "Boot or Logon Autostart Execution".to_string(),
                description: "Malware establishes persistence through autostart mechanisms".to_string(),
                evidence: threat_indicators.persistence_mechanisms.clone(),
            });
        }

        if !threat_indicators.data_exfiltration_attempts.is_empty() {
            techniques.push(AttackTechnique {
                mitre_id: Some("T1041".to_string()),
                technique_name: "Exfiltration Over C2 Channel".to_string(),
                description: "Data exfiltration over command and control channel".to_string(),
                evidence: threat_indicators.data_exfiltration_attempts.clone(),
            });
        }

        if !threat_indicators.evasion_techniques.is_empty() {
            techniques.push(AttackTechnique {
                mitre_id: Some("T1497".to_string()),
                technique_name: "Virtualization/Sandbox Evasion".to_string(),
                description: "Attempts to evade sandbox detection".to_string(),
                evidence: threat_indicators.evasion_techniques.clone(),
            });
        }

        techniques
    }

    /// Extract indicators of compromise
    fn extract_iocs(
        &self,
        behavior: &DynamicBehavior,
        threat_indicators: &DynamicThreatIndicators,
    ) -> Vec<IndicatorOfCompromise> {
        let mut iocs = Vec::new();

        // Extract network IOCs
        for conn in &threat_indicators.malicious_network_connections {
            if let Some((ip, _)) = conn.split_once(':') {
                iocs.push(IndicatorOfCompromise {
                    ioc_type: IocType::IpAddress,
                    value: ip.to_string(),
                    confidence: 0.8,
                    context: "Malicious network connection".to_string(),
                    first_seen: Utc::now(),
                });
            }
        }

        // Extract file IOCs
        for file_op in &behavior.file_operations {
            if matches!(file_op.operation_type, FileOperationType::Create) {
                iocs.push(IndicatorOfCompromise {
                    ioc_type: IocType::FilePath,
                    value: file_op.source_path.display().to_string(),
                    confidence: 0.6,
                    context: "File created during execution".to_string(),
                    first_seen: file_op.timestamp,
                });
            }
        }

        // Extract process IOCs
        for proc in &threat_indicators.malicious_processes {
            iocs.push(IndicatorOfCompromise {
                ioc_type: IocType::Process,
                value: proc.clone(),
                confidence: 0.7,
                context: "Suspicious process execution".to_string(),
                first_seen: Utc::now(),
            });
        }

        iocs
    }

    /// Analyze network activity
    fn analyze_network_activity(
        &self,
        network_operations: &[NetworkOperation],
    ) -> NetworkAnalysisSection {
        let total_connections = network_operations.len() as u32;

        let unique_destinations: HashSet<String> = network_operations
            .iter()
            .map(|op| op.destination_ip.clone())
            .collect();

        let protocols_used: HashSet<String> = network_operations
            .iter()
            .map(|op| op.protocol.clone())
            .collect();

        let mut suspicious_connections = Vec::new();
        for op in network_operations {
            if self.is_suspicious_port(op.destination_port) {
                suspicious_connections.push(SuspiciousConnection {
                    destination: op.destination_ip.clone(),
                    port: op.destination_port,
                    protocol: op.protocol.clone(),
                    reason: "Suspicious port number".to_string(),
                    bytes_transferred: op.bytes_sent + op.bytes_received,
                });
            }
        }

        let (total_sent, total_received) = network_operations.iter().fold(
            (0u64, 0u64),
            |(sent, recv), op| (sent + op.bytes_sent, recv + op.bytes_received),
        );

        NetworkAnalysisSection {
            total_connections,
            unique_destinations: unique_destinations.len() as u32,
            protocols_used: protocols_used.into_iter().collect(),
            suspicious_connections,
            dns_queries: vec![],
            data_transfer_summary: DataTransferSummary {
                total_sent_bytes: total_sent,
                total_received_bytes: total_received,
                outbound_connections: total_connections,
                inbound_connections: 0,
            },
        }
    }

    /// Analyze file activity
    fn analyze_file_activity(&self, file_operations: &[FileOperation]) -> FileActivitySection {
        let mut files_created = Vec::new();
        let mut files_modified = Vec::new();
        let mut files_deleted = Vec::new();
        let mut files_accessed = Vec::new();
        let mut suspicious_file_operations = Vec::new();

        for op in file_operations {
            let path = op.source_path.display().to_string();
            let suspicious = self.is_sensitive_path(&path);

            if suspicious {
                suspicious_file_operations.push(format!(
                    "{:?} on {}",
                    op.operation_type, path
                ));
            }

            let detail = FileActivityDetail {
                path: path.clone(),
                operation: format!("{:?}", op.operation_type),
                timestamp: op.timestamp,
                suspicious,
                reason: if suspicious {
                    Some("Sensitive path".to_string())
                } else {
                    None
                },
            };

            match op.operation_type {
                FileOperationType::Create => files_created.push(detail),
                FileOperationType::Modify => files_modified.push(detail),
                FileOperationType::Delete => files_deleted.push(detail),
                FileOperationType::Read => files_accessed.push(detail),
                _ => {}
            }
        }

        FileActivitySection {
            files_created,
            files_modified,
            files_deleted,
            files_accessed,
            suspicious_file_operations,
        }
    }

    /// Analyze process activity
    fn analyze_process_activity(
        &self,
        process_operations: &[ProcessOperation],
    ) -> ProcessActivitySection {
        let mut processes_created = Vec::new();
        let mut process_injections = Vec::new();
        let mut suspicious_commands = Vec::new();

        for op in process_operations {
            let suspicious = self.is_suspicious_command(&op.command_line);

            if suspicious {
                suspicious_commands.push(op.command_line.clone());
            }

            processes_created.push(ProcessDetail {
                name: op.process_name.clone(),
                pid: op.process_id,
                command_line: op.command_line.clone(),
                timestamp: op.timestamp,
                parent_pid: op.parent_process_id,
                suspicious,
            });

            if matches!(op.operation_type, crate::analyzers::dynamic_analyzer::ProcessOperationType::Inject) {
                process_injections.push(format!(
                    "{} injected into PID {}",
                    op.process_name, op.process_id
                ));
            }
        }

        ProcessActivitySection {
            processes_created,
            process_injections,
            process_tree: vec![],
            suspicious_commands,
        }
    }

    /// Generate security recommendations
    fn generate_recommendations(&self, threat_assessment: &ThreatAssessment) -> Vec<String> {
        let mut recommendations = Vec::new();

        if threat_assessment.is_malicious {
            recommendations.push("Quarantine the file immediately".to_string());
            recommendations.push("Block all IOCs at network perimeter".to_string());
        }

        if threat_assessment.capability_assessment.can_persist {
            recommendations.push("Check for persistence mechanisms on potentially affected systems".to_string());
        }

        if threat_assessment.capability_assessment.can_exfiltrate {
            recommendations.push("Monitor for unusual outbound network traffic".to_string());
        }

        if threat_assessment.capability_assessment.can_evade_detection {
            recommendations.push("Update detection signatures to account for evasion techniques".to_string());
        }

        recommendations.push("Conduct full forensic analysis if deployed in production".to_string());

        recommendations
    }

    /// Helper methods
    fn load_malicious_ips() -> HashSet<String> {
        // In production, load from threat intelligence feeds
        HashSet::new()
    }

    fn load_malicious_domains() -> HashSet<String> {
        HashSet::new()
    }

    fn load_suspicious_paths() -> HashSet<String> {
        let mut paths = HashSet::new();
        paths.insert("/tmp/".to_string());
        paths.insert("/var/tmp/".to_string());
        paths.insert("system32".to_string());
        paths.insert("startup".to_string());
        paths
    }

    fn is_malicious_ip(&self, ip: &str) -> bool {
        self.threat_intel_db.known_malicious_ips.contains(ip)
    }

    fn is_sensitive_path(&self, path: &str) -> bool {
        self.threat_intel_db
            .suspicious_file_paths
            .iter()
            .any(|p| path.contains(p))
    }

    fn is_suspicious_port(&self, port: u16) -> bool {
        matches!(port, 4444 | 5555 | 6666 | 7777 | 8888 | 9999 | 1337 | 31337)
    }

    fn is_suspicious_command(&self, command: &str) -> bool {
        let cmd_lower = command.to_lowercase();
        let suspicious_cmds = [
            "powershell",
            "cmd.exe",
            "wmic",
            "reg.exe",
            "net.exe",
            "sc.exe",
            "schtasks",
        ];
        suspicious_cmds.iter().any(|&cmd| cmd_lower.contains(cmd))
    }

    fn detect_propagation_capability(&self, behavior: &DynamicBehavior) -> bool {
        // Check for network scanning, file sharing access, etc.
        behavior.network_operations.len() > 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_report_generator_creation() {
        let generator = ReportGenerator::new();
        assert!(generator.is_ok());
    }

    #[test]
    fn test_threat_level_calculation() {
        let generator = ReportGenerator::new().unwrap();

        let behavior = DynamicBehavior {
            file_operations: vec![],
            network_operations: vec![],
            process_operations: vec![],
            registry_operations: vec![],
            system_calls: vec![],
            screenshots: vec![],
            network_capture: None,
        };

        let threat_indicators = DynamicThreatIndicators {
            malicious_network_connections: vec![],
            suspicious_file_operations: vec![],
            malicious_processes: vec![],
            registry_modifications: vec![],
            persistence_mechanisms: vec![],
            evasion_techniques: vec![],
            data_exfiltration_attempts: vec![],
        };

        let summary = generator.generate_executive_summary(&behavior, &threat_indicators);
        assert_eq!(summary.threat_level, ThreatLevel::Clean);
    }

    #[test]
    fn test_suspicious_port_detection() {
        let generator = ReportGenerator::new().unwrap();
        assert!(generator.is_suspicious_port(4444));
        assert!(generator.is_suspicious_port(1337));
        assert!(!generator.is_suspicious_port(80));
        assert!(!generator.is_suspicious_port(443));
    }
}
