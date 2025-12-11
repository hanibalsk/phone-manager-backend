//! Data Subject Request domain models for admin API.
//!
//! AP-11.4-6: GDPR/CCPA Data Subject Request endpoints

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Type of data subject request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSubjectRequestType {
    /// Right to access personal data
    Access,
    /// Right to erasure (right to be forgotten)
    Deletion,
    /// Right to data portability
    Portability,
    /// Right to rectification (correction)
    Rectification,
    /// Right to restriction of processing
    Restriction,
    /// Right to object to processing
    Objection,
}

impl std::fmt::Display for DataSubjectRequestType {
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

/// Status of data subject request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSubjectRequestStatus {
    /// Request submitted, awaiting processing
    Pending,
    /// Request being processed
    InProgress,
    /// Request fulfilled
    Completed,
    /// Request rejected (with reason)
    Rejected,
    /// Request cancelled by requester
    Cancelled,
}

impl std::fmt::Display for DataSubjectRequestStatus {
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

/// Processor information.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProcessorInfo {
    /// User ID of the processor.
    pub user_id: Uuid,
    /// Email of the processor.
    pub email: Option<String>,
    /// When the request was processed.
    pub processed_at: DateTime<Utc>,
}

/// Data subject request response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DataSubjectRequestResponse {
    /// Unique identifier.
    pub id: Uuid,
    /// Type of request.
    pub request_type: DataSubjectRequestType,
    /// Current status.
    pub status: DataSubjectRequestStatus,
    /// Email of the data subject.
    pub subject_email: String,
    /// Name of the data subject.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_name: Option<String>,
    /// User ID if the subject is a known user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_user_id: Option<Uuid>,
    /// Description of the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Reason for rejection (if rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
    /// Processor information (if processed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processor: Option<ProcessorInfo>,
    /// Result data (for access/portability requests).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_data: Option<serde_json::Value>,
    /// URL to download result file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_file_url: Option<String>,
    /// When the result expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_expires_at: Option<DateTime<Utc>>,
    /// Due date for compliance.
    pub due_date: DateTime<Utc>,
    /// Whether the request is overdue.
    pub is_overdue: bool,
    /// When the request was created.
    pub created_at: DateTime<Utc>,
    /// When the request was last updated.
    pub updated_at: DateTime<Utc>,
}

/// List data subject requests response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListDataSubjectRequestsResponse {
    /// List of requests.
    pub requests: Vec<DataSubjectRequestResponse>,
    /// Pagination information.
    pub pagination: DataSubjectRequestPagination,
}

/// Pagination information.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DataSubjectRequestPagination {
    /// Current page.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Total items.
    pub total: i64,
    /// Total pages.
    pub total_pages: u32,
}

/// Query parameters for listing data subject requests.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ListDataSubjectRequestsQuery {
    /// Filter by status.
    #[serde(default)]
    pub status: Option<DataSubjectRequestStatus>,
    /// Filter by request type.
    #[serde(default)]
    pub request_type: Option<DataSubjectRequestType>,
    /// Filter by subject email (partial match).
    #[serde(default)]
    pub subject_email: Option<String>,
    /// Filter by creation date (from).
    #[serde(default)]
    pub from: Option<DateTime<Utc>>,
    /// Filter by creation date (to).
    #[serde(default)]
    pub to: Option<DateTime<Utc>>,
    /// Page number (1-indexed).
    #[serde(default)]
    pub page: Option<u32>,
    /// Items per page (max 100).
    #[serde(default)]
    pub per_page: Option<u32>,
}

/// Request to create a data subject request.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateDataSubjectRequestRequest {
    /// Type of request.
    pub request_type: DataSubjectRequestType,
    /// Email of the data subject.
    #[validate(email(message = "Invalid email format"))]
    #[validate(length(max = 255, message = "Email too long"))]
    pub subject_email: String,
    /// Name of the data subject.
    #[validate(length(max = 255, message = "Name too long"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_name: Option<String>,
    /// User ID if the subject is a known user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_user_id: Option<Uuid>,
    /// Description of the request.
    #[validate(length(max = 2000, message = "Description too long"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Custom due date in days (default: 30).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_days: Option<i64>,
}

/// Action to take on a data subject request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSubjectRequestAction {
    /// Start processing the request.
    StartProcessing,
    /// Complete the request successfully.
    Complete,
    /// Reject the request.
    Reject,
    /// Cancel the request.
    Cancel,
}

/// Request to process a data subject request.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct ProcessDataSubjectRequestRequest {
    /// Action to take.
    pub action: DataSubjectRequestAction,
    /// Reason for rejection (required when action is reject).
    #[validate(length(max = 1000, message = "Rejection reason too long"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
    /// Result data (for access/portability requests).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_data: Option<serde_json::Value>,
    /// URL to download result file.
    #[validate(url(message = "Invalid URL format"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_file_url: Option<String>,
    /// Days until result expires (default: 30).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_expires_days: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_type_display() {
        assert_eq!(DataSubjectRequestType::Access.to_string(), "access");
        assert_eq!(DataSubjectRequestType::Deletion.to_string(), "deletion");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(DataSubjectRequestStatus::Pending.to_string(), "pending");
        assert_eq!(
            DataSubjectRequestStatus::InProgress.to_string(),
            "in_progress"
        );
    }

    #[test]
    fn test_create_request_validation() {
        let request = CreateDataSubjectRequestRequest {
            request_type: DataSubjectRequestType::Access,
            subject_email: "test@example.com".to_string(),
            subject_name: Some("John Doe".to_string()),
            subject_user_id: None,
            description: Some("Request for data export".to_string()),
            due_days: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_request_invalid_email() {
        let request = CreateDataSubjectRequestRequest {
            request_type: DataSubjectRequestType::Access,
            subject_email: "invalid-email".to_string(),
            subject_name: None,
            subject_user_id: None,
            description: None,
            due_days: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_request_serialization() {
        let request_type = DataSubjectRequestType::Access;
        let json = serde_json::to_string(&request_type).unwrap();
        assert_eq!(json, "\"access\"");
    }

    #[test]
    fn test_action_serialization() {
        let action = DataSubjectRequestAction::StartProcessing;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"start_processing\"");
    }
}
