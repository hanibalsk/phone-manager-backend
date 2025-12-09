//! System-level role domain models for admin panel access control.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// Available system-level permissions.
pub const SYSTEM_PERMISSIONS: &[&str] = &[
    "system:read",
    "system:manage",
    "org:create",
    "org:read",
    "org:manage",
    "org:delete",
    "user:read_all",
    "user:manage_all",
    "audit:read_all",
];

/// System-level roles for admin panel access.
/// Users can have multiple system roles simultaneously.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemRole {
    /// Full system access, manage all organizations
    SuperAdmin,
    /// Full management for assigned organizations only
    OrgAdmin,
    /// Manage users/devices in assigned organizations only
    OrgManager,
    /// View-only access for customer support (global read)
    Support,
    /// Read-only system metrics
    Viewer,
}

impl SystemRole {
    /// Check if this role has at least the specified role level.
    /// Hierarchy: SuperAdmin > OrgAdmin > OrgManager > Support > Viewer
    pub fn has_at_least(&self, required: SystemRole) -> bool {
        self.priority() >= required.priority()
    }

    /// Get the priority level for role comparison.
    /// Higher value = more privileges.
    fn priority(&self) -> u8 {
        match self {
            SystemRole::SuperAdmin => 100,
            SystemRole::OrgAdmin => 80,
            SystemRole::OrgManager => 60,
            SystemRole::Support => 40,
            SystemRole::Viewer => 20,
        }
    }

    /// All system roles are built-in and cannot be deleted.
    pub fn is_system_defined(&self) -> bool {
        true
    }

    /// Check if this role requires organization assignment for access.
    /// OrgAdmin and OrgManager must be assigned to specific organizations.
    pub fn requires_org_assignment(&self) -> bool {
        matches!(self, SystemRole::OrgAdmin | SystemRole::OrgManager)
    }

    /// Check if this role has global access (doesn't need org assignment).
    pub fn has_global_access(&self) -> bool {
        matches!(self, SystemRole::SuperAdmin | SystemRole::Support | SystemRole::Viewer)
    }

    /// Get default permissions for this role.
    pub fn default_permissions(&self) -> Vec<&'static str> {
        match self {
            SystemRole::SuperAdmin => SYSTEM_PERMISSIONS.to_vec(),
            SystemRole::OrgAdmin => vec![
                "org:read",
                "org:manage",
                "user:read_all",
                "user:manage_all",
                "audit:read_all",
            ],
            SystemRole::OrgManager => vec![
                "org:read",
                "user:read_all",
                "user:manage_all",
            ],
            SystemRole::Support => vec![
                "system:read",
                "org:read",
                "user:read_all",
                "audit:read_all",
            ],
            SystemRole::Viewer => vec![
                "system:read",
            ],
        }
    }

    /// Get a human-readable description of this role.
    pub fn description(&self) -> &'static str {
        match self {
            SystemRole::SuperAdmin => "Full system access, manage all organizations",
            SystemRole::OrgAdmin => "Full management for assigned organizations",
            SystemRole::OrgManager => "Manage users/devices in assigned organizations",
            SystemRole::Support => "View-only access for customer support",
            SystemRole::Viewer => "Read-only system metrics",
        }
    }

    /// Get all available system roles.
    pub fn all() -> &'static [SystemRole] {
        &[
            SystemRole::SuperAdmin,
            SystemRole::OrgAdmin,
            SystemRole::OrgManager,
            SystemRole::Support,
            SystemRole::Viewer,
        ]
    }
}

impl FromStr for SystemRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "super_admin" | "superadmin" => Ok(SystemRole::SuperAdmin),
            "org_admin" | "orgadmin" => Ok(SystemRole::OrgAdmin),
            "org_manager" | "orgmanager" => Ok(SystemRole::OrgManager),
            "support" => Ok(SystemRole::Support),
            "viewer" => Ok(SystemRole::Viewer),
            _ => Err(format!("Unknown system role: {}", s)),
        }
    }
}

impl std::fmt::Display for SystemRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemRole::SuperAdmin => write!(f, "super_admin"),
            SystemRole::OrgAdmin => write!(f, "org_admin"),
            SystemRole::OrgManager => write!(f, "org_manager"),
            SystemRole::Support => write!(f, "support"),
            SystemRole::Viewer => write!(f, "viewer"),
        }
    }
}

/// User's system role assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserSystemRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: SystemRole,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
}

/// Admin's organization assignment (for org_admin/org_manager roles).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminOrgAssignment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
}

/// Response for listing available system roles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SystemRoleInfo {
    pub role: SystemRole,
    pub is_system_defined: bool,
    pub requires_org_assignment: bool,
    pub description: String,
    pub default_permissions: Vec<String>,
}

impl From<SystemRole> for SystemRoleInfo {
    fn from(role: SystemRole) -> Self {
        Self {
            role,
            is_system_defined: role.is_system_defined(),
            requires_org_assignment: role.requires_org_assignment(),
            description: role.description().to_string(),
            default_permissions: role.default_permissions().iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Response for GET /api/admin/v1/system-roles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListSystemRolesResponse {
    pub data: Vec<SystemRoleInfo>,
}

/// Response for GET /api/admin/v1/users/:user_id/system-roles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserSystemRolesResponse {
    pub user_id: Uuid,
    pub roles: Vec<UserSystemRoleDetail>,
}

/// Detailed info about a user's system role.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserSystemRoleDetail {
    pub id: Uuid,
    pub role: SystemRole,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
}

impl From<UserSystemRole> for UserSystemRoleDetail {
    fn from(usr: UserSystemRole) -> Self {
        Self {
            id: usr.id,
            role: usr.role,
            granted_at: usr.granted_at,
            granted_by: usr.granted_by,
        }
    }
}

/// Request to add a system role to a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AddSystemRoleRequest {
    pub role: SystemRole,
}

/// Response for adding a system role.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AddSystemRoleResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: SystemRole,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
}

impl From<UserSystemRole> for AddSystemRoleResponse {
    fn from(usr: UserSystemRole) -> Self {
        Self {
            id: usr.id,
            user_id: usr.user_id,
            role: usr.role,
            granted_at: usr.granted_at,
            granted_by: usr.granted_by,
        }
    }
}

/// Response for removing a system role.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RemoveSystemRoleResponse {
    pub success: bool,
    pub message: String,
}

/// Response for GET /api/admin/v1/users/:user_id/org-assignments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserOrgAssignmentsResponse {
    pub user_id: Uuid,
    pub assignments: Vec<OrgAssignmentDetail>,
}

/// Detailed info about an org assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrgAssignmentDetail {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub organization_name: Option<String>,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
}

/// Request to assign an organization to a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AssignOrgRequest {
    pub organization_id: Uuid,
}

/// Response for assigning an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AssignOrgResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
}

impl From<AdminOrgAssignment> for AssignOrgResponse {
    fn from(aoa: AdminOrgAssignment) -> Self {
        Self {
            id: aoa.id,
            user_id: aoa.user_id,
            organization_id: aoa.organization_id,
            assigned_at: aoa.assigned_at,
            assigned_by: aoa.assigned_by,
        }
    }
}

/// Response for removing an org assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RemoveOrgAssignmentResponse {
    pub success: bool,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_role_serialization() {
        assert_eq!(
            serde_json::to_string(&SystemRole::SuperAdmin).unwrap(),
            "\"super_admin\""
        );
        assert_eq!(
            serde_json::to_string(&SystemRole::OrgAdmin).unwrap(),
            "\"org_admin\""
        );
    }

    #[test]
    fn test_system_role_deserialization() {
        let role: SystemRole = serde_json::from_str("\"super_admin\"").unwrap();
        assert_eq!(role, SystemRole::SuperAdmin);

        let role: SystemRole = serde_json::from_str("\"org_manager\"").unwrap();
        assert_eq!(role, SystemRole::OrgManager);
    }

    #[test]
    fn test_system_role_from_str() {
        assert_eq!(SystemRole::from_str("super_admin").unwrap(), SystemRole::SuperAdmin);
        assert_eq!(SystemRole::from_str("SUPER_ADMIN").unwrap(), SystemRole::SuperAdmin);
        assert_eq!(SystemRole::from_str("superadmin").unwrap(), SystemRole::SuperAdmin);
        assert_eq!(SystemRole::from_str("org_admin").unwrap(), SystemRole::OrgAdmin);
        assert_eq!(SystemRole::from_str("support").unwrap(), SystemRole::Support);
        assert!(SystemRole::from_str("invalid").is_err());
    }

    #[test]
    fn test_role_has_at_least() {
        // SuperAdmin has at least everything
        assert!(SystemRole::SuperAdmin.has_at_least(SystemRole::SuperAdmin));
        assert!(SystemRole::SuperAdmin.has_at_least(SystemRole::OrgAdmin));
        assert!(SystemRole::SuperAdmin.has_at_least(SystemRole::Viewer));

        // OrgAdmin doesn't have SuperAdmin level
        assert!(!SystemRole::OrgAdmin.has_at_least(SystemRole::SuperAdmin));
        assert!(SystemRole::OrgAdmin.has_at_least(SystemRole::OrgAdmin));
        assert!(SystemRole::OrgAdmin.has_at_least(SystemRole::OrgManager));

        // Viewer only has Viewer level
        assert!(!SystemRole::Viewer.has_at_least(SystemRole::SuperAdmin));
        assert!(!SystemRole::Viewer.has_at_least(SystemRole::Support));
        assert!(SystemRole::Viewer.has_at_least(SystemRole::Viewer));
    }

    #[test]
    fn test_requires_org_assignment() {
        assert!(!SystemRole::SuperAdmin.requires_org_assignment());
        assert!(SystemRole::OrgAdmin.requires_org_assignment());
        assert!(SystemRole::OrgManager.requires_org_assignment());
        assert!(!SystemRole::Support.requires_org_assignment());
        assert!(!SystemRole::Viewer.requires_org_assignment());
    }

    #[test]
    fn test_has_global_access() {
        assert!(SystemRole::SuperAdmin.has_global_access());
        assert!(!SystemRole::OrgAdmin.has_global_access());
        assert!(!SystemRole::OrgManager.has_global_access());
        assert!(SystemRole::Support.has_global_access());
        assert!(SystemRole::Viewer.has_global_access());
    }

    #[test]
    fn test_default_permissions() {
        let super_perms = SystemRole::SuperAdmin.default_permissions();
        assert_eq!(super_perms.len(), SYSTEM_PERMISSIONS.len());

        let org_admin_perms = SystemRole::OrgAdmin.default_permissions();
        assert!(org_admin_perms.contains(&"org:manage"));
        assert!(!org_admin_perms.contains(&"system:manage"));

        let viewer_perms = SystemRole::Viewer.default_permissions();
        assert!(viewer_perms.contains(&"system:read"));
        assert_eq!(viewer_perms.len(), 1);
    }

    #[test]
    fn test_system_role_display() {
        assert_eq!(format!("{}", SystemRole::SuperAdmin), "super_admin");
        assert_eq!(format!("{}", SystemRole::OrgManager), "org_manager");
    }

    #[test]
    fn test_system_role_all() {
        let all = SystemRole::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&SystemRole::SuperAdmin));
        assert!(all.contains(&SystemRole::Viewer));
    }

    #[test]
    fn test_system_role_info_from() {
        let info: SystemRoleInfo = SystemRole::OrgAdmin.into();
        assert_eq!(info.role, SystemRole::OrgAdmin);
        assert!(info.is_system_defined);
        assert!(info.requires_org_assignment);
        assert!(!info.description.is_empty());
    }
}
