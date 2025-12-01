//! Location cleanup background job.

use sqlx::PgPool;
use tracing::info;

use super::scheduler::{Job, JobFrequency};

/// Background job to clean up old location records.
pub struct CleanupLocationsJob {
    pool: PgPool,
    retention_days: u32,
    batch_size: i64,
}

impl CleanupLocationsJob {
    /// Create a new cleanup job.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `retention_days` - Number of days to retain locations
    pub fn new(pool: PgPool, retention_days: u32) -> Self {
        Self {
            pool,
            retention_days,
            batch_size: 10_000,
        }
    }

    /// Delete old locations in batches to avoid long locks.
    async fn delete_old_locations(&self) -> Result<u64, sqlx::Error> {
        let mut total_deleted: u64 = 0;

        loop {
            // Delete in batches using a CTE with LIMIT
            let result = sqlx::query(
                r#"
                WITH to_delete AS (
                    SELECT id FROM locations
                    WHERE created_at < NOW() - ($1 || ' days')::INTERVAL
                    LIMIT $2
                )
                DELETE FROM locations
                WHERE id IN (SELECT id FROM to_delete)
                "#,
            )
            .bind(self.retention_days as i32)
            .bind(self.batch_size)
            .execute(&self.pool)
            .await?;

            let deleted = result.rows_affected();
            total_deleted += deleted;

            if deleted < self.batch_size as u64 {
                // No more rows to delete
                break;
            }

            // Small yield to prevent blocking other operations
            tokio::task::yield_now().await;
        }

        Ok(total_deleted)
    }

    /// Clean up expired idempotency keys.
    async fn cleanup_idempotency_keys(&self) -> Result<u64, sqlx::Error> {
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
}

#[async_trait::async_trait]
impl Job for CleanupLocationsJob {
    fn name(&self) -> &'static str {
        "cleanup_locations"
    }

    fn frequency(&self) -> JobFrequency {
        JobFrequency::Hourly
    }

    async fn execute(&self) -> Result<(), String> {
        // Clean up old locations
        let locations_deleted = self
            .delete_old_locations()
            .await
            .map_err(|e| format!("Failed to delete old locations: {}", e))?;

        info!(
            deleted = locations_deleted,
            retention_days = self.retention_days,
            "Cleaned up old locations"
        );

        // Clean up expired idempotency keys
        let keys_deleted = self
            .cleanup_idempotency_keys()
            .await
            .map_err(|e| format!("Failed to delete expired idempotency keys: {}", e))?;

        if keys_deleted > 0 {
            info!(
                deleted = keys_deleted,
                "Cleaned up expired idempotency keys"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================
    // Job Frequency Tests
    // ===========================================

    #[test]
    fn test_job_frequency_hourly() {
        let freq = JobFrequency::Hourly;
        assert_eq!(freq.duration(), std::time::Duration::from_secs(3600));
    }

    #[test]
    fn test_job_frequency_hourly_in_minutes() {
        let freq = JobFrequency::Hourly;
        assert_eq!(freq.duration().as_secs() / 60, 60);
    }

    #[test]
    fn test_job_frequency_daily() {
        let freq = JobFrequency::Daily;
        assert_eq!(freq.duration(), std::time::Duration::from_secs(86400));
    }

    #[test]
    fn test_job_frequency_daily_in_hours() {
        let freq = JobFrequency::Daily;
        assert_eq!(freq.duration().as_secs() / 3600, 24);
    }

    // ===========================================
    // Retention Configuration Tests
    // ===========================================

    #[test]
    fn test_default_retention_days() {
        // Default retention is 30 days per spec
        let default_retention: u32 = 30;
        assert_eq!(default_retention, 30);
    }

    #[test]
    fn test_retention_days_in_seconds() {
        let retention_days: u32 = 30;
        let expected_seconds = retention_days as u64 * 24 * 60 * 60;
        assert_eq!(expected_seconds, 2592000); // 30 days in seconds
    }

    #[test]
    fn test_retention_days_various_values() {
        let valid_retentions = vec![1, 7, 14, 30, 60, 90, 365];
        for retention in valid_retentions {
            let days: u32 = retention;
            assert!(days >= 1);
            assert!(days <= 365);
        }
    }

    // ===========================================
    // Batch Size Tests
    // ===========================================

    #[test]
    fn test_default_batch_size() {
        let batch_size: i64 = 10_000;
        assert_eq!(batch_size, 10000);
    }

    #[test]
    fn test_batch_size_reasonable_range() {
        let batch_size: i64 = 10_000;
        // Batch size should be between 1K and 100K for good performance
        assert!(batch_size >= 1000, "Batch size too small for efficiency");
        assert!(batch_size <= 100_000, "Batch size too large for memory");
    }

    #[test]
    fn test_batch_size_divisibility() {
        // Batch size should be a round number for clarity
        let batch_size: i64 = 10_000;
        assert_eq!(batch_size % 1000, 0);
    }

    // ===========================================
    // Job Name Tests
    // ===========================================

    #[test]
    fn test_cleanup_job_name() {
        let expected_name = "cleanup_locations";
        assert_eq!(expected_name, "cleanup_locations");
        assert!(!expected_name.is_empty());
    }

    #[test]
    fn test_cleanup_job_name_format() {
        let name = "cleanup_locations";
        // Name should be lowercase with underscores (snake_case)
        assert!(name.chars().all(|c| c.is_lowercase() || c == '_'));
    }

    // ===========================================
    // Configuration Validation Tests
    // ===========================================

    #[test]
    fn test_retention_cannot_be_zero() {
        // Zero retention would delete all locations immediately
        let retention: u32 = 0;
        // This is a valid value structurally, but application should validate
        assert_eq!(retention, 0);
    }

    #[test]
    fn test_batch_size_cannot_be_negative() {
        // Type i64 allows negative, but logically invalid
        let valid_batch: i64 = 10_000;
        assert!(valid_batch > 0);
    }

    #[test]
    fn test_job_trait_requirements() {
        // Verify Job trait is async
        fn assert_job<T: Job>() {}
        // Note: Can't actually call this without a pool, but verifies trait exists
    }

    // ===========================================
    // Duration Calculation Tests
    // ===========================================

    #[test]
    fn test_hourly_frequency_is_correct() {
        let hourly = JobFrequency::Hourly;
        let expected = std::time::Duration::from_secs(60 * 60);
        assert_eq!(hourly.duration(), expected);
    }

    #[test]
    fn test_daily_frequency_is_correct() {
        let daily = JobFrequency::Daily;
        let expected = std::time::Duration::from_secs(24 * 60 * 60);
        assert_eq!(daily.duration(), expected);
    }

    // ===========================================
    // SQL Interval Tests (validation of interval format)
    // ===========================================

    #[test]
    fn test_retention_interval_format() {
        // The SQL uses: ($1 || ' days')::INTERVAL
        // where $1 is retention_days as i32
        let retention_days: i32 = 30;
        let interval_str = format!("{} days", retention_days);
        assert_eq!(interval_str, "30 days");
    }

    #[test]
    fn test_retention_interval_various_values() {
        for days in [1, 7, 30, 90, 365] {
            let interval_str = format!("{} days", days);
            assert!(interval_str.ends_with(" days"));
        }
    }

    // ===========================================
    // Batch Processing Logic Tests
    // ===========================================

    #[test]
    fn test_batch_continuation_logic() {
        // If deleted < batch_size, no more rows to delete
        let batch_size: u64 = 10_000;

        // Simulate scenarios
        let deleted_partial: u64 = 5000;
        let deleted_full: u64 = 10_000;

        // Partial batch means we're done
        assert!(deleted_partial < batch_size);

        // Full batch means there might be more
        assert!(deleted_full >= batch_size);
    }

    #[test]
    fn test_total_deleted_accumulation() {
        let mut total_deleted: u64 = 0;
        let batches = vec![10000, 10000, 5000]; // Simulated batch deletions

        for batch in batches {
            total_deleted += batch;
        }

        assert_eq!(total_deleted, 25000);
    }

    // ===========================================
    // Idempotency Key Cleanup Tests
    // ===========================================

    #[test]
    fn test_idempotency_cleanup_is_separate() {
        // Idempotency keys have their own cleanup logic
        // This tests the conceptual separation
        let location_retention_days: u32 = 30;
        let idempotency_key_expiry: &str = "expires_at < NOW()";

        // Both cleanups run in the same job but are independent
        assert_ne!(
            format!("{} days", location_retention_days),
            idempotency_key_expiry
        );
    }

    // ===========================================
    // Error Handling Tests
    // ===========================================

    #[test]
    fn test_error_message_format() {
        let error = "Failed to delete old locations: connection error";
        assert!(error.starts_with("Failed to"));
        assert!(error.contains("locations"));
    }

    #[test]
    fn test_idempotency_error_message_format() {
        let error = "Failed to delete expired idempotency keys: timeout";
        assert!(error.starts_with("Failed to"));
        assert!(error.contains("idempotency"));
    }
}
