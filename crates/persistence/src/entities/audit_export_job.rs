//! Audit export job entity.
//!
//! Story 13.10: Audit Query and Export Endpoints

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database entity for audit export jobs.
#[derive(Debug, Clone, FromRow)]
pub struct AuditExportJobEntity {
    /// Unique database identifier.
    pub id: Uuid,

    /// User-facing job identifier (export_<random>).
    pub job_id: String,

    /// Organization this export belongs to.
    pub organization_id: Uuid,

    /// Current job status.
    pub status: String,

    /// Export format (json or csv).
    pub format: String,

    /// Filter parameters used for the export.
    pub filters: Option<serde_json::Value>,

    /// Number of records in the export.
    pub record_count: Option<i64>,

    /// URL or data URL to download the export.
    pub download_url: Option<String>,

    /// Error message if job failed.
    pub error_message: Option<String>,

    /// When the job was created.
    pub created_at: DateTime<Utc>,

    /// When the job was last updated.
    pub updated_at: DateTime<Utc>,

    /// When the job expires.
    pub expires_at: DateTime<Utc>,

    /// When the job completed.
    pub completed_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_export_job_entity_creation() {
        let now = Utc::now();
        let entity = AuditExportJobEntity {
            id: Uuid::new_v4(),
            job_id: "export_abc123".to_string(),
            organization_id: Uuid::new_v4(),
            status: "pending".to_string(),
            format: "json".to_string(),
            filters: Some(serde_json::json!({"action": "device.assign"})),
            record_count: None,
            download_url: None,
            error_message: None,
            created_at: now,
            updated_at: now,
            expires_at: now + chrono::Duration::hours(24),
            completed_at: None,
        };

        assert_eq!(entity.status, "pending");
        assert_eq!(entity.format, "json");
    }
}
