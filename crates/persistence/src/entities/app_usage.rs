//! App usage database entities.
//!
//! AP-8.1-8.2, AP-8.7: App usage summary, history, and analytics

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// App usage record entity.
#[derive(Debug, Clone, FromRow)]
pub struct AppUsageEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub device_id: Uuid,
    pub package_name: String,
    pub app_name: Option<String>,
    pub category: Option<String>,
    pub foreground_time_ms: i64,
    pub background_time_ms: i64,
    pub launch_count: i32,
    pub notification_count: i32,
    pub usage_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// App usage summary entity for aggregated queries.
#[derive(Debug, Clone, FromRow)]
pub struct AppUsageSummaryEntity {
    pub device_id: Uuid,
    pub total_foreground_time_ms: i64,
    pub total_background_time_ms: i64,
    pub total_launches: i64,
    pub unique_apps: i64,
}

/// Top app entity for rankings.
#[derive(Debug, Clone, FromRow)]
pub struct TopAppEntity {
    pub package_name: String,
    pub app_name: Option<String>,
    pub category: Option<String>,
    pub foreground_time_ms: i64,
    pub background_time_ms: i64,
    pub launch_count: i64,
    pub notification_count: i64,
}

/// App usage daily aggregate entity.
#[derive(Debug, Clone, FromRow)]
pub struct AppUsageDailyAggregateEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub usage_date: NaiveDate,
    pub total_devices: i32,
    pub total_foreground_time_ms: i64,
    pub total_background_time_ms: i64,
    pub total_launches: i32,
    pub unique_apps: i32,
    pub top_apps_by_time: Option<serde_json::Value>,
    pub top_categories: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Analytics trend entity for time-series data.
#[derive(Debug, Clone, FromRow)]
pub struct AnalyticsTrendEntity {
    pub date: NaiveDate,
    pub active_devices: i64,
    pub foreground_time_ms: i64,
    pub background_time_ms: i64,
    pub launches: i64,
}

/// Category usage entity for category breakdown.
#[derive(Debug, Clone, FromRow)]
pub struct CategoryUsageEntity {
    pub category: Option<String>,
    pub foreground_time_ms: i64,
    pub app_count: i64,
}

/// Organization-wide analytics summary entity.
#[derive(Debug, Clone, FromRow)]
pub struct OrgAnalyticsSummaryEntity {
    pub total_devices: i64,
    pub total_foreground_time_ms: i64,
    pub total_background_time_ms: i64,
    pub total_launches: i64,
    pub unique_apps: i64,
}
