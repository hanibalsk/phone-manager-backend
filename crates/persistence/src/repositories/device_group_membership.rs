//! Device-group membership repository for database operations.
//!
//! Story UGM-3.1: Device-Group Membership Table

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::device_group_membership::{
    DeviceGroupInfoEntity, DeviceGroupMembershipEntity, DeviceInGroupEntity,
    DeviceInGroupWithLocationEntity,
};
use crate::metrics::QueryTimer;

/// Repository for device-group membership database operations.
#[derive(Clone)]
pub struct DeviceGroupMembershipRepository {
    pool: PgPool,
}

impl DeviceGroupMembershipRepository {
    /// Creates a new DeviceGroupMembershipRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Add a device to a group.
    pub async fn add_device_to_group(
        &self,
        device_id: Uuid,
        group_id: Uuid,
        added_by: Uuid,
    ) -> Result<DeviceGroupMembershipEntity, sqlx::Error> {
        let timer = QueryTimer::new("add_device_to_group");
        let result = sqlx::query_as::<_, DeviceGroupMembershipEntity>(
            r#"
            INSERT INTO device_group_memberships (device_id, group_id, added_by)
            VALUES ($1, $2, $3)
            RETURNING id, device_id, group_id, added_by, added_at
            "#,
        )
        .bind(device_id)
        .bind(group_id)
        .bind(added_by)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Remove a device from a group.
    pub async fn remove_device_from_group(
        &self,
        device_id: Uuid,
        group_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let timer = QueryTimer::new("remove_device_from_group");
        let result = sqlx::query(
            r#"
            DELETE FROM device_group_memberships
            WHERE device_id = $1 AND group_id = $2
            "#,
        )
        .bind(device_id)
        .bind(group_id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(result.rows_affected())
    }

    /// Check if a device is in a group.
    pub async fn is_device_in_group(
        &self,
        device_id: Uuid,
        group_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let timer = QueryTimer::new("is_device_in_group");
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM device_group_memberships
                WHERE device_id = $1 AND group_id = $2
            )
            "#,
        )
        .bind(device_id)
        .bind(group_id)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Get a device's membership in a group.
    pub async fn get_membership(
        &self,
        device_id: Uuid,
        group_id: Uuid,
    ) -> Result<Option<DeviceGroupMembershipEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_device_group_membership");
        let result = sqlx::query_as::<_, DeviceGroupMembershipEntity>(
            r#"
            SELECT id, device_id, group_id, added_by, added_at
            FROM device_group_memberships
            WHERE device_id = $1 AND group_id = $2
            "#,
        )
        .bind(device_id)
        .bind(group_id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// List all devices in a group (without location).
    pub async fn list_devices_in_group(
        &self,
        group_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DeviceInGroupEntity>, sqlx::Error> {
        let timer = QueryTimer::new("list_devices_in_group");
        let result = sqlx::query_as::<_, DeviceInGroupEntity>(
            r#"
            SELECT
                d.device_id,
                d.display_name,
                d.last_seen_at,
                d.owner_user_id,
                u.display_name as owner_display_name,
                dgm.id as membership_id,
                dgm.added_by,
                dgm.added_at
            FROM device_group_memberships dgm
            JOIN devices d ON d.device_id = dgm.device_id AND d.active = true
            LEFT JOIN users u ON d.owner_user_id = u.id
            WHERE dgm.group_id = $1
            ORDER BY dgm.added_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(group_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// List all devices in a group with their last location.
    pub async fn list_devices_in_group_with_location(
        &self,
        group_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DeviceInGroupWithLocationEntity>, sqlx::Error> {
        let timer = QueryTimer::new("list_devices_in_group_with_location");
        let result = sqlx::query_as::<_, DeviceInGroupWithLocationEntity>(
            r#"
            SELECT
                d.device_id,
                d.display_name,
                d.last_seen_at,
                d.owner_user_id,
                u.display_name as owner_display_name,
                dgm.id as membership_id,
                dgm.added_by,
                dgm.added_at,
                ll.latitude,
                ll.longitude,
                ll.accuracy,
                ll.recorded_at as location_timestamp
            FROM device_group_memberships dgm
            JOIN devices d ON d.device_id = dgm.device_id AND d.active = true
            LEFT JOIN users u ON d.owner_user_id = u.id
            LEFT JOIN LATERAL (
                SELECT latitude, longitude, accuracy, recorded_at
                FROM locations
                WHERE device_id = d.device_id
                ORDER BY recorded_at DESC
                LIMIT 1
            ) ll ON true
            WHERE dgm.group_id = $1
            ORDER BY dgm.added_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(group_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Count devices in a group.
    pub async fn count_devices_in_group(&self, group_id: Uuid) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_devices_in_group");
        let result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM device_group_memberships dgm
            JOIN devices d ON d.device_id = dgm.device_id AND d.active = true
            WHERE dgm.group_id = $1
            "#,
        )
        .bind(group_id)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// List all groups a device belongs to.
    pub async fn list_device_groups(
        &self,
        device_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<DeviceGroupInfoEntity>, sqlx::Error> {
        let timer = QueryTimer::new("list_device_groups");
        let result = sqlx::query_as::<_, DeviceGroupInfoEntity>(
            r#"
            SELECT
                g.id as group_id,
                g.name as group_name,
                g.slug as group_slug,
                gm.role::text as user_role,
                dgm.id as membership_id,
                dgm.added_at
            FROM device_group_memberships dgm
            JOIN groups g ON g.id = dgm.group_id AND g.is_active = true
            JOIN group_memberships gm ON gm.group_id = g.id AND gm.user_id = $2
            WHERE dgm.device_id = $1
            ORDER BY dgm.added_at DESC
            "#,
        )
        .bind(device_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Count device's memberships in groups.
    pub async fn count_device_groups(&self, device_id: Uuid) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_device_groups");
        let result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM device_group_memberships dgm
            JOIN groups g ON g.id = dgm.group_id AND g.is_active = true
            WHERE dgm.device_id = $1
            "#,
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Count devices per user in a group (for member list with device counts).
    pub async fn count_devices_per_user_in_group(
        &self,
        group_id: Uuid,
    ) -> Result<Vec<(Uuid, i64)>, sqlx::Error> {
        let timer = QueryTimer::new("count_devices_per_user_in_group");

        let rows = sqlx::query_as::<_, (Uuid, i64)>(
            r#"
            SELECT d.owner_user_id, COUNT(*) as device_count
            FROM device_group_memberships dgm
            JOIN devices d ON d.device_id = dgm.device_id AND d.active = true
            WHERE dgm.group_id = $1 AND d.owner_user_id IS NOT NULL
            GROUP BY d.owner_user_id
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await;

        timer.record();
        rows
    }
}

#[cfg(test)]
mod tests {
    // Note: DeviceGroupMembershipRepository tests require database connection
    // and are covered by integration tests
}
