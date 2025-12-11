//! Audit export job repository for database operations.
//!
//! Story 13.10: Audit Query and Export Endpoints

use base64::{engine::general_purpose::URL_SAFE, Engine};
use chrono::{Duration, Utc};
use domain::models::{ExportFormat, ExportJobStatus, EXPORT_JOB_EXPIRY_HOURS};
use rand::Rng;
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::AuditExportJobEntity;

/// Repository for audit export job database operations.
#[derive(Clone)]
pub struct AuditExportJobRepository {
    pool: PgPool,
}

/// Domain model for export job.
#[derive(Debug, Clone)]
pub struct ExportJob {
    pub id: Uuid,
    pub job_id: String,
    pub organization_id: Uuid,
    pub status: ExportJobStatus,
    pub format: ExportFormat,
    pub filters: Option<JsonValue>,
    pub record_count: Option<i64>,
    pub download_url: Option<String>,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl AuditExportJobRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Generate a unique job ID.
    pub fn generate_job_id() -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 12] = rng.gen();
        let encoded = URL_SAFE.encode(random_bytes);
        format!("export_{}", encoded)
    }

    /// Create a new export job.
    pub async fn create(
        &self,
        organization_id: Uuid,
        format: ExportFormat,
        filters: Option<JsonValue>,
    ) -> Result<ExportJob, sqlx::Error> {
        let job_id = Self::generate_job_id();
        let expires_at = Utc::now() + Duration::hours(EXPORT_JOB_EXPIRY_HOURS);
        let format_str = match format {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
        };

        let entity = sqlx::query_as::<_, AuditExportJobEntity>(
            r#"
            INSERT INTO audit_export_jobs (job_id, organization_id, format, filters, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, job_id, organization_id, status, format, filters, record_count,
                      download_url, error_message, created_at, updated_at, expires_at, completed_at
            "#,
        )
        .bind(&job_id)
        .bind(organization_id)
        .bind(format_str)
        .bind(&filters)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity_to_domain(entity))
    }

    /// Find export job by job_id and organization.
    pub async fn find_by_job_id(
        &self,
        org_id: Uuid,
        job_id: &str,
    ) -> Result<Option<ExportJob>, sqlx::Error> {
        let entity = sqlx::query_as::<_, AuditExportJobEntity>(
            r#"
            SELECT id, job_id, organization_id, status, format, filters, record_count,
                   download_url, error_message, created_at, updated_at, expires_at, completed_at
            FROM audit_export_jobs
            WHERE job_id = $1 AND organization_id = $2
            "#,
        )
        .bind(job_id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(entity_to_domain))
    }

    /// Update job status to processing.
    pub async fn mark_processing(&self, job_id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE audit_export_jobs
            SET status = 'processing', updated_at = NOW()
            WHERE job_id = $1 AND status = 'pending'
            "#,
        )
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Mark job as completed with download URL.
    pub async fn mark_completed(
        &self,
        job_id: &str,
        record_count: i64,
        download_url: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE audit_export_jobs
            SET status = 'completed', record_count = $2, download_url = $3,
                updated_at = NOW(), completed_at = NOW()
            WHERE job_id = $1 AND status IN ('pending', 'processing')
            "#,
        )
        .bind(job_id)
        .bind(record_count)
        .bind(download_url)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Mark job as failed with error message.
    pub async fn mark_failed(&self, job_id: &str, error: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE audit_export_jobs
            SET status = 'failed', error_message = $2, updated_at = NOW()
            WHERE job_id = $1 AND status IN ('pending', 'processing')
            "#,
        )
        .bind(job_id)
        .bind(error)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Mark expired jobs.
    pub async fn mark_expired(&self) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE audit_export_jobs
            SET status = 'expired', updated_at = NOW()
            WHERE expires_at < NOW() AND status NOT IN ('expired', 'failed')
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Delete old expired jobs (older than 7 days).
    pub async fn cleanup_old_jobs(&self) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM audit_export_jobs
            WHERE expires_at < NOW() - INTERVAL '7 days'
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// List export jobs for an organization.
    pub async fn list_for_org(
        &self,
        org_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ExportJob>, sqlx::Error> {
        let entities = sqlx::query_as::<_, AuditExportJobEntity>(
            r#"
            SELECT id, job_id, organization_id, status, format, filters, record_count,
                   download_url, error_message, created_at, updated_at, expires_at, completed_at
            FROM audit_export_jobs
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(org_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities.into_iter().map(entity_to_domain).collect())
    }
}

fn entity_to_domain(entity: AuditExportJobEntity) -> ExportJob {
    let status = entity
        .status
        .parse::<ExportJobStatus>()
        .unwrap_or(ExportJobStatus::Pending);
    let format = match entity.format.as_str() {
        "csv" => ExportFormat::Csv,
        _ => ExportFormat::Json,
    };

    ExportJob {
        id: entity.id,
        job_id: entity.job_id,
        organization_id: entity.organization_id,
        status,
        format,
        filters: entity.filters,
        record_count: entity.record_count,
        download_url: entity.download_url,
        error_message: entity.error_message,
        created_at: entity.created_at,
        updated_at: entity.updated_at,
        expires_at: entity.expires_at,
        completed_at: entity.completed_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_job_id() {
        let job_id = AuditExportJobRepository::generate_job_id();
        assert!(job_id.starts_with("export_"));
        assert!(job_id.len() > 10);

        // Generate multiple and ensure uniqueness
        let job_id2 = AuditExportJobRepository::generate_job_id();
        assert_ne!(job_id, job_id2);
    }
}
