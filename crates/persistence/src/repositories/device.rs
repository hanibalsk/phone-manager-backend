//! Device repository for database operations.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{DeviceEntity, DeviceWithLastLocationEntity, FleetDeviceEntity};
use crate::metrics::QueryTimer;
use domain::models::{
    AssignedUserInfo, FleetDeviceItem, FleetGroupInfo, FleetLastLocation, FleetPolicyInfo,
    FleetSortField, SortOrder,
};

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
        let timer = QueryTimer::new("find_device_by_id");
        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            SELECT id, device_id, display_name, group_id, platform, fcm_token,
                   active, created_at, updated_at, last_seen_at,
                   owner_user_id, organization_id, is_primary, linked_at
            FROM devices
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Count active devices in a group.
    pub async fn count_active_devices_in_group(&self, group_id: &str) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_active_devices_in_group");
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
        timer.record();
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
        let timer = QueryTimer::new("upsert_device");

        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            INSERT INTO devices (device_id, display_name, group_id, platform, fcm_token, active, created_at, updated_at, last_seen_at, is_primary)
            VALUES ($1, $2, $3, $4, $5, true, $6, $6, $6, false)
            ON CONFLICT (device_id) DO UPDATE SET
                display_name = EXCLUDED.display_name,
                group_id = EXCLUDED.group_id,
                platform = EXCLUDED.platform,
                fcm_token = EXCLUDED.fcm_token,
                active = true,
                updated_at = EXCLUDED.updated_at,
                last_seen_at = EXCLUDED.last_seen_at
            RETURNING id, device_id, display_name, group_id, platform, fcm_token, active, created_at, updated_at, last_seen_at,
                      owner_user_id, organization_id, is_primary, linked_at
            "#,
        )
        .bind(device_id)
        .bind(display_name)
        .bind(group_id)
        .bind(platform)
        .bind(fcm_token)
        .bind(now)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Deactivate a device (soft delete).
    /// Returns the number of rows affected (0 if device not found).
    pub async fn deactivate_device(&self, device_id: Uuid) -> Result<u64, sqlx::Error> {
        let timer = QueryTimer::new("deactivate_device");
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
        timer.record();
        Ok(result.rows_affected())
    }

    /// Find all active devices in a group, sorted by display name.
    pub async fn find_active_devices_by_group(
        &self,
        group_id: &str,
    ) -> Result<Vec<DeviceEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_active_devices_by_group");
        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            SELECT id, device_id, display_name, group_id, platform, fcm_token,
                   active, created_at, updated_at, last_seen_at,
                   owner_user_id, organization_id, is_primary, linked_at
            FROM devices
            WHERE group_id = $1 AND active = true
            ORDER BY display_name ASC
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find all active devices in a group with their last location (from view).
    pub async fn find_devices_with_last_location(
        &self,
        group_id: &str,
    ) -> Result<Vec<DeviceWithLastLocationEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_devices_with_last_location");
        let result = sqlx::query_as::<_, DeviceWithLastLocationEntity>(
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
        .await;
        timer.record();
        result
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

    /// Link a device to a user.
    /// Returns the updated device entity.
    pub async fn link_device_to_user(
        &self,
        device_id: Uuid,
        user_id: Uuid,
        display_name: Option<&str>,
        is_primary: bool,
    ) -> Result<DeviceEntity, sqlx::Error> {
        let now = Utc::now();
        let timer = QueryTimer::new("link_device_to_user");

        // If setting as primary, first clear other primary devices for this user
        if is_primary {
            sqlx::query(
                r#"
                UPDATE devices
                SET is_primary = false, updated_at = $2
                WHERE owner_user_id = $1 AND is_primary = true
                "#,
            )
            .bind(user_id)
            .bind(now)
            .execute(&self.pool)
            .await?;
        }

        // Now link the device
        let result = if let Some(name) = display_name {
            sqlx::query_as::<_, DeviceEntity>(
                r#"
                UPDATE devices
                SET owner_user_id = $2,
                    linked_at = $3,
                    display_name = $4,
                    is_primary = $5,
                    updated_at = $3
                WHERE device_id = $1
                RETURNING id, device_id, display_name, group_id, platform, fcm_token,
                          active, created_at, updated_at, last_seen_at,
                          owner_user_id, organization_id, is_primary, linked_at
                "#,
            )
            .bind(device_id)
            .bind(user_id)
            .bind(now)
            .bind(name)
            .bind(is_primary)
            .fetch_one(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, DeviceEntity>(
                r#"
                UPDATE devices
                SET owner_user_id = $2,
                    linked_at = $3,
                    is_primary = $4,
                    updated_at = $3
                WHERE device_id = $1
                RETURNING id, device_id, display_name, group_id, platform, fcm_token,
                          active, created_at, updated_at, last_seen_at,
                          owner_user_id, organization_id, is_primary, linked_at
                "#,
            )
            .bind(device_id)
            .bind(user_id)
            .bind(now)
            .bind(is_primary)
            .fetch_one(&self.pool)
            .await
        };

        timer.record();
        result
    }

    /// Find all devices owned by a user.
    /// Returns devices ordered by: primary first, then by linked_at descending.
    pub async fn find_devices_by_user(
        &self,
        user_id: Uuid,
        include_inactive: bool,
    ) -> Result<Vec<DeviceEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_devices_by_user");
        let result = if include_inactive {
            sqlx::query_as::<_, DeviceEntity>(
                r#"
                SELECT id, device_id, display_name, group_id, platform, fcm_token,
                       active, created_at, updated_at, last_seen_at,
                       owner_user_id, organization_id, is_primary, linked_at
                FROM devices
                WHERE owner_user_id = $1
                ORDER BY is_primary DESC, linked_at DESC NULLS LAST
                "#,
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, DeviceEntity>(
                r#"
                SELECT id, device_id, display_name, group_id, platform, fcm_token,
                       active, created_at, updated_at, last_seen_at,
                       owner_user_id, organization_id, is_primary, linked_at
                FROM devices
                WHERE owner_user_id = $1 AND active = true
                ORDER BY is_primary DESC, linked_at DESC NULLS LAST
                "#,
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
        };
        timer.record();
        result
    }

    /// Unlink a device from its owner.
    pub async fn unlink_device(&self, device_id: Uuid) -> Result<u64, sqlx::Error> {
        let now = Utc::now();
        let timer = QueryTimer::new("unlink_device");
        let result = sqlx::query(
            r#"
            UPDATE devices
            SET owner_user_id = NULL,
                linked_at = NULL,
                is_primary = false,
                updated_at = $2
            WHERE device_id = $1 AND owner_user_id IS NOT NULL
            "#,
        )
        .bind(device_id)
        .bind(now)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(result.rows_affected())
    }

    /// Transfer device ownership to a new user.
    pub async fn transfer_device_ownership(
        &self,
        device_id: Uuid,
        new_owner_id: Uuid,
    ) -> Result<DeviceEntity, sqlx::Error> {
        let now = Utc::now();
        let timer = QueryTimer::new("transfer_device_ownership");
        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            UPDATE devices
            SET owner_user_id = $2,
                linked_at = $3,
                is_primary = false,
                updated_at = $3
            WHERE device_id = $1
            RETURNING id, device_id, display_name, group_id, platform, fcm_token,
                      active, created_at, updated_at, last_seen_at,
                      owner_user_id, organization_id, is_primary, linked_at
            "#,
        )
        .bind(device_id)
        .bind(new_owner_id)
        .bind(now)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Create a new managed device (for enrollment).
    #[allow(clippy::too_many_arguments)]
    pub async fn create_managed_device(
        &self,
        device_id: Uuid,
        display_name: &str,
        group_id: &str,
        platform: &str,
        fcm_token: Option<&str>,
        organization_id: Uuid,
        policy_id: Option<Uuid>,
        enrollment_token_id: Uuid,
    ) -> Result<DeviceEntity, sqlx::Error> {
        let now = Utc::now();
        let timer = QueryTimer::new("create_managed_device");

        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            INSERT INTO devices (
                device_id, display_name, group_id, platform, fcm_token,
                active, created_at, updated_at, last_seen_at, is_primary,
                organization_id, is_managed, enrollment_status, policy_id,
                enrolled_at, enrolled_via_token_id
            )
            VALUES ($1, $2, $3, $4, $5, true, $6, $6, $6, false, $7, true, 'enrolled', $8, $6, $9)
            ON CONFLICT (device_id) DO UPDATE SET
                display_name = EXCLUDED.display_name,
                group_id = EXCLUDED.group_id,
                platform = EXCLUDED.platform,
                fcm_token = EXCLUDED.fcm_token,
                active = true,
                updated_at = EXCLUDED.updated_at,
                last_seen_at = EXCLUDED.last_seen_at,
                organization_id = EXCLUDED.organization_id,
                is_managed = true,
                enrollment_status = 'enrolled',
                policy_id = EXCLUDED.policy_id,
                enrolled_at = COALESCE(devices.enrolled_at, EXCLUDED.enrolled_at),
                enrolled_via_token_id = EXCLUDED.enrolled_via_token_id
            RETURNING id, device_id, display_name, group_id, platform, fcm_token,
                      active, created_at, updated_at, last_seen_at,
                      owner_user_id, organization_id, is_primary, linked_at
            "#,
        )
        .bind(device_id)
        .bind(display_name)
        .bind(group_id)
        .bind(platform)
        .bind(fcm_token)
        .bind(now)
        .bind(organization_id)
        .bind(policy_id)
        .bind(enrollment_token_id)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Update device enrollment fields.
    pub async fn update_enrollment(
        &self,
        id: i64,
        organization_id: Uuid,
        group_id: Option<&str>,
        policy_id: Option<Uuid>,
        enrollment_status: &str,
        enrollment_token_id: Option<Uuid>,
    ) -> Result<DeviceEntity, sqlx::Error> {
        let now = Utc::now();
        let timer = QueryTimer::new("update_enrollment");

        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            UPDATE devices
            SET organization_id = $2,
                group_id = COALESCE($3, group_id),
                policy_id = $4,
                is_managed = true,
                enrollment_status = $5::enrollment_status,
                enrolled_at = COALESCE(enrolled_at, $6),
                enrolled_via_token_id = COALESCE($7, enrolled_via_token_id),
                updated_at = $6
            WHERE id = $1
            RETURNING id, device_id, display_name, group_id, platform, fcm_token,
                      active, created_at, updated_at, last_seen_at,
                      owner_user_id, organization_id, is_primary, linked_at
            "#,
        )
        .bind(id)
        .bind(organization_id)
        .bind(group_id)
        .bind(policy_id)
        .bind(enrollment_status)
        .bind(now)
        .bind(enrollment_token_id)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Assign a user to a managed device.
    pub async fn assign_user(
        &self,
        device_id: i64,
        user_id: Uuid,
    ) -> Result<DeviceEntity, sqlx::Error> {
        let now = Utc::now();
        let timer = QueryTimer::new("assign_user_to_device");
        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            UPDATE devices
            SET assigned_user_id = $2,
                updated_at = $3
            WHERE id = $1
            RETURNING id, device_id, display_name, group_id, platform, fcm_token,
                      active, created_at, updated_at, last_seen_at,
                      owner_user_id, organization_id, is_primary, linked_at
            "#,
        )
        .bind(device_id)
        .bind(user_id)
        .bind(now)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Unassign user from a managed device.
    pub async fn unassign_user(&self, device_id: i64) -> Result<DeviceEntity, sqlx::Error> {
        let now = Utc::now();
        let timer = QueryTimer::new("unassign_user_from_device");
        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            UPDATE devices
            SET assigned_user_id = NULL,
                updated_at = $2
            WHERE id = $1
            RETURNING id, device_id, display_name, group_id, platform, fcm_token,
                      active, created_at, updated_at, last_seen_at,
                      owner_user_id, organization_id, is_primary, linked_at
            "#,
        )
        .bind(device_id)
        .bind(now)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Update device enrollment status.
    pub async fn update_enrollment_status(
        &self,
        device_id: i64,
        new_status: &str,
    ) -> Result<DeviceEntity, sqlx::Error> {
        let now = Utc::now();
        let timer = QueryTimer::new("update_enrollment_status");
        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            UPDATE devices
            SET enrollment_status = $2::enrollment_status,
                updated_at = $3
            WHERE id = $1
            RETURNING id, device_id, display_name, group_id, platform, fcm_token,
                      active, created_at, updated_at, last_seen_at,
                      owner_user_id, organization_id, is_primary, linked_at
            "#,
        )
        .bind(device_id)
        .bind(new_status)
        .bind(now)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Get device enrollment status.
    pub async fn get_enrollment_status(
        &self,
        device_id: i64,
    ) -> Result<Option<String>, sqlx::Error> {
        let result: Option<(Option<String>,)> = sqlx::query_as(
            r#"
            SELECT enrollment_status::TEXT
            FROM devices
            WHERE id = $1
            "#,
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.and_then(|r| r.0))
    }

    /// Get fleet summary counts for an organization.
    pub async fn get_fleet_summary(
        &self,
        organization_id: Uuid,
    ) -> Result<FleetSummaryCounts, sqlx::Error> {
        let result: FleetSummaryCounts = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE enrollment_status = 'enrolled') as enrolled,
                COUNT(*) FILTER (WHERE enrollment_status = 'pending') as pending,
                COUNT(*) FILTER (WHERE enrollment_status = 'suspended') as suspended,
                COUNT(*) FILTER (WHERE enrollment_status = 'retired') as retired,
                COUNT(*) FILTER (WHERE assigned_user_id IS NOT NULL) as assigned,
                COUNT(*) FILTER (WHERE assigned_user_id IS NULL AND is_managed = true) as unassigned
            FROM devices
            WHERE organization_id = $1 AND is_managed = true
            "#,
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Count devices in organization matching filters.
    pub async fn count_fleet_devices(
        &self,
        organization_id: Uuid,
        status_filter: Option<&str>,
        group_id_filter: Option<&str>,
        policy_id_filter: Option<Uuid>,
        assigned_filter: Option<bool>,
        search_filter: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT COUNT(*)
            FROM devices
            WHERE organization_id = $1 AND is_managed = true
            "#,
        );

        let mut param_idx = 2;

        if status_filter.is_some() {
            query.push_str(&format!(
                " AND enrollment_status = ${}::enrollment_status",
                param_idx
            ));
            param_idx += 1;
        }
        if group_id_filter.is_some() {
            query.push_str(&format!(" AND group_id = ${}", param_idx));
            param_idx += 1;
        }
        if policy_id_filter.is_some() {
            query.push_str(&format!(" AND policy_id = ${}", param_idx));
            param_idx += 1;
        }
        if let Some(assigned) = assigned_filter {
            if assigned {
                query.push_str(" AND assigned_user_id IS NOT NULL");
            } else {
                query.push_str(" AND assigned_user_id IS NULL");
            }
        }
        if search_filter.is_some() {
            query.push_str(&format!(
                " AND (display_name ILIKE ${} OR device_id::TEXT ILIKE ${})",
                param_idx, param_idx
            ));
        }

        let mut q = sqlx::query_as::<_, (i64,)>(&query).bind(organization_id);

        if let Some(status) = status_filter {
            q = q.bind(status);
        }
        if let Some(group_id) = group_id_filter {
            q = q.bind(group_id);
        }
        if let Some(policy_id) = policy_id_filter {
            q = q.bind(policy_id);
        }
        if let Some(search) = search_filter {
            q = q.bind(format!("%{}%", search));
        }

        let result = q.fetch_one(&self.pool).await?;
        Ok(result.0)
    }

    /// List fleet devices with filtering, sorting, and pagination.
    ///
    /// Returns devices with joined user, policy, and location data.
    #[allow(clippy::too_many_arguments)]
    pub async fn list_fleet_devices(
        &self,
        organization_id: Uuid,
        status_filter: Option<&str>,
        group_id_filter: Option<&str>,
        policy_id_filter: Option<Uuid>,
        assigned_filter: Option<bool>,
        search_filter: Option<&str>,
        sort_field: FleetSortField,
        sort_order: SortOrder,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<FleetDeviceItem>, sqlx::Error> {
        // Build the base SELECT with LEFT JOINs
        let mut query = String::from(
            r#"
            SELECT
                d.id,
                d.device_id,
                d.display_name,
                d.platform,
                d.is_managed,
                d.enrollment_status::TEXT as enrollment_status,
                d.enrolled_at,
                d.created_at,
                d.last_seen_at,
                d.group_id,
                u.id as assigned_user_id,
                u.email as assigned_user_email,
                u.display_name as assigned_user_display_name,
                p.id as policy_id,
                p.name as policy_name,
                ll.latitude as last_latitude,
                ll.longitude as last_longitude,
                ll.timestamp as last_location_time
            FROM devices d
            LEFT JOIN users u ON d.assigned_user_id = u.id
            LEFT JOIN device_policies p ON d.policy_id = p.id
            LEFT JOIN LATERAL (
                SELECT latitude, longitude, timestamp
                FROM locations
                WHERE device_id = d.id
                ORDER BY timestamp DESC
                LIMIT 1
            ) ll ON true
            WHERE d.organization_id = $1 AND d.is_managed = true
            "#,
        );

        let mut param_idx = 2;

        // Add filters
        if status_filter.is_some() {
            query.push_str(&format!(
                " AND d.enrollment_status = ${}::enrollment_status",
                param_idx
            ));
            param_idx += 1;
        }
        if group_id_filter.is_some() {
            query.push_str(&format!(" AND d.group_id = ${}", param_idx));
            param_idx += 1;
        }
        if policy_id_filter.is_some() {
            query.push_str(&format!(" AND d.policy_id = ${}", param_idx));
            param_idx += 1;
        }
        if let Some(assigned) = assigned_filter {
            if assigned {
                query.push_str(" AND d.assigned_user_id IS NOT NULL");
            } else {
                query.push_str(" AND d.assigned_user_id IS NULL");
            }
        }
        if search_filter.is_some() {
            query.push_str(&format!(
                " AND (d.display_name ILIKE ${} OR d.device_id::TEXT ILIKE ${})",
                param_idx, param_idx
            ));
            param_idx += 1;
        }

        // Add sorting
        let sort_column = match sort_field {
            FleetSortField::LastSeenAt => "d.last_seen_at",
            FleetSortField::DisplayName => "d.display_name",
            FleetSortField::CreatedAt => "d.created_at",
            FleetSortField::EnrolledAt => "d.enrolled_at",
        };
        let order_str = sort_order.as_str();
        query.push_str(&format!(
            " ORDER BY {} {} NULLS LAST",
            sort_column, order_str
        ));

        // Add pagination
        query.push_str(&format!(" LIMIT ${} OFFSET ${}", param_idx, param_idx + 1));

        // Build and execute query
        let mut q = sqlx::query_as::<_, FleetDeviceEntity>(&query).bind(organization_id);

        if let Some(status) = status_filter {
            q = q.bind(status);
        }
        if let Some(group_id) = group_id_filter {
            q = q.bind(group_id);
        }
        if let Some(policy_id) = policy_id_filter {
            q = q.bind(policy_id);
        }
        if let Some(search) = search_filter {
            q = q.bind(format!("%{}%", search));
        }

        q = q.bind(limit as i32).bind(offset as i32);

        let entities = q.fetch_all(&self.pool).await?;

        // Map entities to domain models
        let items = entities
            .into_iter()
            .map(|e| {
                let assigned_user = e.assigned_user_id.map(|id| AssignedUserInfo {
                    id,
                    email: e.assigned_user_email.unwrap_or_default(),
                    display_name: e.assigned_user_display_name,
                });

                let group = if !e.group_id.is_empty() {
                    Some(FleetGroupInfo {
                        id: e.group_id.clone(),
                        name: None, // Group name not available without another join
                    })
                } else {
                    None
                };

                let policy = e.policy_id.map(|id| FleetPolicyInfo {
                    id,
                    name: e.policy_name.unwrap_or_default(),
                });

                let last_location = if let (Some(lat), Some(lon), Some(ts)) =
                    (e.last_latitude, e.last_longitude, e.last_location_time)
                {
                    Some(FleetLastLocation {
                        latitude: lat,
                        longitude: lon,
                        timestamp: ts,
                    })
                } else {
                    None
                };

                let enrollment_status = e.enrollment_status.as_deref().and_then(|s| s.parse().ok());

                FleetDeviceItem {
                    id: e.id,
                    device_uuid: e.device_id,
                    display_name: e.display_name,
                    platform: e.platform,
                    enrollment_status,
                    is_managed: e.is_managed,
                    assigned_user,
                    group,
                    policy,
                    last_seen_at: e.last_seen_at,
                    last_location,
                    enrolled_at: e.enrolled_at,
                    created_at: e.created_at,
                }
            })
            .collect();

        Ok(items)
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

    /// Find a device by external_id within an organization.
    pub async fn find_by_external_id(
        &self,
        organization_id: Uuid,
        external_id: &str,
    ) -> Result<Option<DeviceEntity>, sqlx::Error> {
        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            SELECT id, device_id, display_name, group_id, platform, fcm_token,
                   active, created_at, updated_at, last_seen_at,
                   owner_user_id, organization_id, is_primary, linked_at
            FROM devices
            WHERE organization_id = $1 AND external_id = $2
            "#,
        )
        .bind(organization_id)
        .bind(external_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Create a device via bulk import.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_bulk_device(
        &self,
        organization_id: Uuid,
        external_id: Option<&str>,
        display_name: &str,
        group_id: Option<&str>,
        policy_id: Option<Uuid>,
        assigned_user_id: Option<Uuid>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<DeviceEntity, sqlx::Error> {
        let device_uuid = Uuid::new_v4();
        let now = Utc::now();

        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            INSERT INTO devices (
                device_id, display_name, group_id, platform, active,
                organization_id, is_managed, enrollment_status, external_id,
                policy_id, assigned_user_id, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, 'unknown', false, $4, true, 'pending', $5, $6, $7, $8, $9, $9)
            RETURNING id, device_id, display_name, group_id, platform, fcm_token,
                      active, created_at, updated_at, last_seen_at,
                      owner_user_id, organization_id, is_primary, linked_at
            "#,
        )
        .bind(device_uuid)
        .bind(display_name)
        .bind(group_id)
        .bind(organization_id)
        .bind(external_id)
        .bind(policy_id)
        .bind(assigned_user_id)
        .bind(metadata)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Update an existing device via bulk import.
    pub async fn update_bulk_device(
        &self,
        device_id: i64,
        display_name: &str,
        group_id: Option<&str>,
        policy_id: Option<Uuid>,
        assigned_user_id: Option<Uuid>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<DeviceEntity, sqlx::Error> {
        let now = Utc::now();

        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            UPDATE devices
            SET display_name = $2,
                group_id = COALESCE($3, group_id),
                policy_id = COALESCE($4, policy_id),
                assigned_user_id = COALESCE($5, assigned_user_id),
                metadata = COALESCE($6, metadata),
                updated_at = $7
            WHERE id = $1
            RETURNING id, device_id, display_name, group_id, platform, fcm_token,
                      active, created_at, updated_at, last_seen_at,
                      owner_user_id, organization_id, is_primary, linked_at
            "#,
        )
        .bind(device_id)
        .bind(display_name)
        .bind(group_id)
        .bind(policy_id)
        .bind(assigned_user_id)
        .bind(metadata)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Bulk update a device (for fleet management).
    /// Returns the list of fields that were updated.
    #[allow(clippy::too_many_arguments)]
    pub async fn bulk_update_device(
        &self,
        device_id: i64,
        organization_id: Uuid,
        display_name: Option<&str>,
        group_id: Option<&str>,
        policy_id: Option<Uuid>,
        assigned_user_id: Option<Uuid>,
        clear_assigned_user: bool,
    ) -> Result<(DeviceEntity, Vec<String>), sqlx::Error> {
        let now = Utc::now();
        let mut updated_fields = Vec::new();

        // Build dynamic update query based on provided fields
        let mut set_clauses = vec!["updated_at = $3".to_string()];
        let mut param_idx = 4;

        if display_name.is_some() {
            set_clauses.push(format!("display_name = ${}", param_idx));
            param_idx += 1;
            updated_fields.push("display_name".to_string());
        }

        if group_id.is_some() {
            set_clauses.push(format!("group_id = ${}", param_idx));
            param_idx += 1;
            updated_fields.push("group_id".to_string());
        }

        if policy_id.is_some() {
            set_clauses.push(format!("policy_id = ${}", param_idx));
            param_idx += 1;
            updated_fields.push("policy_id".to_string());
        }

        if clear_assigned_user {
            set_clauses.push("assigned_user_id = NULL".to_string());
            updated_fields.push("assigned_user_id".to_string());
        } else if assigned_user_id.is_some() {
            set_clauses.push(format!("assigned_user_id = ${}", param_idx));
            updated_fields.push("assigned_user_id".to_string());
        }

        let query = format!(
            r#"
            UPDATE devices
            SET {}
            WHERE id = $1 AND organization_id = $2 AND is_managed = true
            RETURNING id, device_id, display_name, group_id, platform, fcm_token,
                      active, created_at, updated_at, last_seen_at,
                      owner_user_id, organization_id, is_primary, linked_at
            "#,
            set_clauses.join(", ")
        );

        let mut q = sqlx::query_as::<_, DeviceEntity>(&query)
            .bind(device_id)
            .bind(organization_id)
            .bind(now);

        if let Some(name) = display_name {
            q = q.bind(name);
        }

        if let Some(gid) = group_id {
            q = q.bind(gid);
        }

        if let Some(pid) = policy_id {
            q = q.bind(pid);
        }

        if !clear_assigned_user {
            if let Some(uid) = assigned_user_id {
                q = q.bind(uid);
            }
        }

        let result = q.fetch_one(&self.pool).await?;
        Ok((result, updated_fields))
    }

    /// Check if a device exists and belongs to the organization.
    pub async fn device_exists_in_org(
        &self,
        device_id: i64,
        organization_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT id
            FROM devices
            WHERE id = $1 AND organization_id = $2 AND is_managed = true
            "#,
        )
        .bind(device_id)
        .bind(organization_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.is_some())
    }

    /// List all managed devices in an organization (simple query for admin operations).
    ///
    /// Returns basic device info for all managed devices in the organization.
    pub async fn list_org_managed_devices(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<DeviceEntity>, sqlx::Error> {
        let timer = QueryTimer::new("list_org_managed_devices");

        let result = sqlx::query_as::<_, DeviceEntity>(
            r#"
            SELECT id, device_id, display_name, group_id, platform, fcm_token,
                   active, created_at, updated_at, last_seen_at,
                   owner_user_id, organization_id, is_primary, linked_at
            FROM devices
            WHERE organization_id = $1 AND is_managed = true
            ORDER BY display_name ASC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await;

        timer.record();
        result
    }
}

/// Admin statistics about the system.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminStats {
    pub total_devices: i64,
    pub active_devices: i64,
    pub inactive_devices: i64,
    pub total_locations: i64,
    pub total_groups: i64,
}

/// Fleet summary counts for an organization.
#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
#[serde(rename_all = "snake_case")]
pub struct FleetSummaryCounts {
    pub enrolled: i64,
    pub pending: i64,
    pub suspended: i64,
    pub retired: i64,
    pub assigned: i64,
    pub unassigned: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================
    // AdminStats Tests
    // ===========================================

    #[test]
    fn test_admin_stats_creation() {
        let stats = AdminStats {
            total_devices: 100,
            active_devices: 75,
            inactive_devices: 25,
            total_locations: 10000,
            total_groups: 10,
        };

        assert_eq!(stats.total_devices, 100);
        assert_eq!(stats.active_devices, 75);
        assert_eq!(stats.inactive_devices, 25);
        assert_eq!(stats.total_locations, 10000);
        assert_eq!(stats.total_groups, 10);
    }

    #[test]
    fn test_admin_stats_consistency() {
        // Inactive = Total - Active
        let stats = AdminStats {
            total_devices: 100,
            active_devices: 75,
            inactive_devices: 25,
            total_locations: 10000,
            total_groups: 10,
        };

        assert_eq!(
            stats.total_devices,
            stats.active_devices + stats.inactive_devices
        );
    }

    #[test]
    fn test_admin_stats_zero_values() {
        let stats = AdminStats {
            total_devices: 0,
            active_devices: 0,
            inactive_devices: 0,
            total_locations: 0,
            total_groups: 0,
        };

        assert_eq!(stats.total_devices, 0);
        assert_eq!(stats.active_devices, 0);
        assert_eq!(stats.inactive_devices, 0);
        assert_eq!(stats.total_locations, 0);
        assert_eq!(stats.total_groups, 0);
    }

    #[test]
    fn test_admin_stats_large_values() {
        let stats = AdminStats {
            total_devices: i64::MAX,
            active_devices: i64::MAX - 1,
            inactive_devices: 1,
            total_locations: i64::MAX,
            total_groups: 1000000,
        };

        assert_eq!(stats.total_devices, i64::MAX);
        assert_eq!(stats.total_locations, i64::MAX);
    }

    #[test]
    fn test_admin_stats_debug() {
        let stats = AdminStats {
            total_devices: 100,
            active_devices: 75,
            inactive_devices: 25,
            total_locations: 10000,
            total_groups: 10,
        };

        let debug = format!("{:?}", stats);
        assert!(debug.contains("AdminStats"));
        assert!(debug.contains("total_devices"));
        assert!(debug.contains("active_devices"));
        assert!(debug.contains("inactive_devices"));
        assert!(debug.contains("total_locations"));
        assert!(debug.contains("total_groups"));
    }

    #[test]
    fn test_admin_stats_clone() {
        let stats = AdminStats {
            total_devices: 100,
            active_devices: 75,
            inactive_devices: 25,
            total_locations: 10000,
            total_groups: 10,
        };

        let cloned = stats.clone();
        assert_eq!(cloned.total_devices, stats.total_devices);
        assert_eq!(cloned.active_devices, stats.active_devices);
        assert_eq!(cloned.inactive_devices, stats.inactive_devices);
        assert_eq!(cloned.total_locations, stats.total_locations);
        assert_eq!(cloned.total_groups, stats.total_groups);
    }

    #[test]
    fn test_admin_stats_serialization() {
        let stats = AdminStats {
            total_devices: 100,
            active_devices: 75,
            inactive_devices: 25,
            total_locations: 10000,
            total_groups: 10,
        };

        let json = serde_json::to_string(&stats).unwrap();

        // Should use snake_case
        assert!(json.contains("\"total_devices\":100"));
        assert!(json.contains("\"active_devices\":75"));
        assert!(json.contains("\"inactive_devices\":25"));
        assert!(json.contains("\"total_locations\":10000"));
        assert!(json.contains("\"total_groups\":10"));
    }

    #[test]
    fn test_admin_stats_serialization_zero() {
        let stats = AdminStats {
            total_devices: 0,
            active_devices: 0,
            inactive_devices: 0,
            total_locations: 0,
            total_groups: 0,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_devices\":0"));
        assert!(json.contains("\"total_groups\":0"));
    }

    // ===========================================
    // DeviceRepository Struct Tests
    // ===========================================

    // Note: Actual database operations are tested in integration tests.
    // These unit tests verify struct creation and basic properties.

    #[test]
    fn test_device_repository_clone() {
        // DeviceRepository derives Clone - verify the derive works
        // We can't actually create one without a pool, but we can verify
        // the Clone trait is implemented
        fn assert_clone<T: Clone>() {}
        assert_clone::<DeviceRepository>();
    }

    // ===========================================
    // Edge Case Tests
    // ===========================================

    #[test]
    fn test_admin_stats_all_inactive() {
        let stats = AdminStats {
            total_devices: 50,
            active_devices: 0,
            inactive_devices: 50,
            total_locations: 5000,
            total_groups: 5,
        };

        assert_eq!(stats.active_devices, 0);
        assert_eq!(stats.inactive_devices, stats.total_devices);
    }

    #[test]
    fn test_admin_stats_all_active() {
        let stats = AdminStats {
            total_devices: 50,
            active_devices: 50,
            inactive_devices: 0,
            total_locations: 5000,
            total_groups: 5,
        };

        assert_eq!(stats.inactive_devices, 0);
        assert_eq!(stats.active_devices, stats.total_devices);
    }

    #[test]
    fn test_admin_stats_single_group() {
        let stats = AdminStats {
            total_devices: 20,
            active_devices: 20,
            inactive_devices: 0,
            total_locations: 1000,
            total_groups: 1,
        };

        assert_eq!(stats.total_groups, 1);
    }

    #[test]
    fn test_admin_stats_many_groups() {
        let stats = AdminStats {
            total_devices: 1000,
            active_devices: 800,
            inactive_devices: 200,
            total_locations: 100000,
            total_groups: 500,
        };

        assert_eq!(stats.total_groups, 500);
    }

    #[test]
    fn test_admin_stats_negative_values() {
        // While unlikely in practice, i64 can hold negative values
        let stats = AdminStats {
            total_devices: -1, // Invalid but structurally possible
            active_devices: 0,
            inactive_devices: -1,
            total_locations: 0,
            total_groups: 0,
        };

        assert_eq!(stats.total_devices, -1);
        assert_eq!(stats.inactive_devices, -1);
    }
}
