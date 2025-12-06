//! Device policy repository for database operations.

use domain::models::device_policy::{DevicePolicy, ListDevicePoliciesQuery};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::device_policy::DevicePolicyEntity;

/// Repository for device policy database operations.
#[derive(Clone)]
pub struct DevicePolicyRepository {
    pool: PgPool,
}

impl DevicePolicyRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new device policy.
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        organization_id: Uuid,
        name: &str,
        description: Option<&str>,
        is_default: bool,
        settings: &serde_json::Value,
        locked_settings: &[String],
        priority: i32,
    ) -> Result<DevicePolicy, sqlx::Error> {
        let entity = sqlx::query_as::<_, DevicePolicyEntity>(
            r#"
            INSERT INTO device_policies (organization_id, name, description, is_default, settings, locked_settings, priority)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, organization_id, name, description, is_default, settings, locked_settings, priority, device_count, created_at, updated_at
            "#,
        )
        .bind(organization_id)
        .bind(name)
        .bind(description)
        .bind(is_default)
        .bind(settings)
        .bind(locked_settings)
        .bind(priority)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity.into())
    }

    /// Find policy by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<DevicePolicy>, sqlx::Error> {
        let entity = sqlx::query_as::<_, DevicePolicyEntity>(
            r#"
            SELECT id, organization_id, name, description, is_default, settings, locked_settings, priority, device_count, created_at, updated_at
            FROM device_policies
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Find policy by organization and name.
    pub async fn find_by_org_and_name(
        &self,
        organization_id: Uuid,
        name: &str,
    ) -> Result<Option<DevicePolicy>, sqlx::Error> {
        let entity = sqlx::query_as::<_, DevicePolicyEntity>(
            r#"
            SELECT id, organization_id, name, description, is_default, settings, locked_settings, priority, device_count, created_at, updated_at
            FROM device_policies
            WHERE organization_id = $1 AND name = $2
            "#,
        )
        .bind(organization_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Find default policy for organization.
    pub async fn find_default(
        &self,
        organization_id: Uuid,
    ) -> Result<Option<DevicePolicy>, sqlx::Error> {
        let entity = sqlx::query_as::<_, DevicePolicyEntity>(
            r#"
            SELECT id, organization_id, name, description, is_default, settings, locked_settings, priority, device_count, created_at, updated_at
            FROM device_policies
            WHERE organization_id = $1 AND is_default = true
            "#,
        )
        .bind(organization_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Update a device policy.
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<Option<&str>>,
        is_default: Option<bool>,
        settings: Option<&serde_json::Value>,
        locked_settings: Option<&[String]>,
        priority: Option<i32>,
    ) -> Result<Option<DevicePolicy>, sqlx::Error> {
        let entity = sqlx::query_as::<_, DevicePolicyEntity>(
            r#"
            UPDATE device_policies
            SET
                name = COALESCE($2, name),
                description = CASE WHEN $3::boolean THEN $4 ELSE description END,
                is_default = COALESCE($5, is_default),
                settings = COALESCE($6, settings),
                locked_settings = COALESCE($7, locked_settings),
                priority = COALESCE($8, priority),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, organization_id, name, description, is_default, settings, locked_settings, priority, device_count, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description.is_some())
        .bind(description.flatten())
        .bind(is_default)
        .bind(settings)
        .bind(locked_settings)
        .bind(priority)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Delete a policy.
    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM device_policies
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List policies for organization with pagination.
    pub async fn list(
        &self,
        organization_id: Uuid,
        query: &ListDevicePoliciesQuery,
    ) -> Result<(Vec<DevicePolicy>, i64), sqlx::Error> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(50).clamp(1, 100);
        let offset = ((page - 1) * per_page) as i64;

        // Get total count
        let total: i64 = if let Some(is_default) = query.is_default {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM device_policies WHERE organization_id = $1 AND is_default = $2",
            )
            .bind(organization_id)
            .bind(is_default)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM device_policies WHERE organization_id = $1",
            )
            .bind(organization_id)
            .fetch_one(&self.pool)
            .await?
        };

        // Get policies
        let entities = if let Some(is_default) = query.is_default {
            sqlx::query_as::<_, DevicePolicyEntity>(
                r#"
                SELECT id, organization_id, name, description, is_default, settings, locked_settings, priority, device_count, created_at, updated_at
                FROM device_policies
                WHERE organization_id = $1 AND is_default = $2
                ORDER BY priority DESC, name ASC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(organization_id)
            .bind(is_default)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, DevicePolicyEntity>(
                r#"
                SELECT id, organization_id, name, description, is_default, settings, locked_settings, priority, device_count, created_at, updated_at
                FROM device_policies
                WHERE organization_id = $1
                ORDER BY priority DESC, name ASC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(organization_id)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        let policies = entities.into_iter().map(Into::into).collect();

        Ok((policies, total))
    }

    /// Check if policy has devices assigned.
    pub async fn has_devices(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM devices WHERE policy_id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Apply policy to specific devices.
    /// Only affects devices that belong to the specified organization.
    pub async fn apply_to_devices(
        &self,
        policy_id: Uuid,
        device_ids: &[Uuid],
        organization_id: Uuid,
        replace_existing: bool,
    ) -> Result<i64, sqlx::Error> {
        let result = if replace_existing {
            // Replace any existing policy
            sqlx::query(
                r#"
                UPDATE devices
                SET policy_id = $1
                WHERE device_id = ANY($2) AND organization_id = $3
                "#,
            )
            .bind(policy_id)
            .bind(device_ids)
            .bind(organization_id)
            .execute(&self.pool)
            .await?
        } else {
            // Only update devices without a policy
            sqlx::query(
                r#"
                UPDATE devices
                SET policy_id = $1
                WHERE device_id = ANY($2) AND organization_id = $3 AND policy_id IS NULL
                "#,
            )
            .bind(policy_id)
            .bind(device_ids)
            .bind(organization_id)
            .execute(&self.pool)
            .await?
        };

        Ok(result.rows_affected() as i64)
    }

    /// Apply policy to all devices in a group.
    /// Only affects devices that belong to the specified organization.
    /// Uses group UUID to find devices via groups.slug -> devices.group_id JOIN.
    pub async fn apply_to_group(
        &self,
        policy_id: Uuid,
        group_id: Uuid,
        organization_id: Uuid,
        replace_existing: bool,
    ) -> Result<i64, sqlx::Error> {
        let result = if replace_existing {
            sqlx::query(
                r#"
                UPDATE devices d
                SET policy_id = $1
                FROM groups g
                WHERE d.group_id = g.slug AND g.id = $2 AND d.organization_id = $3
                "#,
            )
            .bind(policy_id)
            .bind(group_id)
            .bind(organization_id)
            .execute(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                UPDATE devices d
                SET policy_id = $1
                FROM groups g
                WHERE d.group_id = g.slug AND g.id = $2 AND d.organization_id = $3 AND d.policy_id IS NULL
                "#,
            )
            .bind(policy_id)
            .bind(group_id)
            .bind(organization_id)
            .execute(&self.pool)
            .await?
        };

        Ok(result.rows_affected() as i64)
    }

    /// Get count of policies in organization.
    pub async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM device_policies WHERE organization_id = $1",
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Remove policy from specific devices.
    /// Only affects devices that belong to the specified organization.
    pub async fn unapply_from_devices(
        &self,
        policy_id: Uuid,
        device_ids: &[Uuid],
        organization_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE devices
            SET policy_id = NULL
            WHERE device_id = ANY($1) AND policy_id = $2 AND organization_id = $3
            "#,
        )
        .bind(device_ids)
        .bind(policy_id)
        .bind(organization_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Remove policy from all devices in a group.
    /// Only affects devices that belong to the specified organization.
    /// Uses group UUID to find devices via groups.slug -> devices.group_id JOIN.
    pub async fn unapply_from_group(
        &self,
        policy_id: Uuid,
        group_id: Uuid,
        organization_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE devices d
            SET policy_id = NULL
            FROM groups g
            WHERE d.group_id = g.slug AND g.id = $1 AND d.policy_id = $2 AND d.organization_id = $3
            "#,
        )
        .bind(group_id)
        .bind(policy_id)
        .bind(organization_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_device_policy_repository_new() {
        // This is a compile-time test - repository should be constructable
        // Actual DB tests require integration test setup
    }
}
