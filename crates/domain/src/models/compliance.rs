//! Compliance domain models for admin API.
//!
//! AP-11.7-8: Compliance Dashboard and Report endpoints

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Compliance dashboard response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ComplianceDashboardResponse {
    /// Data Subject Request statistics.
    pub data_subject_requests: DataSubjectRequestStats,
    /// Audit log statistics.
    pub audit_logs: AuditLogStats,
    /// Data retention status.
    pub data_retention: DataRetentionStatus,
    /// Compliance score (0-100).
    pub compliance_score: u8,
    /// Compliance status summary.
    pub status: ComplianceStatus,
}

/// Data Subject Request statistics.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DataSubjectRequestStats {
    /// Total requests.
    pub total: i64,
    /// Pending requests.
    pub pending: i64,
    /// In-progress requests.
    pub in_progress: i64,
    /// Completed requests.
    pub completed: i64,
    /// Rejected requests.
    pub rejected: i64,
    /// Overdue requests (pending/in-progress past due date).
    pub overdue: i64,
    /// Average processing time in days (for completed requests).
    pub average_processing_days: Option<f64>,
    /// Compliance rate (completed on time / total completed) as percentage.
    pub compliance_rate: Option<f64>,
}

/// Audit log statistics.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuditLogStats {
    /// Total audit log entries.
    pub total_entries: i64,
    /// Entries in the last 24 hours.
    pub last_24_hours: i64,
    /// Entries in the last 7 days.
    pub last_7_days: i64,
    /// Entries in the last 30 days.
    pub last_30_days: i64,
}

/// Data retention status.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DataRetentionStatus {
    /// Location retention policy in days.
    pub location_retention_days: u32,
    /// Audit log retention policy in days (if any).
    pub audit_log_retention_days: Option<u32>,
    /// Whether data cleanup is enabled.
    pub cleanup_enabled: bool,
}

/// Overall compliance status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    /// All compliance requirements met.
    Compliant,
    /// Some issues need attention.
    NeedsAttention,
    /// Critical compliance issues.
    NonCompliant,
}

/// Compliance report request.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ComplianceReportQuery {
    /// Start date for the report period.
    #[serde(default)]
    pub from: Option<DateTime<Utc>>,
    /// End date for the report period.
    #[serde(default)]
    pub to: Option<DateTime<Utc>>,
    /// Report format.
    #[serde(default)]
    pub format: Option<ComplianceReportFormat>,
}

/// Report format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceReportFormat {
    /// JSON format.
    #[default]
    Json,
    /// PDF format (not yet implemented).
    Pdf,
}

/// Compliance report response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ComplianceReportResponse {
    /// Report generation timestamp.
    pub generated_at: DateTime<Utc>,
    /// Report period start.
    pub period_start: DateTime<Utc>,
    /// Report period end.
    pub period_end: DateTime<Utc>,
    /// Organization summary.
    pub organization: OrganizationReportSummary,
    /// Data Subject Request summary.
    pub data_subject_requests: DataSubjectRequestReportSummary,
    /// Audit activity summary.
    pub audit_activity: AuditActivitySummary,
    /// Compliance assessment.
    pub compliance_assessment: ComplianceAssessment,
}

/// Organization summary for report.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OrganizationReportSummary {
    /// Total active devices.
    pub total_devices: i64,
    /// Total users.
    pub total_users: i64,
    /// Total groups.
    pub total_groups: i64,
}

/// Data Subject Request summary for report.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DataSubjectRequestReportSummary {
    /// Total requests in period.
    pub total_requests: i64,
    /// Requests by type.
    pub by_type: Vec<RequestTypeCount>,
    /// Requests by status.
    pub by_status: RequestStatusCounts,
    /// Average processing time in days.
    pub average_processing_days: Option<f64>,
    /// Compliance rate percentage.
    pub compliance_rate: Option<f64>,
}

/// Request count by type.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RequestTypeCount {
    /// Request type.
    pub request_type: String,
    /// Count.
    pub count: i64,
}

/// Request counts by status.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RequestStatusCounts {
    pub pending: i64,
    pub in_progress: i64,
    pub completed: i64,
    pub rejected: i64,
    pub cancelled: i64,
}

/// Audit activity summary.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuditActivitySummary {
    /// Total audit entries in period.
    pub total_entries: i64,
    /// Top actions by count.
    pub top_actions: Vec<ActionCount>,
}

/// Action count.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ActionCount {
    /// Action name.
    pub action: String,
    /// Count.
    pub count: i64,
}

/// Compliance assessment.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ComplianceAssessment {
    /// Overall compliance score (0-100).
    pub score: u8,
    /// Overall status.
    pub status: ComplianceStatus,
    /// Assessment findings.
    pub findings: Vec<ComplianceFinding>,
    /// Recommendations.
    pub recommendations: Vec<String>,
}

/// Compliance finding.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ComplianceFinding {
    /// Finding category.
    pub category: String,
    /// Finding severity.
    pub severity: FindingSeverity,
    /// Finding description.
    pub description: String,
}

/// Finding severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    /// Informational finding.
    Info,
    /// Warning - should be addressed.
    Warning,
    /// Critical - must be addressed.
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_status_serialization() {
        let status = ComplianceStatus::Compliant;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"compliant\"");
    }

    #[test]
    fn test_report_format_default() {
        let format = ComplianceReportFormat::default();
        assert_eq!(format, ComplianceReportFormat::Json);
    }

    #[test]
    fn test_finding_severity_serialization() {
        let severity = FindingSeverity::Warning;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"warning\"");
    }
}
