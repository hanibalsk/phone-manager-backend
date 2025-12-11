//! Report generation service.
//!
//! FR-10.5-10.9: Async Report Generation System
//! Handles background generation of user and device analytics reports
//! with CSV and JSON export formats.

use chrono::NaiveDate;
use persistence::entities::ReportJobEntity;
use persistence::repositories::AnalyticsRepository;
use serde::Serialize;
use sqlx::PgPool;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Report generation errors.
#[derive(Error, Debug)]
pub enum ReportGenerationError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Report type not supported: {0}")]
    UnsupportedReportType(String),
}

/// Report format types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Csv,
    Json,
}

impl ReportFormat {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "csv" => ReportFormat::Csv,
            _ => ReportFormat::Json, // Default to JSON
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            ReportFormat::Csv => "csv",
            ReportFormat::Json => "json",
        }
    }
}

/// User analytics report row for export.
#[derive(Debug, Serialize)]
pub struct UserReportRow {
    pub date: String,
    pub active_users: i64,
    pub new_users: i64,
    pub returning_users: i64,
    pub total_sessions: i64,
    pub avg_session_duration_seconds: f64,
}

/// Device analytics report row for export.
#[derive(Debug, Serialize)]
pub struct DeviceReportRow {
    pub date: String,
    pub active_devices: i64,
    pub new_enrollments: i64,
    pub unenrollments: i64,
    pub total_locations_reported: i64,
    pub total_geofence_events: i64,
    pub total_commands_issued: i64,
}

/// Service for generating reports.
pub struct ReportGenerationService {
    pool: PgPool,
    reports_dir: PathBuf,
}

impl ReportGenerationService {
    /// Create a new report generation service.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `reports_dir` - Directory to store generated reports
    pub fn new(pool: PgPool, reports_dir: PathBuf) -> Self {
        Self { pool, reports_dir }
    }

    /// Process pending report jobs.
    ///
    /// Returns the number of jobs processed.
    pub async fn process_pending_jobs(
        &self,
        batch_size: i64,
    ) -> Result<u32, ReportGenerationError> {
        let repo = AnalyticsRepository::new(self.pool.clone());

        let pending = repo.find_pending_report_jobs(batch_size).await?;
        let mut processed = 0u32;

        for job in pending {
            match self.process_job(&repo, &job).await {
                Ok(_) => {
                    processed += 1;
                    info!(
                        job_id = %job.id,
                        report_type = %job.report_type,
                        "Report generated successfully"
                    );
                }
                Err(e) => {
                    error!(
                        job_id = %job.id,
                        report_type = %job.report_type,
                        error = %e,
                        "Failed to generate report"
                    );
                    // Mark job as failed
                    if let Err(update_err) = repo.mark_report_failed(job.id, &e.to_string()).await {
                        error!(
                            job_id = %job.id,
                            error = %update_err,
                            "Failed to mark report as failed"
                        );
                    }
                }
            }
        }

        Ok(processed)
    }

    /// Process a single report job.
    async fn process_job(
        &self,
        repo: &AnalyticsRepository,
        job: &ReportJobEntity,
    ) -> Result<(), ReportGenerationError> {
        // Mark job as processing
        repo.mark_report_processing(job.id).await?;

        // Parse parameters
        let from = self.parse_date_param(&job.parameters, "from")?;
        let to = self.parse_date_param(&job.parameters, "to")?;
        let format = job
            .parameters
            .get("format")
            .and_then(|v| v.as_str())
            .map(ReportFormat::from_str)
            .unwrap_or(ReportFormat::Json);

        // Generate report based on type
        let (file_path, file_size) = match job.report_type.as_str() {
            "user_analytics" => {
                self.generate_user_report(repo, job.id, job.organization_id, from, to, format)
                    .await?
            }
            "device_analytics" => {
                self.generate_device_report(repo, job.id, job.organization_id, from, to, format)
                    .await?
            }
            _ => {
                return Err(ReportGenerationError::UnsupportedReportType(
                    job.report_type.clone(),
                ))
            }
        };

        // Mark job as completed
        repo.mark_report_completed(job.id, &file_path, file_size)
            .await?;

        Ok(())
    }

    /// Generate user analytics report.
    async fn generate_user_report(
        &self,
        repo: &AnalyticsRepository,
        job_id: Uuid,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        format: ReportFormat,
    ) -> Result<(String, i64), ReportGenerationError> {
        // Fetch user activity trends
        let trends = repo.get_user_activity_trends(org_id, from, to).await?;

        // Convert to report rows
        let rows: Vec<UserReportRow> = trends
            .into_iter()
            .map(|t| UserReportRow {
                date: t.activity_date.to_string(),
                active_users: t.active_users,
                new_users: t.new_users,
                returning_users: t.returning_users,
                total_sessions: t.total_sessions,
                avg_session_duration_seconds: t.avg_session_duration_seconds.unwrap_or(0.0),
            })
            .collect();

        // Generate file
        let filename = format!(
            "user_report_{}_{}.{}",
            job_id,
            chrono::Utc::now().format("%Y%m%d%H%M%S"),
            format.extension()
        );
        let file_path = self.reports_dir.join(&filename);

        let content = match format {
            ReportFormat::Json => self.to_json(&rows)?,
            ReportFormat::Csv => self.to_csv_user(&rows)?,
        };

        self.write_file(&file_path, &content)?;

        let file_size = fs::metadata(&file_path)?.len() as i64;
        Ok((filename, file_size))
    }

    /// Generate device analytics report.
    async fn generate_device_report(
        &self,
        repo: &AnalyticsRepository,
        job_id: Uuid,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        format: ReportFormat,
    ) -> Result<(String, i64), ReportGenerationError> {
        // Fetch device activity trends
        let trends = repo.get_device_activity_trends(org_id, from, to).await?;

        // Convert to report rows
        let rows: Vec<DeviceReportRow> = trends
            .into_iter()
            .map(|t| DeviceReportRow {
                date: t.activity_date.to_string(),
                active_devices: t.active_devices,
                new_enrollments: t.new_enrollments,
                unenrollments: t.unenrollments,
                total_locations_reported: t.total_locations_reported,
                total_geofence_events: t.total_geofence_events,
                total_commands_issued: t.total_commands_issued,
            })
            .collect();

        // Generate file
        let filename = format!(
            "device_report_{}_{}.{}",
            job_id,
            chrono::Utc::now().format("%Y%m%d%H%M%S"),
            format.extension()
        );
        let file_path = self.reports_dir.join(&filename);

        let content = match format {
            ReportFormat::Json => self.to_json(&rows)?,
            ReportFormat::Csv => self.to_csv_device(&rows)?,
        };

        self.write_file(&file_path, &content)?;

        let file_size = fs::metadata(&file_path)?.len() as i64;
        Ok((filename, file_size))
    }

    /// Clean up expired reports and their files.
    pub async fn cleanup_expired_reports(&self) -> Result<u32, ReportGenerationError> {
        let repo = AnalyticsRepository::new(self.pool.clone());
        let expired = repo.delete_expired_report_jobs().await?;

        let mut deleted = 0u32;
        for job in expired {
            if let Some(file_path) = &job.file_path {
                let full_path = self.reports_dir.join(file_path);
                if full_path.exists() {
                    if let Err(e) = fs::remove_file(&full_path) {
                        warn!(
                            file_path = %file_path,
                            error = %e,
                            "Failed to delete expired report file"
                        );
                    } else {
                        deleted += 1;
                    }
                }
            }
        }

        if deleted > 0 {
            info!(deleted = deleted, "Cleaned up expired report files");
        }

        Ok(deleted)
    }

    /// Parse a date parameter from job parameters.
    fn parse_date_param(
        &self,
        params: &serde_json::Value,
        key: &str,
    ) -> Result<NaiveDate, ReportGenerationError> {
        // Try to get from nested "additional" first, then top level
        let date_str = params
            .get(key)
            .or_else(|| params.get("additional").and_then(|a| a.get(key)))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ReportGenerationError::InvalidParameters(format!("Missing '{}' parameter", key))
            })?;

        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
            ReportGenerationError::InvalidParameters(format!(
                "Invalid date format for '{}': {} (expected YYYY-MM-DD)",
                key, e
            ))
        })
    }

    /// Serialize data to JSON.
    fn to_json<T: Serialize>(&self, data: &[T]) -> Result<String, ReportGenerationError> {
        serde_json::to_string_pretty(data).map_err(ReportGenerationError::from)
    }

    /// Convert user report rows to CSV.
    fn to_csv_user(&self, rows: &[UserReportRow]) -> Result<String, ReportGenerationError> {
        let mut csv = String::new();
        csv.push_str(
            "date,active_users,new_users,returning_users,total_sessions,avg_session_duration_seconds\n",
        );

        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{:.2}\n",
                row.date,
                row.active_users,
                row.new_users,
                row.returning_users,
                row.total_sessions,
                row.avg_session_duration_seconds
            ));
        }

        Ok(csv)
    }

    /// Convert device report rows to CSV.
    fn to_csv_device(&self, rows: &[DeviceReportRow]) -> Result<String, ReportGenerationError> {
        let mut csv = String::new();
        csv.push_str("date,active_devices,new_enrollments,unenrollments,total_locations_reported,total_geofence_events,total_commands_issued\n");

        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                row.date,
                row.active_devices,
                row.new_enrollments,
                row.unenrollments,
                row.total_locations_reported,
                row.total_geofence_events,
                row.total_commands_issued
            ));
        }

        Ok(csv)
    }

    /// Write content to a file.
    fn write_file(&self, path: &Path, content: &str) -> Result<(), ReportGenerationError> {
        // Ensure reports directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Get the full path for a report file.
    pub fn get_report_path(&self, filename: &str) -> PathBuf {
        self.reports_dir.join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_format_from_str() {
        assert_eq!(ReportFormat::from_str("csv"), ReportFormat::Csv);
        assert_eq!(ReportFormat::from_str("CSV"), ReportFormat::Csv);
        assert_eq!(ReportFormat::from_str("json"), ReportFormat::Json);
        assert_eq!(ReportFormat::from_str("JSON"), ReportFormat::Json);
        assert_eq!(ReportFormat::from_str("unknown"), ReportFormat::Json);
    }

    #[test]
    fn test_report_format_extension() {
        assert_eq!(ReportFormat::Csv.extension(), "csv");
        assert_eq!(ReportFormat::Json.extension(), "json");
    }

    #[test]
    fn test_user_report_row_serialization() {
        let row = UserReportRow {
            date: "2024-01-01".to_string(),
            active_users: 100,
            new_users: 10,
            returning_users: 90,
            total_sessions: 500,
            avg_session_duration_seconds: 300.5,
        };

        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("\"active_users\":100"));
        assert!(json.contains("\"date\":\"2024-01-01\""));
    }

    #[test]
    fn test_device_report_row_serialization() {
        let row = DeviceReportRow {
            date: "2024-01-01".to_string(),
            active_devices: 50,
            new_enrollments: 5,
            unenrollments: 2,
            total_locations_reported: 1000,
            total_geofence_events: 100,
            total_commands_issued: 25,
        };

        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("\"active_devices\":50"));
        assert!(json.contains("\"total_locations_reported\":1000"));
    }
}
