//! Admin user management domain models.
//!
//! Story 14.3: User Management Endpoints

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::org_user::OrgUserRole;

/// Admin user list item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminUserItem {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: OrgUserRole,
    pub permissions: Vec<String>,
    pub device_count: i64,
    pub group_count: i64,
    pub granted_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// User summary statistics for the admin list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminUserSummary {
    pub owners: i64,
    pub admins: i64,
    pub members: i64,
    pub with_devices: i64,
    pub without_devices: i64,
}

/// Pagination for admin user list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminUserPagination {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

/// Response for admin user list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminUserListResponse {
    pub data: Vec<AdminUserItem>,
    pub pagination: AdminUserPagination,
    pub summary: AdminUserSummary,
}

/// Query parameters for listing admin users.
#[derive(Debug, Clone, Deserialize, Validate, Default)]
#[serde(rename_all = "snake_case")]
pub struct AdminUserQuery {
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    pub page: Option<u32>,
    #[validate(range(min = 1, max = 100, message = "Per page must be between 1 and 100"))]
    pub per_page: Option<u32>,
    pub role: Option<OrgUserRole>,
    pub has_device: Option<bool>,
    pub search: Option<String>,
    pub sort: Option<AdminUserSortField>,
    pub order: Option<SortOrder>,
}

/// Sort field for admin user list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AdminUserSortField {
    DisplayName,
    Email,
    #[default]
    GrantedAt,
}

impl AdminUserSortField {
    pub fn as_sql_column(&self) -> &'static str {
        match self {
            AdminUserSortField::DisplayName => "u.display_name",
            AdminUserSortField::Email => "u.email",
            AdminUserSortField::GrantedAt => "ou.granted_at",
        }
    }
}

/// Sort order for queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

impl SortOrder {
    pub fn as_sql(&self) -> &'static str {
        match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
}

/// Device info for user detail response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserDeviceInfo {
    pub id: i64,
    pub device_uuid: Uuid,
    pub display_name: String,
    pub platform: String,
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// Group info for user detail response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserGroupInfo {
    pub id: String,
    pub name: String,
    pub role: String,
}

/// Recent action for activity summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RecentAction {
    pub action: String,
    pub resource_type: String,
    pub timestamp: DateTime<Utc>,
}

/// Activity summary for user detail.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserActivitySummary {
    pub total_actions: i64,
    pub last_action_at: Option<DateTime<Utc>>,
    pub recent_actions: Vec<RecentAction>,
}

/// Full user profile for detail view.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminUserProfile {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub email_verified: bool,
    pub role: OrgUserRole,
    pub permissions: Vec<String>,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Response for user detail endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminUserDetailResponse {
    pub user: AdminUserProfile,
    pub devices: Vec<UserDeviceInfo>,
    pub groups: Vec<UserGroupInfo>,
    pub activity_summary: UserActivitySummary,
}

/// Request for updating user role/permissions.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateAdminUserRequest {
    pub role: Option<OrgUserRole>,
    pub permissions: Option<Vec<String>>,
}

/// Response for updating user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateAdminUserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub role: OrgUserRole,
    pub permissions: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

/// Response for removing user from org.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RemoveUserResponse {
    pub removed: bool,
    pub user_id: Uuid,
    pub removed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_user_sort_field_sql() {
        assert_eq!(
            AdminUserSortField::DisplayName.as_sql_column(),
            "u.display_name"
        );
        assert_eq!(AdminUserSortField::Email.as_sql_column(), "u.email");
        assert_eq!(
            AdminUserSortField::GrantedAt.as_sql_column(),
            "ou.granted_at"
        );
    }

    #[test]
    fn test_sort_order_sql() {
        assert_eq!(SortOrder::Asc.as_sql(), "ASC");
        assert_eq!(SortOrder::Desc.as_sql(), "DESC");
    }

    #[test]
    fn test_admin_user_summary_default() {
        let summary = AdminUserSummary::default();
        assert_eq!(summary.owners, 0);
        assert_eq!(summary.admins, 0);
        assert_eq!(summary.members, 0);
    }

    #[test]
    fn test_admin_user_query_validation() {
        let query = AdminUserQuery {
            page: Some(1),
            per_page: Some(50),
            ..Default::default()
        };
        assert!(query.validate().is_ok());
    }

    #[test]
    fn test_admin_user_query_invalid_page() {
        let query = AdminUserQuery {
            page: Some(0),
            ..Default::default()
        };
        assert!(query.validate().is_err());
    }

    #[test]
    fn test_admin_user_query_invalid_per_page() {
        let query = AdminUserQuery {
            per_page: Some(101),
            ..Default::default()
        };
        assert!(query.validate().is_err());
    }

    #[test]
    fn test_admin_user_item_serialization() {
        let item = AdminUserItem {
            id: Uuid::nil(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            avatar_url: None,
            role: OrgUserRole::Admin,
            permissions: vec!["device:read".to_string()],
            device_count: 2,
            group_count: 1,
            granted_at: Utc::now(),
            last_login_at: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("device_count"));
        assert!(json.contains("group_count"));
        assert!(json.contains("granted_at"));
    }
}
