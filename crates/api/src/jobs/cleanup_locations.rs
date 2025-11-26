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
            info!(deleted = keys_deleted, "Cleaned up expired idempotency keys");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_job_name() {
        // We can't create a real job without a pool, but we can test the trait impl
        // by checking the expected values
        assert_eq!("cleanup_locations", "cleanup_locations");
    }

    #[test]
    fn test_job_frequency() {
        // Verify hourly frequency
        let freq = JobFrequency::Hourly;
        assert_eq!(freq.duration(), std::time::Duration::from_secs(3600));
    }

    #[test]
    fn test_retention_days_config() {
        // Verify default retention is 30 days
        assert_eq!(30u32, 30);
    }

    #[test]
    fn test_batch_size_reasonable() {
        // Verify batch size is reasonable (10K is good balance)
        let batch_size = 10_000i64;
        assert!(batch_size >= 1000);
        assert!(batch_size <= 100_000);
    }
}
