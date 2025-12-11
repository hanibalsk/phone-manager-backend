//! Data Subject Request entity.
//!
//! AP-11.4-6: GDPR/CCPA Data Subject Request management

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database enum for data subject request types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "data_subject_request_type", rename_all = "snake_case")]
pub enum DataSubjectRequestTypeDb {
    Access,
    Deletion,
    Portability,
    Rectification,
    Restriction,
    Objection,
}

impl std::fmt::Display for DataSubjectRequestTypeDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Access => write!(f, "access"),
            Self::Deletion => write!(f, "deletion"),
            Self::Portability => write!(f, "portability"),
            Self::Rectification => write!(f, "rectification"),
            Self::Restriction => write!(f, "restriction"),
            Self::Objection => write!(f, "objection"),
        }
    }
}

/// Database enum for data subject request status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "data_subject_request_status", rename_all = "snake_case")]
pub enum DataSubjectRequestStatusDb {
    Pending,
    InProgress,
    Completed,
    Rejected,
    Cancelled,
}

impl std::fmt::Display for DataSubjectRequestStatusDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Rejected => write!(f, "rejected"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Database entity for data subject requests.
#[derive(Debug, Clone, FromRow)]
pub struct DataSubjectRequestEntity {
    /// Unique identifier.
    pub id: Uuid,

    /// Organization this request belongs to.
    pub organization_id: Uuid,

    /// Type of request.
    pub request_type: DataSubjectRequestTypeDb,

    /// Current status.
    pub status: DataSubjectRequestStatusDb,

    /// Email of the data subject.
    pub subject_email: String,

    /// Name of the data subject.
    pub subject_name: Option<String>,

    /// User ID if the subject is a known user.
    pub subject_user_id: Option<Uuid>,

    /// Description of the request.
    pub description: Option<String>,

    /// Reason for rejection (if rejected).
    pub rejection_reason: Option<String>,

    /// User who processed the request.
    pub processed_by: Option<Uuid>,

    /// When the request was processed.
    pub processed_at: Option<DateTime<Utc>>,

    /// Result data (for access/portability requests).
    pub result_data: Option<serde_json::Value>,

    /// URL to download result file.
    pub result_file_url: Option<String>,

    /// When the result expires.
    pub result_expires_at: Option<DateTime<Utc>>,

    /// Due date for compliance (typically 30 days from creation).
    pub due_date: DateTime<Utc>,

    /// When the request was created.
    pub created_at: DateTime<Utc>,

    /// When the request was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Entity with processor information.
#[derive(Debug, Clone, FromRow)]
pub struct DataSubjectRequestWithProcessorEntity {
    /// Unique identifier.
    pub id: Uuid,

    /// Organization this request belongs to.
    pub organization_id: Uuid,

    /// Type of request.
    pub request_type: DataSubjectRequestTypeDb,

    /// Current status.
    pub status: DataSubjectRequestStatusDb,

    /// Email of the data subject.
    pub subject_email: String,

    /// Name of the data subject.
    pub subject_name: Option<String>,

    /// User ID if the subject is a known user.
    pub subject_user_id: Option<Uuid>,

    /// Description of the request.
    pub description: Option<String>,

    /// Reason for rejection (if rejected).
    pub rejection_reason: Option<String>,

    /// User who processed the request.
    pub processed_by: Option<Uuid>,

    /// Email of the processor.
    pub processor_email: Option<String>,

    /// When the request was processed.
    pub processed_at: Option<DateTime<Utc>>,

    /// Result data (for access/portability requests).
    pub result_data: Option<serde_json::Value>,

    /// URL to download result file.
    pub result_file_url: Option<String>,

    /// When the result expires.
    pub result_expires_at: Option<DateTime<Utc>>,

    /// Due date for compliance.
    pub due_date: DateTime<Utc>,

    /// When the request was created.
    pub created_at: DateTime<Utc>,

    /// When the request was last updated.
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_type_display() {
        assert_eq!(DataSubjectRequestTypeDb::Access.to_string(), "access");
        assert_eq!(DataSubjectRequestTypeDb::Deletion.to_string(), "deletion");
        assert_eq!(
            DataSubjectRequestTypeDb::Portability.to_string(),
            "portability"
        );
        assert_eq!(
            DataSubjectRequestTypeDb::Rectification.to_string(),
            "rectification"
        );
        assert_eq!(
            DataSubjectRequestTypeDb::Restriction.to_string(),
            "restriction"
        );
        assert_eq!(DataSubjectRequestTypeDb::Objection.to_string(), "objection");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(DataSubjectRequestStatusDb::Pending.to_string(), "pending");
        assert_eq!(
            DataSubjectRequestStatusDb::InProgress.to_string(),
            "in_progress"
        );
        assert_eq!(
            DataSubjectRequestStatusDb::Completed.to_string(),
            "completed"
        );
        assert_eq!(DataSubjectRequestStatusDb::Rejected.to_string(), "rejected");
        assert_eq!(
            DataSubjectRequestStatusDb::Cancelled.to_string(),
            "cancelled"
        );
    }

    #[test]
    fn test_entity_creation() {
        let now = Utc::now();
        let entity = DataSubjectRequestEntity {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            request_type: DataSubjectRequestTypeDb::Access,
            status: DataSubjectRequestStatusDb::Pending,
            subject_email: "user@example.com".to_string(),
            subject_name: Some("John Doe".to_string()),
            subject_user_id: Some(Uuid::new_v4()),
            description: Some("Request for data export".to_string()),
            rejection_reason: None,
            processed_by: None,
            processed_at: None,
            result_data: None,
            result_file_url: None,
            result_expires_at: None,
            due_date: now + chrono::Duration::days(30),
            created_at: now,
            updated_at: now,
        };

        assert_eq!(entity.request_type, DataSubjectRequestTypeDb::Access);
        assert_eq!(entity.status, DataSubjectRequestStatusDb::Pending);
    }
}
