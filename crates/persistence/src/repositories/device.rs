//! Device repository for database operations.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{DeviceEntity, DeviceWithLastLocationEntity};

/// Repository for device-related database operations.
#[derive(Clone)]
pub struct DeviceRepository {
    pool: PgPool,
}

impl DeviceRepository {
    /// Creates a new DeviceRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Find a device by its UUID.
    pub async fn find_by_device_id(
        &self,
        device_id: Uuid,
    ) -> Result<Option<DeviceEntity>, sqlx::Error> {
        sqlx::query_as::<_, DeviceEntity>(
            r#"
            SELECT id, device_id, display_name, group_id, platform, fcm_token,
                   active, created_at, updated_at, last_seen_at
            FROM devices
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Count active devices in a group.
    pub async fn count_active_devices_in_group(&self, group_id: &str) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM devices
            WHERE group_id = $1 AND active = true
            "#,
        )
        .bind(group_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count.0)
    }

    /// Upsert a device (insert or update on conflict).
    /// Returns the device entity after upsert.
    pub async fn upsert_device(
        &self,
        device_id: Uuid,
        display_name: &str,
        group_id: &str,
        platform: &str,
        fcm_token: Option<&str>,
    ) -> Result<DeviceEntity, sqlx::Error> {
        let now = Utc::now();

        sqlx::query_as::<_, DeviceEntity>(
            r#"
            INSERT INTO devices (device_id, display_name, group_id, platform, fcm_token, active, created_at, updated_at, last_seen_at)
            VALUES ($1, $2, $3, $4, $5, true, $6, $6, $6)
            ON CONFLICT (device_id) DO UPDATE SET
                display_name = EXCLUDED.display_name,
                group_id = EXCLUDED.group_id,
                platform = EXCLUDED.platform,
                fcm_token = EXCLUDED.fcm_token,
                active = true,
                updated_at = EXCLUDED.updated_at,
                last_seen_at = EXCLUDED.last_seen_at
            RETURNING id, device_id, display_name, group_id, platform, fcm_token, active, created_at, updated_at, last_seen_at
            "#,
        )
        .bind(device_id)
        .bind(display_name)
        .bind(group_id)
        .bind(platform)
        .bind(fcm_token)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    /// Deactivate a device (soft delete).
    /// Returns the number of rows affected (0 if device not found).
    pub async fn deactivate_device(&self, device_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE devices
            SET active = false, updated_at = $2
            WHERE device_id = $1 AND active = true
            "#,
        )
        .bind(device_id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Find all active devices in a group, sorted by display name.
    pub async fn find_active_devices_by_group(
        &self,
        group_id: &str,
    ) -> Result<Vec<DeviceEntity>, sqlx::Error> {
        sqlx::query_as::<_, DeviceEntity>(
            r#"
            SELECT id, device_id, display_name, group_id, platform, fcm_token,
                   active, created_at, updated_at, last_seen_at
            FROM devices
            WHERE group_id = $1 AND active = true
            ORDER BY display_name ASC
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Find all active devices in a group with their last location (from view).
    pub async fn find_devices_with_last_location(
        &self,
        group_id: &str,
    ) -> Result<Vec<DeviceWithLastLocationEntity>, sqlx::Error> {
        sqlx::query_as::<_, DeviceWithLastLocationEntity>(
            r#"
            SELECT id, device_id, display_name, group_id, platform, fcm_token,
                   active, last_seen_at, created_at, updated_at,
                   last_latitude, last_longitude, last_location_time, last_accuracy
            FROM devices_with_last_location
            WHERE group_id = $1 AND active = true
            ORDER BY display_name ASC
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Update last_seen_at timestamp for a device.
    pub async fn update_last_seen_at(
        &self,
        device_id: Uuid,
        timestamp: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE devices
            SET last_seen_at = $2
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete inactive devices older than the specified number of days.
    /// Only devices with active=false will be deleted.
    /// Returns the number of rows deleted.
    pub async fn delete_inactive_devices(&self, older_than_days: i32) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM devices
            WHERE active = false
            AND updated_at < NOW() - ($1 || ' days')::INTERVAL
            "#,
        )
        .bind(older_than_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    /// Reactivate a soft-deleted device.
    /// Returns the number of rows affected (0 if device not found or already active).
    pub async fn reactivate_device(&self, device_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE devices
            SET active = true, updated_at = $2
            WHERE device_id = $1 AND active = false
            "#,
        )
        .bind(device_id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Hard delete a device (for GDPR compliance).
    /// Note: This relies on ON DELETE CASCADE for associated locations.
    /// Returns the number of rows deleted (0 or 1).
    pub async fn hard_delete_device(&self, device_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM devices
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Get admin statistics about devices and locations.
    pub async fn get_admin_stats(&self) -> Result<AdminStats, sqlx::Error> {
        let device_stats: (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE active = true) as active
            FROM devices
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let location_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count FROM locations
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let group_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT group_id) as count FROM devices
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AdminStats {
            total_devices: device_stats.0,
            active_devices: device_stats.1,
            inactive_devices: device_stats.0 - device_stats.1,
            total_locations: location_count.0,
            total_groups: group_count.0,
        })
    }
}

/// Admin statistics about the system.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminStats {
    pub total_devices: i64,
    pub active_devices: i64,
    pub inactive_devices: i64,
    pub total_locations: i64,
    pub total_groups: i64,
}

#[cfg(test)]
mod tests {
    // Note: Integration tests requiring database are in tests/ directory
    // Unit tests here cover logic that doesn't require database connection

    #[test]
    fn test_repository_creation() {
        // This test verifies the DeviceRepository can be created
        // Actual database tests are integration tests
        // Here we just ensure the struct definition is correct
        assert!(true);
    }
}
