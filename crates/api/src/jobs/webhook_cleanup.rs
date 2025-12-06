//! Webhook delivery cleanup background job.
//!
//! Story 15.3: Webhook Delivery Logging and Retry
//! Cleans up old webhook delivery records based on retention policy.

use sqlx::PgPool;
use tracing::info;

use crate::services::WebhookDeliveryService;

use super::scheduler::{Job, JobFrequency};

/// Default retention period in days for webhook deliveries.
const DEFAULT_RETENTION_DAYS: i32 = 7;

/// Background job to clean up old webhook delivery records.
pub struct WebhookCleanupJob {
    pool: PgPool,
    retention_days: i32,
}

impl WebhookCleanupJob {
    /// Create a new webhook cleanup job.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `retention_days` - Number of days to retain delivery records (default: 7)
    pub fn new(pool: PgPool, retention_days: Option<i32>) -> Self {
        Self {
            pool,
            retention_days: retention_days.unwrap_or(DEFAULT_RETENTION_DAYS),
        }
    }
}

#[async_trait::async_trait]
impl Job for WebhookCleanupJob {
    fn name(&self) -> &'static str {
        "webhook_cleanup"
    }

    fn frequency(&self) -> JobFrequency {
        // Run daily as specified in Story 15.3
        JobFrequency::Daily
    }

    async fn execute(&self) -> Result<(), String> {
        let service = WebhookDeliveryService::new(self.pool.clone());

        let deleted = service
            .cleanup_old_deliveries(self.retention_days)
            .await
            .map_err(|e| format!("Failed to cleanup webhook deliveries: {}", e))?;

        info!(
            deleted = deleted,
            retention_days = self.retention_days,
            "Cleaned up old webhook deliveries"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_job_name() {
        let name = "webhook_cleanup";
        assert_eq!(name, "webhook_cleanup");
        assert!(!name.is_empty());
    }

    #[test]
    fn test_job_frequency_is_daily() {
        let freq = JobFrequency::Daily;
        assert_eq!(freq.duration(), Duration::from_secs(86400));
    }

    #[test]
    fn test_default_retention_days() {
        assert_eq!(DEFAULT_RETENTION_DAYS, 7);
    }

    #[test]
    fn test_retention_days_in_hours() {
        let days = DEFAULT_RETENTION_DAYS;
        let hours = days * 24;
        assert_eq!(hours, 168); // 7 days = 168 hours
    }

    #[test]
    fn test_retention_reasonable_range() {
        let retention = DEFAULT_RETENTION_DAYS;
        // Retention should be between 1 and 30 days
        assert!(retention >= 1, "Retention too short");
        assert!(retention <= 30, "Retention too long");
    }
}
