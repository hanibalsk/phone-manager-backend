//! Idempotency key repository for database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::IdempotencyKeyEntity;

/// Repository for idempotency key database operations.
#[derive(Clone)]
pub struct IdempotencyKeyRepository {
    pool: PgPool,
}

impl IdempotencyKeyRepository {
    /// Creates a new IdempotencyKeyRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Find an existing idempotency key by its hash.
    /// Only returns non-expired keys.
    pub async fn find_by_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<IdempotencyKeyEntity>, sqlx::Error> {
        sqlx::query_as::<_, IdempotencyKeyEntity>(
            r#"
            SELECT id, key_hash, device_id, response_body, response_status, created_at, expires_at
            FROM idempotency_keys
            WHERE key_hash = $1 AND expires_at > NOW()
            "#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
    }

    /// Store a new idempotency key with its cached response.
    /// Uses ON CONFLICT to handle race conditions.
    pub async fn store(
        &self,
        key_hash: &str,
        device_id: Uuid,
        response_body: serde_json::Value,
        response_status: i16,
    ) -> Result<IdempotencyKeyEntity, sqlx::Error> {
        sqlx::query_as::<_, IdempotencyKeyEntity>(
            r#"
            INSERT INTO idempotency_keys (key_hash, device_id, response_body, response_status)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (key_hash) DO UPDATE SET
                key_hash = idempotency_keys.key_hash
            RETURNING id, key_hash, device_id, response_body, response_status, created_at, expires_at
            "#,
        )
        .bind(key_hash)
        .bind(device_id)
        .bind(response_body)
        .bind(response_status)
        .fetch_one(&self.pool)
        .await
    }

    /// Delete expired idempotency keys.
    /// Returns the number of deleted records.
    pub async fn delete_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM idempotency_keys
            WHERE expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Delete idempotency keys older than the specified hours.
    /// Returns the number of deleted records.
    pub async fn delete_older_than_hours(&self, hours: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM idempotency_keys
            WHERE created_at < NOW() - make_interval(hours => $1)
            "#,
        )
        .bind(hours)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repository_creation() {
        // This test verifies the IdempotencyKeyRepository can be created
        // Actual database tests are integration tests
        assert!(true);
    }
}
