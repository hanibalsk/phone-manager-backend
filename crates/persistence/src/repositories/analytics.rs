//! Analytics repository.
//!
//! AP-10: Dashboard & Analytics persistence operations

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{
    ApiUsageDailyEntity, ApiUsageSummaryEntity, DeviceActivityDailyEntity,
    DeviceAnalyticsSummaryEntity, DeviceStatusCountEntity, EndpointUsageEntity, ReportJobEntity,
    RoleCountEntity, UserActivityDailyEntity, UserAnalyticsSummaryEntity,
};

/// Repository for analytics operations.
#[derive(Clone)]
pub struct AnalyticsRepository {
    pool: PgPool,
}

impl AnalyticsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // User Analytics
    // ========================================================================

    /// Get user analytics summary for a period.
    pub async fn get_user_analytics_summary(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<UserAnalyticsSummaryEntity, sqlx::Error> {
        sqlx::query_as::<_, UserAnalyticsSummaryEntity>(
            r#"
            SELECT
                (SELECT COUNT(*) FROM org_users WHERE organization_id = $1) as total_users,
                COALESCE(SUM(active_users), 0)::bigint as active_users,
                COALESCE(SUM(new_users), 0)::bigint as new_users_period,
                COALESCE(SUM(total_sessions), 0)::bigint as total_sessions,
                COALESCE(AVG(avg_session_duration_seconds), 0)::float8 as avg_session_duration
            FROM user_activity_daily
            WHERE organization_id = $1
              AND activity_date >= $2
              AND activity_date <= $3
            "#,
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await
    }

    /// Get user activity trends for a period.
    pub async fn get_user_activity_trends(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<UserActivityDailyEntity>, sqlx::Error> {
        sqlx::query_as::<_, UserActivityDailyEntity>(
            r#"
            SELECT id, organization_id, activity_date, active_users, new_users, returning_users,
                   total_sessions, avg_session_duration_seconds, created_at, updated_at
            FROM user_activity_daily
            WHERE organization_id = $1
              AND activity_date >= $2
              AND activity_date <= $3
            ORDER BY activity_date ASC
            "#,
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
    }

    /// Get user role breakdown for organization.
    pub async fn get_user_role_breakdown(
        &self,
        org_id: Uuid,
    ) -> Result<Vec<RoleCountEntity>, sqlx::Error> {
        sqlx::query_as::<_, RoleCountEntity>(
            r#"
            SELECT role::text as role, COUNT(*)::bigint as count
            FROM org_users
            WHERE organization_id = $1
            GROUP BY role
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
    }

    // ========================================================================
    // Device Analytics
    // ========================================================================

    /// Get device analytics summary for a period.
    pub async fn get_device_analytics_summary(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<DeviceAnalyticsSummaryEntity, sqlx::Error> {
        sqlx::query_as::<_, DeviceAnalyticsSummaryEntity>(
            r#"
            SELECT
                (SELECT COUNT(*) FROM devices d
                 JOIN device_enrollments de ON d.id = de.device_id
                 WHERE de.organization_id = $1) as total_devices,
                COALESCE(SUM(active_devices), 0)::bigint as active_devices,
                COALESCE(SUM(new_enrollments), 0)::bigint as new_enrollments,
                COALESCE(SUM(unenrollments), 0)::bigint as unenrollments,
                COALESCE(SUM(total_locations_reported), 0)::bigint as total_locations,
                COALESCE(SUM(total_geofence_events), 0)::bigint as total_geofence_events,
                COALESCE(SUM(total_commands_issued), 0)::bigint as total_commands
            FROM device_activity_daily
            WHERE organization_id = $1
              AND activity_date >= $2
              AND activity_date <= $3
            "#,
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await
    }

    /// Get device activity trends for a period.
    pub async fn get_device_activity_trends(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<DeviceActivityDailyEntity>, sqlx::Error> {
        sqlx::query_as::<_, DeviceActivityDailyEntity>(
            r#"
            SELECT id, organization_id, activity_date, active_devices, new_enrollments,
                   unenrollments, total_locations_reported, total_geofence_events,
                   total_commands_issued, created_at, updated_at
            FROM device_activity_daily
            WHERE organization_id = $1
              AND activity_date >= $2
              AND activity_date <= $3
            ORDER BY activity_date ASC
            "#,
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
    }

    /// Get device status breakdown for organization.
    pub async fn get_device_status_breakdown(
        &self,
        org_id: Uuid,
    ) -> Result<Vec<DeviceStatusCountEntity>, sqlx::Error> {
        sqlx::query_as::<_, DeviceStatusCountEntity>(
            r#"
            SELECT enrollment_status::text as status, COUNT(*)::bigint as count
            FROM device_enrollments
            WHERE organization_id = $1
            GROUP BY enrollment_status
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
    }

    // ========================================================================
    // API Usage Analytics
    // ========================================================================

    /// Get API usage summary for a period.
    pub async fn get_api_usage_summary(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<ApiUsageSummaryEntity, sqlx::Error> {
        sqlx::query_as::<_, ApiUsageSummaryEntity>(
            r#"
            SELECT
                COALESCE(SUM(total_requests), 0)::bigint as total_requests,
                COALESCE(SUM(success_count), 0)::bigint as success_count,
                COALESCE(SUM(error_count), 0)::bigint as error_count,
                COALESCE(AVG(avg_response_time_ms), 0)::float8 as avg_response_time_ms,
                COALESCE(MAX(p95_response_time_ms), 0)::int4 as p95_response_time_ms,
                COALESCE(SUM(total_request_bytes) + SUM(total_response_bytes), 0)::bigint as total_bytes
            FROM api_usage_daily
            WHERE organization_id = $1
              AND usage_date >= $2
              AND usage_date <= $3
            "#,
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await
    }

    /// Get API usage trends for a period.
    pub async fn get_api_usage_trends(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<ApiUsageDailyEntity>, sqlx::Error> {
        sqlx::query_as::<_, ApiUsageDailyEntity>(
            r#"
            SELECT id, organization_id, usage_date, endpoint_path, method,
                   total_requests, success_count, error_count, avg_response_time_ms,
                   p95_response_time_ms, total_request_bytes, total_response_bytes,
                   created_at, updated_at
            FROM api_usage_daily
            WHERE organization_id = $1
              AND usage_date >= $2
              AND usage_date <= $3
            ORDER BY usage_date ASC
            "#,
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
    }

    /// Get top endpoints by request count.
    pub async fn get_top_endpoints(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        limit: i32,
    ) -> Result<Vec<EndpointUsageEntity>, sqlx::Error> {
        sqlx::query_as::<_, EndpointUsageEntity>(
            r#"
            SELECT
                endpoint_path,
                method,
                SUM(total_requests)::bigint as total_requests,
                SUM(success_count)::bigint as success_count,
                AVG(avg_response_time_ms)::float8 as avg_response_time_ms
            FROM api_usage_daily
            WHERE organization_id = $1
              AND usage_date >= $2
              AND usage_date <= $3
            GROUP BY endpoint_path, method
            ORDER BY total_requests DESC
            LIMIT $4
            "#,
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // ========================================================================
    // Report Jobs
    // ========================================================================

    /// Create a new report job.
    pub async fn create_report_job(
        &self,
        org_id: Uuid,
        report_type: &str,
        parameters: serde_json::Value,
        created_by: Uuid,
    ) -> Result<ReportJobEntity, sqlx::Error> {
        sqlx::query_as::<_, ReportJobEntity>(
            r#"
            INSERT INTO report_jobs (organization_id, report_type, parameters, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING id, organization_id, report_type, status, parameters, file_path,
                      file_size_bytes, error_message, created_by, started_at, completed_at,
                      expires_at, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(report_type)
        .bind(parameters)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
    }

    /// Get a report job by ID.
    pub async fn get_report_job(
        &self,
        org_id: Uuid,
        job_id: Uuid,
    ) -> Result<Option<ReportJobEntity>, sqlx::Error> {
        sqlx::query_as::<_, ReportJobEntity>(
            r#"
            SELECT id, organization_id, report_type, status, parameters, file_path,
                   file_size_bytes, error_message, created_by, started_at, completed_at,
                   expires_at, created_at, updated_at
            FROM report_jobs
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(job_id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Update report job status.
    pub async fn update_report_job_status(
        &self,
        job_id: Uuid,
        status: &str,
        file_path: Option<&str>,
        file_size_bytes: Option<i64>,
        error_message: Option<&str>,
    ) -> Result<ReportJobEntity, sqlx::Error> {
        let now = chrono::Utc::now();
        let (started_at, completed_at) = match status {
            "processing" => (Some(now), None),
            "completed" | "failed" => (None, Some(now)),
            _ => (None, None),
        };

        sqlx::query_as::<_, ReportJobEntity>(
            r#"
            UPDATE report_jobs
            SET status = $2,
                file_path = COALESCE($3, file_path),
                file_size_bytes = COALESCE($4, file_size_bytes),
                error_message = $5,
                started_at = COALESCE($6, started_at),
                completed_at = COALESCE($7, completed_at),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, organization_id, report_type, status, parameters, file_path,
                      file_size_bytes, error_message, created_by, started_at, completed_at,
                      expires_at, created_at, updated_at
            "#,
        )
        .bind(job_id)
        .bind(status)
        .bind(file_path)
        .bind(file_size_bytes)
        .bind(error_message)
        .bind(started_at)
        .bind(completed_at)
        .fetch_one(&self.pool)
        .await
    }

    /// List report jobs for organization.
    pub async fn list_report_jobs(
        &self,
        org_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ReportJobEntity>, sqlx::Error> {
        sqlx::query_as::<_, ReportJobEntity>(
            r#"
            SELECT id, organization_id, report_type, status, parameters, file_path,
                   file_size_bytes, error_message, created_by, started_at, completed_at,
                   expires_at, created_at, updated_at
            FROM report_jobs
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(org_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }
}
