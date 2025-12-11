//! Unlock request domain models for setting unlock workflow.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of an unlock request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnlockRequestStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

impl std::fmt::Display for UnlockRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnlockRequestStatus::Pending => write!(f, "pending"),
            UnlockRequestStatus::Approved => write!(f, "approved"),
            UnlockRequestStatus::Denied => write!(f, "denied"),
            UnlockRequestStatus::Expired => write!(f, "expired"),
        }
    }
}

/// Request to create an unlock request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateUnlockRequestRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Response after creating an unlock request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateUnlockRequestResponse {
    pub id: Uuid,
    pub device_id: Uuid,
    pub setting_key: String,
    pub status: UnlockRequestStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Brief device info for listing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceInfo {
    pub id: Uuid,
    pub display_name: String,
}

/// Brief user info for listing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UserInfo {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Unlock request for listing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UnlockRequestItem {
    pub id: Uuid,
    pub device: DeviceInfo,
    pub setting_key: String,
    pub setting_display_name: String,
    pub status: UnlockRequestStatus,
    pub requested_by: UserInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responded_by: Option<UserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responded_at: Option<DateTime<Utc>>,
}

/// Pagination info for list responses.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

/// Response for listing unlock requests.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListUnlockRequestsResponse {
    pub data: Vec<UnlockRequestItem>,
    pub pagination: Pagination,
}

/// Request to respond to an unlock request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RespondToUnlockRequestRequest {
    pub status: UnlockRequestStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Response after responding to an unlock request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RespondToUnlockRequestResponse {
    pub id: Uuid,
    pub status: UnlockRequestStatus,
    pub responded_by: Uuid,
    pub responded_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// True if the setting was automatically unlocked (for approved requests)
    pub setting_unlocked: bool,
}

/// Query parameters for listing unlock requests.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListUnlockRequestsQuery {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

// Admin unlock request models (AP-8)

/// Query parameters for listing admin unlock requests.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminListUnlockRequestsQuery {
    /// Filter by status (pending, approved, denied, expired).
    pub status: Option<String>,
    /// Filter by device ID.
    pub device_id: Option<Uuid>,
    /// Page number (1-based).
    #[serde(default = "default_admin_page")]
    pub page: u32,
    /// Items per page.
    #[serde(default = "default_admin_per_page")]
    pub per_page: u32,
}

fn default_admin_page() -> u32 {
    1
}

fn default_admin_per_page() -> u32 {
    20
}

/// Admin unlock request item for listing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminUnlockRequestItem {
    pub id: Uuid,
    pub device_id: Uuid,
    pub device_name: String,
    pub setting_key: String,
    pub setting_name: String,
    pub status: UnlockRequestStatus,
    pub requested_by: AdminUserBrief,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responded_by: Option<AdminUserBrief>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responded_at: Option<DateTime<Utc>>,
}

/// Brief user info for admin unlock request listing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminUserBrief {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Admin pagination info.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminUnlockPagination {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

/// Response for listing admin unlock requests.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminListUnlockRequestsResponse {
    pub requests: Vec<AdminUnlockRequestItem>,
    pub pagination: AdminUnlockPagination,
}

/// Request to approve an unlock request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ApproveUnlockRequestRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Request to deny an unlock request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DenyUnlockRequestRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Response for approve/deny unlock request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminUnlockRequestActionResponse {
    pub id: Uuid,
    pub status: UnlockRequestStatus,
    pub responded_by: Uuid,
    pub responded_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Request for bulk processing unlock requests.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BulkProcessUnlockRequestsRequest {
    /// List of request IDs to process.
    pub request_ids: Vec<Uuid>,
    /// Action to take: "approve" or "deny".
    pub action: String,
    /// Optional note for all requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Response for bulk processing unlock requests.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BulkProcessUnlockRequestsResponse {
    /// Number of requests processed.
    pub processed: i64,
    /// Number of requests requested.
    pub requested: usize,
    /// Action taken.
    pub action: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unlock_request_status_display() {
        assert_eq!(UnlockRequestStatus::Pending.to_string(), "pending");
        assert_eq!(UnlockRequestStatus::Approved.to_string(), "approved");
        assert_eq!(UnlockRequestStatus::Denied.to_string(), "denied");
        assert_eq!(UnlockRequestStatus::Expired.to_string(), "expired");
    }

    #[test]
    fn test_create_unlock_request_deserialize() {
        let json = r#"{"reason":"I need to change this setting"}"#;
        let req: CreateUnlockRequestRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            req.reason,
            Some("I need to change this setting".to_string())
        );
    }

    #[test]
    fn test_respond_to_unlock_request_deserialize() {
        let json = r#"{"status":"approved","note":"OK for now"}"#;
        let req: RespondToUnlockRequestRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, UnlockRequestStatus::Approved);
        assert_eq!(req.note, Some("OK for now".to_string()));
    }

    #[test]
    fn test_list_query_defaults() {
        let query: ListUnlockRequestsQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, 20);
        assert!(query.status.is_none());
    }
}
