// backend/bounty-manager/src/handlers/validation.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use shared::types::ApiResponse;
use super::bounty_crud::PaginationParams;
use crate::handlers::bounty_crud::{BountyManagerState, ThreatVerdict};
use crate::handlers::submission::{Submission, AnalysisDetails};

/// Validation result for a submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub bounty_id: Uuid,
    pub validator_id: String,
    pub validator_type: ValidatorType,
    pub validation_status: ValidationStatus,
    pub quality_score: f32, // 0.0 to 1.0
    pub checks_performed: Vec<ValidationCheck>,
    pub issues_found: Vec<ValidationIssue>,
    pub recommendations: Vec<String>,
    pub validated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidatorType {
    Automated,     // Automated validation system
    Human,         // Manual review by expert
    Hybrid,        // Combination of both
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    Pending,       // Awaiting validation
    Validating,    // Currently being validated
    Passed,        // All checks passed
    PassedWithWarnings, // Passed but has minor issues
    Failed,        // Critical issues found
    RequiresReview, // Needs human review
}

/// Individual validation check performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub check_type: ValidationCheckType,
    pub check_name: String,
    pub passed: bool,
    pub severity: CheckSeverity,
    pub description: String,
    pub details: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationCheckType {
    // Completeness checks
    RequiredFieldsPresent,
    AnalysisDetailsComplete,

    // Quality checks
    ConfidenceReasonable,
    AnalysisDepth,
    ThreatIndicatorsValid,

    // Consistency checks
    VerdictAlignedWithEvidence,
    ConfidenceMatchesEvidence,
    CrossFieldConsistency,

    // Technical checks
    HashesValid,
    TimestampsValid,
    DataIntegrity,
    FormatCompliance,

    // Security checks
    MaliciousDataDetection,
    InjectionAttempts,
    SuspiciousPatterns,

    // Business rules
    StakeRequirementsMet,
    DeadlineNotExceeded,
    NoDuplicateSubmission,
    ReputationRequirementsMet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Issues discovered during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub issue_type: IssueType,
    pub severity: IssueSeverity,
    pub field: Option<String>, // Field where issue was found
    pub message: String,
    pub details: String,
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueType {
    MissingData,
    InvalidFormat,
    InconsistentData,
    LowQualityAnalysis,
    SuspiciousActivity,
    PolicyViolation,
    TechnicalError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Minor,      // Can be ignored
    Moderate,   // Should be addressed
    Major,      // Must be fixed
    Critical,   // Submission should be rejected
}

/// Submission quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub overall_score: f32,
    pub completeness_score: f32,
    pub accuracy_score: f32,
    pub detail_score: f32,
    pub consistency_score: f32,
    pub timeliness_score: f32,
}

/// Validation configuration/rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub min_quality_score: f32,
    pub required_checks: Vec<ValidationCheckType>,
    pub min_analysis_depth: AnalysisDepthLevel,
    pub max_validation_time_seconds: u64,
    pub strict_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisDepthLevel {
    Basic,      // Minimal analysis
    Standard,   // Normal depth
    Detailed,   // Comprehensive analysis
    Expert,     // Deep dive analysis
}

// Request/Response DTOs

#[derive(Debug, Deserialize)]
pub struct ValidateSubmissionRequest {
    pub submission_id: Uuid,
    pub validation_rules: Option<ValidationRules>,
    pub force_revalidation: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BulkValidateRequest {
    pub submission_ids: Vec<Uuid>,
    pub validation_rules: Option<ValidationRules>,
}

#[derive(Debug, Serialize)]
pub struct BulkValidateResponse {
    pub results: Vec<ValidationResult>,
    pub total_processed: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
}

#[derive(Debug, Deserialize)]
pub struct ValidationFilters {
    pub bounty_id: Option<Uuid>,
    pub submission_id: Option<Uuid>,
    pub status: Option<ValidationStatus>,
    pub validator_type: Option<ValidatorType>,
    pub min_quality_score: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct ValidationListResponse {
    pub validations: Vec<ValidationResult>,
    pub total_count: usize,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize)]
pub struct ValidationStatsResponse {
    pub total_validations: u64,
    pub passed_count: u64,
    pub failed_count: u64,
    pub avg_quality_score: f32,
    pub avg_validation_time_ms: u64,
    pub common_issues: Vec<CommonIssue>,
}

#[derive(Debug, Serialize)]
pub struct CommonIssue {
    pub issue_type: IssueType,
    pub count: u64,
    pub percentage: f32,
}

// Handler implementations

/// Validate a single submission
pub async fn validate_submission(
    State(_state): State<BountyManagerState>,
    Extension(validator_id): Extension<String>,
    Json(req): Json<ValidateSubmissionRequest>,
) -> Result<Json<ApiResponse<ValidationResult>>, StatusCode> {
    // TODO: Fetch submission from database
    let submission = create_mock_submission(req.submission_id);

    // TODO: Check if already validated and if revalidation is needed
    if req.force_revalidation.unwrap_or(false) {
        // Force revalidation
    }

    // Get validation rules (use provided or default)
    let rules = req.validation_rules.unwrap_or_else(get_default_validation_rules);

    // Perform validation
    let validation_result = perform_validation(&submission, &rules, validator_id);

    // TODO: Save validation result to database
    // TODO: Update submission status based on validation
    // TODO: Emit validation event

    Ok(Json(ApiResponse::success(validation_result)))
}

/// Get validation result for a submission
pub async fn get_validation_result(
    State(_state): State<BountyManagerState>,
    Path(validation_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ValidationResult>>, StatusCode> {
    // TODO: Fetch from database
    let mock_result = create_mock_validation_result(validation_id);

    Ok(Json(ApiResponse::success(mock_result)))
}

/// List validation results with filters
pub async fn list_validations(
    State(_state): State<BountyManagerState>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<ValidationFilters>,
) -> Result<Json<ApiResponse<ValidationListResponse>>, StatusCode> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20).min(100);

    // TODO: Implement database query with filters
    let validations = vec![
        create_mock_validation_result(Uuid::new_v4()),
        create_mock_validation_result(Uuid::new_v4()),
    ];

    let response_data = ValidationListResponse {
        validations: validations.clone(),
        total_count: validations.len(),
        page,
        per_page,
    };

    Ok(Json(ApiResponse::success(response_data)))
}

/// Bulk validate multiple submissions
pub async fn bulk_validate_submissions(
    State(_state): State<BountyManagerState>,
    Extension(validator_id): Extension<String>,
    Json(req): Json<BulkValidateRequest>,
) -> Result<Json<ApiResponse<BulkValidateResponse>>, StatusCode> {
    if req.submission_ids.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let rules = req.validation_rules.unwrap_or_else(get_default_validation_rules);
    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;
    let mut warnings = 0;

    // TODO: Fetch all submissions from database
    for submission_id in &req.submission_ids {
        let submission = create_mock_submission(*submission_id);
        let result = perform_validation(&submission, &rules, validator_id.clone());

        match result.validation_status {
            ValidationStatus::Passed => passed += 1,
            ValidationStatus::PassedWithWarnings => warnings += 1,
            ValidationStatus::Failed => failed += 1,
            _ => {}
        }

        results.push(result);
    }

    let response_data = BulkValidateResponse {
        results,
        total_processed: req.submission_ids.len(),
        passed,
        failed,
        warnings,
    };

    // TODO: Save all validation results to database
    // TODO: Emit bulk validation event

    Ok(Json(ApiResponse::success(response_data)))
}

/// Get validation statistics
pub async fn get_validation_stats(
    State(_state): State<BountyManagerState>,
) -> Result<Json<ApiResponse<ValidationStatsResponse>>, StatusCode> {
    // TODO: Implement real statistics from database
    let stats = ValidationStatsResponse {
        total_validations: 450,
        passed_count: 378,
        failed_count: 52,
        avg_quality_score: 0.82,
        avg_validation_time_ms: 850,
        common_issues: vec![
            CommonIssue {
                issue_type: IssueType::LowQualityAnalysis,
                count: 35,
                percentage: 7.8,
            },
            CommonIssue {
                issue_type: IssueType::InconsistentData,
                count: 28,
                percentage: 6.2,
            },
            CommonIssue {
                issue_type: IssueType::MissingData,
                count: 19,
                percentage: 4.2,
            },
        ],
    };

    Ok(Json(ApiResponse::success(stats)))
}

/// Re-validate a submission (admin only)
pub async fn revalidate_submission(
    State(_state): State<BountyManagerState>,
    Extension(validator_id): Extension<String>,
    Path(submission_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ValidationResult>>, StatusCode> {
    // TODO: Verify validator has admin role
    // TODO: Fetch submission from database
    let submission = create_mock_submission(submission_id);

    let rules = get_default_validation_rules();
    let validation_result = perform_validation(&submission, &rules, validator_id);

    // TODO: Save validation result
    // TODO: Update submission status

    Ok(Json(ApiResponse::success(validation_result)))
}

// Core validation logic

/// Perform comprehensive validation on a submission
fn perform_validation(
    submission: &Submission,
    rules: &ValidationRules,
    validator_id: String,
) -> ValidationResult {
    let start_time = Utc::now();
    let mut checks = Vec::new();
    let mut issues = Vec::new();
    let mut recommendations = Vec::new();

    // 1. Check required fields
    let required_fields_check = check_required_fields(submission);
    checks.push(required_fields_check.clone());
    if !required_fields_check.passed {
        issues.push(ValidationIssue {
            issue_type: IssueType::MissingData,
            severity: IssueSeverity::Critical,
            field: Some("required_fields".to_string()),
            message: "Missing required fields".to_string(),
            details: "Submission must include all required fields".to_string(),
            suggested_fix: Some("Ensure verdict, confidence, and analysis_details are provided".to_string()),
        });
    }

    // 2. Validate confidence value
    let confidence_check = check_confidence_reasonable(submission);
    checks.push(confidence_check.clone());
    if !confidence_check.passed {
        issues.push(ValidationIssue {
            issue_type: IssueType::InvalidFormat,
            severity: IssueSeverity::Major,
            field: Some("confidence".to_string()),
            message: "Confidence value out of range".to_string(),
            details: format!("Confidence must be between 0.0 and 1.0, got {}", submission.confidence),
            suggested_fix: Some("Provide a valid confidence value between 0.0 and 1.0".to_string()),
        });
    }

    // 3. Check analysis depth
    let depth_check = check_analysis_depth(submission, &rules.min_analysis_depth);
    checks.push(depth_check.clone());
    if !depth_check.passed {
        issues.push(ValidationIssue {
            issue_type: IssueType::LowQualityAnalysis,
            severity: IssueSeverity::Moderate,
            field: Some("analysis_details".to_string()),
            message: "Insufficient analysis depth".to_string(),
            details: "Analysis lacks required detail and depth".to_string(),
            suggested_fix: Some("Provide more comprehensive analysis including behavioral and static data".to_string()),
        });
        recommendations.push("Include more detailed threat indicators".to_string());
    }

    // 4. Verify verdict alignment
    let alignment_check = check_verdict_alignment(submission);
    checks.push(alignment_check.clone());
    if !alignment_check.passed {
        issues.push(ValidationIssue {
            issue_type: IssueType::InconsistentData,
            severity: IssueSeverity::Major,
            field: Some("verdict".to_string()),
            message: "Verdict doesn't align with evidence".to_string(),
            details: "The stated verdict is inconsistent with the analysis details provided".to_string(),
            suggested_fix: Some("Review analysis and ensure verdict matches the evidence".to_string()),
        });
    }

    // 5. Validate stake requirements
    let stake_check = check_stake_requirements(submission);
    checks.push(stake_check.clone());
    if !stake_check.passed {
        issues.push(ValidationIssue {
            issue_type: IssueType::PolicyViolation,
            severity: IssueSeverity::Critical,
            field: Some("stake_amount".to_string()),
            message: "Stake amount below minimum".to_string(),
            details: "Submission doesn't meet minimum stake requirements".to_string(),
            suggested_fix: Some("Increase stake amount to meet minimum requirements".to_string()),
        });
    }

    // 6. Check for suspicious patterns
    let security_check = check_security_issues(submission);
    checks.push(security_check.clone());
    if !security_check.passed {
        issues.push(ValidationIssue {
            issue_type: IssueType::SuspiciousActivity,
            severity: IssueSeverity::Critical,
            field: None,
            message: "Potential security issue detected".to_string(),
            details: "Submission contains patterns that may indicate malicious intent".to_string(),
            suggested_fix: None,
        });
    }

    // Calculate quality score
    let quality_metrics = calculate_quality_metrics(submission, &checks);
    let quality_score = quality_metrics.overall_score;

    // Determine validation status
    let validation_status = determine_validation_status(&checks, &issues, quality_score, rules);

    // Add general recommendations
    if quality_score < 0.8 {
        recommendations.push("Consider providing more detailed analysis to improve quality score".to_string());
    }

    ValidationResult {
        id: Uuid::new_v4(),
        submission_id: submission.id,
        bounty_id: submission.bounty_id,
        validator_id,
        validator_type: ValidatorType::Automated,
        validation_status,
        quality_score,
        checks_performed: checks,
        issues_found: issues,
        recommendations,
        validated_at: start_time,
        metadata: HashMap::new(),
    }
}

// Individual check functions

fn check_required_fields(submission: &Submission) -> ValidationCheck {
    let has_verdict = true; // Verdict is always present in the struct
    let has_confidence = submission.confidence >= 0.0;
    let has_analysis = !submission.analysis_details.malware_families.is_empty()
        || !submission.analysis_details.threat_indicators.is_empty();

    let passed = has_verdict && has_confidence && has_analysis;

    ValidationCheck {
        check_type: ValidationCheckType::RequiredFieldsPresent,
        check_name: "Required Fields Check".to_string(),
        passed,
        severity: CheckSeverity::Critical,
        description: "Verify all required fields are present".to_string(),
        details: Some(format!("Verdict: {}, Confidence: {}, Analysis: {}",
            has_verdict, has_confidence, has_analysis)),
        execution_time_ms: 5,
    }
}

fn check_confidence_reasonable(submission: &Submission) -> ValidationCheck {
    let passed = submission.confidence >= 0.0 && submission.confidence <= 1.0;

    ValidationCheck {
        check_type: ValidationCheckType::ConfidenceReasonable,
        check_name: "Confidence Range Check".to_string(),
        passed,
        severity: CheckSeverity::High,
        description: "Ensure confidence is within valid range [0.0, 1.0]".to_string(),
        details: Some(format!("Confidence value: {}", submission.confidence)),
        execution_time_ms: 2,
    }
}

fn check_analysis_depth(submission: &Submission, _min_depth: &AnalysisDepthLevel) -> ValidationCheck {
    let details = &submission.analysis_details;

    // Score based on completeness
    let has_malware_families = !details.malware_families.is_empty();
    let has_threat_indicators = !details.threat_indicators.is_empty();
    let has_behavioral = details.behavioral_analysis.is_some();
    let has_static = details.static_analysis.is_some();
    let has_network = details.network_analysis.is_some();

    let completeness_count = vec![
        has_malware_families,
        has_threat_indicators,
        has_behavioral,
        has_static,
        has_network,
    ].iter().filter(|&&x| x).count();

    // Require at least 3 out of 5 for standard depth
    let passed = completeness_count >= 3;

    ValidationCheck {
        check_type: ValidationCheckType::AnalysisDepth,
        check_name: "Analysis Depth Check".to_string(),
        passed,
        severity: CheckSeverity::Medium,
        description: "Verify analysis has sufficient depth and detail".to_string(),
        details: Some(format!("Analysis components present: {}/5", completeness_count)),
        execution_time_ms: 15,
    }
}

fn check_verdict_alignment(submission: &Submission) -> ValidationCheck {
    let details = &submission.analysis_details;

    // Simple heuristic: if verdict is Malicious, should have threat indicators
    let passed = match submission.verdict {
        ThreatVerdict::Malicious => !details.threat_indicators.is_empty() || !details.malware_families.is_empty(),
        ThreatVerdict::Benign => true, // For now, always pass for benign
        ThreatVerdict::Suspicious => !details.threat_indicators.is_empty(),
        ThreatVerdict::Unknown => true, // Unknown can have any indicators
    };

    ValidationCheck {
        check_type: ValidationCheckType::VerdictAlignedWithEvidence,
        check_name: "Verdict Alignment Check".to_string(),
        passed,
        severity: CheckSeverity::High,
        description: "Ensure verdict matches the provided evidence".to_string(),
        details: Some(format!("Verdict: {:?}, Indicators: {}",
            submission.verdict, details.threat_indicators.len())),
        execution_time_ms: 10,
    }
}

fn check_stake_requirements(submission: &Submission) -> ValidationCheck {
    // TODO: Get actual minimum stake from bounty
    let min_stake = 1000u64;
    let passed = submission.stake_amount >= min_stake;

    ValidationCheck {
        check_type: ValidationCheckType::StakeRequirementsMet,
        check_name: "Stake Requirements Check".to_string(),
        passed,
        severity: CheckSeverity::Critical,
        description: "Verify stake meets minimum requirements".to_string(),
        details: Some(format!("Stake: {}, Required: {}", submission.stake_amount, min_stake)),
        execution_time_ms: 3,
    }
}

fn check_security_issues(submission: &Submission) -> ValidationCheck {
    // Check for potential injection attempts or malicious data
    let mut suspicious = false;

    // Basic checks for suspicious patterns in strings
    for indicator in &submission.analysis_details.threat_indicators {
        if indicator.value.contains("<script>") || indicator.value.contains("'; DROP") {
            suspicious = true;
            break;
        }
    }

    ValidationCheck {
        check_type: ValidationCheckType::MaliciousDataDetection,
        check_name: "Security Issues Check".to_string(),
        passed: !suspicious,
        severity: CheckSeverity::Critical,
        description: "Detect potential malicious data or injection attempts".to_string(),
        details: Some(if suspicious { "Suspicious patterns detected".to_string() } else { "No issues found".to_string() }),
        execution_time_ms: 8,
    }
}

fn calculate_quality_metrics(submission: &Submission, checks: &[ValidationCheck]) -> QualityMetrics {
    let total_checks = checks.len() as f32;
    let passed_checks = checks.iter().filter(|c| c.passed).count() as f32;
    let overall_score = if total_checks > 0.0 { passed_checks / total_checks } else { 0.0 };

    // Calculate individual scores
    let completeness_score = if submission.confidence > 0.0 { 0.9 } else { 0.5 };
    let accuracy_score = overall_score; // Simplified
    let detail_score = (submission.analysis_details.threat_indicators.len() as f32 / 5.0).min(1.0);
    let consistency_score = if checks.iter().any(|c| c.check_type == ValidationCheckType::VerdictAlignedWithEvidence && c.passed) { 1.0 } else { 0.5 };
    let timeliness_score = 1.0; // Placeholder

    QualityMetrics {
        overall_score,
        completeness_score,
        accuracy_score,
        detail_score,
        consistency_score,
        timeliness_score,
    }
}

fn determine_validation_status(
    checks: &[ValidationCheck],
    issues: &[ValidationIssue],
    quality_score: f32,
    rules: &ValidationRules,
) -> ValidationStatus {
    // Check for critical failures
    let has_critical_issues = issues.iter().any(|i| matches!(i.severity, IssueSeverity::Critical));
    if has_critical_issues {
        return ValidationStatus::Failed;
    }

    // Check critical checks
    let critical_check_failed = checks.iter().any(|c|
        matches!(c.severity, CheckSeverity::Critical) && !c.passed
    );
    if critical_check_failed {
        return ValidationStatus::Failed;
    }

    // Check quality score
    if quality_score < rules.min_quality_score {
        return ValidationStatus::Failed;
    }

    // Check for major issues
    let has_major_issues = issues.iter().any(|i| matches!(i.severity, IssueSeverity::Major));
    if has_major_issues {
        return ValidationStatus::PassedWithWarnings;
    }

    // Check for moderate issues
    let has_moderate_issues = issues.iter().any(|i| matches!(i.severity, IssueSeverity::Moderate));
    if has_moderate_issues {
        return ValidationStatus::PassedWithWarnings;
    }

    ValidationStatus::Passed
}

fn get_default_validation_rules() -> ValidationRules {
    ValidationRules {
        min_quality_score: 0.7,
        required_checks: vec![
            ValidationCheckType::RequiredFieldsPresent,
            ValidationCheckType::ConfidenceReasonable,
            ValidationCheckType::StakeRequirementsMet,
        ],
        min_analysis_depth: AnalysisDepthLevel::Standard,
        max_validation_time_seconds: 30,
        strict_mode: false,
    }
}

// Mock data helpers

fn create_mock_submission(id: Uuid) -> Submission {
    use crate::handlers::submission::*;

    Submission {
        id,
        bounty_id: Uuid::new_v4(),
        engine_id: "engine_123".to_string(),
        engine_type: EngineType::Automated,
        verdict: ThreatVerdict::Malicious,
        confidence: 0.92,
        stake_amount: 50000,
        analysis_details: AnalysisDetails {
            malware_families: vec!["Trojan.Generic".to_string()],
            threat_indicators: vec![
                ThreatIndicator {
                    indicator_type: "hash".to_string(),
                    value: "abc123def456".to_string(),
                    severity: ThreatSeverity::High,
                    description: Some("Known malicious hash".to_string()),
                }
            ],
            behavioral_analysis: Some(BehavioralAnalysis {
                network_connections: vec!["192.168.1.100:8080".to_string()],
                file_operations: vec!["CreateFile: C:\\temp\\malware.exe".to_string()],
                registry_modifications: vec!["HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string()],
                process_creation: vec!["cmd.exe".to_string()],
                api_calls: vec!["CreateProcessA".to_string()],
            }),
            static_analysis: None,
            network_analysis: None,
            metadata: HashMap::new(),
        },
        status: SubmissionStatus::Active,
        transaction_hash: Some("0xabc123def456...".to_string()),
        submitted_at: Utc::now() - chrono::Duration::minutes(30),
        processed_at: None,
        accuracy_score: None,
    }
}

fn create_mock_validation_result(id: Uuid) -> ValidationResult {
    let now = Utc::now();

    ValidationResult {
        id,
        submission_id: Uuid::new_v4(),
        bounty_id: Uuid::new_v4(),
        validator_id: "validator_auto_001".to_string(),
        validator_type: ValidatorType::Automated,
        validation_status: ValidationStatus::Passed,
        quality_score: 0.87,
        checks_performed: vec![
            ValidationCheck {
                check_type: ValidationCheckType::RequiredFieldsPresent,
                check_name: "Required Fields Check".to_string(),
                passed: true,
                severity: CheckSeverity::Critical,
                description: "Verify all required fields are present".to_string(),
                details: Some("All required fields found".to_string()),
                execution_time_ms: 5,
            },
            ValidationCheck {
                check_type: ValidationCheckType::ConfidenceReasonable,
                check_name: "Confidence Range Check".to_string(),
                passed: true,
                severity: CheckSeverity::High,
                description: "Ensure confidence is within valid range".to_string(),
                details: Some("Confidence: 0.92".to_string()),
                execution_time_ms: 2,
            },
        ],
        issues_found: Vec::new(),
        recommendations: vec![
            "Consider adding more static analysis details".to_string(),
        ],
        validated_at: now,
        metadata: HashMap::new(),
    }
}
