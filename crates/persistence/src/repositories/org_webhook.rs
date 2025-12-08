//! Repository for organization webhook database operations.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::OrgWebhookEntity;

/// Repository for organization webhook operations.
#[derive(Clone)]
pub struct OrgWebhookRepository {
    pool: PgPool,
}

impl OrgWebhookRepository {
    /// Creates a new organization webhook repository.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new organization webhook.
    pub async fn create(
        &self,
        org_id: Uuid,
        name: &str,
        target_url: &str,
        secret: &str,
        event_types: &[String],
    ) -> Result<OrgWebhookEntity, sqlx::Error> {
        sqlx::query_as::<_, OrgWebhookEntity>(
            r#"
            INSERT INTO org_webhooks (organization_id, name, target_url, secret, event_types)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, webhook_id, organization_id, name, target_url, secret, enabled,
                      event_types, consecutive_failures, circuit_open_until, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(name)
        .bind(target_url)
        .bind(secret)
        .bind(event_types)
        .fetch_one(&self.pool)
        .await
    }

    /// Finds a webhook by its UUID and organization.
    pub async fn find_by_id(
        &self,
        webhook_id: Uuid,
        org_id: Uuid,
    ) -> Result<Option<OrgWebhookEntity>, sqlx::Error> {
        sqlx::query_as::<_, OrgWebhookEntity>(
            r#"
            SELECT id, webhook_id, organization_id, name, target_url, secret, enabled,
                   event_types, consecutive_failures, circuit_open_until, created_at, updated_at
            FROM org_webhooks
            WHERE webhook_id = $1 AND organization_id = $2
            "#,
        )
        .bind(webhook_id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Lists all webhooks for an organization.
    pub async fn list_by_organization(
        &self,
        org_id: Uuid,
    ) -> Result<Vec<OrgWebhookEntity>, sqlx::Error> {
        sqlx::query_as::<_, OrgWebhookEntity>(
            r#"
            SELECT id, webhook_id, organization_id, name, target_url, secret, enabled,
                   event_types, consecutive_failures, circuit_open_until, created_at, updated_at
            FROM org_webhooks
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Counts webhooks for an organization.
    pub async fn count_by_organization(&self, org_id: Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM org_webhooks WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }

    /// Checks if a webhook name exists in the organization.
    pub async fn name_exists(&self, org_id: Uuid, name: &str) -> Result<bool, sqlx::Error> {
        let result: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM org_webhooks
                WHERE organization_id = $1 AND LOWER(name) = LOWER($2)
            )
            "#,
        )
        .bind(org_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }

    /// Updates a webhook (partial update).
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        webhook_id: Uuid,
        org_id: Uuid,
        name: Option<&str>,
        target_url: Option<&str>,
        secret: Option<&str>,
        enabled: Option<bool>,
        event_types: Option<&[String]>,
    ) -> Result<Option<OrgWebhookEntity>, sqlx::Error> {
        sqlx::query_as::<_, OrgWebhookEntity>(
            r#"
            UPDATE org_webhooks SET
                name = COALESCE($3, name),
                target_url = COALESCE($4, target_url),
                secret = COALESCE($5, secret),
                enabled = COALESCE($6, enabled),
                event_types = COALESCE($7, event_types),
                updated_at = NOW()
            WHERE webhook_id = $1 AND organization_id = $2
            RETURNING id, webhook_id, organization_id, name, target_url, secret, enabled,
                      event_types, consecutive_failures, circuit_open_until, created_at, updated_at
            "#,
        )
        .bind(webhook_id)
        .bind(org_id)
        .bind(name)
        .bind(target_url)
        .bind(secret)
        .bind(enabled)
        .bind(event_types)
        .fetch_optional(&self.pool)
        .await
    }

    /// Deletes a webhook.
    /// Returns true if a webhook was deleted.
    pub async fn delete(&self, webhook_id: Uuid, org_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM org_webhooks
            WHERE webhook_id = $1 AND organization_id = $2
            "#,
        )
        .bind(webhook_id)
        .bind(org_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Finds all enabled webhooks for an organization that subscribe to an event type.
    /// Excludes webhooks with open circuit breakers.
    pub async fn find_enabled_for_event(
        &self,
        org_id: Uuid,
        event_type: &str,
    ) -> Result<Vec<OrgWebhookEntity>, sqlx::Error> {
        sqlx::query_as::<_, OrgWebhookEntity>(
            r#"
            SELECT id, webhook_id, organization_id, name, target_url, secret, enabled,
                   event_types, consecutive_failures, circuit_open_until, created_at, updated_at
            FROM org_webhooks
            WHERE organization_id = $1
              AND enabled = true
              AND $2 = ANY(event_types)
              AND (circuit_open_until IS NULL OR circuit_open_until <= NOW())
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id)
        .bind(event_type)
        .fetch_all(&self.pool)
        .await
    }

    /// Increments consecutive failures counter for a webhook.
    /// Returns the new failure count.
    pub async fn increment_failures(&self, webhook_id: Uuid) -> Result<i32, sqlx::Error> {
        let result: (i32,) = sqlx::query_as(
            r#"
            UPDATE org_webhooks
            SET consecutive_failures = consecutive_failures + 1,
                updated_at = NOW()
            WHERE webhook_id = $1
            RETURNING consecutive_failures
            "#,
        )
        .bind(webhook_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }

    /// Resets consecutive failures counter after a successful delivery.
    pub async fn reset_failures(&self, webhook_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE org_webhooks
            SET consecutive_failures = 0,
                circuit_open_until = NULL,
                updated_at = NOW()
            WHERE webhook_id = $1
            "#,
        )
        .bind(webhook_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Opens the circuit breaker for a webhook.
    pub async fn open_circuit(
        &self,
        webhook_id: Uuid,
        open_until: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE org_webhooks
            SET circuit_open_until = $2,
                consecutive_failures = 0,
                updated_at = NOW()
            WHERE webhook_id = $1
            "#,
        )
        .bind(webhook_id)
        .bind(open_until)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Closes the circuit breaker for a webhook.
    pub async fn close_circuit(&self, webhook_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE org_webhooks
            SET circuit_open_until = NULL,
                updated_at = NOW()
            WHERE webhook_id = $1
            "#,
        )
        .bind(webhook_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repository_creation() {
        // This test verifies the OrgWebhookRepository structure
        // Actual database tests are integration tests
    }
}
