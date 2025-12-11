//! Invite repository for database operations.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{
    GroupInviteEntity, GroupRoleDb, InviteWithCreatorEntity, InviteWithGroupEntity,
};
use crate::metrics::QueryTimer;

/// Repository for invite-related database operations.
#[derive(Clone)]
pub struct InviteRepository {
    pool: PgPool,
}

impl InviteRepository {
    /// Creates a new InviteRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Create a new invite.
    pub async fn create_invite(
        &self,
        group_id: Uuid,
        code: &str,
        preset_role: GroupRoleDb,
        max_uses: i32,
        expires_at: DateTime<Utc>,
        created_by: Uuid,
    ) -> Result<GroupInviteEntity, sqlx::Error> {
        let timer = QueryTimer::new("create_invite");
        let result = sqlx::query_as::<_, GroupInviteEntity>(
            r#"
            INSERT INTO group_invites (group_id, code, preset_role, max_uses, expires_at, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, group_id, code, preset_role, max_uses, current_uses, expires_at, created_by, created_at, is_active
            "#,
        )
        .bind(group_id)
        .bind(code)
        .bind(preset_role)
        .bind(max_uses)
        .bind(expires_at)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find invite by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<GroupInviteEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_invite_by_id");
        let result = sqlx::query_as::<_, GroupInviteEntity>(
            r#"
            SELECT id, group_id, code, preset_role, max_uses, current_uses, expires_at, created_by, created_at, is_active
            FROM group_invites
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find invite by code.
    pub async fn find_by_code(&self, code: &str) -> Result<Option<GroupInviteEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_invite_by_code");
        let result = sqlx::query_as::<_, GroupInviteEntity>(
            r#"
            SELECT id, group_id, code, preset_role, max_uses, current_uses, expires_at, created_by, created_at, is_active
            FROM group_invites
            WHERE code = $1 AND is_active = true
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find invite by code with group info (for public lookup).
    pub async fn find_by_code_with_group(
        &self,
        code: &str,
    ) -> Result<Option<InviteWithGroupEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_invite_by_code_with_group");
        let result = sqlx::query_as::<_, InviteWithGroupEntity>(
            r#"
            SELECT
                i.id, i.group_id, i.code, i.preset_role, i.max_uses, i.current_uses,
                i.expires_at, i.is_active,
                g.name as group_name, g.icon_emoji as group_icon_emoji,
                (SELECT COUNT(*) FROM group_memberships WHERE group_id = g.id) as member_count
            FROM group_invites i
            JOIN groups g ON i.group_id = g.id
            WHERE i.code = $1 AND i.is_active = true AND g.is_active = true
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// List active invites for a group (non-expired, not fully used).
    pub async fn list_active_invites(
        &self,
        group_id: Uuid,
    ) -> Result<Vec<InviteWithCreatorEntity>, sqlx::Error> {
        let timer = QueryTimer::new("list_active_invites");
        let now = Utc::now();
        let result = sqlx::query_as::<_, InviteWithCreatorEntity>(
            r#"
            SELECT
                i.id, i.group_id, i.code, i.preset_role, i.max_uses, i.current_uses,
                i.expires_at, i.created_by, i.created_at,
                u.display_name as creator_display_name
            FROM group_invites i
            JOIN users u ON i.created_by = u.id
            WHERE i.group_id = $1
              AND i.is_active = true
              AND i.expires_at > $2
              AND i.current_uses < i.max_uses
            ORDER BY i.created_at DESC
            "#,
        )
        .bind(group_id)
        .bind(now)
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Revoke (soft delete) an invite.
    pub async fn revoke_invite(&self, id: Uuid) -> Result<u64, sqlx::Error> {
        let timer = QueryTimer::new("revoke_invite");
        let result = sqlx::query(
            r#"
            UPDATE group_invites
            SET is_active = false
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(result.rows_affected())
    }

    /// Increment use count for an invite (when someone joins).
    pub async fn increment_use_count(&self, id: Uuid) -> Result<GroupInviteEntity, sqlx::Error> {
        let timer = QueryTimer::new("increment_invite_use_count");
        let result = sqlx::query_as::<_, GroupInviteEntity>(
            r#"
            UPDATE group_invites
            SET current_uses = current_uses + 1
            WHERE id = $1 AND is_active = true AND current_uses < max_uses
            RETURNING id, group_id, code, preset_role, max_uses, current_uses, expires_at, created_by, created_at, is_active
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Check if code exists.
    pub async fn code_exists(&self, code: &str) -> Result<bool, sqlx::Error> {
        let timer = QueryTimer::new("check_invite_code_exists");
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(SELECT 1 FROM group_invites WHERE code = $1)
            "#,
        )
        .bind(code)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Generate unique invite code by retrying if collision.
    pub async fn generate_unique_code<F>(&self, generator: F) -> Result<String, sqlx::Error>
    where
        F: Fn() -> String,
    {
        let mut code = generator();
        let mut attempts = 0;

        while self.code_exists(&code).await? {
            code = generator();
            attempts += 1;
            if attempts > 100 {
                return Err(sqlx::Error::Protocol(
                    "Could not generate unique invite code".to_string(),
                ));
            }
        }

        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    // Note: InviteRepository tests require database connection and are covered by integration tests
}
