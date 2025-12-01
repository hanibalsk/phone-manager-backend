//! Admin group repository for organization group management.
//!
//! Story 14.4: Group Management Admin Endpoints

use domain::models::{
    AdminGroupItem, AdminGroupProfile, AdminGroupSortField, AdminGroupSummary, AdminSortOrder,
    GroupDeviceInfo, GroupMemberInfo, GroupOwnerInfo,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{
    AdminGroupEntity, AdminGroupProfileEntity, AdminGroupSummaryEntity, GroupDeviceEntity,
    GroupMemberEntity,
};

/// Repository for admin group management operations.
#[derive(Clone)]
pub struct AdminGroupRepository {
    pool: PgPool,
}

impl AdminGroupRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get group summary statistics for an organization.
    /// Groups belong to org via group members who are org users.
    pub async fn get_group_summary(&self, org_id: Uuid) -> Result<AdminGroupSummary, sqlx::Error> {
        let row = sqlx::query_as::<_, AdminGroupSummaryEntity>(
            r#"
            WITH org_groups AS (
                SELECT DISTINCT g.id
                FROM groups g
                JOIN group_memberships gm ON gm.group_id = g.id
                JOIN org_users ou ON ou.user_id = gm.user_id
                WHERE ou.organization_id = $1
            )
            SELECT
                COUNT(DISTINCT g.id) as total_groups,
                COUNT(DISTINCT g.id) FILTER (WHERE g.is_active = true) as active_groups,
                COALESCE(SUM((SELECT COUNT(*) FROM group_memberships gm2 WHERE gm2.group_id = g.id)), 0) as total_members,
                COALESCE(SUM((SELECT COUNT(*) FROM devices d WHERE d.group_id = g.slug AND d.active = true)), 0) as total_devices
            FROM groups g
            WHERE g.id IN (SELECT id FROM org_groups)
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(AdminGroupSummary {
            total_groups: row.total_groups,
            active_groups: row.active_groups,
            total_members: row.total_members,
            total_devices: row.total_devices,
        })
    }

    /// Count groups matching filters.
    pub async fn count_groups(
        &self,
        org_id: Uuid,
        active_filter: Option<bool>,
        has_devices_filter: Option<bool>,
        search_filter: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let mut query = r#"
            WITH org_groups AS (
                SELECT DISTINCT g.id
                FROM groups g
                JOIN group_memberships gm ON gm.group_id = g.id
                JOIN org_users ou ON ou.user_id = gm.user_id
                WHERE ou.organization_id = $1
            )
            SELECT COUNT(DISTINCT g.id)
            FROM groups g
            WHERE g.id IN (SELECT id FROM org_groups)
            "#
        .to_string();

        let mut param_idx = 2;
        let mut conditions = Vec::new();

        if active_filter.is_some() {
            conditions.push(format!("g.is_active = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(has_devices) = has_devices_filter {
            if has_devices {
                conditions.push(
                    "EXISTS (SELECT 1 FROM devices d WHERE d.group_id = g.slug AND d.active = true)"
                        .to_string(),
                );
            } else {
                conditions.push(
                    "NOT EXISTS (SELECT 1 FROM devices d WHERE d.group_id = g.slug AND d.active = true)"
                        .to_string(),
                );
            }
        }

        if search_filter.is_some() {
            conditions.push(format!("g.name ILIKE ${}", param_idx));
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        let mut q = sqlx::query_scalar::<_, i64>(&query).bind(org_id);

        if let Some(active) = active_filter {
            q = q.bind(active);
        }

        if let Some(search) = search_filter {
            let pattern = format!("%{}%", search);
            q = q.bind(pattern);
        }

        q.fetch_one(&self.pool).await
    }

    /// List admin groups with filtering, sorting, and pagination.
    #[allow(clippy::too_many_arguments)]
    pub async fn list_groups(
        &self,
        org_id: Uuid,
        active_filter: Option<bool>,
        has_devices_filter: Option<bool>,
        search_filter: Option<&str>,
        sort_field: AdminGroupSortField,
        sort_order: AdminSortOrder,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<AdminGroupItem>, sqlx::Error> {
        let sort_col = sort_field.as_sql_column();
        let sort_dir = sort_order.as_sql();

        let mut query = r#"
            WITH org_groups AS (
                SELECT DISTINCT g.id
                FROM groups g
                JOIN group_memberships gm ON gm.group_id = g.id
                JOIN org_users ou ON ou.user_id = gm.user_id
                WHERE ou.organization_id = $1
            )
            SELECT
                g.id as group_id,
                g.name,
                g.slug,
                g.description,
                g.icon_emoji,
                g.is_active,
                g.created_at,
                COALESCE((
                    SELECT COUNT(*) FROM group_memberships gm WHERE gm.group_id = g.id
                ), 0) as member_count,
                COALESCE((
                    SELECT COUNT(*) FROM devices d WHERE d.group_id = g.slug AND d.active = true
                ), 0) as device_count,
                owner.id as owner_id,
                owner.email as owner_email,
                owner.display_name as owner_display_name
            FROM groups g
            LEFT JOIN group_memberships owner_gm ON owner_gm.group_id = g.id AND owner_gm.role = 'owner'
            LEFT JOIN users owner ON owner.id = owner_gm.user_id
            WHERE g.id IN (SELECT id FROM org_groups)
            "#
        .to_string();

        let mut param_idx = 2;
        let mut conditions = Vec::new();

        if active_filter.is_some() {
            conditions.push(format!("g.is_active = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(has_devices) = has_devices_filter {
            if has_devices {
                conditions.push(
                    "EXISTS (SELECT 1 FROM devices d WHERE d.group_id = g.slug AND d.active = true)"
                        .to_string(),
                );
            } else {
                conditions.push(
                    "NOT EXISTS (SELECT 1 FROM devices d WHERE d.group_id = g.slug AND d.active = true)"
                        .to_string(),
                );
            }
        }

        if search_filter.is_some() {
            conditions.push(format!("g.name ILIKE ${}", param_idx));
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

        let mut q = sqlx::query_as::<_, AdminGroupEntity>(&query).bind(org_id);

        if let Some(active) = active_filter {
            q = q.bind(active);
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
                let owner = e.owner_id.map(|id| GroupOwnerInfo {
                    id,
                    email: e.owner_email.unwrap_or_default(),
                    display_name: e.owner_display_name,
                });

                AdminGroupItem {
                    id: e.group_id,
                    name: e.name,
                    slug: e.slug,
                    description: e.description,
                    icon_emoji: e.icon_emoji,
                    member_count: e.member_count,
                    device_count: e.device_count,
                    is_active: e.is_active,
                    owner,
                    created_at: e.created_at,
                }
            })
            .collect();

        Ok(items)
    }

    /// Get group profile details.
    pub async fn get_group_profile(
        &self,
        org_id: Uuid,
        group_id: Uuid,
    ) -> Result<Option<AdminGroupProfile>, sqlx::Error> {
        let entity = sqlx::query_as::<_, AdminGroupProfileEntity>(
            r#"
            WITH org_groups AS (
                SELECT DISTINCT g.id
                FROM groups g
                JOIN group_memberships gm ON gm.group_id = g.id
                JOIN org_users ou ON ou.user_id = gm.user_id
                WHERE ou.organization_id = $1
            )
            SELECT
                g.id as group_id,
                g.name,
                g.slug,
                g.description,
                g.icon_emoji,
                g.max_devices,
                g.is_active,
                g.created_by,
                g.created_at,
                COALESCE((
                    SELECT COUNT(*) FROM group_memberships gm WHERE gm.group_id = g.id
                ), 0) as member_count,
                COALESCE((
                    SELECT COUNT(*) FROM devices d WHERE d.group_id = g.slug AND d.active = true
                ), 0) as device_count
            FROM groups g
            WHERE g.id = $2 AND g.id IN (SELECT id FROM org_groups)
            "#,
        )
        .bind(org_id)
        .bind(group_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(|e| AdminGroupProfile {
            id: e.group_id,
            name: e.name,
            slug: e.slug,
            description: e.description,
            icon_emoji: e.icon_emoji,
            max_devices: e.max_devices,
            member_count: e.member_count,
            device_count: e.device_count,
            is_active: e.is_active,
            created_by: e.created_by,
            created_at: e.created_at,
        }))
    }

    /// Get members of a group.
    pub async fn get_group_members(
        &self,
        group_id: Uuid,
    ) -> Result<Vec<GroupMemberInfo>, sqlx::Error> {
        let entities = sqlx::query_as::<_, GroupMemberEntity>(
            r#"
            SELECT
                gm.user_id,
                u.email,
                u.display_name,
                gm.role,
                gm.joined_at
            FROM group_memberships gm
            JOIN users u ON u.id = gm.user_id
            WHERE gm.group_id = $1
            ORDER BY gm.role ASC, u.display_name ASC
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities
            .into_iter()
            .map(|e| GroupMemberInfo {
                user_id: e.user_id,
                email: e.email,
                display_name: e.display_name,
                role: e.role,
                joined_at: e.joined_at,
            })
            .collect())
    }

    /// Get devices in a group.
    pub async fn get_group_devices(
        &self,
        group_slug: &str,
    ) -> Result<Vec<GroupDeviceInfo>, sqlx::Error> {
        let entities = sqlx::query_as::<_, GroupDeviceEntity>(
            r#"
            SELECT
                d.id,
                d.device_id,
                d.display_name,
                d.last_seen_at
            FROM devices d
            WHERE d.group_id = $1 AND d.active = true
            ORDER BY d.display_name ASC
            "#,
        )
        .bind(group_slug)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities
            .into_iter()
            .map(|e| GroupDeviceInfo {
                id: e.id,
                device_uuid: e.device_id,
                display_name: e.display_name,
                last_seen_at: e.last_seen_at,
            })
            .collect())
    }

    /// Update group settings.
    pub async fn update_group(
        &self,
        group_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        max_devices: Option<i32>,
        is_active: Option<bool>,
    ) -> Result<bool, sqlx::Error> {
        let mut updates = Vec::new();
        let mut param_idx = 2;

        if name.is_some() {
            updates.push(format!("name = ${}", param_idx));
            param_idx += 1;
        }
        if description.is_some() {
            updates.push(format!("description = ${}", param_idx));
            param_idx += 1;
        }
        if max_devices.is_some() {
            updates.push(format!("max_devices = ${}", param_idx));
            param_idx += 1;
        }
        if is_active.is_some() {
            updates.push(format!("is_active = ${}", param_idx));
        }

        if updates.is_empty() {
            return Ok(false);
        }

        updates.push("updated_at = NOW()".to_string());

        let query = format!(
            "UPDATE groups SET {} WHERE id = $1",
            updates.join(", ")
        );

        let mut q = sqlx::query(&query).bind(group_id);

        if let Some(n) = name {
            q = q.bind(n);
        }
        if let Some(d) = description {
            q = q.bind(d);
        }
        if let Some(m) = max_devices {
            q = q.bind(m);
        }
        if let Some(a) = is_active {
            q = q.bind(a);
        }

        let result = q.execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    /// Deactivate a group.
    pub async fn deactivate_group(&self, group_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE groups
            SET is_active = false, updated_at = NOW()
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(group_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a group belongs to an organization.
    pub async fn group_belongs_to_org(
        &self,
        org_id: Uuid,
        group_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM groups g
                JOIN group_memberships gm ON gm.group_id = g.id
                JOIN org_users ou ON ou.user_id = gm.user_id
                WHERE g.id = $1 AND ou.organization_id = $2
            )
            "#,
        )
        .bind(group_id)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_field_sql_column() {
        assert_eq!(AdminGroupSortField::Name.as_sql_column(), "g.name");
        assert_eq!(AdminGroupSortField::CreatedAt.as_sql_column(), "g.created_at");
        assert_eq!(AdminGroupSortField::MemberCount.as_sql_column(), "member_count");
        assert_eq!(AdminGroupSortField::DeviceCount.as_sql_column(), "device_count");
    }
}
