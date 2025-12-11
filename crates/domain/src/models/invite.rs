//! Invite domain models for group invitations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::group::GroupRole;

/// Represents a group invitation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GroupInvite {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub preset_role: GroupRole,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Request to create a new invite.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateInviteRequest {
    /// Role to assign when joining (default: member). Cannot be owner.
    pub preset_role: Option<GroupRole>,

    /// Maximum uses (1-100, default: 1)
    #[validate(range(min = 1, max = 100, message = "max_uses must be between 1 and 100"))]
    pub max_uses: Option<i32>,

    /// Hours until expiry (1-168, default: 24)
    #[validate(range(
        min = 1,
        max = 168,
        message = "expires_in_hours must be between 1 and 168"
    ))]
    pub expires_in_hours: Option<i32>,
}

/// Response after creating an invite.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateInviteResponse {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub preset_role: GroupRole,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub invite_url: String,
}

/// Summary of an invite for listing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InviteSummary {
    pub id: Uuid,
    pub code: String,
    pub preset_role: GroupRole,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: DateTime<Utc>,
    pub created_by: CreatorInfo,
    pub created_at: DateTime<Utc>,
}

/// Creator info for invite listing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CreatorInfo {
    pub id: Uuid,
    pub display_name: Option<String>,
}

/// Response for listing invites.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListInvitesResponse {
    pub data: Vec<InviteSummary>,
}

/// Public invite info (for GET /invites/:code without auth).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PublicInviteInfo {
    pub group: PublicGroupInfo,
    pub preset_role: GroupRole,
    pub expires_at: DateTime<Utc>,
    pub is_valid: bool,
}

/// Public group info for invite preview.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PublicGroupInfo {
    pub name: String,
    pub icon_emoji: Option<String>,
    pub member_count: i64,
}

/// Request to join a group using an invite code.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct JoinGroupRequest {
    /// The invite code in XXX-XXX-XXX format.
    #[validate(length(equal = 11, message = "Invalid invite code format"))]
    #[validate(regex(
        path = *INVITE_CODE_REGEX,
        message = "Invalid invite code format. Expected XXX-XXX-XXX"
    ))]
    pub code: String,
}

lazy_static::lazy_static! {
    static ref INVITE_CODE_REGEX: regex::Regex =
        regex::Regex::new(r"^[A-Z0-9]{3}-[A-Z0-9]{3}-[A-Z0-9]{3}$").unwrap();
}

/// Summary of group info for join response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct JoinGroupInfo {
    pub id: Uuid,
    pub name: String,
    pub member_count: i64,
}

/// Summary of membership info for join response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct JoinMembershipInfo {
    pub id: Uuid,
    pub role: GroupRole,
    pub joined_at: DateTime<Utc>,
}

/// Response after joining a group.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct JoinGroupResponse {
    pub group: JoinGroupInfo,
    pub membership: JoinMembershipInfo,
}

/// Generate a random invite code in XXX-XXX-XXX format.
pub fn generate_invite_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chars: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Avoiding confusing chars: 0, O, I, 1

    let mut generate_segment = || -> String {
        (0..3)
            .map(|_| {
                let idx = rng.gen_range(0..chars.len());
                chars[idx] as char
            })
            .collect()
    };

    format!(
        "{}-{}-{}",
        generate_segment(),
        generate_segment(),
        generate_segment()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_invite_code_format() {
        let code = generate_invite_code();
        assert_eq!(code.len(), 11); // XXX-XXX-XXX
        assert_eq!(&code[3..4], "-");
        assert_eq!(&code[7..8], "-");

        // Check all chars are valid (uppercase letters or digits, excluding confusing ones)
        for (i, c) in code.chars().enumerate() {
            if i == 3 || i == 7 {
                assert_eq!(c, '-');
            } else {
                assert!(
                    c.is_ascii_uppercase() || c.is_ascii_digit(),
                    "Invalid char: {}",
                    c
                );
                assert!(c != 'O' && c != 'I' && c != '0' && c != '1');
            }
        }
    }

    #[test]
    fn test_generate_invite_code_uniqueness() {
        // Generate multiple codes and check they're all different
        let codes: Vec<String> = (0..100).map(|_| generate_invite_code()).collect();
        let unique_codes: std::collections::HashSet<_> = codes.iter().collect();
        // With such a large character space, duplicates should be extremely rare
        assert!(unique_codes.len() >= 99);
    }

    #[test]
    fn test_create_invite_request_validation() {
        let valid = CreateInviteRequest {
            preset_role: Some(GroupRole::Member),
            max_uses: Some(5),
            expires_in_hours: Some(24),
        };
        assert!(valid.validate().is_ok());

        let too_many_uses = CreateInviteRequest {
            preset_role: None,
            max_uses: Some(200),
            expires_in_hours: None,
        };
        assert!(too_many_uses.validate().is_err());

        let too_long_expiry = CreateInviteRequest {
            preset_role: None,
            max_uses: None,
            expires_in_hours: Some(1000),
        };
        assert!(too_long_expiry.validate().is_err());
    }
}
