//! Organization member invite entity (database row mapping).
//!
//! Used for inviting new members to organizations.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the org_member_invites table.
#[derive(Debug, Clone, FromRow)]
pub struct OrgMemberInviteEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub token: String,
    pub email: String,
    pub role: String,
    pub invited_by: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub accepted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub note: Option<String>,
}

impl OrgMemberInviteEntity {
    /// Check if this invite is valid (not expired, not accepted).
    pub fn is_valid(&self) -> bool {
        self.accepted_at.is_none() && self.expires_at > Utc::now()
    }

    /// Check if this invite is pending (not accepted).
    pub fn is_pending(&self) -> bool {
        self.accepted_at.is_none()
    }

    /// Check if this invite is expired.
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }

    /// Check if this invite can be used by the given email.
    pub fn can_be_used_by(&self, email: &str) -> bool {
        self.email.eq_ignore_ascii_case(email)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_invite(
        accepted_at: Option<DateTime<Utc>>,
        expires_at: DateTime<Utc>,
    ) -> OrgMemberInviteEntity {
        OrgMemberInviteEntity {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            token: "test_token_abc123".to_string(),
            email: "invitee@example.com".to_string(),
            role: "member".to_string(),
            invited_by: Some(Uuid::new_v4()),
            expires_at,
            accepted_at,
            accepted_by: None,
            created_at: Utc::now(),
            note: Some("Welcome to the team!".to_string()),
        }
    }

    #[test]
    fn test_is_valid_pending_not_expired() {
        let invite = create_test_invite(None, Utc::now() + Duration::days(7));
        assert!(invite.is_valid());
    }

    #[test]
    fn test_is_valid_accepted() {
        let invite = create_test_invite(Some(Utc::now()), Utc::now() + Duration::days(7));
        assert!(!invite.is_valid());
    }

    #[test]
    fn test_is_valid_expired() {
        let invite = create_test_invite(None, Utc::now() - Duration::days(1));
        assert!(!invite.is_valid());
    }

    #[test]
    fn test_is_pending() {
        let pending = create_test_invite(None, Utc::now() + Duration::days(7));
        assert!(pending.is_pending());

        let accepted = create_test_invite(Some(Utc::now()), Utc::now() + Duration::days(7));
        assert!(!accepted.is_pending());
    }

    #[test]
    fn test_is_expired() {
        let expired = create_test_invite(None, Utc::now() - Duration::days(1));
        assert!(expired.is_expired());

        let not_expired = create_test_invite(None, Utc::now() + Duration::days(7));
        assert!(!not_expired.is_expired());
    }

    #[test]
    fn test_can_be_used_by_exact_match() {
        let invite = create_test_invite(None, Utc::now() + Duration::days(7));
        assert!(invite.can_be_used_by("invitee@example.com"));
    }

    #[test]
    fn test_can_be_used_by_case_insensitive() {
        let invite = create_test_invite(None, Utc::now() + Duration::days(7));
        assert!(invite.can_be_used_by("INVITEE@EXAMPLE.COM"));
        assert!(invite.can_be_used_by("Invitee@Example.Com"));
    }

    #[test]
    fn test_can_be_used_by_wrong_email() {
        let invite = create_test_invite(None, Utc::now() + Duration::days(7));
        assert!(!invite.can_be_used_by("other@example.com"));
    }

    #[test]
    fn test_entity_debug() {
        let invite = create_test_invite(None, Utc::now() + Duration::days(7));
        let debug_str = format!("{:?}", invite);
        assert!(debug_str.contains("OrgMemberInviteEntity"));
        assert!(debug_str.contains("invitee@example.com"));
    }

    #[test]
    fn test_entity_clone() {
        let invite = create_test_invite(None, Utc::now() + Duration::days(7));
        let cloned = invite.clone();
        assert_eq!(cloned.id, invite.id);
        assert_eq!(cloned.email, invite.email);
        assert_eq!(cloned.role, invite.role);
    }
}
