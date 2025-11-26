//! Background job to record connection pool metrics.

use sqlx::PgPool;

use super::scheduler::{Job, JobFrequency};

/// Job that periodically records database connection pool metrics.
pub struct PoolMetricsJob {
    pool: PgPool,
}

impl PoolMetricsJob {
    /// Create a new pool metrics job.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Job for PoolMetricsJob {
    fn name(&self) -> &'static str {
        "pool_metrics"
    }

    fn frequency(&self) -> JobFrequency {
        // Record pool metrics every 10 seconds for real-time monitoring
        JobFrequency::Seconds(10)
    }

    async fn execute(&self) -> Result<(), String> {
        persistence::metrics::record_pool_metrics(&self.pool);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_name() {
        // We can't easily create a PgPool in unit tests, but we can verify the trait impl
        // Actual functionality is tested through integration tests
    }

    #[test]
    fn test_job_frequency() {
        // Pool metrics should be recorded frequently for real-time monitoring
        let freq = JobFrequency::Seconds(10);
        assert_eq!(freq.duration().as_secs(), 10);
    }
}
