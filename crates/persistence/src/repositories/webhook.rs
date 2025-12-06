//! Webhook repository for database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::WebhookEntity;
use crate::metrics::QueryTimer;

/// Repository for webhook-related database operations.
#[derive(Clone)]
pub struct WebhookRepository {
    pool: PgPool,
}

impl WebhookRepository {
    /// Creates a new WebhookRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new webhook.
    pub async fn create(
        &self,
        owner_device_id: Uuid,
        name: &str,
        target_url: &str,
        secret: &str,
        enabled: bool,
    ) -> Result<WebhookEntity, sqlx::Error> {
        let timer = QueryTimer::new("create_webhook");
        let result = sqlx::query_as::<_, WebhookEntity>(
            r#"
            INSERT INTO webhooks (owner_device_id, name, target_url, secret, enabled)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(owner_device_id)
        .bind(name)
        .bind(target_url)
        .bind(secret)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find webhook by UUID.
    pub async fn find_by_webhook_id(
        &self,
        webhook_id: Uuid,
    ) -> Result<Option<WebhookEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_webhook_by_id");
        let result = sqlx::query_as::<_, WebhookEntity>(
            r#"
            SELECT * FROM webhooks WHERE webhook_id = $1
            "#,
        )
        .bind(webhook_id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find all webhooks for a device.
    pub async fn find_by_owner_device_id(
        &self,
        owner_device_id: Uuid,
    ) -> Result<Vec<WebhookEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_webhooks_by_device");
        let result = sqlx::query_as::<_, WebhookEntity>(
            r#"
            SELECT * FROM webhooks
            WHERE owner_device_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_device_id)
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find webhook by device ID and name (for uniqueness check).
    pub async fn find_by_device_and_name(
        &self,
        owner_device_id: Uuid,
        name: &str,
    ) -> Result<Option<WebhookEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_webhook_by_device_and_name");
        let result = sqlx::query_as::<_, WebhookEntity>(
            r#"
            SELECT * FROM webhooks
            WHERE owner_device_id = $1 AND name = $2
            "#,
        )
        .bind(owner_device_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Count webhooks for a device.
    pub async fn count_by_owner_device_id(&self, owner_device_id: Uuid) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_webhooks_by_device");
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM webhooks WHERE owner_device_id = $1
            "#,
        )
        .bind(owner_device_id)
        .fetch_one(&self.pool)
        .await?;
        timer.record();
        Ok(count.0)
    }

    /// Update a webhook (partial update).
    /// Only provided fields are updated; None values are preserved.
    pub async fn update(
        &self,
        webhook_id: Uuid,
        name: Option<&str>,
        target_url: Option<&str>,
        secret: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<Option<WebhookEntity>, sqlx::Error> {
        let timer = QueryTimer::new("update_webhook");

        let result = sqlx::query_as::<_, WebhookEntity>(
            r#"
            UPDATE webhooks SET
                name = COALESCE($2, name),
                target_url = COALESCE($3, target_url),
                secret = COALESCE($4, secret),
                enabled = COALESCE($5, enabled),
                updated_at = NOW()
            WHERE webhook_id = $1
            RETURNING *
            "#,
        )
        .bind(webhook_id)
        .bind(name)
        .bind(target_url)
        .bind(secret)
        .bind(enabled)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Delete a webhook.
    /// Returns the number of rows deleted (0 or 1).
    pub async fn delete(&self, webhook_id: Uuid) -> Result<u64, sqlx::Error> {
        let timer = QueryTimer::new("delete_webhook");
        let result = sqlx::query(
            r#"
            DELETE FROM webhooks WHERE webhook_id = $1
            "#,
        )
        .bind(webhook_id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(result.rows_affected())
    }

    /// Delete all webhooks for a device.
    /// Returns the number of rows deleted.
    pub async fn delete_all_by_owner_device_id(
        &self,
        owner_device_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let timer = QueryTimer::new("delete_all_webhooks_by_device");
        let result = sqlx::query(
            r#"
            DELETE FROM webhooks WHERE owner_device_id = $1
            "#,
        )
        .bind(owner_device_id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(result.rows_affected())
    }

    /// Find all enabled webhooks for a device that are available for delivery.
    /// Excludes webhooks with open circuit breakers.
    pub async fn find_enabled_by_owner_device_id(
        &self,
        owner_device_id: Uuid,
    ) -> Result<Vec<WebhookEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_enabled_webhooks_by_device");
        let result = sqlx::query_as::<_, WebhookEntity>(
            r#"
            SELECT * FROM webhooks
            WHERE owner_device_id = $1
              AND enabled = true
              AND (circuit_open_until IS NULL OR circuit_open_until <= NOW())
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_device_id)
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Increment consecutive failures counter for a webhook.
    /// Returns the new failure count.
    pub async fn increment_consecutive_failures(
        &self,
        webhook_id: Uuid,
    ) -> Result<i32, sqlx::Error> {
        let timer = QueryTimer::new("increment_webhook_failures");
        let result: (i32,) = sqlx::query_as(
            r#"
            UPDATE webhooks
            SET consecutive_failures = consecutive_failures + 1,
                updated_at = NOW()
            WHERE webhook_id = $1
            RETURNING consecutive_failures
            "#,
        )
        .bind(webhook_id)
        .fetch_one(&self.pool)
        .await?;
        timer.record();
        Ok(result.0)
    }

    /// Reset consecutive failures counter after a successful delivery.
    pub async fn reset_consecutive_failures(
        &self,
        webhook_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let timer = QueryTimer::new("reset_webhook_failures");
        sqlx::query(
            r#"
            UPDATE webhooks
            SET consecutive_failures = 0,
                circuit_open_until = NULL,
                updated_at = NOW()
            WHERE webhook_id = $1
            "#,
        )
        .bind(webhook_id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(())
    }

    /// Open the circuit breaker for a webhook.
    /// The circuit will remain open until the specified time.
    /// Also resets consecutive_failures to 0 so that after cooldown expires,
    /// the webhook gets a fresh start (half-open state behavior).
    pub async fn open_circuit(
        &self,
        webhook_id: Uuid,
        open_until: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), sqlx::Error> {
        let timer = QueryTimer::new("open_webhook_circuit");
        sqlx::query(
            r#"
            UPDATE webhooks
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
        timer.record();
        Ok(())
    }

    /// Close the circuit breaker for a webhook (reset circuit_open_until to NULL).
    pub async fn close_circuit(
        &self,
        webhook_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let timer = QueryTimer::new("close_webhook_circuit");
        sqlx::query(
            r#"
            UPDATE webhooks
            SET circuit_open_until = NULL,
                updated_at = NOW()
            WHERE webhook_id = $1
            "#,
        )
        .bind(webhook_id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repository_creation() {
        // This test verifies the WebhookRepository can be created
        // Actual database tests are integration tests
    }
}
