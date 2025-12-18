//! Group domain models for location sharing groups.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

/// Role within a group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl GroupRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            GroupRole::Owner => "owner",
            GroupRole::Admin => "admin",
            GroupRole::Member => "member",
            GroupRole::Viewer => "viewer",
        }
    }

    /// Returns true if this role can manage group settings
    pub fn can_manage_group(&self) -> bool {
        matches!(self, GroupRole::Owner | GroupRole::Admin)
    }

    /// Returns true if this role can manage members
    pub fn can_manage_members(&self) -> bool {
        matches!(self, GroupRole::Owner | GroupRole::Admin)
    }

    /// Returns true if this role can view member locations
    pub fn can_view_locations(&self) -> bool {
        // All roles can view locations
        true
    }

    /// Returns true if this role can delete the group
    pub fn can_delete_group(&self) -> bool {
        matches!(self, GroupRole::Owner)
    }

    /// Returns true if this role can transfer ownership
    pub fn can_transfer_ownership(&self) -> bool {
        matches!(self, GroupRole::Owner)
    }
}

impl FromStr for GroupRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(GroupRole::Owner),
            "admin" => Ok(GroupRole::Admin),
            "member" => Ok(GroupRole::Member),
            "viewer" => Ok(GroupRole::Viewer),
            _ => Err(format!("Invalid group role: {}", s)),
        }
    }
}

impl fmt::Display for GroupRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a location sharing group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon_emoji: Option<String>,
    pub max_devices: i32,
    pub is_active: bool,
    pub settings: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a user's membership in a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GroupMembership {
    pub id: Uuid,
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub role: GroupRole,
    pub invited_by: Option<Uuid>,
    pub joined_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a group.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateGroupRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    pub name: String,

    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,

    #[validate(length(max = 10, message = "Icon emoji must be at most 10 characters"))]
    pub icon_emoji: Option<String>,

    #[validate(range(min = 1, max = 100, message = "Max devices must be between 1 and 100"))]
    pub max_devices: Option<i32>,
}

/// Request payload for updating a group.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateGroupRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    pub name: Option<String>,

    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,

    #[validate(length(max = 10, message = "Icon emoji must be at most 10 characters"))]
    pub icon_emoji: Option<String>,

    #[validate(range(min = 1, max = 100, message = "Max devices must be between 1 and 100"))]
    pub max_devices: Option<i32>,
}

/// Response for group listing (minimal info).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GroupSummary {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub icon_emoji: Option<String>,
    pub member_count: i64,
    pub device_count: i64,
    pub your_role: GroupRole,
    pub joined_at: DateTime<Utc>,
}

/// Response for group detail.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GroupDetail {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon_emoji: Option<String>,
    pub max_devices: i32,
    pub member_count: i64,
    pub device_count: i64,
    pub is_active: bool,
    pub settings: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub your_role: GroupRole,
    pub your_membership: MembershipInfo,
}

/// Basic membership info for group responses.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MembershipInfo {
    pub id: Uuid,
    pub role: GroupRole,
    pub joined_at: DateTime<Utc>,
}

/// Response for creating a group.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateGroupResponse {
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
    pub your_role: GroupRole,
}

/// Query parameters for listing groups.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListGroupsQuery {
    pub role: Option<String>,
}

/// Response for listing groups.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListGroupsResponse {
    pub data: Vec<GroupSummary>,
    pub count: usize,
}

// ============================================================================
// Membership DTOs (Story 11.2)
// ============================================================================

/// Query parameters for listing members.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ListMembersQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub role: Option<String>,
    pub include_devices: Option<bool>,
}

/// Pagination info for list responses.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

/// Public user info (no sensitive data like email).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UserPublic {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// Device info for member listing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MemberDeviceInfo {
    /// The device's UUID
    pub device_id: Uuid,
    /// Display name of the device
    pub name: Option<String>,
    /// Whether the device is currently online
    pub is_online: bool,
    /// Last known location
    pub last_location: Option<LastLocationInfo>,
}

/// Last location info for device.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LastLocationInfo {
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: DateTime<Utc>,
}

/// Member response in list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MemberResponse {
    pub id: Uuid,
    pub user: UserPublic,
    pub role: GroupRole,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devices: Option<Vec<MemberDeviceInfo>>,
    /// Number of devices this member has in the group (Story UGM-3.6)
    pub device_count: i64,
}

/// Response for listing members.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListMembersResponse {
    pub data: Vec<MemberResponse>,
    pub pagination: Pagination,
}

/// Response when removing a member.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RemoveMemberResponse {
    pub removed: bool,
    pub user_id: Uuid,
    pub group_id: Uuid,
}

// ============================================================================
// Role Management DTOs (Story 11.3)
// ============================================================================

/// Request to update a member's role.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateRoleRequest {
    pub role: GroupRole,
}

/// Response after updating a member's role.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateRoleResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub role: GroupRole,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Ownership Transfer DTOs (Story 11.6)
// ============================================================================

/// Request to transfer group ownership.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TransferOwnershipRequest {
    /// The user ID of the new owner (must be existing group member).
    pub new_owner_id: Uuid,
}

/// Response after transferring group ownership.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TransferOwnershipResponse {
    pub group_id: Uuid,
    pub previous_owner_id: Uuid,
    pub new_owner_id: Uuid,
    pub transferred_at: DateTime<Utc>,
}

/// Helper function to generate URL-safe slug from name.
pub fn generate_slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                ' ' // Will be filtered out
            }
        })
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_role_as_str() {
        assert_eq!(GroupRole::Owner.as_str(), "owner");
        assert_eq!(GroupRole::Admin.as_str(), "admin");
        assert_eq!(GroupRole::Member.as_str(), "member");
        assert_eq!(GroupRole::Viewer.as_str(), "viewer");
    }

    #[test]
    fn test_group_role_from_str() {
        assert_eq!(GroupRole::from_str("owner").unwrap(), GroupRole::Owner);
        assert_eq!(GroupRole::from_str("ADMIN").unwrap(), GroupRole::Admin);
        assert_eq!(GroupRole::from_str("Member").unwrap(), GroupRole::Member);
        assert_eq!(GroupRole::from_str("viewer").unwrap(), GroupRole::Viewer);
        assert!(GroupRole::from_str("invalid").is_err());
    }

    #[test]
    fn test_group_role_display() {
        assert_eq!(format!("{}", GroupRole::Owner), "owner");
        assert_eq!(format!("{}", GroupRole::Admin), "admin");
    }

    #[test]
    fn test_group_role_permissions() {
        // Owner can do everything
        assert!(GroupRole::Owner.can_manage_group());
        assert!(GroupRole::Owner.can_manage_members());
        assert!(GroupRole::Owner.can_view_locations());
        assert!(GroupRole::Owner.can_delete_group());
        assert!(GroupRole::Owner.can_transfer_ownership());

        // Admin can manage but not delete/transfer
        assert!(GroupRole::Admin.can_manage_group());
        assert!(GroupRole::Admin.can_manage_members());
        assert!(GroupRole::Admin.can_view_locations());
        assert!(!GroupRole::Admin.can_delete_group());
        assert!(!GroupRole::Admin.can_transfer_ownership());

        // Member can only view
        assert!(!GroupRole::Member.can_manage_group());
        assert!(!GroupRole::Member.can_manage_members());
        assert!(GroupRole::Member.can_view_locations());
        assert!(!GroupRole::Member.can_delete_group());
        assert!(!GroupRole::Member.can_transfer_ownership());

        // Viewer can only view
        assert!(!GroupRole::Viewer.can_manage_group());
        assert!(!GroupRole::Viewer.can_manage_members());
        assert!(GroupRole::Viewer.can_view_locations());
        assert!(!GroupRole::Viewer.can_delete_group());
        assert!(!GroupRole::Viewer.can_transfer_ownership());
    }

    #[test]
    fn test_generate_slug() {
        assert_eq!(generate_slug("Smith Family"), "smith-family");
        assert_eq!(generate_slug("My Awesome Group!"), "my-awesome-group");
        assert_eq!(generate_slug("Test   Group"), "test-group");
        assert_eq!(generate_slug("family-group"), "family-group");
        assert_eq!(generate_slug("Group123"), "group123");
        assert_eq!(generate_slug("  Spaces  Everywhere  "), "spaces-everywhere");
    }

    #[test]
    fn test_create_group_request_validation() {
        let valid_request = CreateGroupRequest {
            name: "My Group".to_string(),
            description: Some("A test group".to_string()),
            icon_emoji: Some("👨‍👩‍👧".to_string()),
            max_devices: Some(20),
        };
        assert!(valid_request.validate().is_ok());

        let empty_name = CreateGroupRequest {
            name: "".to_string(),
            description: None,
            icon_emoji: None,
            max_devices: None,
        };
        assert!(empty_name.validate().is_err());

        let too_many_devices = CreateGroupRequest {
            name: "Test".to_string(),
            description: None,
            icon_emoji: None,
            max_devices: Some(200),
        };
        assert!(too_many_devices.validate().is_err());
    }
}
