//! Repository for organization settings operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::OrganizationSettingsEntity;

/// Repository for organization settings database operations.
#[derive(Clone)]
pub struct OrganizationSettingsRepository {
    pool: PgPool,
}

impl OrganizationSettingsRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Gets settings for an organization.
    /// Returns None if settings don't exist yet.
    pub async fn get_by_organization_id(
        &self,
        organization_id: Uuid,
    ) -> Result<Option<OrganizationSettingsEntity>, sqlx::Error> {
        sqlx::query_as::<_, OrganizationSettingsEntity>(
            r#"
            SELECT id, organization_id, unlock_pin_hash, default_daily_limit_minutes,
                   notifications_enabled, auto_approve_unlock_requests, created_at, updated_at
            FROM organization_settings
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Gets settings for an organization, creating defaults if they don't exist.
    pub async fn get_or_create(
        &self,
        organization_id: Uuid,
    ) -> Result<OrganizationSettingsEntity, sqlx::Error> {
        // Try to get existing settings first
        if let Some(settings) = self.get_by_organization_id(organization_id).await? {
            return Ok(settings);
        }

        // Create default settings
        self.create_default(organization_id).await
    }

    /// Creates default settings for an organization.
    pub async fn create_default(
        &self,
        organization_id: Uuid,
    ) -> Result<OrganizationSettingsEntity, sqlx::Error> {
        sqlx::query_as::<_, OrganizationSettingsEntity>(
            r#"
            INSERT INTO organization_settings (organization_id)
            VALUES ($1)
            ON CONFLICT (organization_id) DO UPDATE SET updated_at = NOW()
            RETURNING id, organization_id, unlock_pin_hash, default_daily_limit_minutes,
                      notifications_enabled, auto_approve_unlock_requests, created_at, updated_at
            "#,
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
    }

    /// Updates organization settings.
    /// Uses upsert pattern: creates if not exists, updates if exists.
    pub async fn upsert(
        &self,
        organization_id: Uuid,
        unlock_pin_hash: Option<String>,
        default_daily_limit_minutes: i32,
        notifications_enabled: bool,
        auto_approve_unlock_requests: bool,
    ) -> Result<OrganizationSettingsEntity, sqlx::Error> {
        sqlx::query_as::<_, OrganizationSettingsEntity>(
            r#"
            INSERT INTO organization_settings (
                organization_id, unlock_pin_hash, default_daily_limit_minutes,
                notifications_enabled, auto_approve_unlock_requests
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (organization_id) DO UPDATE SET
                unlock_pin_hash = EXCLUDED.unlock_pin_hash,
                default_daily_limit_minutes = EXCLUDED.default_daily_limit_minutes,
                notifications_enabled = EXCLUDED.notifications_enabled,
                auto_approve_unlock_requests = EXCLUDED.auto_approve_unlock_requests,
                updated_at = NOW()
            RETURNING id, organization_id, unlock_pin_hash, default_daily_limit_minutes,
                      notifications_enabled, auto_approve_unlock_requests, created_at, updated_at
            "#,
        )
        .bind(organization_id)
        .bind(unlock_pin_hash)
        .bind(default_daily_limit_minutes)
        .bind(notifications_enabled)
        .bind(auto_approve_unlock_requests)
        .fetch_one(&self.pool)
        .await
    }

    /// Updates only the unlock PIN hash.
    pub async fn update_pin_hash(
        &self,
        organization_id: Uuid,
        unlock_pin_hash: Option<String>,
    ) -> Result<OrganizationSettingsEntity, sqlx::Error> {
        // Ensure settings exist first
        let current = self.get_or_create(organization_id).await?;

        sqlx::query_as::<_, OrganizationSettingsEntity>(
            r#"
            UPDATE organization_settings
            SET unlock_pin_hash = $2, updated_at = NOW()
            WHERE organization_id = $1
            RETURNING id, organization_id, unlock_pin_hash, default_daily_limit_minutes,
                      notifications_enabled, auto_approve_unlock_requests, created_at, updated_at
            "#,
        )
        .bind(organization_id)
        .bind(unlock_pin_hash)
        .fetch_one(&self.pool)
        .await
        .or_else(|_| Ok(current))
    }

    /// Deletes settings for an organization.
    pub async fn delete(&self, organization_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM organization_settings
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would go here with a test database
    // For now, we just verify the repository compiles correctly

    #[test]
    fn test_repository_new() {
        // This test just verifies the struct is properly defined
        // Actual integration tests would require a database connection
    }
}
