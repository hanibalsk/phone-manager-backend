//! Permission domain models for organization-level RBAC.
//!
//! Story AP-1.1: List Permissions

use serde::{Deserialize, Serialize};

/// Permission category for grouping related permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionCategory {
    Devices,
    Users,
    Policies,
    Audit,
}

impl std::fmt::Display for PermissionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionCategory::Devices => write!(f, "devices"),
            PermissionCategory::Users => write!(f, "users"),
            PermissionCategory::Policies => write!(f, "policies"),
            PermissionCategory::Audit => write!(f, "audit"),
        }
    }
}

/// A permission with full metadata including name, description, and category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Permission {
    /// The permission identifier (e.g., "device:read")
    pub name: String,
    /// Human-readable description of what this permission allows
    pub description: String,
    /// Category for grouping permissions
    pub category: PermissionCategory,
}

/// Response for listing permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListPermissionsResponse {
    pub data: Vec<Permission>,
    /// Permissions grouped by category
    pub by_category: PermissionsByCategory,
}

/// Permissions organized by category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PermissionsByCategory {
    pub devices: Vec<Permission>,
    pub users: Vec<Permission>,
    pub policies: Vec<Permission>,
    pub audit: Vec<Permission>,
}

/// Query parameters for listing permissions.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ListPermissionsQuery {
    /// Filter by category
    pub category: Option<String>,
}

/// All available organization-level permissions with metadata.
pub fn get_all_permissions() -> Vec<Permission> {
    vec![
        // Device permissions
        Permission {
            name: "device:read".to_string(),
            description: "View devices, their status, and location data".to_string(),
            category: PermissionCategory::Devices,
        },
        Permission {
            name: "device:manage".to_string(),
            description: "Create, update, delete, and manage device settings".to_string(),
            category: PermissionCategory::Devices,
        },
        // User permissions
        Permission {
            name: "user:read".to_string(),
            description: "View organization members and their roles".to_string(),
            category: PermissionCategory::Users,
        },
        Permission {
            name: "user:manage".to_string(),
            description: "Add, remove, and modify organization member roles".to_string(),
            category: PermissionCategory::Users,
        },
        // Policy permissions
        Permission {
            name: "policy:read".to_string(),
            description: "View device policies and their assignments".to_string(),
            category: PermissionCategory::Policies,
        },
        Permission {
            name: "policy:manage".to_string(),
            description: "Create, update, delete, and assign device policies".to_string(),
            category: PermissionCategory::Policies,
        },
        // Audit permissions
        Permission {
            name: "audit:read".to_string(),
            description: "View audit logs and compliance reports".to_string(),
            category: PermissionCategory::Audit,
        },
    ]
}

/// Get permissions grouped by category.
pub fn get_permissions_by_category() -> PermissionsByCategory {
    let all = get_all_permissions();

    PermissionsByCategory {
        devices: all
            .iter()
            .filter(|p| p.category == PermissionCategory::Devices)
            .cloned()
            .collect(),
        users: all
            .iter()
            .filter(|p| p.category == PermissionCategory::Users)
            .cloned()
            .collect(),
        policies: all
            .iter()
            .filter(|p| p.category == PermissionCategory::Policies)
            .cloned()
            .collect(),
        audit: all
            .iter()
            .filter(|p| p.category == PermissionCategory::Audit)
            .cloned()
            .collect(),
    }
}

/// Get permissions filtered by category.
pub fn get_permissions_by_category_filter(category: &str) -> Vec<Permission> {
    let all = get_all_permissions();
    let category_lower = category.to_lowercase();

    all.into_iter()
        .filter(|p| p.category.to_string() == category_lower)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_permissions() {
        let permissions = get_all_permissions();
        assert_eq!(permissions.len(), 7);

        // Verify all expected permissions exist
        let names: Vec<_> = permissions.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"device:read"));
        assert!(names.contains(&"device:manage"));
        assert!(names.contains(&"user:read"));
        assert!(names.contains(&"user:manage"));
        assert!(names.contains(&"policy:read"));
        assert!(names.contains(&"policy:manage"));
        assert!(names.contains(&"audit:read"));
    }

    #[test]
    fn test_get_permissions_by_category() {
        let by_category = get_permissions_by_category();

        assert_eq!(by_category.devices.len(), 2);
        assert_eq!(by_category.users.len(), 2);
        assert_eq!(by_category.policies.len(), 2);
        assert_eq!(by_category.audit.len(), 1);
    }

    #[test]
    fn test_get_permissions_by_category_filter() {
        let devices = get_permissions_by_category_filter("devices");
        assert_eq!(devices.len(), 2);
        assert!(devices
            .iter()
            .all(|p| p.category == PermissionCategory::Devices));

        let users = get_permissions_by_category_filter("Users"); // Test case-insensitive
        assert_eq!(users.len(), 2);

        let unknown = get_permissions_by_category_filter("unknown");
        assert!(unknown.is_empty());
    }

    #[test]
    fn test_permission_category_display() {
        assert_eq!(PermissionCategory::Devices.to_string(), "devices");
        assert_eq!(PermissionCategory::Users.to_string(), "users");
        assert_eq!(PermissionCategory::Policies.to_string(), "policies");
        assert_eq!(PermissionCategory::Audit.to_string(), "audit");
    }

    #[test]
    fn test_permission_serialization() {
        let perm = Permission {
            name: "device:read".to_string(),
            description: "View devices".to_string(),
            category: PermissionCategory::Devices,
        };

        let json = serde_json::to_string(&perm).unwrap();
        assert!(json.contains("device:read"));
        assert!(json.contains("devices"));
    }
}
