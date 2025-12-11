//! Organization role domain models for custom RBAC.
//!
//! Story AP-1.2: Create Custom Role
//! Story AP-1.3: Delete Custom Role

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Organization role domain model.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "snake_case")]
pub struct OrganizationRole {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub is_system_role: bool,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

/// Request to create a custom organization role.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateOrganizationRoleRequest {
    #[validate(length(min = 1, max = 50, message = "Name must be 1-50 characters"))]
    pub name: String,
    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    pub display_name: String,
    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,
    #[validate(length(min = 1, message = "At least one permission is required"))]
    pub permissions: Vec<String>,
}

/// Response for creating an organization role.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrganizationRoleResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub is_system_role: bool,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<OrganizationRole> for OrganizationRoleResponse {
    fn from(role: OrganizationRole) -> Self {
        Self {
            id: role.id,
            organization_id: role.organization_id,
            name: role.name,
            display_name: role.display_name,
            description: role.description,
            permissions: role.permissions,
            is_system_role: role.is_system_role,
            priority: role.priority,
            created_at: role.created_at,
            updated_at: role.updated_at,
        }
    }
}

/// Response for listing organization roles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListOrganizationRolesResponse {
    pub data: Vec<OrganizationRoleResponse>,
    pub system_roles: Vec<OrganizationRoleResponse>,
    pub custom_roles: Vec<OrganizationRoleResponse>,
}

/// Query parameters for listing organization roles.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ListOrganizationRolesQuery {
    /// Filter to show only system roles
    pub system_only: Option<bool>,
    /// Filter to show only custom roles
    pub custom_only: Option<bool>,
}

/// Response for deleting an organization role.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeleteOrganizationRoleResponse {
    pub deleted: bool,
    pub role_id: Uuid,
    pub deleted_at: DateTime<Utc>,
}

/// System role names that cannot be deleted or duplicated.
pub const SYSTEM_ROLE_NAMES: &[&str] = &["owner", "admin", "member"];

/// Check if a role name is a system role.
pub fn is_system_role_name(name: &str) -> bool {
    SYSTEM_ROLE_NAMES.contains(&name.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_system_role_name() {
        assert!(is_system_role_name("owner"));
        assert!(is_system_role_name("Owner"));
        assert!(is_system_role_name("ADMIN"));
        assert!(is_system_role_name("member"));
        assert!(!is_system_role_name("custom_role"));
        assert!(!is_system_role_name("viewer"));
    }

    #[test]
    fn test_organization_role_response_from() {
        let role = OrganizationRole {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "test_role".to_string(),
            display_name: "Test Role".to_string(),
            description: Some("A test role".to_string()),
            permissions: vec!["device:read".to_string()],
            is_system_role: false,
            priority: 50,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
        };

        let response: OrganizationRoleResponse = role.clone().into();
        assert_eq!(response.id, role.id);
        assert_eq!(response.name, "test_role");
        assert!(!response.is_system_role);
    }

    #[test]
    fn test_create_request_validation() {
        let request = CreateOrganizationRoleRequest {
            name: "custom_role".to_string(),
            display_name: "Custom Role".to_string(),
            description: None,
            permissions: vec!["device:read".to_string()],
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_request_validation_empty_name() {
        let request = CreateOrganizationRoleRequest {
            name: "".to_string(),
            display_name: "Custom Role".to_string(),
            description: None,
            permissions: vec!["device:read".to_string()],
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_request_validation_empty_permissions() {
        let request = CreateOrganizationRoleRequest {
            name: "custom_role".to_string(),
            display_name: "Custom Role".to_string(),
            description: None,
            permissions: vec![],
        };

        assert!(request.validate().is_err());
    }
}
