//! Admin group management domain models.
//!
//! Story 14.4: Group Management Admin Endpoints

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::admin_user::SortOrder;

/// Owner info for admin group list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupOwnerInfo {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
}

/// Admin group list item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminGroupItem {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon_emoji: Option<String>,
    pub member_count: i64,
    pub device_count: i64,
    pub is_active: bool,
    pub owner: Option<GroupOwnerInfo>,
    pub created_at: DateTime<Utc>,
}

/// Group summary statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminGroupSummary {
    pub total_groups: i64,
    pub active_groups: i64,
    pub total_members: i64,
    pub total_devices: i64,
}

/// Pagination for admin group list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminGroupPagination {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

/// Response for admin group list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminGroupListResponse {
    pub data: Vec<AdminGroupItem>,
    pub pagination: AdminGroupPagination,
    pub summary: AdminGroupSummary,
}

/// Query parameters for listing admin groups.
#[derive(Debug, Clone, Deserialize, Validate, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdminGroupQuery {
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    pub page: Option<u32>,
    #[validate(range(min = 1, max = 100, message = "Per page must be between 1 and 100"))]
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub has_devices: Option<bool>,
    pub sort: Option<AdminGroupSortField>,
    pub order: Option<SortOrder>,
}

/// Sort field for admin group list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AdminGroupSortField {
    Name,
    #[default]
    CreatedAt,
    MemberCount,
    DeviceCount,
}

impl AdminGroupSortField {
    pub fn as_sql_column(&self) -> &'static str {
        match self {
            AdminGroupSortField::Name => "g.name",
            AdminGroupSortField::CreatedAt => "g.created_at",
            AdminGroupSortField::MemberCount => "member_count",
            AdminGroupSortField::DeviceCount => "device_count",
        }
    }
}

/// Member info for group detail response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMemberInfo {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

/// Device info for group detail response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupDeviceInfo {
    pub id: i64,
    pub device_uuid: Uuid,
    pub display_name: String,
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// Full group info for detail view.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminGroupProfile {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon_emoji: Option<String>,
    pub max_devices: i32,
    pub member_count: i64,
    pub device_count: i64,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Response for group detail endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminGroupDetailResponse {
    pub group: AdminGroupProfile,
    pub members: Vec<GroupMemberInfo>,
    pub devices: Vec<GroupDeviceInfo>,
}

/// Request for updating group settings.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAdminGroupRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: Option<String>,
    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,
    #[validate(range(min = 1, max = 100, message = "Max devices must be between 1 and 100"))]
    pub max_devices: Option<i32>,
    pub is_active: Option<bool>,
}

/// Response for updating group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAdminGroupResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub max_devices: i32,
    pub is_active: bool,
    pub updated_at: DateTime<Utc>,
}

/// Response for deactivating group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeactivateGroupResponse {
    pub deactivated: bool,
    pub group_id: Uuid,
    pub deactivated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_group_sort_field_sql() {
        assert_eq!(AdminGroupSortField::Name.as_sql_column(), "g.name");
        assert_eq!(AdminGroupSortField::CreatedAt.as_sql_column(), "g.created_at");
        assert_eq!(AdminGroupSortField::MemberCount.as_sql_column(), "member_count");
        assert_eq!(AdminGroupSortField::DeviceCount.as_sql_column(), "device_count");
    }

    #[test]
    fn test_admin_group_summary_default() {
        let summary = AdminGroupSummary::default();
        assert_eq!(summary.total_groups, 0);
        assert_eq!(summary.active_groups, 0);
    }

    #[test]
    fn test_admin_group_query_validation() {
        let query = AdminGroupQuery {
            page: Some(1),
            per_page: Some(50),
            ..Default::default()
        };
        assert!(query.validate().is_ok());
    }

    #[test]
    fn test_update_admin_group_request_validation() {
        let request = UpdateAdminGroupRequest {
            name: Some("Test Group".to_string()),
            description: None,
            max_devices: Some(50),
            is_active: Some(true),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_update_admin_group_request_invalid_name() {
        let request = UpdateAdminGroupRequest {
            name: Some("".to_string()),
            description: None,
            max_devices: None,
            is_active: None,
        };
        assert!(request.validate().is_err());
    }
}
