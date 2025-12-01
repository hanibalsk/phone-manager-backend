//! Enrollment token domain model for device provisioning.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Enrollment token prefix.
pub const TOKEN_PREFIX: &str = "enroll_";

/// Length of random bytes for token generation.
const TOKEN_RANDOM_BYTES: usize = 45;

/// Enrollment token domain model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EnrollmentToken {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub token: String,
    pub token_prefix: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<i32>,
    pub current_uses: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_assign_user_by_email: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
}

impl EnrollmentToken {
    /// Check if the token is valid for use.
    pub fn is_valid(&self) -> bool {
        // Check if revoked
        if self.revoked_at.is_some() {
            return false;
        }

        // Check expiration
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }

        // Check usage limit
        if let Some(max_uses) = self.max_uses {
            if self.current_uses >= max_uses {
                return false;
            }
        }

        true
    }

    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| Utc::now() > exp)
    }

    /// Check if the token is revoked.
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    /// Check if the token has reached its usage limit.
    pub fn is_exhausted(&self) -> bool {
        self.max_uses.is_some_and(|max| self.current_uses >= max)
    }

    /// Get remaining uses (None if unlimited).
    pub fn remaining_uses(&self) -> Option<i32> {
        self.max_uses.map(|max| max - self.current_uses)
    }
}

/// Response format for enrollment token.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EnrollmentTokenResponse {
    pub id: Uuid,
    pub token: String,
    pub token_prefix: String,
    pub organization_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<i32>,
    pub current_uses: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_assign_user_by_email: bool,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
    pub is_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_uses: Option<i32>,
}

impl From<EnrollmentToken> for EnrollmentTokenResponse {
    fn from(token: EnrollmentToken) -> Self {
        let is_valid = token.is_valid();
        let remaining_uses = token.remaining_uses();

        Self {
            id: token.id,
            token: token.token,
            token_prefix: token.token_prefix,
            organization_id: token.organization_id,
            group_id: token.group_id,
            policy_id: token.policy_id,
            max_uses: token.max_uses,
            current_uses: token.current_uses,
            expires_at: token.expires_at,
            auto_assign_user_by_email: token.auto_assign_user_by_email,
            created_at: token.created_at,
            revoked_at: token.revoked_at,
            is_valid,
            remaining_uses,
        }
    }
}

/// Request to create a new enrollment token.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateEnrollmentTokenRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<Uuid>,
    #[validate(range(min = 1, max = 10000, message = "max_uses must be between 1 and 10000"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<i32>,
    #[validate(range(min = 1, max = 365, message = "expires_in_days must be between 1 and 365"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in_days: Option<i32>,
    #[serde(default)]
    pub auto_assign_user_by_email: bool,
}

/// Query parameters for listing enrollment tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListEnrollmentTokensQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub include_revoked: Option<bool>,
    pub include_expired: Option<bool>,
}

/// Response for listing enrollment tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListEnrollmentTokensResponse {
    pub data: Vec<EnrollmentTokenResponse>,
    pub pagination: EnrollmentTokenPagination,
}

/// Pagination metadata for enrollment tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EnrollmentTokenPagination {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
}

/// Response for QR code generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QrCodeResponse {
    pub qr_data: String,
    pub enrollment_url: String,
}

/// Generate a new enrollment token.
pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..TOKEN_RANDOM_BYTES).map(|_| rng.gen()).collect();
    let encoded = URL_SAFE_NO_PAD.encode(&random_bytes);
    format!("{}{}", TOKEN_PREFIX, encoded)
}

/// Extract token prefix from a full token.
pub fn extract_prefix(token: &str) -> String {
    // Return first 8 characters as prefix
    token.chars().take(8).collect()
}

/// Calculate expiry date from days.
pub fn calculate_expiry(days: i32) -> DateTime<Utc> {
    Utc::now() + Duration::days(days as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token() {
        let token = generate_token();
        assert!(token.starts_with(TOKEN_PREFIX));
        assert!(token.len() > 20); // Should be reasonably long
    }

    #[test]
    fn test_generate_token_uniqueness() {
        let token1 = generate_token();
        let token2 = generate_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_extract_prefix() {
        let token = "enroll_abc123xyz";
        let prefix = extract_prefix(token);
        assert_eq!(prefix, "enroll_a");
    }

    #[test]
    fn test_calculate_expiry() {
        let expiry = calculate_expiry(30);
        let now = Utc::now();
        assert!(expiry > now);
        assert!(expiry < now + Duration::days(31));
    }

    #[test]
    fn test_enrollment_token_is_valid() {
        let token = EnrollmentToken {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            token: "enroll_test".to_string(),
            token_prefix: "enroll_t".to_string(),
            group_id: None,
            policy_id: None,
            max_uses: Some(10),
            current_uses: 5,
            expires_at: Some(Utc::now() + Duration::days(1)),
            auto_assign_user_by_email: false,
            created_by: None,
            created_at: Utc::now(),
            revoked_at: None,
        };
        assert!(token.is_valid());
    }

    #[test]
    fn test_enrollment_token_expired() {
        let token = EnrollmentToken {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            token: "enroll_test".to_string(),
            token_prefix: "enroll_t".to_string(),
            group_id: None,
            policy_id: None,
            max_uses: None,
            current_uses: 0,
            expires_at: Some(Utc::now() - Duration::days(1)),
            auto_assign_user_by_email: false,
            created_by: None,
            created_at: Utc::now(),
            revoked_at: None,
        };
        assert!(!token.is_valid());
        assert!(token.is_expired());
    }

    #[test]
    fn test_enrollment_token_revoked() {
        let token = EnrollmentToken {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            token: "enroll_test".to_string(),
            token_prefix: "enroll_t".to_string(),
            group_id: None,
            policy_id: None,
            max_uses: None,
            current_uses: 0,
            expires_at: None,
            auto_assign_user_by_email: false,
            created_by: None,
            created_at: Utc::now(),
            revoked_at: Some(Utc::now()),
        };
        assert!(!token.is_valid());
        assert!(token.is_revoked());
    }

    #[test]
    fn test_enrollment_token_exhausted() {
        let token = EnrollmentToken {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            token: "enroll_test".to_string(),
            token_prefix: "enroll_t".to_string(),
            group_id: None,
            policy_id: None,
            max_uses: Some(10),
            current_uses: 10,
            expires_at: None,
            auto_assign_user_by_email: false,
            created_by: None,
            created_at: Utc::now(),
            revoked_at: None,
        };
        assert!(!token.is_valid());
        assert!(token.is_exhausted());
    }

    #[test]
    fn test_enrollment_token_remaining_uses() {
        let token = EnrollmentToken {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            token: "enroll_test".to_string(),
            token_prefix: "enroll_t".to_string(),
            group_id: None,
            policy_id: None,
            max_uses: Some(10),
            current_uses: 7,
            expires_at: None,
            auto_assign_user_by_email: false,
            created_by: None,
            created_at: Utc::now(),
            revoked_at: None,
        };
        assert_eq!(token.remaining_uses(), Some(3));
    }

    #[test]
    fn test_enrollment_token_unlimited_uses() {
        let token = EnrollmentToken {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            token: "enroll_test".to_string(),
            token_prefix: "enroll_t".to_string(),
            group_id: None,
            policy_id: None,
            max_uses: None,
            current_uses: 100,
            expires_at: None,
            auto_assign_user_by_email: false,
            created_by: None,
            created_at: Utc::now(),
            revoked_at: None,
        };
        assert!(token.is_valid());
        assert_eq!(token.remaining_uses(), None);
    }

    #[test]
    fn test_create_request_validation() {
        let request = CreateEnrollmentTokenRequest {
            group_id: Some("grp123".to_string()),
            policy_id: Some(Uuid::new_v4()),
            max_uses: Some(50),
            expires_in_days: Some(30),
            auto_assign_user_by_email: true,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_request_invalid_max_uses() {
        let request = CreateEnrollmentTokenRequest {
            group_id: None,
            policy_id: None,
            max_uses: Some(0),
            expires_in_days: None,
            auto_assign_user_by_email: false,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_request_invalid_expires_in_days() {
        let request = CreateEnrollmentTokenRequest {
            group_id: None,
            policy_id: None,
            max_uses: None,
            expires_in_days: Some(400),
            auto_assign_user_by_email: false,
        };
        assert!(request.validate().is_err());
    }
}
