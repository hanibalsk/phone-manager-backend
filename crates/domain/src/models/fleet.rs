//! Fleet management domain models.
//!
//! Story 13.7: Fleet Management Endpoints

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::device_token::EnrollmentStatus;

/// Device command types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceCommandType {
    Wipe,
    Lock,
    Unlock,
    Restart,
    UpdatePolicy,
    SyncSettings,
}

impl DeviceCommandType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wipe => "wipe",
            Self::Lock => "lock",
            Self::Unlock => "unlock",
            Self::Restart => "restart",
            Self::UpdatePolicy => "update_policy",
            Self::SyncSettings => "sync_settings",
        }
    }
}

impl std::fmt::Display for DeviceCommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for DeviceCommandType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "wipe" => Ok(Self::Wipe),
            "lock" => Ok(Self::Lock),
            "unlock" => Ok(Self::Unlock),
            "restart" => Ok(Self::Restart),
            "update_policy" => Ok(Self::UpdatePolicy),
            "sync_settings" => Ok(Self::SyncSettings),
            _ => Err(format!("Invalid command type: {}", s)),
        }
    }
}

/// Device command status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceCommandStatus {
    Pending,
    Acknowledged,
    Completed,
    Failed,
    Expired,
}

impl DeviceCommandStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Acknowledged => "acknowledged",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DeviceCommandStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for DeviceCommandStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "acknowledged" => Ok(Self::Acknowledged),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "expired" => Ok(Self::Expired),
            _ => Err(format!("Invalid command status: {}", s)),
        }
    }
}

/// Device command domain model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceCommand {
    pub id: Uuid,
    pub device_id: i64,
    pub organization_id: Uuid,
    pub command_type: DeviceCommandType,
    pub status: DeviceCommandStatus,
    pub payload: Option<serde_json::Value>,
    pub issued_by: Uuid,
    pub issued_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub expires_at: DateTime<Utc>,
}

/// Fleet device listing query parameters.
#[derive(Debug, Clone, Default, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct FleetDeviceQuery {
    /// Page number (1-indexed)
    #[validate(range(min = 1))]
    pub page: Option<u32>,
    /// Items per page
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u32>,
    /// Filter by enrollment status
    pub status: Option<EnrollmentStatus>,
    /// Filter by group ID
    pub group_id: Option<String>,
    /// Filter by policy ID
    pub policy_id: Option<Uuid>,
    /// Filter by assignment status
    pub assigned: Option<bool>,
    /// Search by name or UUID
    #[validate(length(max = 100))]
    pub search: Option<String>,
    /// Sort field
    pub sort: Option<FleetSortField>,
    /// Sort order
    pub order: Option<SortOrder>,
}

/// Sort fields for fleet device listing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetSortField {
    #[default]
    LastSeenAt,
    DisplayName,
    CreatedAt,
    EnrolledAt,
}

impl FleetSortField {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LastSeenAt => "last_seen_at",
            Self::DisplayName => "display_name",
            Self::CreatedAt => "created_at",
            Self::EnrolledAt => "enrolled_at",
        }
    }
}

/// Sort order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// Assigned user info in fleet device response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AssignedUserInfo {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
}

/// Group info in fleet device response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FleetGroupInfo {
    pub id: String,
    pub name: Option<String>,
}

/// Policy info in fleet device response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FleetPolicyInfo {
    pub id: Uuid,
    pub name: String,
}

/// Last location info in fleet device response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FleetLastLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: DateTime<Utc>,
}

/// Fleet device item in listing response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FleetDeviceItem {
    pub id: i64,
    pub device_uuid: Uuid,
    pub display_name: String,
    pub platform: String,
    pub enrollment_status: Option<EnrollmentStatus>,
    pub is_managed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_user: Option<AssignedUserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<FleetGroupInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<FleetPolicyInfo>,
    pub last_seen_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_location: Option<FleetLastLocation>,
    pub enrolled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Pagination info in fleet response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FleetPagination {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

/// Summary counts in fleet response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FleetSummary {
    pub enrolled: i64,
    pub pending: i64,
    pub suspended: i64,
    pub retired: i64,
    pub assigned: i64,
    pub unassigned: i64,
}

/// Fleet device list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FleetDeviceListResponse {
    pub data: Vec<FleetDeviceItem>,
    pub pagination: FleetPagination,
    pub summary: FleetSummary,
}

/// Request to assign a user to a device.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct AssignDeviceRequest {
    pub user_id: Uuid,
    #[serde(default)]
    pub notify_user: bool,
}

/// Response for device assignment.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AssignDeviceResponse {
    pub device_id: i64,
    pub assigned_user: AssignedUserInfo,
    pub assigned_at: DateTime<Utc>,
    pub notification_sent: bool,
}

/// Response for device unassignment.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UnassignDeviceResponse {
    pub device_id: i64,
    pub unassigned_at: DateTime<Utc>,
}

/// Response for device status change (suspend/retire).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceStatusChangeResponse {
    pub device_id: i64,
    pub previous_status: Option<EnrollmentStatus>,
    pub new_status: EnrollmentStatus,
    pub changed_at: DateTime<Utc>,
}

/// Request to issue a device command.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct IssueCommandRequest {
    /// Optional payload for the command
    pub payload: Option<serde_json::Value>,
    /// Expiry time for the command (defaults to 24 hours)
    pub expires_in_hours: Option<u32>,
}

/// Response for device command issuance.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct IssueCommandResponse {
    pub command_id: Uuid,
    pub device_id: i64,
    pub command_type: DeviceCommandType,
    pub status: DeviceCommandStatus,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_command_type_from_str() {
        assert_eq!(
            "wipe".parse::<DeviceCommandType>().unwrap(),
            DeviceCommandType::Wipe
        );
        assert_eq!(
            "lock".parse::<DeviceCommandType>().unwrap(),
            DeviceCommandType::Lock
        );
        assert!("invalid".parse::<DeviceCommandType>().is_err());
    }

    #[test]
    fn test_device_command_status_from_str() {
        assert_eq!(
            "pending".parse::<DeviceCommandStatus>().unwrap(),
            DeviceCommandStatus::Pending
        );
        assert_eq!(
            "completed".parse::<DeviceCommandStatus>().unwrap(),
            DeviceCommandStatus::Completed
        );
        assert!("invalid".parse::<DeviceCommandStatus>().is_err());
    }

    #[test]
    fn test_fleet_device_query_validation() {
        let query = FleetDeviceQuery {
            page: Some(0),
            ..Default::default()
        };
        assert!(query.validate().is_err());

        let query = FleetDeviceQuery {
            page: Some(1),
            per_page: Some(50),
            ..Default::default()
        };
        assert!(query.validate().is_ok());
    }

    #[test]
    fn test_fleet_sort_field_default() {
        assert_eq!(FleetSortField::default(), FleetSortField::LastSeenAt);
    }

    #[test]
    fn test_sort_order_default() {
        assert_eq!(SortOrder::default(), SortOrder::Desc);
    }

    #[test]
    fn test_device_command_type_display() {
        assert_eq!(DeviceCommandType::Wipe.to_string(), "wipe");
        assert_eq!(DeviceCommandType::UpdatePolicy.to_string(), "update_policy");
    }

    #[test]
    fn test_device_command_status_display() {
        assert_eq!(DeviceCommandStatus::Pending.to_string(), "pending");
        assert_eq!(DeviceCommandStatus::Completed.to_string(), "completed");
    }

    #[test]
    fn test_assign_device_request_validation() {
        let request = AssignDeviceRequest {
            user_id: Uuid::new_v4(),
            notify_user: true,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_fleet_summary_default() {
        let summary = FleetSummary::default();
        assert_eq!(summary.enrolled, 0);
        assert_eq!(summary.pending, 0);
        assert_eq!(summary.suspended, 0);
        assert_eq!(summary.retired, 0);
        assert_eq!(summary.assigned, 0);
        assert_eq!(summary.unassigned, 0);
    }
}
