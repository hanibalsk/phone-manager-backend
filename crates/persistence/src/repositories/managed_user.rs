//! Managed user repository for admin user management.
//!
//! Handles queries for listing and managing users that an admin can control.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{ManagedUserEntity, UserLocationEntity};
use crate::metrics::QueryTimer;

/// Repository for managed user database operations.
#[derive(Clone)]
pub struct ManagedUserRepository {
    pool: PgPool,
}

impl ManagedUserRepository {
    /// Creates a new ManagedUserRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List users manageable by an admin.
    ///
    /// For org admins: returns users in their organizations.
    /// For non-org admins: returns users not in any organization.
    #[allow(clippy::too_many_arguments)]
    pub async fn list_managed_users(
        &self,
        org_ids: &[Uuid],
        search: Option<&str>,
        tracking_enabled: Option<bool>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ManagedUserEntity>, sqlx::Error> {
        let timer = QueryTimer::new("list_managed_users");

        let result = if org_ids.is_empty() {
            // Non-org admin: users not in any organization
            sqlx::query_as::<_, ManagedUserEntity>(
                r#"
                WITH user_locations AS (
                    SELECT DISTINCT ON (d.owner_user_id)
                        d.owner_user_id,
                        d.device_id as last_device_id,
                        d.display_name as last_device_name,
                        l.latitude as last_latitude,
                        l.longitude as last_longitude,
                        l.accuracy as last_accuracy,
                        l.captured_at as last_captured_at
                    FROM devices d
                    INNER JOIN locations l ON l.device_id = d.device_id
                    WHERE d.active = true
                    ORDER BY d.owner_user_id, l.captured_at DESC
                )
                SELECT
                    u.id,
                    u.email,
                    u.display_name,
                    u.tracking_enabled,
                    COUNT(DISTINCT d.device_id)::bigint as device_count,
                    NULL::UUID as organization_id,
                    NULL::TEXT as organization_name,
                    u.created_at,
                    ul.last_device_id,
                    ul.last_device_name,
                    ul.last_latitude,
                    ul.last_longitude,
                    ul.last_accuracy,
                    ul.last_captured_at
                FROM users u
                LEFT JOIN devices d ON d.owner_user_id = u.id AND d.active = true
                LEFT JOIN user_locations ul ON ul.owner_user_id = u.id
                WHERE u.is_active = true
                  AND NOT EXISTS (SELECT 1 FROM org_users ou WHERE ou.user_id = u.id)
                  AND ($1::TEXT IS NULL OR u.email ILIKE '%' || $1 || '%' OR u.display_name ILIKE '%' || $1 || '%')
                  AND ($2::BOOLEAN IS NULL OR u.tracking_enabled = $2)
                GROUP BY u.id, ul.last_device_id, ul.last_device_name, ul.last_latitude,
                         ul.last_longitude, ul.last_accuracy, ul.last_captured_at
                ORDER BY u.created_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(search)
            .bind(tracking_enabled)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
        } else {
            // Org admin: users in their organizations
            sqlx::query_as::<_, ManagedUserEntity>(
                r#"
                WITH user_locations AS (
                    SELECT DISTINCT ON (d.owner_user_id)
                        d.owner_user_id,
                        d.device_id as last_device_id,
                        d.display_name as last_device_name,
                        l.latitude as last_latitude,
                        l.longitude as last_longitude,
                        l.accuracy as last_accuracy,
                        l.captured_at as last_captured_at
                    FROM devices d
                    INNER JOIN locations l ON l.device_id = d.device_id
                    WHERE d.active = true
                    ORDER BY d.owner_user_id, l.captured_at DESC
                ),
                device_counts AS (
                    SELECT owner_user_id, COUNT(*)::bigint as device_count
                    FROM devices
                    WHERE active = true
                    GROUP BY owner_user_id
                )
                SELECT DISTINCT ON (u.id)
                    u.id,
                    u.email,
                    u.display_name,
                    u.tracking_enabled,
                    COALESCE(dc.device_count, 0)::bigint as device_count,
                    o.id as organization_id,
                    o.name as organization_name,
                    u.created_at,
                    ul.last_device_id,
                    ul.last_device_name,
                    ul.last_latitude,
                    ul.last_longitude,
                    ul.last_accuracy,
                    ul.last_captured_at
                FROM users u
                INNER JOIN org_users ou ON ou.user_id = u.id
                INNER JOIN organizations o ON o.id = ou.organization_id
                LEFT JOIN device_counts dc ON dc.owner_user_id = u.id
                LEFT JOIN user_locations ul ON ul.owner_user_id = u.id
                WHERE u.is_active = true
                  AND ou.organization_id = ANY($1)
                  AND ($2::TEXT IS NULL OR u.email ILIKE '%' || $2 || '%' OR u.display_name ILIKE '%' || $2 || '%')
                  AND ($3::BOOLEAN IS NULL OR u.tracking_enabled = $3)
                ORDER BY u.id, u.created_at DESC
                LIMIT $4 OFFSET $5
                "#,
            )
            .bind(org_ids)
            .bind(search)
            .bind(tracking_enabled)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
        };

        timer.record();
        result
    }

    /// Count managed users.
    pub async fn count_managed_users(
        &self,
        org_ids: &[Uuid],
        search: Option<&str>,
        tracking_enabled: Option<bool>,
    ) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_managed_users");

        let count: i64 = if org_ids.is_empty() {
            sqlx::query_scalar(
                r#"
                SELECT COUNT(*)
                FROM users u
                WHERE u.is_active = true
                  AND NOT EXISTS (SELECT 1 FROM org_users ou WHERE ou.user_id = u.id)
                  AND ($1::TEXT IS NULL OR u.email ILIKE '%' || $1 || '%' OR u.display_name ILIKE '%' || $1 || '%')
                  AND ($2::BOOLEAN IS NULL OR u.tracking_enabled = $2)
                "#,
            )
            .bind(search)
            .bind(tracking_enabled)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_scalar(
                r#"
                SELECT COUNT(DISTINCT u.id)
                FROM users u
                INNER JOIN org_users ou ON ou.user_id = u.id
                WHERE u.is_active = true
                  AND ou.organization_id = ANY($1)
                  AND ($2::TEXT IS NULL OR u.email ILIKE '%' || $2 || '%' OR u.display_name ILIKE '%' || $2 || '%')
                  AND ($3::BOOLEAN IS NULL OR u.tracking_enabled = $3)
                "#,
            )
            .bind(org_ids)
            .bind(search)
            .bind(tracking_enabled)
            .fetch_one(&self.pool)
            .await?
        };

        timer.record();
        Ok(count)
    }

    /// Check if an admin can manage a specific user.
    ///
    /// Org admins can manage users in their organizations.
    /// Non-org admins can manage users not in any organization.
    pub async fn can_manage_user(
        &self,
        target_user_id: Uuid,
        org_ids: &[Uuid],
    ) -> Result<bool, sqlx::Error> {
        let timer = QueryTimer::new("can_manage_user");

        let exists: bool = if org_ids.is_empty() {
            // Non-org admin can manage users not in any org
            sqlx::query_scalar(
                r#"
                SELECT EXISTS(
                    SELECT 1 FROM users u
                    WHERE u.id = $1 AND u.is_active = true
                      AND NOT EXISTS (SELECT 1 FROM org_users ou WHERE ou.user_id = u.id)
                )
                "#,
            )
            .bind(target_user_id)
            .fetch_one(&self.pool)
            .await?
        } else {
            // Org admin can manage users in their orgs
            sqlx::query_scalar(
                r#"
                SELECT EXISTS(
                    SELECT 1 FROM users u
                    INNER JOIN org_users ou ON ou.user_id = u.id
                    WHERE u.id = $1 AND u.is_active = true
                      AND ou.organization_id = ANY($2)
                )
                "#,
            )
            .bind(target_user_id)
            .bind(org_ids)
            .fetch_one(&self.pool)
            .await?
        };

        timer.record();
        Ok(exists)
    }

    /// Update user tracking status.
    pub async fn update_tracking(&self, user_id: Uuid, enabled: bool) -> Result<bool, sqlx::Error> {
        let timer = QueryTimer::new("update_user_tracking");

        let result =
            sqlx::query("UPDATE users SET tracking_enabled = $2, updated_at = NOW() WHERE id = $1")
                .bind(user_id)
                .bind(enabled)
                .execute(&self.pool)
                .await?;

        timer.record();
        Ok(result.rows_affected() > 0)
    }

    /// Get user's latest location across all their devices.
    pub async fn get_user_location(
        &self,
        user_id: Uuid,
    ) -> Result<Option<UserLocationEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_user_location");

        let result = sqlx::query_as::<_, UserLocationEntity>(
            r#"
            SELECT
                d.device_id,
                d.display_name as device_name,
                l.latitude,
                l.longitude,
                l.accuracy,
                l.captured_at
            FROM devices d
            INNER JOIN locations l ON l.device_id = d.device_id
            WHERE d.owner_user_id = $1 AND d.active = true
            ORDER BY l.captured_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Deactivate a user account.
    pub async fn deactivate_user(&self, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let timer = QueryTimer::new("deactivate_user");

        let result = sqlx::query(
            "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1 AND is_active = true",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        timer.record();
        Ok(result.rows_affected() > 0)
    }

    /// Get organizations where a user has admin/owner role.
    pub async fn get_admin_org_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
        let timer = QueryTimer::new("get_admin_org_ids");

        let org_ids: Vec<Uuid> = sqlx::query_scalar(
            r#"
            SELECT organization_id FROM org_users
            WHERE user_id = $1 AND role IN ('owner', 'admin')
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        timer.record();
        Ok(org_ids)
    }

    /// Remove user from specified organizations.
    pub async fn remove_from_organizations(
        &self,
        user_id: Uuid,
        org_ids: &[Uuid],
    ) -> Result<u64, sqlx::Error> {
        let timer = QueryTimer::new("remove_from_organizations");

        let result =
            sqlx::query("DELETE FROM org_users WHERE user_id = $1 AND organization_id = ANY($2)")
                .bind(user_id)
                .bind(org_ids)
                .execute(&self.pool)
                .await?;

        timer.record();
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repository_creation() {
        // This test verifies the ManagedUserRepository can be created
        // Actual database tests are integration tests
    }
}
