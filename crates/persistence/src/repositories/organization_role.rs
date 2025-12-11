//! Organization role repository for custom RBAC.
//!
//! Story AP-1.2: Create Custom Role
//! Story AP-1.3: Delete Custom Role

use anyhow::Result;
use domain::models::OrganizationRole;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for organization role operations.
pub struct OrganizationRoleRepository {
    pool: PgPool,
}

impl OrganizationRoleRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize system roles for an organization.
    /// This should be called when a new organization is created.
    pub async fn init_system_roles(&self, org_id: Uuid, creator_id: Option<Uuid>) -> Result<()> {
        sqlx::query("SELECT init_organization_system_roles($1, $2)")
            .bind(org_id)
            .bind(creator_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// List all roles for an organization.
    pub async fn list(&self, org_id: Uuid) -> Result<Vec<OrganizationRole>> {
        let rows = sqlx::query_as::<_, OrganizationRole>(
            r#"
            SELECT
                id,
                organization_id,
                name,
                display_name,
                description,
                permissions,
                is_system_role,
                priority,
                created_at,
                updated_at,
                created_by
            FROM organization_roles
            WHERE organization_id = $1
            ORDER BY priority DESC, name ASC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// List system roles for an organization.
    pub async fn list_system_roles(&self, org_id: Uuid) -> Result<Vec<OrganizationRole>> {
        let rows = sqlx::query_as::<_, OrganizationRole>(
            r#"
            SELECT
                id,
                organization_id,
                name,
                display_name,
                description,
                permissions,
                is_system_role,
                priority,
                created_at,
                updated_at,
                created_by
            FROM organization_roles
            WHERE organization_id = $1 AND is_system_role = TRUE
            ORDER BY priority DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// List custom (non-system) roles for an organization.
    pub async fn list_custom_roles(&self, org_id: Uuid) -> Result<Vec<OrganizationRole>> {
        let rows = sqlx::query_as::<_, OrganizationRole>(
            r#"
            SELECT
                id,
                organization_id,
                name,
                display_name,
                description,
                permissions,
                is_system_role,
                priority,
                created_at,
                updated_at,
                created_by
            FROM organization_roles
            WHERE organization_id = $1 AND is_system_role = FALSE
            ORDER BY priority DESC, name ASC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Get a role by ID.
    pub async fn find_by_id(&self, role_id: Uuid) -> Result<Option<OrganizationRole>> {
        let row = sqlx::query_as::<_, OrganizationRole>(
            r#"
            SELECT
                id,
                organization_id,
                name,
                display_name,
                description,
                permissions,
                is_system_role,
                priority,
                created_at,
                updated_at,
                created_by
            FROM organization_roles
            WHERE id = $1
            "#,
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Get a role by name within an organization.
    pub async fn find_by_name(&self, org_id: Uuid, name: &str) -> Result<Option<OrganizationRole>> {
        let row = sqlx::query_as::<_, OrganizationRole>(
            r#"
            SELECT
                id,
                organization_id,
                name,
                display_name,
                description,
                permissions,
                is_system_role,
                priority,
                created_at,
                updated_at,
                created_by
            FROM organization_roles
            WHERE organization_id = $1 AND name = $2
            "#,
        )
        .bind(org_id)
        .bind(name.to_lowercase())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Create a new custom role.
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        org_id: Uuid,
        name: &str,
        display_name: &str,
        description: Option<&str>,
        permissions: &[String],
        priority: i32,
        created_by: Option<Uuid>,
    ) -> Result<OrganizationRole> {
        let row = sqlx::query_as::<_, OrganizationRole>(
            r#"
            INSERT INTO organization_roles (
                organization_id, name, display_name, description, permissions, priority, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id,
                organization_id,
                name,
                display_name,
                description,
                permissions,
                is_system_role,
                priority,
                created_at,
                updated_at,
                created_by
            "#,
        )
        .bind(org_id)
        .bind(name.to_lowercase())
        .bind(display_name)
        .bind(description)
        .bind(permissions)
        .bind(priority)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Delete a custom role by ID.
    /// Returns true if deleted, false if not found.
    /// Will fail if the role is a system role or has assigned users.
    pub async fn delete(&self, role_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM organization_roles
            WHERE id = $1 AND is_system_role = FALSE
            "#,
        )
        .bind(role_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Count users assigned to a specific role (by role name) in an organization.
    pub async fn count_users_with_role(&self, org_id: Uuid, role_name: &str) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM org_users
            WHERE organization_id = $1 AND role::text = $2
            "#,
        )
        .bind(org_id)
        .bind(role_name.to_lowercase())
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    /// Check if a role exists in an organization.
    pub async fn exists(&self, org_id: Uuid, role_id: Uuid) -> Result<bool> {
        let exists: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM organization_roles
                WHERE id = $1 AND organization_id = $2
            ) as exists
            "#,
        )
        .bind(role_id)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists.0)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repository_new() {
        // Just verify the struct can be created
        // Full integration tests would require a database connection
    }
}
