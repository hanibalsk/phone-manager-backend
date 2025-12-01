//! Bulk device import models.
//!
//! Story 13.8: Bulk Device Import Endpoint

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Maximum devices per bulk import request.
pub const MAX_BULK_IMPORT_DEVICES: usize = 200;

/// Maximum metadata size in bytes (10KB).
pub const MAX_METADATA_SIZE: usize = 10 * 1024;

/// Request to bulk import devices.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BulkDeviceImportRequest {
    /// List of devices to import.
    #[validate(length(min = 1, max = 200, message = "devices must contain 1-200 items"))]
    #[validate(nested)]
    pub devices: Vec<BulkDeviceItem>,

    /// Import options.
    #[serde(default)]
    pub options: BulkImportOptions,
}

/// Single device item in bulk import.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BulkDeviceItem {
    /// External identifier (unique within organization).
    #[validate(length(max = 255, message = "external_id must be at most 255 characters"))]
    pub external_id: Option<String>,

    /// Display name for the device.
    #[validate(length(
        min = 2,
        max = 100,
        message = "display_name must be 2-100 characters"
    ))]
    pub display_name: String,

    /// Group to assign the device to.
    pub group_id: Option<Uuid>,

    /// Policy to assign to the device.
    pub policy_id: Option<Uuid>,

    /// Email of user to assign the device to.
    #[validate(email(message = "assigned_user_email must be a valid email"))]
    pub assigned_user_email: Option<String>,

    /// Device metadata (JSON object).
    pub metadata: Option<serde_json::Value>,
}

/// Options for bulk import operation.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BulkImportOptions {
    /// Update existing devices matched by external_id.
    #[serde(default)]
    pub update_existing: bool,

    /// Create users if not found (sends invite).
    #[serde(default)]
    pub create_missing_users: bool,

    /// Send welcome email to assigned users.
    #[serde(default)]
    pub send_welcome_email: bool,
}

/// Response from bulk device import.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkDeviceImportResponse {
    /// Total number of devices processed.
    pub processed: u32,

    /// Number of devices created.
    pub created: u32,

    /// Number of devices updated.
    pub updated: u32,

    /// Number of devices skipped.
    pub skipped: u32,

    /// List of errors encountered.
    pub errors: Vec<BulkImportError>,
}

/// Error encountered during bulk import.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkImportError {
    /// Row number (1-indexed) where error occurred.
    pub row: usize,

    /// External ID of the device (if provided).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,

    /// Error message.
    pub error: String,
}

/// Result of processing a single device in bulk import.
#[derive(Debug, Clone)]
pub enum BulkImportResult {
    /// Device was created.
    Created(i64),
    /// Device was updated.
    Updated(i64),
    /// Device was skipped (exists, no update).
    Skipped,
    /// Error occurred.
    Error(String),
}

/// Input for creating a device via bulk import.
#[derive(Debug, Clone)]
pub struct BulkDeviceInput {
    /// Organization ID.
    pub organization_id: Uuid,

    /// External identifier.
    pub external_id: Option<String>,

    /// Display name.
    pub display_name: String,

    /// Group ID to assign.
    pub group_id: Option<Uuid>,

    /// Policy ID to assign.
    pub policy_id: Option<Uuid>,

    /// User ID to assign.
    pub assigned_user_id: Option<Uuid>,

    /// Device metadata.
    pub metadata: Option<serde_json::Value>,
}

/// Status of bulk import job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkImportJobStatus {
    /// Job is pending.
    Pending,
    /// Job is in progress.
    InProgress,
    /// Job completed successfully.
    Completed,
    /// Job completed with errors.
    CompletedWithErrors,
    /// Job failed.
    Failed,
}

impl BulkImportJobStatus {
    /// Convert to database string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::CompletedWithErrors => "completed_with_errors",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for BulkImportJobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_bulk_import_request_deserialize() {
        let json = json!({
            "devices": [
                {
                    "externalId": "ASSET-001",
                    "displayName": "Field Tablet 1",
                    "groupId": "550e8400-e29b-41d4-a716-446655440001",
                    "metadata": {
                        "assetTag": "AT001"
                    }
                }
            ],
            "options": {
                "updateExisting": true,
                "createMissingUsers": false
            }
        });

        let request: BulkDeviceImportRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.devices.len(), 1);
        assert_eq!(request.devices[0].external_id, Some("ASSET-001".to_string()));
        assert!(request.options.update_existing);
        assert!(!request.options.create_missing_users);
    }

    #[test]
    fn test_bulk_import_response_serialize() {
        let response = BulkDeviceImportResponse {
            processed: 100,
            created: 85,
            updated: 10,
            skipped: 2,
            errors: vec![BulkImportError {
                row: 52,
                external_id: Some("ASSET-052".to_string()),
                error: "User not found".to_string(),
            }],
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["processed"], 100);
        assert_eq!(json["errors"][0]["row"], 52);
    }

    #[test]
    fn test_bulk_import_options_default() {
        let options = BulkImportOptions::default();
        assert!(!options.update_existing);
        assert!(!options.create_missing_users);
        assert!(!options.send_welcome_email);
    }

    #[test]
    fn test_bulk_import_job_status_as_str() {
        assert_eq!(BulkImportJobStatus::Pending.as_str(), "pending");
        assert_eq!(BulkImportJobStatus::InProgress.as_str(), "in_progress");
        assert_eq!(BulkImportJobStatus::Completed.as_str(), "completed");
    }
}
