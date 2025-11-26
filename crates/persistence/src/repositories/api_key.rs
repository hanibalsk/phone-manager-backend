//! Repository for API key database operations.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::entities::ApiKeyEntity;

/// Repository for API key operations.
#[derive(Clone)]
pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    /// Creates a new API key repository.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Finds an API key by its hash.
    ///
    /// Returns `None` if no key with the given hash exists.
    pub async fn find_by_key_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<ApiKeyEntity>, sqlx::Error> {
        let result = sqlx::query_as::<_, ApiKeyEntity>(
            r#"
            SELECT id, key_hash, key_prefix, name, is_active, is_admin,
                   last_used_at, created_at, expires_at
            FROM api_keys
            WHERE key_hash = $1
            "#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Updates the last_used_at timestamp for an API key.
    ///
    /// This is typically called asynchronously after successful authentication.
    pub async fn update_last_used(&self, key_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE api_keys
            SET last_used_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(key_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Checks if an API key is valid for authentication.
    ///
    /// Returns `true` if the key is active and not expired.
    pub fn is_key_valid(key: &ApiKeyEntity) -> bool {
        if !key.is_active {
            return false;
        }

        if let Some(expires_at) = key.expires_at {
            if expires_at < Utc::now() {
                return false;
            }
        }

        true
    }

    /// Checks if an API key is valid at a specific time.
    ///
    /// Useful for testing with fixed timestamps.
    pub fn is_key_valid_at(key: &ApiKeyEntity, at: DateTime<Utc>) -> bool {
        if !key.is_active {
            return false;
        }

        if let Some(expires_at) = key.expires_at {
            if expires_at < at {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_test_key(is_active: bool, expires_at: Option<DateTime<Utc>>) -> ApiKeyEntity {
        ApiKeyEntity {
            id: 1,
            key_hash: "test_hash".to_string(),
            key_prefix: "pm_aBcDe".to_string(),
            name: "Test Key".to_string(),
            is_active,
            is_admin: false,
            last_used_at: None,
            created_at: Utc::now(),
            expires_at,
        }
    }

    #[test]
    fn test_is_key_valid_active_no_expiry() {
        let key = make_test_key(true, None);
        assert!(ApiKeyRepository::is_key_valid(&key));
    }

    #[test]
    fn test_is_key_valid_active_future_expiry() {
        let key = make_test_key(true, Some(Utc::now() + Duration::days(30)));
        assert!(ApiKeyRepository::is_key_valid(&key));
    }

    #[test]
    fn test_is_key_valid_active_past_expiry() {
        let key = make_test_key(true, Some(Utc::now() - Duration::days(1)));
        assert!(!ApiKeyRepository::is_key_valid(&key));
    }

    #[test]
    fn test_is_key_valid_inactive() {
        let key = make_test_key(false, None);
        assert!(!ApiKeyRepository::is_key_valid(&key));
    }

    #[test]
    fn test_is_key_valid_inactive_future_expiry() {
        let key = make_test_key(false, Some(Utc::now() + Duration::days(30)));
        assert!(!ApiKeyRepository::is_key_valid(&key));
    }
}
