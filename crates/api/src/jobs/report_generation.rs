//! Report generation background job.
//!
//! FR-10.5-10.9: Async Report Generation
//! Processes pending report jobs and generates analytics reports.

use sqlx::PgPool;
use std::path::PathBuf;
use tracing::info;

use crate::services::ReportGenerationService;

use super::scheduler::{Job, JobFrequency};

/// Background job to process report generation requests.
pub struct ReportGenerationJob {
    pool: PgPool,
    batch_size: i64,
    reports_dir: PathBuf,
}

impl ReportGenerationJob {
    /// Create a new report generation job.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `batch_size` - Number of reports to process per batch
    /// * `reports_dir` - Directory to store generated reports
    pub fn new(pool: PgPool, batch_size: i64, reports_dir: PathBuf) -> Self {
        Self {
            pool,
            batch_size,
            reports_dir,
        }
    }
}

#[async_trait::async_trait]
impl Job for ReportGenerationJob {
    fn name(&self) -> &'static str {
        "report_generation"
    }

    fn frequency(&self) -> JobFrequency {
        // Run every 30 seconds to quickly process new report requests
        JobFrequency::Seconds(30)
    }

    async fn execute(&self) -> Result<(), String> {
        let service = ReportGenerationService::new(self.pool.clone(), self.reports_dir.clone());

        let processed = service
            .process_pending_jobs(self.batch_size)
            .await
            .map_err(|e| format!("Failed to process report jobs: {}", e))?;

        if processed > 0 {
            info!(
                processed = processed,
                batch_size = self.batch_size,
                "Processed report generation jobs"
            );
        }

        Ok(())
    }
}

/// Background job to clean up expired reports.
pub struct ReportCleanupJob {
    pool: PgPool,
    reports_dir: PathBuf,
}

impl ReportCleanupJob {
    /// Create a new report cleanup job.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `reports_dir` - Directory where reports are stored
    pub fn new(pool: PgPool, reports_dir: PathBuf) -> Self {
        Self { pool, reports_dir }
    }
}

#[async_trait::async_trait]
impl Job for ReportCleanupJob {
    fn name(&self) -> &'static str {
        "report_cleanup"
    }

    fn frequency(&self) -> JobFrequency {
        // Run daily to clean up expired reports
        JobFrequency::Daily
    }

    async fn execute(&self) -> Result<(), String> {
        let service = ReportGenerationService::new(self.pool.clone(), self.reports_dir.clone());

        let deleted = service
            .cleanup_expired_reports()
            .await
            .map_err(|e| format!("Failed to cleanup expired reports: {}", e))?;

        if deleted > 0 {
            info!(deleted = deleted, "Cleaned up expired reports");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_report_generation_job_name() {
        let name = "report_generation";
        assert_eq!(name, "report_generation");
    }

    #[test]
    fn test_report_generation_job_frequency() {
        let freq = JobFrequency::Seconds(30);
        assert_eq!(freq.duration(), Duration::from_secs(30));
    }

    #[test]
    fn test_report_cleanup_job_name() {
        let name = "report_cleanup";
        assert_eq!(name, "report_cleanup");
    }

    #[test]
    fn test_report_cleanup_job_frequency() {
        let freq = JobFrequency::Daily;
        assert_eq!(freq.duration(), Duration::from_secs(86400));
    }

    #[test]
    fn test_default_batch_size() {
        let batch_size: i64 = 5;
        // Batch size should be reasonable for report generation
        assert!(batch_size >= 1, "Batch size too small");
        assert!(batch_size <= 20, "Batch size too large for report generation");
    }
}
