//! Analytics domain models.
//!
//! AP-10: Dashboard & Analytics - User, Device, and API analytics

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// Re-export AnalyticsGroupBy from app_usage to avoid duplication
pub use super::app_usage::AnalyticsGroupBy;

// ============================================================================
// User Analytics (FR-10.1)
// ============================================================================

/// Query parameters for user analytics.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UserAnalyticsQuery {
    /// Start date for analytics
    #[serde(default)]
    pub from: Option<NaiveDate>,
    /// End date for analytics
    #[serde(default)]
    pub to: Option<NaiveDate>,
    /// Group by: day, week, or month
    #[serde(default)]
    pub group_by: Option<AnalyticsGroupBy>,
}

/// User analytics response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UserAnalyticsResponse {
    pub organization_id: Uuid,
    pub period: AnalyticsPeriod,
    pub summary: UserAnalyticsSummary,
    pub trends: Vec<UserActivityTrend>,
    pub by_role: UserRoleBreakdown,
}

/// Analytics period.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AnalyticsPeriod {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

/// User analytics summary.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UserAnalyticsSummary {
    pub total_users: i64,
    pub active_users: i64,
    pub new_users_period: i64,
    pub avg_sessions_per_user: f64,
    pub avg_session_duration_seconds: f64,
}

/// User activity trend point.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UserActivityTrend {
    pub date: NaiveDate,
    pub active_users: i64,
    pub new_users: i64,
    pub returning_users: i64,
    pub total_sessions: i64,
}

/// User role breakdown.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UserRoleBreakdown {
    pub owners: i64,
    pub admins: i64,
    pub members: i64,
}

// ============================================================================
// Device Analytics (FR-10.2)
// ============================================================================

/// Query parameters for device analytics.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct DeviceAnalyticsQuery {
    /// Start date for analytics
    #[serde(default)]
    pub from: Option<NaiveDate>,
    /// End date for analytics
    #[serde(default)]
    pub to: Option<NaiveDate>,
    /// Group by: day, week, or month
    #[serde(default)]
    pub group_by: Option<AnalyticsGroupBy>,
}

/// Device analytics response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceAnalyticsResponse {
    pub organization_id: Uuid,
    pub period: AnalyticsPeriod,
    pub summary: DeviceAnalyticsSummary,
    pub trends: Vec<DeviceActivityTrend>,
    pub by_status: DeviceStatusBreakdown,
}

/// Device analytics summary.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceAnalyticsSummary {
    pub total_devices: i64,
    pub active_devices: i64,
    pub new_enrollments_period: i64,
    pub unenrollments_period: i64,
    pub total_locations_reported: i64,
    pub total_geofence_events: i64,
    pub total_commands_issued: i64,
}

/// Device activity trend point.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceActivityTrend {
    pub date: NaiveDate,
    pub active_devices: i64,
    pub new_enrollments: i64,
    pub unenrollments: i64,
    pub locations_reported: i64,
    pub geofence_events: i64,
}

/// Device status breakdown.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceStatusBreakdown {
    pub registered: i64,
    pub enrolled: i64,
    pub suspended: i64,
    pub retired: i64,
}

// ============================================================================
// API Usage Analytics (FR-10.3)
// ============================================================================

/// Query parameters for API usage analytics.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ApiUsageAnalyticsQuery {
    /// Start date for analytics
    #[serde(default)]
    pub from: Option<NaiveDate>,
    /// End date for analytics
    #[serde(default)]
    pub to: Option<NaiveDate>,
    /// Group by: day, week, or month
    #[serde(default)]
    pub group_by: Option<AnalyticsGroupBy>,
}

/// API usage analytics response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiUsageAnalyticsResponse {
    pub organization_id: Uuid,
    pub period: AnalyticsPeriod,
    pub summary: ApiUsageSummary,
    pub trends: Vec<ApiUsageTrend>,
    pub top_endpoints: Vec<EndpointUsage>,
}

/// API usage summary.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiUsageSummary {
    pub total_requests: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: i32,
    pub total_data_transferred_bytes: i64,
}

/// API usage trend point.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiUsageTrend {
    pub date: NaiveDate,
    pub total_requests: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub avg_response_time_ms: f64,
}

/// Endpoint usage statistics.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EndpointUsage {
    pub endpoint: String,
    pub method: String,
    pub total_requests: i64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub percentage: f64,
}

// ============================================================================
// Report Generation (FR-10.4, FR-10.5, FR-10.6, FR-10.7)
// ============================================================================

/// Request to generate a report.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct GenerateReportRequest {
    /// Start date for report data
    pub from: NaiveDate,
    /// End date for report data
    pub to: NaiveDate,
    /// Report format
    #[serde(default)]
    pub format: ReportFormat,
    /// Additional parameters (depends on report type)
    #[serde(default)]
    pub parameters: Option<serde_json::Value>,
}

/// Report format options.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReportFormat {
    #[default]
    Csv,
    Json,
    Xlsx,
    Pdf,
}

/// Report job response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ReportJobResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub report_type: String,
    pub status: ReportStatus,
    pub parameters: serde_json::Value,
    pub file_size_bytes: Option<i64>,
    pub error_message: Option<String>,
    pub created_by: Uuid,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Report status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl From<&str> for ReportStatus {
    fn from(s: &str) -> Self {
        match s {
            "pending" => ReportStatus::Pending,
            "processing" => ReportStatus::Processing,
            "completed" => ReportStatus::Completed,
            "failed" => ReportStatus::Failed,
            _ => ReportStatus::Pending,
        }
    }
}

impl ReportStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportStatus::Pending => "pending",
            ReportStatus::Processing => "processing",
            ReportStatus::Completed => "completed",
            ReportStatus::Failed => "failed",
        }
    }
}

/// Report download response with presigned URL or file content.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ReportDownloadResponse {
    pub report_id: Uuid,
    pub file_name: String,
    pub content_type: String,
    pub file_size_bytes: i64,
    /// For small files, the base64-encoded content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_base64: Option<String>,
    /// For larger files, a presigned download URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_expires_at: Option<DateTime<Utc>>,
}
