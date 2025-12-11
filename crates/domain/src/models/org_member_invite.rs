//! Organization member invitation domain models.
//!
//! Request/response DTOs for the member invitation API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Maximum invitations per organization.
pub const MAX_INVITATIONS_PER_ORG: i64 = 100;

/// Default expiration days for invitations.
pub const DEFAULT_EXPIRATION_DAYS: i32 = 7;

/// Maximum expiration days for invitations.
pub const MAX_EXPIRATION_DAYS: i32 = 30;

/// Minimum expiration days for invitations.
pub const MIN_EXPIRATION_DAYS: i32 = 1;

/// Request to create a new member invitation.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateInvitationRequest {
    /// Email address of the invitee.
    #[validate(email(message = "Invalid email address"))]
    #[validate(length(max = 255, message = "Email must be at most 255 characters"))]
    pub email: String,

    /// Role to assign when accepted ("admin" or "member").
    #[validate(custom(function = "validate_role"))]
    pub role: Option<String>,

    /// Optional note for admin tracking.
    #[validate(length(max = 255, message = "Note must be at most 255 characters"))]
    pub note: Option<String>,

    /// Days until expiration (1-30, default: 7).
    #[validate(range(
        min = 1,
        max = 30,
        message = "Expiration must be between 1 and 30 days"
    ))]
    pub expires_in_days: Option<i32>,
}

/// Response after creating an invitation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateInvitationResponse {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub token: String,
    pub invite_url: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Invitation response (for listing/getting).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InvitationResponse {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub status: InvitationStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invited_by: Option<InvitedByInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_at: Option<DateTime<Utc>>,
}

/// Information about who created the invitation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InvitedByInfo {
    pub id: Uuid,
    pub email: String,
}

/// Invitation status.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
}

/// Query parameters for listing invitations.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ListInvitationsQuery {
    /// Filter by status: "pending", "accepted", "expired", "all" (default: "pending").
    pub status: Option<String>,

    /// Page number (default: 1).
    pub page: Option<i64>,

    /// Items per page (default: 50, max: 100).
    pub per_page: Option<i64>,
}

impl ListInvitationsQuery {
    /// Get the page number (1-indexed).
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    /// Get items per page (clamped to 1-100).
    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(50).clamp(1, 100)
    }

    /// Get the offset for pagination.
    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }

    /// Check if we should include accepted invitations.
    pub fn include_accepted(&self) -> bool {
        matches!(self.status.as_deref(), Some("accepted") | Some("all"))
    }
}

/// Response for listing invitations.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListInvitationsResponse {
    pub invitations: Vec<InvitationResponse>,
    pub pagination: InvitationPagination,
    pub summary: InvitationSummary,
}

/// Pagination info for invitations.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InvitationPagination {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

impl InvitationPagination {
    pub fn new(page: i64, per_page: i64, total: i64) -> Self {
        let total_pages = (total + per_page - 1) / per_page;
        Self {
            page,
            per_page,
            total,
            total_pages,
        }
    }
}

/// Summary counts for invitations.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InvitationSummary {
    pub pending: i64,
    pub accepted: i64,
    pub expired: i64,
}

/// Request to accept an invitation.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct AcceptInvitationRequest {
    /// Password for the new account.
    #[validate(length(
        min = 8,
        max = 128,
        message = "Password must be between 8 and 128 characters"
    ))]
    pub password: String,

    /// Display name for the new user.
    #[validate(length(
        min = 1,
        max = 100,
        message = "Display name must be between 1 and 100 characters"
    ))]
    pub display_name: String,
}

/// Response after accepting an invitation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AcceptInvitationResponse {
    pub user: AcceptedUserInfo,
    pub organization: AcceptedOrgInfo,
    pub role: String,
    pub access_token: String,
    pub token_expires_at: DateTime<Utc>,
}

/// User info in accept response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AcceptedUserInfo {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
}

/// Organization info in accept response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AcceptedOrgInfo {
    pub id: Uuid,
    pub name: String,
}

/// Validate role value.
fn validate_role(role: &str) -> Result<(), validator::ValidationError> {
    match role {
        "admin" | "member" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_role");
            err.message = Some("Role must be 'admin' or 'member'".into());
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_invitation_request_validation() {
        let valid = CreateInvitationRequest {
            email: "test@example.com".to_string(),
            role: Some("member".to_string()),
            note: Some("Welcome!".to_string()),
            expires_in_days: Some(7),
        };
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_create_invitation_invalid_email() {
        let invalid = CreateInvitationRequest {
            email: "not-an-email".to_string(),
            role: None,
            note: None,
            expires_in_days: None,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_create_invitation_invalid_role() {
        let invalid = CreateInvitationRequest {
            email: "test@example.com".to_string(),
            role: Some("superadmin".to_string()),
            note: None,
            expires_in_days: None,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_create_invitation_invalid_expiration() {
        let invalid = CreateInvitationRequest {
            email: "test@example.com".to_string(),
            role: None,
            note: None,
            expires_in_days: Some(365), // Too long
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_list_invitations_query_defaults() {
        let query = ListInvitationsQuery::default();
        assert_eq!(query.page(), 1);
        assert_eq!(query.per_page(), 50);
        assert_eq!(query.offset(), 0);
        assert!(!query.include_accepted());
    }

    #[test]
    fn test_list_invitations_query_with_values() {
        let query = ListInvitationsQuery {
            status: Some("all".to_string()),
            page: Some(3),
            per_page: Some(25),
        };
        assert_eq!(query.page(), 3);
        assert_eq!(query.per_page(), 25);
        assert_eq!(query.offset(), 50);
        assert!(query.include_accepted());
    }

    #[test]
    fn test_list_invitations_query_clamping() {
        let query = ListInvitationsQuery {
            status: None,
            page: Some(-5),
            per_page: Some(500),
        };
        assert_eq!(query.page(), 1);
        assert_eq!(query.per_page(), 100);
    }

    #[test]
    fn test_pagination_new() {
        let pagination = InvitationPagination::new(2, 25, 75);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 25);
        assert_eq!(pagination.total, 75);
        assert_eq!(pagination.total_pages, 3);
    }

    #[test]
    fn test_accept_invitation_request_validation() {
        let valid = AcceptInvitationRequest {
            password: "SecurePass123!".to_string(),
            display_name: "John Smith".to_string(),
        };
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_accept_invitation_short_password() {
        let invalid = AcceptInvitationRequest {
            password: "short".to_string(),
            display_name: "John Smith".to_string(),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_accept_invitation_empty_display_name() {
        let invalid = AcceptInvitationRequest {
            password: "SecurePass123!".to_string(),
            display_name: "".to_string(),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_invitation_status_serialization() {
        assert_eq!(
            serde_json::to_string(&InvitationStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&InvitationStatus::Accepted).unwrap(),
            "\"accepted\""
        );
        assert_eq!(
            serde_json::to_string(&InvitationStatus::Expired).unwrap(),
            "\"expired\""
        );
    }
}
