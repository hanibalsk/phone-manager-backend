//! Organization user repository for database operations.

use domain::models::{ListOrgUsersQuery, OrgUser, OrgUserRole, OrgUserWithDetails};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::org_user::{OrgUserEntity, OrgUserRoleDb, OrgUserWithDetailsEntity};

/// Repository for organization user database operations.
#[derive(Clone)]
pub struct OrgUserRepository {
    pool: PgPool,
}

impl OrgUserRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Add a user to an organization.
    pub async fn create(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
        role: OrgUserRole,
        permissions: &[String],
        granted_by: Option<Uuid>,
    ) -> Result<OrgUserWithDetails, sqlx::Error> {
        let permissions_json = serde_json::to_value(permissions).unwrap_or_default();

        let entity = sqlx::query_as::<_, OrgUserWithDetailsEntity>(
            r#"
            WITH inserted AS (
                INSERT INTO org_users (organization_id, user_id, role, permissions, granted_by)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, organization_id, user_id, role, permissions, granted_at, granted_by
            )
            SELECT
                i.id, i.organization_id, i.user_id, i.role, i.permissions, i.granted_at, i.granted_by,
                u.email as user_email, u.display_name as user_display_name
            FROM inserted i
            JOIN users u ON u.id = i.user_id
            "#,
        )
        .bind(organization_id)
        .bind(user_id)
        .bind(OrgUserRoleDb::from(role))
        .bind(permissions_json)
        .bind(granted_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity.into())
    }

    /// Find organization user by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<OrgUser>, sqlx::Error> {
        let entity = sqlx::query_as::<_, OrgUserEntity>(
            r#"
            SELECT id, organization_id, user_id, role, permissions, granted_at, granted_by
            FROM org_users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Find organization user by organization and user ID.
    pub async fn find_by_org_and_user(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<OrgUser>, sqlx::Error> {
        let entity = sqlx::query_as::<_, OrgUserEntity>(
            r#"
            SELECT id, organization_id, user_id, role, permissions, granted_at, granted_by
            FROM org_users
            WHERE organization_id = $1 AND user_id = $2
            "#,
        )
        .bind(organization_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Get organization user with details.
    pub async fn find_with_details(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<OrgUserWithDetails>, sqlx::Error> {
        let entity = sqlx::query_as::<_, OrgUserWithDetailsEntity>(
            r#"
            SELECT
                ou.id, ou.organization_id, ou.user_id, ou.role, ou.permissions, ou.granted_at, ou.granted_by,
                u.email as user_email, u.display_name as user_display_name
            FROM org_users ou
            JOIN users u ON u.id = ou.user_id
            WHERE ou.organization_id = $1 AND ou.user_id = $2
            "#,
        )
        .bind(organization_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Update organization user.
    pub async fn update(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
        role: Option<OrgUserRole>,
        permissions: Option<&[String]>,
    ) -> Result<Option<OrgUserWithDetails>, sqlx::Error> {
        let permissions_json = permissions.map(|p| serde_json::to_value(p).unwrap_or_default());

        let entity = sqlx::query_as::<_, OrgUserWithDetailsEntity>(
            r#"
            WITH updated AS (
                UPDATE org_users
                SET
                    role = COALESCE($3, role),
                    permissions = COALESCE($4, permissions)
                WHERE organization_id = $1 AND user_id = $2
                RETURNING id, organization_id, user_id, role, permissions, granted_at, granted_by
            )
            SELECT
                u2.id, u2.organization_id, u2.user_id, u2.role, u2.permissions, u2.granted_at, u2.granted_by,
                u.email as user_email, u.display_name as user_display_name
            FROM updated u2
            JOIN users u ON u.id = u2.user_id
            "#,
        )
        .bind(organization_id)
        .bind(user_id)
        .bind(role.map(OrgUserRoleDb::from))
        .bind(permissions_json)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Remove user from organization.
    pub async fn delete(&self, organization_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM org_users
            WHERE organization_id = $1 AND user_id = $2
            "#,
        )
        .bind(organization_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List organization users with pagination.
    pub async fn list(
        &self,
        organization_id: Uuid,
        query: &ListOrgUsersQuery,
    ) -> Result<(Vec<OrgUserWithDetails>, i64), sqlx::Error> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(50).clamp(1, 100);
        let offset = ((page - 1) * per_page) as i64;

        // Get total count
        let total: i64 = if let Some(ref role) = query.role {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM org_users WHERE organization_id = $1 AND role = $2",
            )
            .bind(organization_id)
            .bind(OrgUserRoleDb::from(*role))
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM org_users WHERE organization_id = $1")
                .bind(organization_id)
                .fetch_one(&self.pool)
                .await?
        };

        // Get org users
        let entities = if let Some(ref role) = query.role {
            sqlx::query_as::<_, OrgUserWithDetailsEntity>(
                r#"
                SELECT
                    ou.id, ou.organization_id, ou.user_id, ou.role, ou.permissions, ou.granted_at, ou.granted_by,
                    u.email as user_email, u.display_name as user_display_name
                FROM org_users ou
                JOIN users u ON u.id = ou.user_id
                WHERE ou.organization_id = $1 AND ou.role = $2
                ORDER BY ou.granted_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(organization_id)
            .bind(OrgUserRoleDb::from(*role))
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, OrgUserWithDetailsEntity>(
                r#"
                SELECT
                    ou.id, ou.organization_id, ou.user_id, ou.role, ou.permissions, ou.granted_at, ou.granted_by,
                    u.email as user_email, u.display_name as user_display_name
                FROM org_users ou
                JOIN users u ON u.id = ou.user_id
                WHERE ou.organization_id = $1
                ORDER BY ou.granted_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(organization_id)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        let org_users = entities.into_iter().map(Into::into).collect();

        Ok((org_users, total))
    }

    /// Count owners in organization.
    pub async fn count_owners(&self, organization_id: Uuid) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM org_users WHERE organization_id = $1 AND role = 'owner'",
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Check if user exists in organization.
    pub async fn exists(&self, organization_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM org_users WHERE organization_id = $1 AND user_id = $2)",
        )
        .bind(organization_id)
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
    fn test_org_user_role_db_conversion() {
        assert_eq!(OrgUserRoleDb::from(OrgUserRole::Owner), OrgUserRoleDb::Owner);
        assert_eq!(OrgUserRole::from(OrgUserRoleDb::Admin), OrgUserRole::Admin);
    }
}
