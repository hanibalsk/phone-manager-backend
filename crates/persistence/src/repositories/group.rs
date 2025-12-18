//! Group repository for database operations.

use domain::models::group::GroupRole;
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{
    GroupEntity, GroupMembershipEntity, GroupRoleDb, GroupWithMembershipEntity,
    MemberWithUserEntity,
};
use crate::metrics::QueryTimer;

/// Repository for group-related database operations.
#[derive(Clone)]
pub struct GroupRepository {
    pool: PgPool,
}

impl GroupRepository {
    /// Creates a new GroupRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Create a new group and add the creator as owner.
    pub async fn create_group(
        &self,
        name: &str,
        slug: &str,
        description: Option<&str>,
        icon_emoji: Option<&str>,
        max_devices: i32,
        created_by: Uuid,
    ) -> Result<GroupEntity, sqlx::Error> {
        let timer = QueryTimer::new("create_group");

        // Start a transaction to ensure both group and membership are created atomically
        let mut tx = self.pool.begin().await?;

        // Create the group
        let group = sqlx::query_as::<_, GroupEntity>(
            r#"
            INSERT INTO groups (name, slug, description, icon_emoji, max_devices, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, slug, description, icon_emoji, max_devices, is_active, settings, created_by, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(slug)
        .bind(description)
        .bind(icon_emoji)
        .bind(max_devices)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // Add the creator as owner
        sqlx::query(
            r#"
            INSERT INTO group_memberships (group_id, user_id, role)
            VALUES ($1, $2, 'owner')
            "#,
        )
        .bind(group.id)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        timer.record();
        Ok(group)
    }

    /// Find a group by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<GroupEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_group_by_id");
        let result = sqlx::query_as::<_, GroupEntity>(
            r#"
            SELECT id, name, slug, description, icon_emoji, max_devices, is_active, settings, created_by, created_at, updated_at
            FROM groups
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find a group by slug.
    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<GroupEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_group_by_slug");
        let result = sqlx::query_as::<_, GroupEntity>(
            r#"
            SELECT id, name, slug, description, icon_emoji, max_devices, is_active, settings, created_by, created_at, updated_at
            FROM groups
            WHERE slug = $1 AND is_active = true
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find all groups a user belongs to.
    ///
    /// # Arguments
    /// * `user_id` - The user to find groups for
    /// * `role_filter` - Optional role filter (e.g., "admin", "member")
    /// * `device_id` - Optional device ID to check membership for. If None, checks if ANY user device is in each group.
    pub async fn find_user_groups(
        &self,
        user_id: Uuid,
        role_filter: Option<&str>,
        device_id: Option<Uuid>,
    ) -> Result<Vec<GroupWithMembershipEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_user_groups");

        let result = if let Some(role) = role_filter {
            sqlx::query_as::<_, GroupWithMembershipEntity>(
                r#"
                SELECT
                    g.id, g.name, g.slug, g.description, g.icon_emoji, g.max_devices, g.is_active,
                    g.settings, g.created_by, g.created_at, g.updated_at,
                    gm.id as membership_id, gm.role, gm.joined_at,
                    (SELECT COUNT(*) FROM group_memberships WHERE group_id = g.id) as member_count,
                    (SELECT COUNT(*) FROM devices WHERE group_id = g.slug AND active = true) as device_count,
                    EXISTS (
                        SELECT 1 FROM device_group_memberships dgm
                        JOIN devices d ON dgm.device_id = d.device_id
                        WHERE dgm.group_id = g.id
                        AND d.owner_user_id = $1
                        AND d.active = true
                        AND (dgm.device_id = $3 OR $3 IS NULL)
                    ) as has_current_device
                FROM groups g
                JOIN group_memberships gm ON g.id = gm.group_id
                WHERE gm.user_id = $1 AND g.is_active = true AND gm.role::text = $2
                ORDER BY gm.joined_at DESC
                "#,
            )
            .bind(user_id)
            .bind(role)
            .bind(device_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, GroupWithMembershipEntity>(
                r#"
                SELECT
                    g.id, g.name, g.slug, g.description, g.icon_emoji, g.max_devices, g.is_active,
                    g.settings, g.created_by, g.created_at, g.updated_at,
                    gm.id as membership_id, gm.role, gm.joined_at,
                    (SELECT COUNT(*) FROM group_memberships WHERE group_id = g.id) as member_count,
                    (SELECT COUNT(*) FROM devices WHERE group_id = g.slug AND active = true) as device_count,
                    EXISTS (
                        SELECT 1 FROM device_group_memberships dgm
                        JOIN devices d ON dgm.device_id = d.device_id
                        WHERE dgm.group_id = g.id
                        AND d.owner_user_id = $1
                        AND d.active = true
                        AND (dgm.device_id = $2 OR $2 IS NULL)
                    ) as has_current_device
                FROM groups g
                JOIN group_memberships gm ON g.id = gm.group_id
                WHERE gm.user_id = $1 AND g.is_active = true
                ORDER BY gm.joined_at DESC
                "#,
            )
            .bind(user_id)
            .bind(device_id)
            .fetch_all(&self.pool)
            .await
        };

        timer.record();
        result
    }

    /// Find group with membership info for a specific user.
    ///
    /// # Arguments
    /// * `group_id` - The group ID to find
    /// * `user_id` - The user to check membership for
    /// * `device_id` - Optional device ID to check assignment for. If None, checks if ANY user device is in the group.
    pub async fn find_group_with_membership(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        device_id: Option<Uuid>,
    ) -> Result<Option<GroupWithMembershipEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_group_with_membership");
        let result = sqlx::query_as::<_, GroupWithMembershipEntity>(
            r#"
            SELECT
                g.id, g.name, g.slug, g.description, g.icon_emoji, g.max_devices, g.is_active,
                g.settings, g.created_by, g.created_at, g.updated_at,
                gm.id as membership_id, gm.role, gm.joined_at,
                (SELECT COUNT(*) FROM group_memberships WHERE group_id = g.id) as member_count,
                (SELECT COUNT(*) FROM devices WHERE group_id = g.slug AND active = true) as device_count,
                EXISTS (
                    SELECT 1 FROM device_group_memberships dgm
                    JOIN devices d ON dgm.device_id = d.device_id
                    WHERE dgm.group_id = g.id
                    AND d.owner_user_id = $2
                    AND d.active = true
                    AND (dgm.device_id = $3 OR $3 IS NULL)
                ) as has_current_device
            FROM groups g
            JOIN group_memberships gm ON g.id = gm.group_id
            WHERE g.id = $1 AND gm.user_id = $2 AND g.is_active = true
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Update a group.
    pub async fn update_group(
        &self,
        group_id: Uuid,
        name: Option<&str>,
        slug: Option<&str>,
        description: Option<&str>,
        icon_emoji: Option<&str>,
        max_devices: Option<i32>,
    ) -> Result<GroupEntity, sqlx::Error> {
        let timer = QueryTimer::new("update_group");

        // Build dynamic update query
        let result = sqlx::query_as::<_, GroupEntity>(
            r#"
            UPDATE groups
            SET
                name = COALESCE($2, name),
                slug = COALESCE($3, slug),
                description = COALESCE($4, description),
                icon_emoji = COALESCE($5, icon_emoji),
                max_devices = COALESCE($6, max_devices),
                updated_at = NOW()
            WHERE id = $1 AND is_active = true
            RETURNING id, name, slug, description, icon_emoji, max_devices, is_active, settings, created_by, created_at, updated_at
            "#,
        )
        .bind(group_id)
        .bind(name)
        .bind(slug)
        .bind(description)
        .bind(icon_emoji)
        .bind(max_devices)
        .fetch_one(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Soft delete a group.
    pub async fn delete_group(&self, group_id: Uuid) -> Result<u64, sqlx::Error> {
        let timer = QueryTimer::new("delete_group");
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
        timer.record();
        Ok(result.rows_affected())
    }

    /// Get user's membership for a group.
    pub async fn get_membership(
        &self,
        group_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<GroupMembershipEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_group_membership");
        let result = sqlx::query_as::<_, GroupMembershipEntity>(
            r#"
            SELECT id, group_id, user_id, role, invited_by, joined_at, updated_at
            FROM group_memberships
            WHERE group_id = $1 AND user_id = $2
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Add a member to a group.
    pub async fn add_member(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        role: GroupRole,
        invited_by: Option<Uuid>,
    ) -> Result<GroupMembershipEntity, sqlx::Error> {
        let timer = QueryTimer::new("add_group_member");
        let role_db: GroupRoleDb = role.into();
        let result = sqlx::query_as::<_, GroupMembershipEntity>(
            r#"
            INSERT INTO group_memberships (group_id, user_id, role, invited_by)
            VALUES ($1, $2, $3, $4)
            RETURNING id, group_id, user_id, role, invited_by, joined_at, updated_at
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .bind(role_db)
        .bind(invited_by)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Update member role.
    pub async fn update_member_role(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        new_role: GroupRole,
    ) -> Result<GroupMembershipEntity, sqlx::Error> {
        let timer = QueryTimer::new("update_member_role");
        let role_db: GroupRoleDb = new_role.into();
        let result = sqlx::query_as::<_, GroupMembershipEntity>(
            r#"
            UPDATE group_memberships
            SET role = $3, updated_at = NOW()
            WHERE group_id = $1 AND user_id = $2
            RETURNING id, group_id, user_id, role, invited_by, joined_at, updated_at
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .bind(role_db)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Remove a member from a group.
    pub async fn remove_member(&self, group_id: Uuid, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let timer = QueryTimer::new("remove_group_member");
        let result = sqlx::query(
            r#"
            DELETE FROM group_memberships
            WHERE group_id = $1 AND user_id = $2
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(result.rows_affected())
    }

    /// Check if slug exists.
    pub async fn slug_exists(&self, slug: &str) -> Result<bool, sqlx::Error> {
        let timer = QueryTimer::new("check_group_slug_exists");
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(SELECT 1 FROM groups WHERE slug = $1)
            "#,
        )
        .bind(slug)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Generate unique slug by appending numbers if needed.
    pub async fn generate_unique_slug(&self, base_slug: &str) -> Result<String, sqlx::Error> {
        let mut slug = base_slug.to_string();
        let mut counter = 1;

        while self.slug_exists(&slug).await? {
            slug = format!("{}-{}", base_slug, counter);
            counter += 1;
            if counter > 100 {
                // Safety limit
                return Err(sqlx::Error::Protocol(
                    "Could not generate unique slug".to_string(),
                ));
            }
        }

        Ok(slug)
    }

    // =========================================================================
    // Membership listing methods (Story 11.2)
    // =========================================================================

    /// List members of a group with pagination.
    pub async fn list_members(
        &self,
        group_id: Uuid,
        role_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MemberWithUserEntity>, sqlx::Error> {
        let timer = QueryTimer::new("list_group_members");

        let result = if let Some(role) = role_filter {
            sqlx::query_as::<_, MemberWithUserEntity>(
                r#"
                SELECT
                    gm.id, gm.group_id, gm.user_id, gm.role, gm.invited_by, gm.joined_at,
                    u.display_name, u.avatar_url
                FROM group_memberships gm
                JOIN users u ON gm.user_id = u.id
                WHERE gm.group_id = $1 AND gm.role::text = $2
                ORDER BY gm.joined_at ASC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(group_id)
            .bind(role)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, MemberWithUserEntity>(
                r#"
                SELECT
                    gm.id, gm.group_id, gm.user_id, gm.role, gm.invited_by, gm.joined_at,
                    u.display_name, u.avatar_url
                FROM group_memberships gm
                JOIN users u ON gm.user_id = u.id
                WHERE gm.group_id = $1
                ORDER BY gm.joined_at ASC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(group_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };

        timer.record();
        result
    }

    /// Count total members in a group (with optional role filter).
    pub async fn count_members(
        &self,
        group_id: Uuid,
        role_filter: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_group_members");

        let result = if let Some(role) = role_filter {
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*)
                FROM group_memberships
                WHERE group_id = $1 AND role::text = $2
                "#,
            )
            .bind(group_id)
            .bind(role)
            .fetch_one(&self.pool)
            .await
        } else {
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*)
                FROM group_memberships
                WHERE group_id = $1
                "#,
            )
            .bind(group_id)
            .fetch_one(&self.pool)
            .await
        };

        timer.record();
        result
    }

    /// Get a single member with user info.
    pub async fn get_member_with_user(
        &self,
        group_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<MemberWithUserEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_member_with_user");
        let result = sqlx::query_as::<_, MemberWithUserEntity>(
            r#"
            SELECT
                gm.id, gm.group_id, gm.user_id, gm.role, gm.invited_by, gm.joined_at,
                u.display_name, u.avatar_url
            FROM group_memberships gm
            JOIN users u ON gm.user_id = u.id
            WHERE gm.group_id = $1 AND gm.user_id = $2
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    // =========================================================================
    // Ownership Transfer (Story 11.6)
    // =========================================================================

    /// Transfer group ownership atomically.
    /// The current owner becomes admin, and the new owner gets the owner role.
    pub async fn transfer_ownership(
        &self,
        group_id: Uuid,
        current_owner_id: Uuid,
        new_owner_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let timer = QueryTimer::new("transfer_ownership");

        // Start a transaction for atomic role swap
        let mut tx = self.pool.begin().await?;

        // IMPORTANT: Promote new owner FIRST, then demote old owner.
        // This order is required because of the check_group_has_owner trigger
        // which prevents demoting the last owner before another owner exists.

        // Promote new owner to owner
        sqlx::query(
            r#"
            UPDATE group_memberships
            SET role = 'owner', updated_at = NOW()
            WHERE group_id = $1 AND user_id = $2
            "#,
        )
        .bind(group_id)
        .bind(new_owner_id)
        .execute(&mut *tx)
        .await?;

        // Demote previous owner to admin
        sqlx::query(
            r#"
            UPDATE group_memberships
            SET role = 'admin', updated_at = NOW()
            WHERE group_id = $1 AND user_id = $2
            "#,
        )
        .bind(group_id)
        .bind(current_owner_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        timer.record();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Note: GroupRepository tests require database connection and are covered by integration tests
}
