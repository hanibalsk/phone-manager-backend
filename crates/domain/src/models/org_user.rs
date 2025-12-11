//! Organization user domain models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

/// Available permissions for organization users.
pub const PERMISSIONS: &[&str] = &[
    "device:read",
    "device:manage",
    "user:read",
    "user:manage",
    "policy:read",
    "policy:manage",
    "audit:read",
];

/// Roles for organization users.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrgUserRole {
    Owner,
    Admin,
    Member,
}

impl OrgUserRole {
    /// Check if this role has at least the specified role level.
    pub fn has_at_least(&self, required: OrgUserRole) -> bool {
        match (self, required) {
            (OrgUserRole::Owner, _) => true,
            (OrgUserRole::Admin, OrgUserRole::Member) => true,
            (OrgUserRole::Admin, OrgUserRole::Admin) => true,
            (OrgUserRole::Admin, OrgUserRole::Owner) => false,
            (OrgUserRole::Member, OrgUserRole::Member) => true,
            (OrgUserRole::Member, _) => false,
        }
    }

    /// Get default permissions for this role.
    pub fn default_permissions(&self) -> Vec<String> {
        match self {
            OrgUserRole::Owner => PERMISSIONS.iter().map(|s| s.to_string()).collect(),
            OrgUserRole::Admin => vec![
                "device:read".to_string(),
                "device:manage".to_string(),
                "user:read".to_string(),
                "user:manage".to_string(),
                "policy:read".to_string(),
            ],
            OrgUserRole::Member => vec!["device:read".to_string(), "user:read".to_string()],
        }
    }
}

impl FromStr for OrgUserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(OrgUserRole::Owner),
            "admin" => Ok(OrgUserRole::Admin),
            "member" => Ok(OrgUserRole::Member),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}

impl std::fmt::Display for OrgUserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrgUserRole::Owner => write!(f, "owner"),
            OrgUserRole::Admin => write!(f, "admin"),
            OrgUserRole::Member => write!(f, "member"),
        }
    }
}

/// Organization user domain model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrgUser {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub role: OrgUserRole,
    pub permissions: Vec<String>,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
    // Suspension fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspended_by: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspension_reason: Option<String>,
}

impl OrgUser {
    /// Check if this user has the specified permission.
    pub fn has_permission(&self, permission: &str) -> bool {
        // Owners have all permissions implicitly
        if self.role == OrgUserRole::Owner {
            return true;
        }
        self.permissions.contains(&permission.to_string())
    }

    /// Check if this user can manage other users with the specified role.
    pub fn can_manage_role(&self, target_role: OrgUserRole) -> bool {
        match self.role {
            OrgUserRole::Owner => true,
            OrgUserRole::Admin => target_role == OrgUserRole::Member,
            OrgUserRole::Member => false,
        }
    }

    /// Check if this user is currently suspended.
    pub fn is_suspended(&self) -> bool {
        self.suspended_at.is_some()
    }
}

/// User info for organization user responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrgUserInfo {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
}

/// Organization user with user details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrgUserWithDetails {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user: OrgUserInfo,
    pub role: OrgUserRole,
    pub permissions: Vec<String>,
    pub granted_at: DateTime<Utc>,
    // Suspension fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspended_by: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspension_reason: Option<String>,
}

impl OrgUserWithDetails {
    /// Check if this user is currently suspended.
    pub fn is_suspended(&self) -> bool {
        self.suspended_at.is_some()
    }
}

/// Request to add a user to an organization.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct AddOrgUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub role: OrgUserRole,
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
}

/// Request to update an organization user.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateOrgUserRequest {
    pub role: Option<OrgUserRole>,
    pub permissions: Option<Vec<String>>,
}

/// Response for add/update organization user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrgUserResponse {
    #[serde(flatten)]
    pub org_user: OrgUserWithDetails,
}

/// Response for list organization users.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListOrgUsersResponse {
    pub data: Vec<OrgUserWithDetails>,
    pub pagination: OrgUserPagination,
}

/// Pagination info for organization users list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrgUserPagination {
    pub page: i32,
    pub per_page: i32,
    pub total: i64,
    pub total_pages: i32,
}

/// Query parameters for listing organization users.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ListOrgUsersQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub role: Option<OrgUserRole>,
}

/// Validate that all permissions in the list are valid.
pub fn validate_permissions(permissions: &[String]) -> Result<(), String> {
    for perm in permissions {
        if !PERMISSIONS.contains(&perm.as_str()) {
            return Err(format!("Invalid permission: {}", perm));
        }
    }
    Ok(())
}

/// Request to suspend an organization user.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct SuspendOrgUserRequest {
    #[validate(length(
        max = 500,
        message = "Suspension reason must not exceed 500 characters"
    ))]
    pub reason: Option<String>,
}

/// Response for suspend organization user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SuspendOrgUserResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspension_reason: Option<String>,
    pub message: String,
}

/// Response for reactivate organization user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReactivateOrgUserResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub reactivated_at: DateTime<Utc>,
    pub message: String,
}

/// Response for admin-triggered password reset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TriggerPasswordResetResponse {
    pub user_id: Uuid,
    pub email: String,
    pub reset_token_sent: bool,
    pub expires_at: DateTime<Utc>,
    pub message: String,
}

/// MFA method types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MfaMethod {
    Totp,
    Sms,
    Email,
}

impl std::fmt::Display for MfaMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MfaMethod::Totp => write!(f, "totp"),
            MfaMethod::Sms => write!(f, "sms"),
            MfaMethod::Email => write!(f, "email"),
        }
    }
}

impl std::str::FromStr for MfaMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "totp" => Ok(MfaMethod::Totp),
            "sms" => Ok(MfaMethod::Sms),
            "email" => Ok(MfaMethod::Email),
            _ => Err(format!("Unknown MFA method: {}", s)),
        }
    }
}

/// Response for getting user MFA status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MfaStatusResponse {
    pub user_id: Uuid,
    pub mfa_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_method: Option<MfaMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrolled_at: Option<DateTime<Utc>>,
    pub mfa_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_by: Option<Uuid>,
}

/// Response for forcing MFA enrollment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ForceMfaResponse {
    pub user_id: Uuid,
    pub mfa_required: bool,
    pub required_at: DateTime<Utc>,
    pub required_by: Uuid,
    pub message: String,
}

/// Response for resetting user MFA.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResetMfaResponse {
    pub user_id: Uuid,
    pub mfa_reset: bool,
    pub reset_at: DateTime<Utc>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_org_user_role_serialization() {
        assert_eq!(
            serde_json::to_string(&OrgUserRole::Admin).unwrap(),
            "\"admin\""
        );
    }

    #[test]
    fn test_org_user_role_deserialization() {
        let role: OrgUserRole = serde_json::from_str("\"owner\"").unwrap();
        assert_eq!(role, OrgUserRole::Owner);
    }

    #[test]
    fn test_org_user_role_from_str() {
        assert_eq!(OrgUserRole::from_str("owner").unwrap(), OrgUserRole::Owner);
        assert_eq!(OrgUserRole::from_str("ADMIN").unwrap(), OrgUserRole::Admin);
        assert!(OrgUserRole::from_str("invalid").is_err());
    }

    #[test]
    fn test_role_has_at_least() {
        assert!(OrgUserRole::Owner.has_at_least(OrgUserRole::Owner));
        assert!(OrgUserRole::Owner.has_at_least(OrgUserRole::Admin));
        assert!(OrgUserRole::Owner.has_at_least(OrgUserRole::Member));
        assert!(OrgUserRole::Admin.has_at_least(OrgUserRole::Admin));
        assert!(OrgUserRole::Admin.has_at_least(OrgUserRole::Member));
        assert!(!OrgUserRole::Admin.has_at_least(OrgUserRole::Owner));
        assert!(!OrgUserRole::Member.has_at_least(OrgUserRole::Admin));
    }

    #[test]
    fn test_default_permissions() {
        let owner_perms = OrgUserRole::Owner.default_permissions();
        assert_eq!(owner_perms.len(), PERMISSIONS.len());

        let admin_perms = OrgUserRole::Admin.default_permissions();
        assert!(admin_perms.contains(&"device:manage".to_string()));
        assert!(!admin_perms.contains(&"audit:read".to_string()));

        let member_perms = OrgUserRole::Member.default_permissions();
        assert!(member_perms.contains(&"device:read".to_string()));
        assert!(!member_perms.contains(&"device:manage".to_string()));
    }

    #[test]
    fn test_has_permission() {
        let owner = OrgUser {
            id: Uuid::nil(),
            organization_id: Uuid::nil(),
            user_id: Uuid::nil(),
            role: OrgUserRole::Owner,
            permissions: vec![],
            granted_at: Utc::now(),
            granted_by: None,
            suspended_at: None,
            suspended_by: None,
            suspension_reason: None,
        };
        // Owner has all permissions implicitly
        assert!(owner.has_permission("audit:read"));

        let admin = OrgUser {
            id: Uuid::nil(),
            organization_id: Uuid::nil(),
            user_id: Uuid::nil(),
            role: OrgUserRole::Admin,
            permissions: vec!["device:manage".to_string()],
            granted_at: Utc::now(),
            granted_by: None,
            suspended_at: None,
            suspended_by: None,
            suspension_reason: None,
        };
        assert!(admin.has_permission("device:manage"));
        assert!(!admin.has_permission("audit:read"));
    }

    #[test]
    fn test_can_manage_role() {
        let owner = OrgUser {
            id: Uuid::nil(),
            organization_id: Uuid::nil(),
            user_id: Uuid::nil(),
            role: OrgUserRole::Owner,
            permissions: vec![],
            granted_at: Utc::now(),
            granted_by: None,
            suspended_at: None,
            suspended_by: None,
            suspension_reason: None,
        };
        assert!(owner.can_manage_role(OrgUserRole::Owner));
        assert!(owner.can_manage_role(OrgUserRole::Admin));

        let admin = OrgUser {
            id: Uuid::nil(),
            organization_id: Uuid::nil(),
            user_id: Uuid::nil(),
            role: OrgUserRole::Admin,
            permissions: vec![],
            granted_at: Utc::now(),
            granted_by: None,
            suspended_at: None,
            suspended_by: None,
            suspension_reason: None,
        };
        assert!(!admin.can_manage_role(OrgUserRole::Owner));
        assert!(!admin.can_manage_role(OrgUserRole::Admin));
        assert!(admin.can_manage_role(OrgUserRole::Member));
    }

    #[test]
    fn test_is_suspended() {
        let active_user = OrgUser {
            id: Uuid::nil(),
            organization_id: Uuid::nil(),
            user_id: Uuid::nil(),
            role: OrgUserRole::Member,
            permissions: vec![],
            granted_at: Utc::now(),
            granted_by: None,
            suspended_at: None,
            suspended_by: None,
            suspension_reason: None,
        };
        assert!(!active_user.is_suspended());

        let suspended_user = OrgUser {
            id: Uuid::nil(),
            organization_id: Uuid::nil(),
            user_id: Uuid::nil(),
            role: OrgUserRole::Member,
            permissions: vec![],
            granted_at: Utc::now(),
            granted_by: None,
            suspended_at: Some(Utc::now()),
            suspended_by: Some(Uuid::nil()),
            suspension_reason: Some("Policy violation".to_string()),
        };
        assert!(suspended_user.is_suspended());
    }

    #[test]
    fn test_validate_permissions() {
        assert!(validate_permissions(&["device:read".to_string()]).is_ok());
        assert!(validate_permissions(&["invalid:perm".to_string()]).is_err());
    }
}
