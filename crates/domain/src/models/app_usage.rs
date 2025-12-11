//! App usage domain models.
//!
//! AP-8.1-8.2, AP-8.7: App usage summary, history, and analytics

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// App usage summary for a device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageSummary {
    /// Device ID
    pub device_id: Uuid,
    /// Device display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    /// Total foreground time in milliseconds
    pub total_foreground_time_ms: i64,
    /// Total background time in milliseconds
    pub total_background_time_ms: i64,
    /// Total app launches
    pub total_launches: i32,
    /// Total unique apps used
    pub unique_apps: i32,
    /// Top apps by foreground time
    pub top_apps: Vec<AppUsageItem>,
    /// Summary period start
    pub period_start: NaiveDate,
    /// Summary period end
    pub period_end: NaiveDate,
}

/// Individual app usage item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageItem {
    /// Package name (e.g., com.example.app)
    pub package_name: String,
    /// Display name of the app
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    /// App category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Foreground time in milliseconds
    pub foreground_time_ms: i64,
    /// Background time in milliseconds
    pub background_time_ms: i64,
    /// Number of launches
    pub launch_count: i32,
    /// Notification count
    pub notification_count: i32,
}

/// App usage history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageHistoryEntry {
    /// Usage date
    pub date: NaiveDate,
    /// Package name
    pub package_name: String,
    /// Display name of the app
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    /// App category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Foreground time in milliseconds
    pub foreground_time_ms: i64,
    /// Background time in milliseconds
    pub background_time_ms: i64,
    /// Number of launches
    pub launch_count: i32,
    /// Notification count
    pub notification_count: i32,
}

/// Query parameters for app usage summary.
#[derive(Debug, Clone, Deserialize)]
pub struct AppUsageSummaryQuery {
    /// Start date for summary (defaults to 7 days ago)
    #[serde(default)]
    pub from: Option<NaiveDate>,
    /// End date for summary (defaults to today)
    #[serde(default)]
    pub to: Option<NaiveDate>,
    /// Limit number of top apps (default 10, max 50)
    #[serde(default)]
    pub top_apps_limit: Option<i32>,
}

/// Query parameters for app usage history.
#[derive(Debug, Clone, Deserialize)]
pub struct AppUsageHistoryQuery {
    /// Start date for history
    #[serde(default)]
    pub from: Option<NaiveDate>,
    /// End date for history
    #[serde(default)]
    pub to: Option<NaiveDate>,
    /// Filter by package name
    #[serde(default)]
    pub package_name: Option<String>,
    /// Filter by category
    #[serde(default)]
    pub category: Option<String>,
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    50
}

/// Response for app usage summary endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct AppUsageSummaryResponse {
    /// Device ID
    pub device_id: Uuid,
    /// Device display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    /// Total foreground time in milliseconds
    pub total_foreground_time_ms: i64,
    /// Total background time in milliseconds
    pub total_background_time_ms: i64,
    /// Total app launches
    pub total_launches: i32,
    /// Total unique apps used
    pub unique_apps: i32,
    /// Top apps by foreground time
    pub top_apps: Vec<AppUsageItem>,
    /// Summary period
    pub period: AppUsagePeriod,
}

/// App usage period.
#[derive(Debug, Clone, Serialize)]
pub struct AppUsagePeriod {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

/// Response for app usage history endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct AppUsageHistoryResponse {
    /// Device ID
    pub device_id: Uuid,
    /// Usage history entries
    pub history: Vec<AppUsageHistoryEntry>,
    /// Pagination info
    pub pagination: AppUsagePagination,
}

/// Pagination info for app usage.
#[derive(Debug, Clone, Serialize)]
pub struct AppUsagePagination {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

/// Query parameters for organization-wide app usage analytics.
#[derive(Debug, Clone, Deserialize)]
pub struct AppUsageAnalyticsQuery {
    /// Start date for analytics (defaults to 30 days ago)
    #[serde(default)]
    pub from: Option<NaiveDate>,
    /// End date for analytics (defaults to today)
    #[serde(default)]
    pub to: Option<NaiveDate>,
    /// Group by: day, week, or month
    #[serde(default)]
    pub group_by: Option<AnalyticsGroupBy>,
}

/// Grouping option for analytics.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsGroupBy {
    #[default]
    Day,
    Week,
    Month,
}

/// Organization-wide app usage analytics response.
#[derive(Debug, Clone, Serialize)]
pub struct AppUsageAnalyticsResponse {
    /// Organization ID
    pub organization_id: Uuid,
    /// Analytics period
    pub period: AppUsagePeriod,
    /// Summary metrics
    pub summary: AnalyticsSummary,
    /// Trend data over time
    pub trends: Vec<AnalyticsTrendPoint>,
    /// Top apps across all devices
    pub top_apps: Vec<TopAppItem>,
    /// Usage by category
    pub by_category: Vec<CategoryUsageItem>,
}

/// Summary metrics for analytics.
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsSummary {
    /// Total devices with usage data
    pub total_devices: i32,
    /// Total foreground time across all devices (ms)
    pub total_foreground_time_ms: i64,
    /// Total background time across all devices (ms)
    pub total_background_time_ms: i64,
    /// Total app launches
    pub total_launches: i64,
    /// Average foreground time per device (ms)
    pub avg_foreground_time_per_device_ms: i64,
    /// Total unique apps used
    pub unique_apps: i32,
}

/// Trend point for analytics time series.
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsTrendPoint {
    /// Date of the data point
    pub date: NaiveDate,
    /// Active devices count
    pub active_devices: i32,
    /// Total foreground time (ms)
    pub foreground_time_ms: i64,
    /// Total background time (ms)
    pub background_time_ms: i64,
    /// Total launches
    pub launches: i64,
}

/// Top app item for analytics.
#[derive(Debug, Clone, Serialize)]
pub struct TopAppItem {
    /// Package name
    pub package_name: String,
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    /// Category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Total foreground time across all devices (ms)
    pub total_foreground_time_ms: i64,
    /// Number of devices using this app
    pub device_count: i32,
    /// Total launches
    pub total_launches: i64,
    /// Percentage of total foreground time
    pub percentage: f64,
}

/// Category usage item for analytics.
#[derive(Debug, Clone, Serialize)]
pub struct CategoryUsageItem {
    /// Category name
    pub category: String,
    /// Total foreground time (ms)
    pub foreground_time_ms: i64,
    /// Number of apps in category
    pub app_count: i32,
    /// Percentage of total foreground time
    pub percentage: f64,
}
