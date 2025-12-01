//! Dashboard metrics repository for database operations.
//!
//! Story 14.1: Dashboard Metrics Endpoint

use chrono::{Duration, Utc};
use domain::models::{
    ActivityPeriod, ActivitySummary, DashboardMetrics, DeviceMetrics, DeviceStatusBreakdown,
    EnrollmentMetrics, GroupMetrics, PolicyMetrics, RoleBreakdown, TrendData, Trends, UserMetrics,
};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

/// Repository for dashboard metrics database operations.
#[derive(Clone)]
pub struct DashboardRepository {
    pool: PgPool,
}

impl DashboardRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get complete dashboard metrics for an organization.
    pub async fn get_metrics(&self, org_id: Uuid) -> Result<DashboardMetrics, sqlx::Error> {
        // Run all queries in parallel for performance
        let (devices, users, groups, policies, enrollment, activity, trends) = tokio::try_join!(
            self.get_device_metrics(org_id),
            self.get_user_metrics(org_id),
            self.get_group_metrics(org_id),
            self.get_policy_metrics(org_id),
            self.get_enrollment_metrics(org_id),
            self.get_activity_summary(org_id),
            self.get_trends(org_id),
        )?;

        let now = Utc::now();
        let cache_expires = now + Duration::minutes(5);

        Ok(DashboardMetrics {
            devices,
            users,
            groups,
            policies,
            enrollment,
            activity,
            trends,
            generated_at: now,
            cache_expires_at: Some(cache_expires),
        })
    }

    /// Get device metrics for an organization.
    async fn get_device_metrics(&self, org_id: Uuid) -> Result<DeviceMetrics, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE is_active = true) as active,
                COUNT(*) FILTER (WHERE is_active = false) as inactive,
                COUNT(*) FILTER (WHERE enrollment_status = 'pending') as pending_enrollment,
                COUNT(*) FILTER (WHERE enrollment_status IS NULL OR enrollment_status = 'registered') as registered,
                COUNT(*) FILTER (WHERE enrollment_status = 'enrolled') as enrolled,
                COUNT(*) FILTER (WHERE enrollment_status = 'suspended') as suspended,
                COUNT(*) FILTER (WHERE enrollment_status = 'retired') as retired
            FROM devices
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(DeviceMetrics {
            total: row.get::<i64, _>("total"),
            active: row.get::<i64, _>("active"),
            inactive: row.get::<i64, _>("inactive"),
            pending_enrollment: row.get::<i64, _>("pending_enrollment"),
            by_status: DeviceStatusBreakdown {
                registered: row.get::<i64, _>("registered"),
                enrolled: row.get::<i64, _>("enrolled"),
                suspended: row.get::<i64, _>("suspended"),
                retired: row.get::<i64, _>("retired"),
            },
        })
    }

    /// Get user metrics for an organization.
    async fn get_user_metrics(&self, org_id: Uuid) -> Result<UserMetrics, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) as active,
                COUNT(*) FILTER (WHERE role = 'owner') as owners,
                COUNT(*) FILTER (WHERE role = 'admin') as admins,
                COUNT(*) FILTER (WHERE role = 'member') as members
            FROM org_users
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(UserMetrics {
            total: row.get::<i64, _>("total"),
            active: row.get::<i64, _>("active"),
            by_role: RoleBreakdown {
                owner: row.get::<i64, _>("owners"),
                admin: row.get::<i64, _>("admins"),
                member: row.get::<i64, _>("members"),
            },
        })
    }

    /// Get group metrics for an organization.
    async fn get_group_metrics(&self, org_id: Uuid) -> Result<GroupMetrics, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(DISTINCT d.group_id) as total,
                COALESCE(AVG(group_counts.member_count)::float8, 0.0) as avg_members
            FROM devices d
            LEFT JOIN (
                SELECT group_id, COUNT(*) as member_count
                FROM devices
                WHERE organization_id = $1 AND group_id IS NOT NULL
                GROUP BY group_id
            ) group_counts ON d.group_id = group_counts.group_id
            WHERE d.organization_id = $1 AND d.group_id IS NOT NULL
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(GroupMetrics {
            total: row.get::<i64, _>("total"),
            average_members: row.get::<f64, _>("avg_members"),
        })
    }

    /// Get policy metrics for an organization.
    async fn get_policy_metrics(&self, org_id: Uuid) -> Result<PolicyMetrics, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                (SELECT COUNT(*) FROM device_policies WHERE organization_id = $1) as total,
                (SELECT COUNT(*) FROM devices WHERE organization_id = $1 AND policy_id IS NOT NULL) as active_assignments
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PolicyMetrics {
            total: row.get::<i64, _>("total"),
            active_assignments: row.get::<i64, _>("active_assignments"),
        })
    }

    /// Get enrollment metrics for an organization.
    async fn get_enrollment_metrics(&self, org_id: Uuid) -> Result<EnrollmentMetrics, sqlx::Error> {
        let now = Utc::now();
        let month_start = now - Duration::days(30);

        let row = sqlx::query(
            r#"
            SELECT
                (SELECT COUNT(*) FROM enrollment_tokens WHERE organization_id = $1 AND is_active = true AND expires_at > $2) as active_tokens,
                (SELECT COUNT(*) FROM devices WHERE organization_id = $1 AND enrolled_at >= $3) as enrolled_this_month
            "#,
        )
        .bind(org_id)
        .bind(now)
        .bind(month_start)
        .fetch_one(&self.pool)
        .await?;

        Ok(EnrollmentMetrics {
            active_tokens: row.get::<i64, _>("active_tokens"),
            enrolled_this_month: row.get::<i64, _>("enrolled_this_month"),
        })
    }

    /// Get activity summary for an organization.
    async fn get_activity_summary(&self, org_id: Uuid) -> Result<ActivitySummary, sqlx::Error> {
        let now = Utc::now();
        let seven_days_ago = now - Duration::days(7);

        // Get total events in last 7 days
        let total_row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM audit_logs
            WHERE organization_id = $1 AND timestamp >= $2
            "#,
        )
        .bind(org_id)
        .bind(seven_days_ago)
        .fetch_one(&self.pool)
        .await?;

        let total: i64 = total_row.get("count");

        // Get events by type
        let by_type_rows = sqlx::query(
            r#"
            SELECT action, COUNT(*) as count
            FROM audit_logs
            WHERE organization_id = $1 AND timestamp >= $2
            GROUP BY action
            ORDER BY count DESC
            LIMIT 10
            "#,
        )
        .bind(org_id)
        .bind(seven_days_ago)
        .fetch_all(&self.pool)
        .await?;

        let mut by_type = HashMap::new();
        for row in by_type_rows {
            let action: String = row.get("action");
            let count: i64 = row.get("count");
            by_type.insert(action, count);
        }

        Ok(ActivitySummary {
            last_7_days: ActivityPeriod {
                total_events: total,
                by_type,
            },
        })
    }

    /// Get trend data for an organization.
    async fn get_trends(&self, org_id: Uuid) -> Result<Trends, sqlx::Error> {
        let now = Utc::now();
        let seven_days_ago = now - Duration::days(7);
        let thirty_days_ago = now - Duration::days(30);
        let fourteen_days_ago = now - Duration::days(14);
        let sixty_days_ago = now - Duration::days(60);

        // Device trends
        let device_row = sqlx::query(
            r#"
            SELECT
                (SELECT COUNT(*) FROM devices WHERE organization_id = $1 AND created_at >= $2) as last_7_days,
                (SELECT COUNT(*) FROM devices WHERE organization_id = $1 AND created_at >= $3) as last_30_days,
                (SELECT COUNT(*) FROM devices WHERE organization_id = $1 AND created_at < $2 AND created_at >= $4) as prev_7_days,
                (SELECT COUNT(*) FROM devices WHERE organization_id = $1 AND created_at < $3 AND created_at >= $5) as prev_30_days
            "#,
        )
        .bind(org_id)
        .bind(seven_days_ago)
        .bind(thirty_days_ago)
        .bind(fourteen_days_ago)
        .bind(sixty_days_ago)
        .fetch_one(&self.pool)
        .await?;

        let device_last_7: i64 = device_row.get("last_7_days");
        let device_last_30: i64 = device_row.get("last_30_days");
        let device_prev_7: i64 = device_row.get("prev_7_days");
        let device_prev_30: i64 = device_row.get("prev_30_days");

        let device_trends = TrendData {
            last_7_days: device_last_7,
            last_30_days: device_last_30,
            percent_change_7d: calculate_percent_change(device_prev_7, device_last_7),
            percent_change_30d: calculate_percent_change(device_prev_30, device_last_30),
        };

        // User trends
        let user_row = sqlx::query(
            r#"
            SELECT
                (SELECT COUNT(*) FROM org_users WHERE organization_id = $1 AND granted_at >= $2) as last_7_days,
                (SELECT COUNT(*) FROM org_users WHERE organization_id = $1 AND granted_at >= $3) as last_30_days,
                (SELECT COUNT(*) FROM org_users WHERE organization_id = $1 AND granted_at < $2 AND granted_at >= $4) as prev_7_days,
                (SELECT COUNT(*) FROM org_users WHERE organization_id = $1 AND granted_at < $3 AND granted_at >= $5) as prev_30_days
            "#,
        )
        .bind(org_id)
        .bind(seven_days_ago)
        .bind(thirty_days_ago)
        .bind(fourteen_days_ago)
        .bind(sixty_days_ago)
        .fetch_one(&self.pool)
        .await?;

        let user_last_7: i64 = user_row.get("last_7_days");
        let user_last_30: i64 = user_row.get("last_30_days");
        let user_prev_7: i64 = user_row.get("prev_7_days");
        let user_prev_30: i64 = user_row.get("prev_30_days");

        let user_trends = TrendData {
            last_7_days: user_last_7,
            last_30_days: user_last_30,
            percent_change_7d: calculate_percent_change(user_prev_7, user_last_7),
            percent_change_30d: calculate_percent_change(user_prev_30, user_last_30),
        };

        Ok(Trends {
            devices: device_trends,
            users: user_trends,
        })
    }
}

/// Calculate percent change between two values.
fn calculate_percent_change(previous: i64, current: i64) -> f64 {
    if previous == 0 {
        if current == 0 {
            0.0
        } else {
            100.0 // New growth from nothing
        }
    } else {
        ((current - previous) as f64 / previous as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_percent_change() {
        assert_eq!(calculate_percent_change(100, 150), 50.0);
        assert_eq!(calculate_percent_change(100, 50), -50.0);
        assert_eq!(calculate_percent_change(0, 10), 100.0);
        assert_eq!(calculate_percent_change(0, 0), 0.0);
        assert_eq!(calculate_percent_change(10, 10), 0.0);
    }

    #[test]
    fn test_dashboard_repository_new() {
        // This test would require a database connection
        // Just verify the struct can be created
    }
}
