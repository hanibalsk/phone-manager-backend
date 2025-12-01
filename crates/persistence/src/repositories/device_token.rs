//! Device token repository for database operations.

use chrono::{DateTime, Utc};
use domain::models::DeviceToken;
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::DeviceTokenEntity;

/// Repository for device token database operations.
#[derive(Clone)]
pub struct DeviceTokenRepository {
    pool: PgPool,
}

impl DeviceTokenRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new device token.
    pub async fn create(
        &self,
        device_id: i64,
        organization_id: Uuid,
        token: &str,
        token_prefix: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<DeviceToken, sqlx::Error> {
        let entity = sqlx::query_as::<_, DeviceTokenEntity>(
            r#"
            INSERT INTO device_tokens (device_id, organization_id, token, token_prefix, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, device_id, organization_id, token, token_prefix, expires_at, created_at, last_used_at, revoked_at
            "#,
        )
        .bind(device_id)
        .bind(organization_id)
        .bind(token)
        .bind(token_prefix)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity.into())
    }

    /// Find token by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<DeviceToken>, sqlx::Error> {
        let entity = sqlx::query_as::<_, DeviceTokenEntity>(
            r#"
            SELECT id, device_id, organization_id, token, token_prefix, expires_at, created_at, last_used_at, revoked_at
            FROM device_tokens
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Find token by token string.
    pub async fn find_by_token(&self, token: &str) -> Result<Option<DeviceToken>, sqlx::Error> {
        let entity = sqlx::query_as::<_, DeviceTokenEntity>(
            r#"
            SELECT id, device_id, organization_id, token, token_prefix, expires_at, created_at, last_used_at, revoked_at
            FROM device_tokens
            WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Find valid token by token string (not revoked, not expired).
    pub async fn find_valid_token(&self, token: &str) -> Result<Option<DeviceToken>, sqlx::Error> {
        let entity = sqlx::query_as::<_, DeviceTokenEntity>(
            r#"
            SELECT id, device_id, organization_id, token, token_prefix, expires_at, created_at, last_used_at, revoked_at
            FROM device_tokens
            WHERE token = $1
              AND revoked_at IS NULL
              AND expires_at > NOW()
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Update last_used_at timestamp.
    pub async fn update_last_used(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE device_tokens
            SET last_used_at = NOW()
            WHERE id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Revoke a token (soft delete).
    pub async fn revoke(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE device_tokens
            SET revoked_at = NOW()
            WHERE id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Revoke all tokens for a device.
    pub async fn revoke_all_for_device(&self, device_id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE device_tokens
            SET revoked_at = NOW()
            WHERE device_id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(device_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// List active tokens for a device.
    pub async fn list_active_for_device(
        &self,
        device_id: i64,
    ) -> Result<Vec<DeviceToken>, sqlx::Error> {
        let entities = sqlx::query_as::<_, DeviceTokenEntity>(
            r#"
            SELECT id, device_id, organization_id, token, token_prefix, expires_at, created_at, last_used_at, revoked_at
            FROM device_tokens
            WHERE device_id = $1 AND revoked_at IS NULL AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .bind(device_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_token_repository_new() {
        // This is a compile-time test - repository should be constructable
        // Actual DB tests require integration test setup
    }
}
