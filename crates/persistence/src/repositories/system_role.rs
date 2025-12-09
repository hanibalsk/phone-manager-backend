//! System role repository for database operations.

use domain::models::{AdminOrgAssignment, OrgAssignmentDetail, SystemRole, UserSystemRole};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::system_role::{
    AdminOrgAssignmentEntity, AdminOrgAssignmentWithNameEntity, SystemRoleDb, UserSystemRoleEntity,
};

/// Repository for system role database operations.
#[derive(Clone)]
pub struct SystemRoleRepository {
    pool: PgPool,
}

impl SystemRoleRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ===========================================
    // System Role Management
    // ===========================================

    /// Add a system role to a user.
    pub async fn add_role(
        &self,
        user_id: Uuid,
        role: SystemRole,
        granted_by: Option<Uuid>,
    ) -> Result<UserSystemRole, sqlx::Error> {
        let entity = sqlx::query_as::<_, UserSystemRoleEntity>(
            r#"
            INSERT INTO user_system_roles (user_id, role, granted_by)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, role, granted_at, granted_by
            "#,
        )
        .bind(user_id)
        .bind(SystemRoleDb::from(role))
        .bind(granted_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity.into())
    }

    /// Remove a system role from a user.
    pub async fn remove_role(&self, user_id: Uuid, role: SystemRole) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_system_roles
            WHERE user_id = $1 AND role = $2
            "#,
        )
        .bind(user_id)
        .bind(SystemRoleDb::from(role))
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get all system roles for a user.
    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<UserSystemRole>, sqlx::Error> {
        let entities = sqlx::query_as::<_, UserSystemRoleEntity>(
            r#"
            SELECT id, user_id, role, granted_at, granted_by
            FROM user_system_roles
            WHERE user_id = $1
            ORDER BY granted_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    /// Check if a user has a specific system role.
    pub async fn has_role(&self, user_id: Uuid, role: SystemRole) -> Result<bool, sqlx::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM user_system_roles WHERE user_id = $1 AND role = $2)",
        )
        .bind(user_id)
        .bind(SystemRoleDb::from(role))
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    /// Check if a user has any of the specified system roles.
    pub async fn has_any_role(
        &self,
        user_id: Uuid,
        roles: &[SystemRole],
    ) -> Result<bool, sqlx::Error> {
        if roles.is_empty() {
            return Ok(false);
        }

        // Build dynamic query with OR conditions since PostgreSQL array binding
        // doesn't work well with custom enum types
        let role_strs: Vec<String> = roles.iter().map(|r| format!("'{}'", r)).collect();
        let roles_condition = role_strs.join(", ");

        let query = format!(
            "SELECT EXISTS(SELECT 1 FROM user_system_roles WHERE user_id = $1 AND role IN ({}))",
            roles_condition
        );

        let exists: bool = sqlx::query_scalar(&query)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(exists)
    }

    // ===========================================
    // Protection Checks
    // ===========================================

    /// Count the number of super admins in the system.
    pub async fn count_super_admins(&self) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_system_roles WHERE role = 'super_admin'",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Count users with a specific system role.
    pub async fn count_users_with_role(&self, role: SystemRole) -> Result<i64, sqlx::Error> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM user_system_roles WHERE role = $1")
                .bind(SystemRoleDb::from(role))
                .fetch_one(&self.pool)
                .await?;

        Ok(count)
    }

    // ===========================================
    // Organization Assignments
    // ===========================================

    /// Assign an organization to an admin user.
    pub async fn assign_org(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        assigned_by: Option<Uuid>,
    ) -> Result<AdminOrgAssignment, sqlx::Error> {
        let entity = sqlx::query_as::<_, AdminOrgAssignmentEntity>(
            r#"
            INSERT INTO admin_org_assignments (user_id, organization_id, assigned_by)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, organization_id, assigned_at, assigned_by
            "#,
        )
        .bind(user_id)
        .bind(organization_id)
        .bind(assigned_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity.into())
    }

    /// Remove an organization assignment from an admin user.
    pub async fn unassign_org(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM admin_org_assignments
            WHERE user_id = $1 AND organization_id = $2
            "#,
        )
        .bind(user_id)
        .bind(organization_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Remove all organization assignments for a user.
    /// Used when removing org_admin or org_manager role.
    pub async fn remove_all_org_assignments(&self, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM admin_org_assignments
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get all organization assignments for a user with org names.
    pub async fn get_assigned_orgs(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<OrgAssignmentDetail>, sqlx::Error> {
        let entities = sqlx::query_as::<_, AdminOrgAssignmentWithNameEntity>(
            r#"
            SELECT
                a.id, a.user_id, a.organization_id, a.assigned_at, a.assigned_by,
                o.name as organization_name
            FROM admin_org_assignments a
            LEFT JOIN organizations o ON o.id = a.organization_id
            WHERE a.user_id = $1
            ORDER BY a.assigned_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    /// Get organization assignment IDs for a user.
    pub async fn get_assigned_org_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
        let org_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT organization_id FROM admin_org_assignments WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(org_ids)
    }

    /// Check if a user is assigned to a specific organization.
    pub async fn is_assigned_to_org(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM admin_org_assignments WHERE user_id = $1 AND organization_id = $2)",
        )
        .bind(user_id)
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    /// Check if user has access to an organization.
    /// SuperAdmin has access to all orgs.
    /// OrgAdmin/OrgManager need explicit assignment.
    /// Support/Viewer have global read access.
    pub async fn has_org_access(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        // Check if user is super_admin (has access to all)
        if self.has_role(user_id, SystemRole::SuperAdmin).await? {
            return Ok(true);
        }

        // Check if user has support or viewer role (global read)
        if self
            .has_any_role(user_id, &[SystemRole::Support, SystemRole::Viewer])
            .await?
        {
            return Ok(true);
        }

        // For org_admin/org_manager, check assignment
        self.is_assigned_to_org(user_id, organization_id).await
    }

    /// Check if user can manage an organization (not just read).
    /// SuperAdmin can manage all.
    /// OrgAdmin/OrgManager can manage assigned orgs.
    pub async fn can_manage_org(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        // Check if user is super_admin
        if self.has_role(user_id, SystemRole::SuperAdmin).await? {
            return Ok(true);
        }

        // Check if user has org_admin or org_manager role AND is assigned to this org
        let has_manage_role = self
            .has_any_role(user_id, &[SystemRole::OrgAdmin, SystemRole::OrgManager])
            .await?;

        if !has_manage_role {
            return Ok(false);
        }

        self.is_assigned_to_org(user_id, organization_id).await
    }

    // ===========================================
    // Utility Methods
    // ===========================================

    /// Get the highest priority role for a user.
    /// Returns None if user has no system roles.
    pub async fn get_highest_role(&self, user_id: Uuid) -> Result<Option<SystemRole>, sqlx::Error> {
        let roles = self.get_user_roles(user_id).await?;

        // Return the highest priority role
        let mut highest: Option<SystemRole> = None;
        for usr in roles {
            match (&highest, &usr.role) {
                (None, _) => highest = Some(usr.role),
                (Some(current), new) if new.has_at_least(*current) => highest = Some(*new),
                _ => {}
            }
        }

        Ok(highest)
    }

    /// Check if a user has any system role.
    pub async fn has_any_system_role(&self, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user_system_roles WHERE user_id = $1)")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_role_db_conversion() {
        assert_eq!(
            SystemRoleDb::from(SystemRole::SuperAdmin),
            SystemRoleDb::SuperAdmin
        );
        assert_eq!(
            SystemRole::from(SystemRoleDb::OrgAdmin),
            SystemRole::OrgAdmin
        );
    }
}
