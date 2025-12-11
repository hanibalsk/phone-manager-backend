//! Device token domain model for managed device authentication.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Device token prefix.
pub const DEVICE_TOKEN_PREFIX: &str = "dt_";

/// Length of random bytes for token generation.
const TOKEN_RANDOM_BYTES: usize = 45;

/// Default token expiry in days.
pub const DEFAULT_TOKEN_EXPIRY_DAYS: i64 = 90;

/// Device token domain model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceToken {
    pub id: Uuid,
    pub device_id: i64,
    pub organization_id: Uuid,
    pub token: String,
    pub token_prefix: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
}

impl DeviceToken {
    /// Check if the token is valid for use.
    pub fn is_valid(&self) -> bool {
        // Check if revoked
        if self.revoked_at.is_some() {
            return false;
        }

        // Check expiration
        if Utc::now() > self.expires_at {
            return false;
        }

        true
    }

    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the token is revoked.
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }
}

/// Generate a new device token.
pub fn generate_device_token() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..TOKEN_RANDOM_BYTES).map(|_| rng.gen()).collect();
    let encoded = URL_SAFE_NO_PAD.encode(&random_bytes);
    format!("{}{}", DEVICE_TOKEN_PREFIX, encoded)
}

/// Extract token prefix from a full token.
pub fn extract_device_token_prefix(token: &str) -> String {
    token.chars().take(8).collect()
}

/// Calculate expiry date from days.
pub fn calculate_device_token_expiry(days: i64) -> DateTime<Utc> {
    Utc::now() + Duration::days(days)
}

/// Enrollment status for managed devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentStatus {
    Pending,
    Enrolled,
    Suspended,
    Retired,
}

impl EnrollmentStatus {
    /// Convert to database string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Enrolled => "enrolled",
            Self::Suspended => "suspended",
            Self::Retired => "retired",
        }
    }

    /// Parse from database string representation.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "enrolled" => Some(Self::Enrolled),
            "suspended" => Some(Self::Suspended),
            "retired" => Some(Self::Retired),
            _ => None,
        }
    }
}

impl std::fmt::Display for EnrollmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for EnrollmentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "enrolled" => Ok(Self::Enrolled),
            "suspended" => Ok(Self::Suspended),
            "retired" => Ok(Self::Retired),
            _ => Err(format!("Unknown enrollment status: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_device_token() {
        let token = generate_device_token();
        assert!(token.starts_with(DEVICE_TOKEN_PREFIX));
        assert!(token.len() > 20);
    }

    #[test]
    fn test_generate_device_token_uniqueness() {
        let token1 = generate_device_token();
        let token2 = generate_device_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_extract_device_token_prefix() {
        let token = "dt_abc123xyz";
        let prefix = extract_device_token_prefix(token);
        assert_eq!(prefix, "dt_abc12");
    }

    #[test]
    fn test_calculate_device_token_expiry() {
        let expiry = calculate_device_token_expiry(90);
        let now = Utc::now();
        assert!(expiry > now);
        assert!(expiry < now + Duration::days(91));
    }

    #[test]
    fn test_device_token_is_valid() {
        let token = DeviceToken {
            id: Uuid::new_v4(),
            device_id: 1,
            organization_id: Uuid::new_v4(),
            token: "dt_test".to_string(),
            token_prefix: "dt_test".to_string(),
            expires_at: Utc::now() + Duration::days(30),
            created_at: Utc::now(),
            last_used_at: None,
            revoked_at: None,
        };
        assert!(token.is_valid());
    }

    #[test]
    fn test_device_token_expired() {
        let token = DeviceToken {
            id: Uuid::new_v4(),
            device_id: 1,
            organization_id: Uuid::new_v4(),
            token: "dt_test".to_string(),
            token_prefix: "dt_test".to_string(),
            expires_at: Utc::now() - Duration::days(1),
            created_at: Utc::now(),
            last_used_at: None,
            revoked_at: None,
        };
        assert!(!token.is_valid());
        assert!(token.is_expired());
    }

    #[test]
    fn test_device_token_revoked() {
        let token = DeviceToken {
            id: Uuid::new_v4(),
            device_id: 1,
            organization_id: Uuid::new_v4(),
            token: "dt_test".to_string(),
            token_prefix: "dt_test".to_string(),
            expires_at: Utc::now() + Duration::days(30),
            created_at: Utc::now(),
            last_used_at: None,
            revoked_at: Some(Utc::now()),
        };
        assert!(!token.is_valid());
        assert!(token.is_revoked());
    }

    #[test]
    fn test_enrollment_status_as_str() {
        assert_eq!(EnrollmentStatus::Pending.as_str(), "pending");
        assert_eq!(EnrollmentStatus::Enrolled.as_str(), "enrolled");
        assert_eq!(EnrollmentStatus::Suspended.as_str(), "suspended");
        assert_eq!(EnrollmentStatus::Retired.as_str(), "retired");
    }

    #[test]
    fn test_enrollment_status_from_str() {
        assert_eq!(
            EnrollmentStatus::parse("pending"),
            Some(EnrollmentStatus::Pending)
        );
        assert_eq!(
            EnrollmentStatus::parse("enrolled"),
            Some(EnrollmentStatus::Enrolled)
        );
        assert_eq!(
            EnrollmentStatus::parse("suspended"),
            Some(EnrollmentStatus::Suspended)
        );
        assert_eq!(
            EnrollmentStatus::parse("retired"),
            Some(EnrollmentStatus::Retired)
        );
        assert_eq!(EnrollmentStatus::parse("invalid"), None);
    }

    #[test]
    fn test_enrollment_status_display() {
        assert_eq!(format!("{}", EnrollmentStatus::Pending), "pending");
        assert_eq!(format!("{}", EnrollmentStatus::Enrolled), "enrolled");
    }
}
