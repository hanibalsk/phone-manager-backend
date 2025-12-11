//! Repository for registration invite database operations.

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::RegistrationInviteEntity;

/// Repository for registration invite operations.
#[derive(Clone)]
pub struct RegistrationInviteRepository {
    pool: PgPool,
}

impl RegistrationInviteRepository {
    /// Creates a new registration invite repository.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Finds a registration invite by its token.
    ///
    /// Returns `None` if no invite with the given token exists.
    pub async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<RegistrationInviteEntity>, sqlx::Error> {
        sqlx::query_as::<_, RegistrationInviteEntity>(
            r#"
            SELECT id, token, email, created_by, expires_at, used_at, used_by, created_at, note
            FROM registration_invites
            WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
    }

    /// Creates a new registration invite.
    ///
    /// Returns the created invite entity.
    pub async fn create(
        &self,
        token: &str,
        email: Option<&str>,
        created_by: Option<Uuid>,
        expires_at: DateTime<Utc>,
        note: Option<&str>,
    ) -> Result<RegistrationInviteEntity, sqlx::Error> {
        sqlx::query_as::<_, RegistrationInviteEntity>(
            r#"
            INSERT INTO registration_invites (token, email, created_by, expires_at, note)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, token, email, created_by, expires_at, used_at, used_by, created_at, note
            "#,
        )
        .bind(token)
        .bind(email)
        .bind(created_by)
        .bind(expires_at)
        .bind(note)
        .fetch_one(&self.pool)
        .await
    }

    /// Claims an invite atomically without associating a user yet.
    ///
    /// This uses `AND used_at IS NULL` to prevent race conditions where
    /// two concurrent registrations could both try to use the same invite.
    /// The `used_by` field is left NULL and should be set via `set_used_by`
    /// after user creation succeeds.
    ///
    /// Returns `true` if the invite was successfully claimed,
    /// `false` if it was already claimed (race condition detected).
    pub async fn claim(&self, invite_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE registration_invites
            SET used_at = NOW()
            WHERE id = $1 AND used_at IS NULL
            "#,
        )
        .bind(invite_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Sets the user who used an already-claimed invite.
    ///
    /// This should be called after user creation succeeds to record
    /// which user consumed the invite for auditing purposes.
    pub async fn set_used_by(&self, invite_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE registration_invites
            SET used_by = $2
            WHERE id = $1
            "#,
        )
        .bind(invite_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Marks an invite as used by a user atomically (single operation).
    ///
    /// This uses `AND used_at IS NULL` to prevent race conditions where
    /// two concurrent registrations could both succeed with the same invite.
    ///
    /// Returns `true` if the invite was successfully marked as used,
    /// `false` if it was already used (race condition detected).
    pub async fn mark_used(&self, invite_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE registration_invites
            SET used_at = NOW(), used_by = $2
            WHERE id = $1 AND used_at IS NULL
            "#,
        )
        .bind(invite_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Lists all invites, optionally filtered by usage status.
    ///
    /// If `include_used` is false, only unused invites are returned.
    pub async fn list(
        &self,
        include_used: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RegistrationInviteEntity>, sqlx::Error> {
        let query = if include_used {
            r#"
            SELECT id, token, email, created_by, expires_at, used_at, used_by, created_at, note
            FROM registration_invites
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        } else {
            r#"
            SELECT id, token, email, created_by, expires_at, used_at, used_by, created_at, note
            FROM registration_invites
            WHERE used_at IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        };

        sqlx::query_as::<_, RegistrationInviteEntity>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
    }

    /// Deletes an invite by ID.
    ///
    /// Returns true if an invite was deleted.
    pub async fn delete(&self, invite_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM registration_invites
            WHERE id = $1
            "#,
        )
        .bind(invite_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Deletes expired invites.
    ///
    /// Returns the number of deleted invites.
    pub async fn delete_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM registration_invites
            WHERE expires_at < NOW() AND used_at IS NULL
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

/// Generate a secure invite token.
pub fn generate_invite_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789";
    let mut rng = rand::thread_rng();

    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Default invite expiration (7 days).
pub fn default_expiration() -> DateTime<Utc> {
    Utc::now() + Duration::days(7)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_invite_token_length() {
        let token = generate_invite_token();
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_generate_invite_token_unique() {
        let token1 = generate_invite_token();
        let token2 = generate_invite_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_invite_token_charset() {
        let token = generate_invite_token();
        // Should not contain confusing characters (0, O, 1, l, I)
        assert!(!token.contains('0'));
        assert!(!token.contains('O'));
        assert!(!token.contains('1'));
        assert!(!token.contains('l'));
        assert!(!token.contains('I'));
    }

    #[test]
    fn test_default_expiration() {
        let expiration = default_expiration();
        let now = Utc::now();
        let diff = expiration - now;
        // Should be approximately 7 days (within a few seconds)
        assert!(diff.num_days() >= 6 && diff.num_days() <= 7);
    }
}
