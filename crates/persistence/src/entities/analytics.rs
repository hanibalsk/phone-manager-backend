//! Analytics database entities.
//!
//! AP-10: Dashboard & Analytics persistence layer

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Report job entity.
#[derive(Debug, Clone, FromRow)]
pub struct ReportJobEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub report_type: String,
    pub status: String,
    pub parameters: serde_json::Value,
    pub file_path: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub error_message: Option<String>,
    pub created_by: Uuid,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API usage daily aggregate entity.
#[derive(Debug, Clone, FromRow)]
pub struct ApiUsageDailyEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub usage_date: NaiveDate,
    pub endpoint_path: String,
    pub method: String,
    pub total_requests: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub avg_response_time_ms: Option<f64>,
    pub p95_response_time_ms: Option<i32>,
    pub total_request_bytes: Option<i64>,
    pub total_response_bytes: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User activity daily entity.
#[derive(Debug, Clone, FromRow)]
pub struct UserActivityDailyEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub activity_date: NaiveDate,
    pub active_users: i64,
    pub new_users: i64,
    pub returning_users: i64,
    pub total_sessions: i64,
    pub avg_session_duration_seconds: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Device activity daily entity.
#[derive(Debug, Clone, FromRow)]
pub struct DeviceActivityDailyEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub activity_date: NaiveDate,
    pub active_devices: i64,
    pub new_enrollments: i64,
    pub unenrollments: i64,
    pub total_locations_reported: i64,
    pub total_geofence_events: i64,
    pub total_commands_issued: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Summary entity for user analytics.
#[derive(Debug, Clone, FromRow)]
pub struct UserAnalyticsSummaryEntity {
    pub total_users: i64,
    pub active_users: i64,
    pub new_users_period: i64,
    pub total_sessions: i64,
    pub avg_session_duration: f64,
}

/// Summary entity for device analytics.
#[derive(Debug, Clone, FromRow)]
pub struct DeviceAnalyticsSummaryEntity {
    pub total_devices: i64,
    pub active_devices: i64,
    pub new_enrollments: i64,
    pub unenrollments: i64,
    pub total_locations: i64,
    pub total_geofence_events: i64,
    pub total_commands: i64,
}

/// Summary entity for API usage analytics.
#[derive(Debug, Clone, FromRow)]
pub struct ApiUsageSummaryEntity {
    pub total_requests: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: i32,
    pub total_bytes: i64,
}

/// Endpoint usage statistics entity.
#[derive(Debug, Clone, FromRow)]
pub struct EndpointUsageEntity {
    pub endpoint_path: String,
    pub method: String,
    pub total_requests: i64,
    pub success_count: i64,
    pub avg_response_time_ms: f64,
}

/// Role count entity.
#[derive(Debug, Clone, FromRow)]
pub struct RoleCountEntity {
    pub role: String,
    pub count: i64,
}

/// Device status count entity.
#[derive(Debug, Clone, FromRow)]
pub struct DeviceStatusCountEntity {
    pub status: String,
    pub count: i64,
}
