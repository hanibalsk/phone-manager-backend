//! Dashboard metrics domain models.
//!
//! Story 14.1: Dashboard Metrics Endpoint

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Device counts by status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceMetrics {
    pub total: i64,
    pub active: i64,
    pub inactive: i64,
    pub pending_enrollment: i64,
    pub by_status: DeviceStatusBreakdown,
}

/// Breakdown of devices by enrollment status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceStatusBreakdown {
    pub registered: i64,
    pub enrolled: i64,
    pub suspended: i64,
    pub retired: i64,
}

/// User counts by role.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserMetrics {
    pub total: i64,
    pub active: i64,
    pub by_role: RoleBreakdown,
}

/// Breakdown of users by role.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RoleBreakdown {
    pub owner: i64,
    pub admin: i64,
    pub member: i64,
}

/// Group metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GroupMetrics {
    pub total: i64,
    pub average_members: f64,
}

/// Policy metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PolicyMetrics {
    pub total: i64,
    pub active_assignments: i64,
}

/// Enrollment metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EnrollmentMetrics {
    pub active_tokens: i64,
    pub enrolled_this_month: i64,
}

/// Activity summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ActivitySummary {
    pub last_7_days: ActivityPeriod,
}

/// Activity for a time period.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ActivityPeriod {
    pub total_events: i64,
    pub by_type: HashMap<String, i64>,
}

/// Trend data for a metric.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrendData {
    pub last_7_days: i64,
    pub last_30_days: i64,
    pub percent_change_7d: f64,
    pub percent_change_30d: f64,
}

/// All trends.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Trends {
    pub devices: TrendData,
    pub users: TrendData,
}

/// Complete dashboard metrics response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DashboardMetrics {
    pub devices: DeviceMetrics,
    pub users: UserMetrics,
    pub groups: GroupMetrics,
    pub policies: PolicyMetrics,
    pub enrollment: EnrollmentMetrics,
    pub activity: ActivitySummary,
    pub trends: Trends,
    pub generated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_expires_at: Option<DateTime<Utc>>,
}

impl DashboardMetrics {
    /// Create a new DashboardMetrics with the current timestamp.
    pub fn new() -> Self {
        Self {
            generated_at: Utc::now(),
            ..Default::default()
        }
    }

    /// Set cache expiration time.
    pub fn with_cache_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.cache_expires_at = Some(expires_at);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_metrics_new() {
        let metrics = DashboardMetrics::new();
        assert!(metrics.generated_at <= Utc::now());
        assert!(metrics.cache_expires_at.is_none());
    }

    #[test]
    fn test_dashboard_metrics_with_cache_expiry() {
        let expires = Utc::now() + chrono::Duration::minutes(5);
        let metrics = DashboardMetrics::new().with_cache_expiry(expires);
        assert_eq!(metrics.cache_expires_at, Some(expires));
    }

    #[test]
    fn test_device_metrics_default() {
        let metrics = DeviceMetrics::default();
        assert_eq!(metrics.total, 0);
        assert_eq!(metrics.active, 0);
        assert_eq!(metrics.inactive, 0);
        assert_eq!(metrics.pending_enrollment, 0);
    }

    #[test]
    fn test_user_metrics_default() {
        let metrics = UserMetrics::default();
        assert_eq!(metrics.total, 0);
        assert_eq!(metrics.active, 0);
    }

    #[test]
    fn test_trend_data_default() {
        let trend = TrendData::default();
        assert_eq!(trend.last_7_days, 0);
        assert_eq!(trend.last_30_days, 0);
        assert_eq!(trend.percent_change_7d, 0.0);
        assert_eq!(trend.percent_change_30d, 0.0);
    }

    #[test]
    fn test_dashboard_metrics_serialization() {
        let metrics = DashboardMetrics::new();
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("devices"));
        assert!(json.contains("users"));
        assert!(json.contains("groups"));
        assert!(json.contains("policies"));
        assert!(json.contains("enrollment"));
        assert!(json.contains("activity"));
        assert!(json.contains("trends"));
        assert!(json.contains("generated_at"));
    }
}
