//! Repository for API key database operations.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

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
                   last_used_at, created_at, expires_at, user_id, description,
                   organization_id
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

    // ============================================
    // Organization-scoped API key methods
    // ============================================

    /// Creates a new API key for an organization.
    ///
    /// The key_hash should be the SHA-256 hash of the raw key.
    /// The key_prefix should be the first 8 characters after `pm_` for identification.
    pub async fn create_for_organization(
        &self,
        org_id: Uuid,
        key_hash: &str,
        key_prefix: &str,
        name: &str,
        description: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<ApiKeyEntity, sqlx::Error> {
        let result = sqlx::query_as::<_, ApiKeyEntity>(
            r#"
            INSERT INTO api_keys (key_hash, key_prefix, name, description, expires_at, organization_id, is_active, is_admin)
            VALUES ($1, $2, $3, $4, $5, $6, true, false)
            RETURNING id, key_hash, key_prefix, name, is_active, is_admin,
                      last_used_at, created_at, expires_at, user_id, description, organization_id
            "#,
        )
        .bind(key_hash)
        .bind(key_prefix)
        .bind(name)
        .bind(description)
        .bind(expires_at)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Lists all API keys for an organization.
    ///
    /// If `include_inactive` is false, only active keys are returned.
    pub async fn list_by_organization(
        &self,
        org_id: Uuid,
        include_inactive: bool,
    ) -> Result<Vec<ApiKeyEntity>, sqlx::Error> {
        let query = if include_inactive {
            r#"
            SELECT id, key_hash, key_prefix, name, is_active, is_admin,
                   last_used_at, created_at, expires_at, user_id, description, organization_id
            FROM api_keys
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#
        } else {
            r#"
            SELECT id, key_hash, key_prefix, name, is_active, is_admin,
                   last_used_at, created_at, expires_at, user_id, description, organization_id
            FROM api_keys
            WHERE organization_id = $1 AND is_active = true
            ORDER BY created_at DESC
            "#
        };

        let result = sqlx::query_as::<_, ApiKeyEntity>(query)
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(result)
    }

    /// Finds an API key by ID and organization.
    ///
    /// Returns `None` if no key with the given ID exists for the organization.
    pub async fn find_by_id_and_org(
        &self,
        key_id: i64,
        org_id: Uuid,
    ) -> Result<Option<ApiKeyEntity>, sqlx::Error> {
        let result = sqlx::query_as::<_, ApiKeyEntity>(
            r#"
            SELECT id, key_hash, key_prefix, name, is_active, is_admin,
                   last_used_at, created_at, expires_at, user_id, description, organization_id
            FROM api_keys
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(key_id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Revokes an API key (soft delete).
    ///
    /// Sets `is_active` to false. Returns true if a key was updated.
    pub async fn revoke(&self, key_id: i64, org_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE api_keys
            SET is_active = false
            WHERE id = $1 AND organization_id = $2 AND is_active = true
            "#,
        )
        .bind(key_id)
        .bind(org_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Updates an API key's metadata (name and/or description).
    ///
    /// Returns the updated key, or `None` if not found.
    pub async fn update(
        &self,
        key_id: i64,
        org_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Option<ApiKeyEntity>, sqlx::Error> {
        // Build dynamic update query based on provided fields
        let result = sqlx::query_as::<_, ApiKeyEntity>(
            r#"
            UPDATE api_keys
            SET name = COALESCE($3, name),
                description = COALESCE($4, description)
            WHERE id = $1 AND organization_id = $2
            RETURNING id, key_hash, key_prefix, name, is_active, is_admin,
                      last_used_at, created_at, expires_at, user_id, description, organization_id
            "#,
        )
        .bind(key_id)
        .bind(org_id)
        .bind(name)
        .bind(description)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Counts the number of API keys for an organization.
    ///
    /// Used to enforce limits on the number of keys per organization.
    pub async fn count_by_organization(&self, org_id: Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM api_keys
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[allow(dead_code)]
    fn make_test_key(is_active: bool, expires_at: Option<DateTime<Utc>>) -> ApiKeyEntity {
        ApiKeyEntity {
            id: 1,
            key_hash: "test_hash".to_string(),
            key_prefix: "pm_aBcDe".to_string(),
            name: Some("Test Key".to_string()),
            is_active,
            is_admin: false,
            last_used_at: None,
            created_at: Utc::now(),
            expires_at,
            user_id: None,
            description: None,
            organization_id: None,
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
