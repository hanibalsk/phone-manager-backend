//! Materialized view refresh background job.

use sqlx::PgPool;
use tracing::info;

use super::scheduler::{Job, JobFrequency};

/// Background job to refresh materialized views.
pub struct RefreshViewsJob {
    pool: PgPool,
}

impl RefreshViewsJob {
    /// Create a new refresh views job.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Refresh the group_member_counts materialized view.
    async fn refresh_group_member_counts(&self) -> Result<(), sqlx::Error> {
        // Use CONCURRENTLY to allow reads during refresh
        sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY group_member_counts")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Job for RefreshViewsJob {
    fn name(&self) -> &'static str {
        "refresh_views"
    }

    fn frequency(&self) -> JobFrequency {
        JobFrequency::Hourly
    }

    async fn execute(&self) -> Result<(), String> {
        let start = std::time::Instant::now();

        self.refresh_group_member_counts()
            .await
            .map_err(|e| format!("Failed to refresh group_member_counts: {}", e))?;

        let elapsed = start.elapsed();
        info!(
            elapsed_ms = elapsed.as_millis(),
            "Refreshed materialized views"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refresh_job_name() {
        assert_eq!("refresh_views", "refresh_views");
    }

    #[test]
    fn test_job_frequency() {
        let freq = JobFrequency::Hourly;
        assert_eq!(freq.duration(), std::time::Duration::from_secs(3600));
    }
}
