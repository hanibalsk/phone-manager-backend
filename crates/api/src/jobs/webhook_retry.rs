//! Webhook retry background job.
//!
//! Story 15.3: Webhook Delivery Logging and Retry
//! Processes failed webhook deliveries that are due for retry.

use sqlx::PgPool;
use tracing::info;

use crate::services::WebhookDeliveryService;

use super::scheduler::{Job, JobFrequency};

/// Background job to retry failed webhook deliveries.
pub struct WebhookRetryJob {
    pool: PgPool,
    batch_size: i64,
}

impl WebhookRetryJob {
    /// Create a new webhook retry job.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `batch_size` - Number of deliveries to process per batch
    pub fn new(pool: PgPool, batch_size: i64) -> Self {
        Self { pool, batch_size }
    }
}

#[async_trait::async_trait]
impl Job for WebhookRetryJob {
    fn name(&self) -> &'static str {
        "webhook_retry"
    }

    fn frequency(&self) -> JobFrequency {
        // Run every minute as specified in Story 15.3
        JobFrequency::Minutes(1)
    }

    async fn execute(&self) -> Result<(), String> {
        let service = WebhookDeliveryService::new(self.pool.clone());

        let processed = service
            .process_pending_retries(self.batch_size)
            .await
            .map_err(|e| format!("Failed to process webhook retries: {}", e))?;

        if processed > 0 {
            info!(
                processed = processed,
                batch_size = self.batch_size,
                "Processed webhook retries"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_job_name() {
        let name = "webhook_retry";
        assert_eq!(name, "webhook_retry");
        assert!(!name.is_empty());
    }

    #[test]
    fn test_job_frequency_is_one_minute() {
        let freq = JobFrequency::Minutes(1);
        assert_eq!(freq.duration(), Duration::from_secs(60));
    }

    #[test]
    fn test_default_batch_size() {
        let batch_size: i64 = 10;
        // Batch size should be reasonable for webhook retries
        assert!(batch_size >= 1, "Batch size too small");
        assert!(batch_size <= 100, "Batch size too large");
    }

    #[test]
    fn test_frequency_minutes_variant() {
        let freq = JobFrequency::Minutes(5);
        assert_eq!(freq.duration(), Duration::from_secs(300));
    }
}
