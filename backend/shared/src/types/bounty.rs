/// Shared bounty-related type definitions
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Bounty status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bounty_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum BountyStatus {
    Draft,
    Open,
    InProgress,
    UnderReview,
    Completed,
    Cancelled,
    Expired,
}

impl Default for BountyStatus {
    fn default() -> Self {
        BountyStatus::Draft
    }
}

impl std::fmt::Display for BountyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BountyStatus::Draft => write!(f, "draft"),
            BountyStatus::Open => write!(f, "open"),
            BountyStatus::InProgress => write!(f, "in_progress"),
            BountyStatus::UnderReview => write!(f, "under_review"),
            BountyStatus::Completed => write!(f, "completed"),
            BountyStatus::Cancelled => write!(f, "cancelled"),
            BountyStatus::Expired => write!(f, "expired"),
        }
    }
}

/// Threat verdict enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "threat_verdict", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ThreatVerdict {
    Malicious,
    Benign,
    Suspicious,
    Unknown,
}

impl Default for ThreatVerdict {
    fn default() -> Self {
        ThreatVerdict::Unknown
    }
}

impl std::fmt::Display for ThreatVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreatVerdict::Malicious => write!(f, "malicious"),
            ThreatVerdict::Benign => write!(f, "benign"),
            ThreatVerdict::Suspicious => write!(f, "suspicious"),
            ThreatVerdict::Unknown => write!(f, "unknown"),
        }
    }
}

/// Artifact type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "artifact_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ArtifactType {
    File,
    Url,
    Hash,
    Ip,
    Domain,
}

impl std::fmt::Display for ArtifactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactType::File => write!(f, "file"),
            ArtifactType::Url => write!(f, "url"),
            ArtifactType::Hash => write!(f, "hash"),
            ArtifactType::Ip => write!(f, "ip"),
            ArtifactType::Domain => write!(f, "domain"),
        }
    }
}

/// Priority level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "priority_level", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PriorityLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for PriorityLevel {
    fn default() -> Self {
        PriorityLevel::Medium
    }
}

/// Severity level enumeration  
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeverityLevel {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Default for SeverityLevel {
    fn default() -> Self {
        SeverityLevel::Info
    }
}
