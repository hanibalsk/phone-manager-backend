//! App usage repository.
//!
//! AP-8.1-8.2, AP-8.7: App usage summary, history, and analytics

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{
    AnalyticsTrendEntity, AppUsageEntity, AppUsageSummaryEntity, CategoryUsageEntity,
    OrgAnalyticsSummaryEntity, TopAppEntity,
};

/// Repository for app usage data.
#[derive(Debug, Clone)]
pub struct AppUsageRepository {
    pool: PgPool,
}

impl AppUsageRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get app usage summary for a device.
    pub async fn get_device_summary(
        &self,
        org_id: Uuid,
        device_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<AppUsageSummaryEntity, sqlx::Error> {
        let result = sqlx::query_as::<_, AppUsageSummaryEntity>(
            r#"
            SELECT
                device_id,
                COALESCE(SUM(foreground_time_ms), 0) as total_foreground_time_ms,
                COALESCE(SUM(background_time_ms), 0) as total_background_time_ms,
                COALESCE(SUM(launch_count), 0) as total_launches,
                COUNT(DISTINCT package_name) as unique_apps
            FROM app_usage
            WHERE organization_id = $1
              AND device_id = $2
              AND usage_date >= $3
              AND usage_date <= $4
            GROUP BY device_id
            "#,
        )
        .bind(org_id)
        .bind(device_id)
        .bind(from)
        .bind(to)
        .fetch_optional(&self.pool)
        .await?;

        // Return empty summary if no data
        Ok(result.unwrap_or(AppUsageSummaryEntity {
            device_id,
            total_foreground_time_ms: 0,
            total_background_time_ms: 0,
            total_launches: 0,
            unique_apps: 0,
        }))
    }

    /// Get top apps for a device.
    pub async fn get_top_apps(
        &self,
        org_id: Uuid,
        device_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        limit: i32,
    ) -> Result<Vec<TopAppEntity>, sqlx::Error> {
        let apps = sqlx::query_as::<_, TopAppEntity>(
            r#"
            SELECT
                package_name,
                MAX(app_name) as app_name,
                MAX(category) as category,
                SUM(foreground_time_ms) as foreground_time_ms,
                SUM(background_time_ms) as background_time_ms,
                SUM(launch_count) as launch_count,
                SUM(notification_count) as notification_count
            FROM app_usage
            WHERE organization_id = $1
              AND device_id = $2
              AND usage_date >= $3
              AND usage_date <= $4
            GROUP BY package_name
            ORDER BY foreground_time_ms DESC
            LIMIT $5
            "#,
        )
        .bind(org_id)
        .bind(device_id)
        .bind(from)
        .bind(to)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(apps)
    }

    /// Get app usage history for a device.
    #[allow(clippy::too_many_arguments)]
    pub async fn get_device_history(
        &self,
        org_id: Uuid,
        device_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        package_name: Option<&str>,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AppUsageEntity>, i64), sqlx::Error> {
        // Build count query
        let mut count_sql = String::from(
            r#"
            SELECT COUNT(*) as count
            FROM app_usage
            WHERE organization_id = $1
              AND device_id = $2
            "#,
        );

        let mut param_idx = 3;
        if from.is_some() {
            count_sql.push_str(&format!(" AND usage_date >= ${}", param_idx));
            param_idx += 1;
        }
        if to.is_some() {
            count_sql.push_str(&format!(" AND usage_date <= ${}", param_idx));
            param_idx += 1;
        }
        if package_name.is_some() {
            count_sql.push_str(&format!(" AND package_name = ${}", param_idx));
            param_idx += 1;
        }
        if category.is_some() {
            count_sql.push_str(&format!(" AND category = ${}", param_idx));
        }

        // Build main query
        let mut sql = String::from(
            r#"
            SELECT
                id, organization_id, device_id, package_name, app_name, category,
                foreground_time_ms, background_time_ms, launch_count, notification_count,
                usage_date, created_at, updated_at
            FROM app_usage
            WHERE organization_id = $1
              AND device_id = $2
            "#,
        );

        param_idx = 3;
        if from.is_some() {
            sql.push_str(&format!(" AND usage_date >= ${}", param_idx));
            param_idx += 1;
        }
        if to.is_some() {
            sql.push_str(&format!(" AND usage_date <= ${}", param_idx));
            param_idx += 1;
        }
        if package_name.is_some() {
            sql.push_str(&format!(" AND package_name = ${}", param_idx));
            param_idx += 1;
        }
        if category.is_some() {
            sql.push_str(&format!(" AND category = ${}", param_idx));
            param_idx += 1;
        }

        sql.push_str(&format!(
            " ORDER BY usage_date DESC, foreground_time_ms DESC LIMIT ${} OFFSET ${}",
            param_idx,
            param_idx + 1
        ));

        // Execute count query
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql)
            .bind(org_id)
            .bind(device_id);
        if let Some(f) = from {
            count_query = count_query.bind(f);
        }
        if let Some(t) = to {
            count_query = count_query.bind(t);
        }
        if let Some(p) = package_name {
            count_query = count_query.bind(p);
        }
        if let Some(c) = category {
            count_query = count_query.bind(c);
        }
        let total = count_query.fetch_one(&self.pool).await?;

        // Execute main query
        let mut main_query = sqlx::query_as::<_, AppUsageEntity>(&sql)
            .bind(org_id)
            .bind(device_id);
        if let Some(f) = from {
            main_query = main_query.bind(f);
        }
        if let Some(t) = to {
            main_query = main_query.bind(t);
        }
        if let Some(p) = package_name {
            main_query = main_query.bind(p);
        }
        if let Some(c) = category {
            main_query = main_query.bind(c);
        }
        main_query = main_query.bind(limit).bind(offset);

        let records = main_query.fetch_all(&self.pool).await?;

        Ok((records, total))
    }

    /// Get organization-wide analytics summary.
    pub async fn get_org_analytics_summary(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<OrgAnalyticsSummaryEntity, sqlx::Error> {
        let result = sqlx::query_as::<_, OrgAnalyticsSummaryEntity>(
            r#"
            SELECT
                COUNT(DISTINCT device_id) as total_devices,
                COALESCE(SUM(foreground_time_ms), 0) as total_foreground_time_ms,
                COALESCE(SUM(background_time_ms), 0) as total_background_time_ms,
                COALESCE(SUM(launch_count), 0) as total_launches,
                COUNT(DISTINCT package_name) as unique_apps
            FROM app_usage
            WHERE organization_id = $1
              AND usage_date >= $2
              AND usage_date <= $3
            "#,
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.unwrap_or(OrgAnalyticsSummaryEntity {
            total_devices: 0,
            total_foreground_time_ms: 0,
            total_background_time_ms: 0,
            total_launches: 0,
            unique_apps: 0,
        }))
    }

    /// Get daily trend data for organization.
    pub async fn get_org_daily_trends(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<AnalyticsTrendEntity>, sqlx::Error> {
        let trends = sqlx::query_as::<_, AnalyticsTrendEntity>(
            r#"
            SELECT
                usage_date as date,
                COUNT(DISTINCT device_id) as active_devices,
                COALESCE(SUM(foreground_time_ms), 0) as foreground_time_ms,
                COALESCE(SUM(background_time_ms), 0) as background_time_ms,
                COALESCE(SUM(launch_count), 0) as launches
            FROM app_usage
            WHERE organization_id = $1
              AND usage_date >= $2
              AND usage_date <= $3
            GROUP BY usage_date
            ORDER BY usage_date ASC
            "#,
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(trends)
    }

    /// Get top apps across organization.
    pub async fn get_org_top_apps(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        limit: i32,
    ) -> Result<Vec<TopAppEntity>, sqlx::Error> {
        let apps = sqlx::query_as::<_, TopAppEntity>(
            r#"
            SELECT
                package_name,
                MAX(app_name) as app_name,
                MAX(category) as category,
                SUM(foreground_time_ms) as foreground_time_ms,
                SUM(background_time_ms) as background_time_ms,
                SUM(launch_count) as launch_count,
                SUM(notification_count) as notification_count
            FROM app_usage
            WHERE organization_id = $1
              AND usage_date >= $2
              AND usage_date <= $3
            GROUP BY package_name
            ORDER BY foreground_time_ms DESC
            LIMIT $4
            "#,
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(apps)
    }

    /// Get device count per app across organization.
    pub async fn get_app_device_counts(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        package_names: &[String],
    ) -> Result<Vec<(String, i64)>, sqlx::Error> {
        if package_names.is_empty() {
            return Ok(vec![]);
        }

        let counts = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT
                package_name,
                COUNT(DISTINCT device_id) as device_count
            FROM app_usage
            WHERE organization_id = $1
              AND usage_date >= $2
              AND usage_date <= $3
              AND package_name = ANY($4)
            GROUP BY package_name
            "#,
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .bind(package_names)
        .fetch_all(&self.pool)
        .await?;

        Ok(counts)
    }

    /// Get usage by category for organization.
    pub async fn get_org_category_usage(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<CategoryUsageEntity>, sqlx::Error> {
        let categories = sqlx::query_as::<_, CategoryUsageEntity>(
            r#"
            SELECT
                category,
                SUM(foreground_time_ms) as foreground_time_ms,
                COUNT(DISTINCT package_name) as app_count
            FROM app_usage
            WHERE organization_id = $1
              AND usage_date >= $2
              AND usage_date <= $3
            GROUP BY category
            ORDER BY foreground_time_ms DESC
            "#,
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repository_creation() {
        // This is a compile-time check that the repository can be created
        // Actual database tests would require a test database
    }
}
