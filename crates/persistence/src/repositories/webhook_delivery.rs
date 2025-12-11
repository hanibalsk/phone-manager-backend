//! Webhook delivery repository.
//!
//! Story 15.3: Webhook Delivery Logging and Retry
//! Provides data access for webhook delivery tracking and retry management.

use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::webhook_delivery::{
    WebhookDeliveryEntity, MAX_RETRY_ATTEMPTS, RETRY_BACKOFF_SECONDS, STATUS_FAILED,
    STATUS_PENDING, STATUS_SUCCESS,
};

/// Repository for webhook delivery operations.
pub struct WebhookDeliveryRepository {
    pool: PgPool,
}

impl WebhookDeliveryRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new webhook delivery record.
    pub async fn create(
        &self,
        webhook_id: Uuid,
        event_id: Option<Uuid>,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Result<WebhookDeliveryEntity, sqlx::Error> {
        let entity = sqlx::query_as::<_, WebhookDeliveryEntity>(
            r#"
            INSERT INTO webhook_deliveries (webhook_id, event_id, event_type, payload, status, attempts)
            VALUES ($1, $2, $3, $4, 'pending', 0)
            RETURNING id, delivery_id, webhook_id, event_id, event_type, payload, status, attempts,
                      last_attempt_at, next_retry_at, response_code, error_message, created_at
            "#,
        )
        .bind(webhook_id)
        .bind(event_id)
        .bind(event_type)
        .bind(payload)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity)
    }

    /// Update delivery status after an attempt.
    pub async fn update_attempt(
        &self,
        delivery_id: Uuid,
        success: bool,
        response_code: Option<i32>,
        error_message: Option<&str>,
    ) -> Result<WebhookDeliveryEntity, sqlx::Error> {
        let now = Utc::now();

        // Get current attempt count
        let current: (i32,) =
            sqlx::query_as(r#"SELECT attempts FROM webhook_deliveries WHERE delivery_id = $1"#)
                .bind(delivery_id)
                .fetch_one(&self.pool)
                .await?;

        let new_attempts = current.0 + 1;

        let (new_status, next_retry) = if success {
            (STATUS_SUCCESS.to_string(), None)
        } else if new_attempts >= MAX_RETRY_ATTEMPTS {
            (STATUS_FAILED.to_string(), None)
        } else {
            // Calculate next retry time based on backoff schedule
            // Use (new_attempts - 1) as index since:
            // - Attempt 1 failed -> use index 0 (immediate retry)
            // - Attempt 2 failed -> use index 1 (60s)
            // - Attempt 3 failed -> use index 2 (300s)
            // - Attempt 4 failed -> use index 3 (900s)
            let backoff_index =
                ((new_attempts - 1).max(0) as usize).min(RETRY_BACKOFF_SECONDS.len() - 1);
            let backoff_seconds = RETRY_BACKOFF_SECONDS[backoff_index];
            let next_retry_at = now + Duration::seconds(backoff_seconds);
            (STATUS_PENDING.to_string(), Some(next_retry_at))
        };

        let entity = sqlx::query_as::<_, WebhookDeliveryEntity>(
            r#"
            UPDATE webhook_deliveries
            SET status = $2,
                attempts = $3,
                last_attempt_at = $4,
                next_retry_at = $5,
                response_code = $6,
                error_message = $7
            WHERE delivery_id = $1
            RETURNING id, delivery_id, webhook_id, event_id, event_type, payload, status, attempts,
                      last_attempt_at, next_retry_at, response_code, error_message, created_at
            "#,
        )
        .bind(delivery_id)
        .bind(new_status)
        .bind(new_attempts)
        .bind(now)
        .bind(next_retry)
        .bind(response_code)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity)
    }

    /// Find deliveries that are due for retry.
    pub async fn find_pending_retries(
        &self,
        limit: i64,
    ) -> Result<Vec<WebhookDeliveryEntity>, sqlx::Error> {
        let now = Utc::now();
        let entities = sqlx::query_as::<_, WebhookDeliveryEntity>(
            r#"
            SELECT id, delivery_id, webhook_id, event_id, event_type, payload, status, attempts,
                   last_attempt_at, next_retry_at, response_code, error_message, created_at
            FROM webhook_deliveries
            WHERE status = 'pending'
              AND (next_retry_at IS NULL OR next_retry_at <= $1)
            ORDER BY COALESCE(next_retry_at, created_at) ASC
            LIMIT $2
            "#,
        )
        .bind(now)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities)
    }

    /// Find delivery by ID.
    pub async fn find_by_delivery_id(
        &self,
        delivery_id: Uuid,
    ) -> Result<Option<WebhookDeliveryEntity>, sqlx::Error> {
        let entity = sqlx::query_as::<_, WebhookDeliveryEntity>(
            r#"
            SELECT id, delivery_id, webhook_id, event_id, event_type, payload, status, attempts,
                   last_attempt_at, next_retry_at, response_code, error_message, created_at
            FROM webhook_deliveries
            WHERE delivery_id = $1
            "#,
        )
        .bind(delivery_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity)
    }

    /// Find deliveries by webhook ID.
    pub async fn find_by_webhook_id(
        &self,
        webhook_id: Uuid,
        limit: i64,
    ) -> Result<Vec<WebhookDeliveryEntity>, sqlx::Error> {
        let entities = sqlx::query_as::<_, WebhookDeliveryEntity>(
            r#"
            SELECT id, delivery_id, webhook_id, event_id, event_type, payload, status, attempts,
                   last_attempt_at, next_retry_at, response_code, error_message, created_at
            FROM webhook_deliveries
            WHERE webhook_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(webhook_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities)
    }

    /// Delete deliveries older than retention period.
    pub async fn delete_old_deliveries(&self, retention_days: i32) -> Result<u64, sqlx::Error> {
        let cutoff = Utc::now() - Duration::days(retention_days as i64);
        let result = sqlx::query(
            r#"
            DELETE FROM webhook_deliveries
            WHERE created_at < $1
            "#,
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Postpone a delivery retry to a later time (e.g., when circuit breaker is open).
    pub async fn postpone_retry(
        &self,
        delivery_id: Uuid,
        next_retry_at: chrono::DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE webhook_deliveries
            SET next_retry_at = $2
            WHERE delivery_id = $1 AND status = 'pending'
            "#,
        )
        .bind(delivery_id)
        .bind(next_retry_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Count pending deliveries for a webhook.
    pub async fn count_pending_by_webhook_id(&self, webhook_id: Uuid) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM webhook_deliveries
            WHERE webhook_id = $1 AND status = 'pending'
            "#,
        )
        .bind(webhook_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    /// Get delivery statistics.
    pub async fn get_stats(&self) -> Result<DeliveryStats, sqlx::Error> {
        let stats: DeliveryStats = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'pending') as pending_count,
                COUNT(*) FILTER (WHERE status = 'success') as success_count,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_count
            FROM webhook_deliveries
            WHERE created_at > NOW() - INTERVAL '24 hours'
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }

    /// List deliveries for a webhook with pagination and optional status filter.
    pub async fn list_by_webhook_id(
        &self,
        webhook_id: Uuid,
        status_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WebhookDeliveryEntity>, sqlx::Error> {
        let entities = sqlx::query_as::<_, WebhookDeliveryEntity>(
            r#"
            SELECT id, delivery_id, webhook_id, event_id, event_type, payload, status, attempts,
                   last_attempt_at, next_retry_at, response_code, error_message, created_at
            FROM webhook_deliveries
            WHERE webhook_id = $1
              AND ($2::TEXT IS NULL OR status = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(webhook_id)
        .bind(status_filter)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities)
    }

    /// Count deliveries for a webhook with optional status filter.
    pub async fn count_by_webhook_id(
        &self,
        webhook_id: Uuid,
        status_filter: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM webhook_deliveries
            WHERE webhook_id = $1
              AND ($2::TEXT IS NULL OR status = $2)
            "#,
        )
        .bind(webhook_id)
        .bind(status_filter)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    /// Get delivery statistics for a specific webhook.
    pub async fn get_webhook_stats(
        &self,
        webhook_id: Uuid,
        hours: i32,
    ) -> Result<WebhookDeliveryStats, sqlx::Error> {
        let stats: WebhookDeliveryStats = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total_count,
                COUNT(*) FILTER (WHERE status = 'pending') as pending_count,
                COUNT(*) FILTER (WHERE status = 'success') as success_count,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_count
            FROM webhook_deliveries
            WHERE webhook_id = $1
              AND created_at > NOW() - make_interval(hours => $2)
            "#,
        )
        .bind(webhook_id)
        .bind(hours)
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }

    /// Reset a delivery for retry by setting status to pending and clearing next_retry_at.
    pub async fn reset_for_retry(
        &self,
        delivery_id: Uuid,
    ) -> Result<Option<WebhookDeliveryEntity>, sqlx::Error> {
        let entity = sqlx::query_as::<_, WebhookDeliveryEntity>(
            r#"
            UPDATE webhook_deliveries
            SET status = 'pending',
                next_retry_at = NULL,
                error_message = NULL
            WHERE delivery_id = $1
              AND status = 'failed'
            RETURNING id, delivery_id, webhook_id, event_id, event_type, payload, status, attempts,
                      last_attempt_at, next_retry_at, response_code, error_message, created_at
            "#,
        )
        .bind(delivery_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity)
    }
}

/// Delivery statistics.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DeliveryStats {
    pub pending_count: Option<i64>,
    pub success_count: Option<i64>,
    pub failed_count: Option<i64>,
}

/// Webhook-specific delivery statistics.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WebhookDeliveryStats {
    pub total_count: Option<i64>,
    pub pending_count: Option<i64>,
    pub success_count: Option<i64>,
    pub failed_count: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_schedule() {
        // Verify backoff schedule is correct
        assert_eq!(RETRY_BACKOFF_SECONDS[0], 0); // Immediate first attempt
        assert_eq!(RETRY_BACKOFF_SECONDS[1], 60); // 1 minute
        assert_eq!(RETRY_BACKOFF_SECONDS[2], 300); // 5 minutes
        assert_eq!(RETRY_BACKOFF_SECONDS[3], 900); // 15 minutes
    }

    #[test]
    fn test_max_retries() {
        assert_eq!(MAX_RETRY_ATTEMPTS, 4);
    }
}
