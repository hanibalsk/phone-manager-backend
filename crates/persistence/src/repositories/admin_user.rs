//! Admin user repository for organization user management.
//!
//! Story 14.3: User Management Endpoints

use domain::models::{
    AdminSortOrder, AdminUserItem, AdminUserProfile, AdminUserSortField, AdminUserSummary,
    OrgUserRole, RecentAction, UserActivitySummary, UserDeviceInfo, UserGroupInfo,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{
    AdminUserEntity, AdminUserProfileEntity, AdminUserSummaryEntity, RecentActionEntity,
    UserDeviceEntity, UserGroupEntity,
};

/// Repository for admin user management operations.
#[derive(Clone)]
pub struct AdminUserRepository {
    pool: PgPool,
}

impl AdminUserRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get user summary statistics for an organization.
    pub async fn get_user_summary(&self, org_id: Uuid) -> Result<AdminUserSummary, sqlx::Error> {
        let row = sqlx::query_as::<_, AdminUserSummaryEntity>(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE ou.role = 'owner') as owners,
                COUNT(*) FILTER (WHERE ou.role = 'admin') as admins,
                COUNT(*) FILTER (WHERE ou.role = 'member') as members,
                COUNT(*) FILTER (WHERE EXISTS (
                    SELECT 1 FROM devices d WHERE d.owner_user_id = ou.user_id AND d.active = true
                )) as with_devices,
                COUNT(*) FILTER (WHERE NOT EXISTS (
                    SELECT 1 FROM devices d WHERE d.owner_user_id = ou.user_id AND d.active = true
                )) as without_devices
            FROM org_users ou
            WHERE ou.organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(AdminUserSummary {
            owners: row.owners,
            admins: row.admins,
            members: row.members,
            with_devices: row.with_devices,
            without_devices: row.without_devices,
        })
    }

    /// Count users matching filters.
    #[allow(clippy::too_many_arguments)]
    pub async fn count_users(
        &self,
        org_id: Uuid,
        role_filter: Option<&str>,
        has_device_filter: Option<bool>,
        search_filter: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT COUNT(DISTINCT ou.user_id)
            FROM org_users ou
            JOIN users u ON u.id = ou.user_id
            WHERE ou.organization_id = $1
            "#,
        );

        let mut param_idx = 2;
        let mut conditions = Vec::new();

        if role_filter.is_some() {
            conditions.push(format!("ou.role = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(has_device) = has_device_filter {
            if has_device {
                conditions.push("EXISTS (SELECT 1 FROM devices d WHERE d.owner_user_id = ou.user_id AND d.active = true)".to_string());
            } else {
                conditions.push("NOT EXISTS (SELECT 1 FROM devices d WHERE d.owner_user_id = ou.user_id AND d.active = true)".to_string());
            }
        }

        if search_filter.is_some() {
            conditions.push(format!(
                "(u.email ILIKE ${0} OR u.display_name ILIKE ${0})",
                param_idx
            ));
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        let mut q = sqlx::query_scalar::<_, i64>(&query).bind(org_id);

        if let Some(role) = role_filter {
            q = q.bind(role);
        }

        if let Some(search) = search_filter {
            let pattern = format!("%{}%", search);
            q = q.bind(pattern);
        }

        q.fetch_one(&self.pool).await
    }

    /// List admin users with filtering, sorting, and pagination.
    #[allow(clippy::too_many_arguments)]
    pub async fn list_users(
        &self,
        org_id: Uuid,
        role_filter: Option<&str>,
        has_device_filter: Option<bool>,
        search_filter: Option<&str>,
        sort_field: AdminUserSortField,
        sort_order: AdminSortOrder,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<AdminUserItem>, sqlx::Error> {
        let sort_col = sort_field.as_sql_column();
        let sort_dir = sort_order.as_sql();

        let mut query = r#"
            SELECT
                ou.user_id,
                u.email,
                u.display_name,
                u.avatar_url,
                u.last_login_at,
                ou.role,
                ou.permissions,
                ou.granted_at,
                COALESCE((
                    SELECT COUNT(*) FROM devices d
                    WHERE d.owner_user_id = ou.user_id AND d.active = true
                ), 0) as device_count,
                COALESCE((
                    SELECT COUNT(*) FROM group_memberships gm
                    WHERE gm.user_id = ou.user_id
                ), 0) as group_count
            FROM org_users ou
            JOIN users u ON u.id = ou.user_id
            WHERE ou.organization_id = $1
            "#
        .to_string();

        let mut param_idx = 2;
        let mut conditions = Vec::new();

        if role_filter.is_some() {
            conditions.push(format!("ou.role = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(has_device) = has_device_filter {
            if has_device {
                conditions.push("EXISTS (SELECT 1 FROM devices d WHERE d.owner_user_id = ou.user_id AND d.active = true)".to_string());
            } else {
                conditions.push("NOT EXISTS (SELECT 1 FROM devices d WHERE d.owner_user_id = ou.user_id AND d.active = true)".to_string());
            }
        }

        if search_filter.is_some() {
            conditions.push(format!(
                "(u.email ILIKE ${0} OR u.display_name ILIKE ${0})",
                param_idx
            ));
            param_idx += 1;
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(&format!(
            " ORDER BY {} {} NULLS LAST LIMIT ${} OFFSET ${}",
            sort_col,
            sort_dir,
            param_idx,
            param_idx + 1
        ));

        let mut q = sqlx::query_as::<_, AdminUserEntity>(&query).bind(org_id);

        if let Some(role) = role_filter {
            q = q.bind(role);
        }

        if let Some(search) = search_filter {
            let pattern = format!("%{}%", search);
            q = q.bind(pattern);
        }

        q = q.bind(limit as i64).bind(offset as i64);

        let entities = q.fetch_all(&self.pool).await?;

        let items = entities
            .into_iter()
            .map(|e| {
                let role = match e.role.as_str() {
                    "owner" => OrgUserRole::Owner,
                    "admin" => OrgUserRole::Admin,
                    _ => OrgUserRole::Member,
                };
                let permissions: Vec<String> =
                    serde_json::from_value(e.permissions).unwrap_or_default();

                AdminUserItem {
                    id: e.user_id,
                    email: e.email,
                    display_name: e.display_name,
                    avatar_url: e.avatar_url,
                    role,
                    permissions,
                    device_count: e.device_count,
                    group_count: e.group_count,
                    granted_at: e.granted_at,
                    last_login_at: e.last_login_at,
                }
            })
            .collect();

        Ok(items)
    }

    /// Get user profile details.
    pub async fn get_user_profile(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<AdminUserProfile>, sqlx::Error> {
        let entity = sqlx::query_as::<_, AdminUserProfileEntity>(
            r#"
            SELECT
                ou.user_id,
                u.email,
                u.display_name,
                u.avatar_url,
                u.email_verified,
                u.created_at,
                u.last_login_at,
                ou.role,
                ou.permissions,
                ou.granted_at,
                granter.email as granted_by_email
            FROM org_users ou
            JOIN users u ON u.id = ou.user_id
            LEFT JOIN users granter ON granter.id = ou.granted_by
            WHERE ou.organization_id = $1 AND ou.user_id = $2
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(|e| {
            let role = match e.role.as_str() {
                "owner" => OrgUserRole::Owner,
                "admin" => OrgUserRole::Admin,
                _ => OrgUserRole::Member,
            };
            let permissions: Vec<String> =
                serde_json::from_value(e.permissions).unwrap_or_default();

            AdminUserProfile {
                id: e.user_id,
                email: e.email,
                display_name: e.display_name,
                avatar_url: e.avatar_url,
                email_verified: e.email_verified,
                role,
                permissions,
                granted_at: e.granted_at,
                granted_by: e.granted_by_email,
                last_login_at: e.last_login_at,
                created_at: e.created_at,
            }
        }))
    }

    /// Get devices assigned to a user.
    pub async fn get_user_devices(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserDeviceInfo>, sqlx::Error> {
        let entities = sqlx::query_as::<_, UserDeviceEntity>(
            r#"
            SELECT
                d.id,
                d.device_id,
                d.display_name,
                d.platform,
                d.last_seen_at
            FROM devices d
            WHERE d.owner_user_id = $1 AND d.active = true
            ORDER BY d.display_name ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities
            .into_iter()
            .map(|e| UserDeviceInfo {
                id: e.id,
                device_uuid: e.device_id,
                display_name: e.display_name,
                platform: e.platform,
                last_seen_at: e.last_seen_at,
            })
            .collect())
    }

    /// Get groups a user belongs to.
    pub async fn get_user_groups(&self, user_id: Uuid) -> Result<Vec<UserGroupInfo>, sqlx::Error> {
        let entities = sqlx::query_as::<_, UserGroupEntity>(
            r#"
            SELECT
                g.id as group_id,
                g.name,
                gm.role
            FROM group_memberships gm
            JOIN groups g ON g.id = gm.group_id
            WHERE gm.user_id = $1
            ORDER BY g.name ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities
            .into_iter()
            .map(|e| UserGroupInfo {
                id: e.group_id,
                name: e.name,
                role: e.role,
            })
            .collect())
    }

    /// Get activity summary for a user (last 30 days).
    pub async fn get_user_activity(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<UserActivitySummary, sqlx::Error> {
        // Get total action count
        let total_actions: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM audit_logs
            WHERE organization_id = $1
              AND actor_id = $2
              AND created_at > NOW() - INTERVAL '30 days'
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Get last action timestamp
        let last_action_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
            r#"
            SELECT MAX(created_at)
            FROM audit_logs
            WHERE organization_id = $1 AND actor_id = $2
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Get recent actions (last 5)
        let recent_entities = sqlx::query_as::<_, RecentActionEntity>(
            r#"
            SELECT
                action,
                resource_type,
                created_at as timestamp
            FROM audit_logs
            WHERE organization_id = $1 AND actor_id = $2
            ORDER BY created_at DESC
            LIMIT 5
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let recent_actions = recent_entities
            .into_iter()
            .map(|e| RecentAction {
                action: e.action,
                resource_type: e.resource_type,
                timestamp: e.timestamp,
            })
            .collect();

        Ok(UserActivitySummary {
            total_actions,
            last_action_at,
            recent_actions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_field_sql_column() {
        assert_eq!(
            AdminUserSortField::DisplayName.as_sql_column(),
            "u.display_name"
        );
        assert_eq!(AdminUserSortField::Email.as_sql_column(), "u.email");
        assert_eq!(
            AdminUserSortField::GrantedAt.as_sql_column(),
            "ou.granted_at"
        );
    }

    #[test]
    fn test_sort_order_sql() {
        assert_eq!(AdminSortOrder::Asc.as_sql(), "ASC");
        assert_eq!(AdminSortOrder::Desc.as_sql(), "DESC");
    }
}
