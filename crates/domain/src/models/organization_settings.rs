//! Organization settings domain models.
//!
//! Per-organization admin settings for the admin portal.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Internal representation of organization settings.
#[derive(Debug, Clone)]
pub struct OrganizationSettings {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Whether a PIN is set (derived from unlock_pin_hash)
    pub has_unlock_pin: bool,
    /// The hashed PIN (internal only, never exposed in API responses)
    pub unlock_pin_hash: Option<String>,
    /// Default daily screen time limit in minutes (0 = unlimited)
    pub default_daily_limit_minutes: i32,
    /// Enable push notifications for this organization
    pub notifications_enabled: bool,
    /// Automatically approve device unlock requests
    pub auto_approve_unlock_requests: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// GET response for organization settings (PIN is never exposed).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OrganizationSettingsResponse {
    /// Whether a PIN is set (true/false, not the actual PIN)
    pub has_unlock_pin: bool,
    /// Default daily screen time limit in minutes (0 = unlimited)
    pub default_daily_limit_minutes: i32,
    /// Enable push notifications for this organization
    pub notifications_enabled: bool,
    /// Automatically approve device unlock requests
    pub auto_approve_unlock_requests: bool,
}

impl From<OrganizationSettings> for OrganizationSettingsResponse {
    fn from(settings: OrganizationSettings) -> Self {
        Self {
            has_unlock_pin: settings.has_unlock_pin,
            default_daily_limit_minutes: settings.default_daily_limit_minutes,
            notifications_enabled: settings.notifications_enabled,
            auto_approve_unlock_requests: settings.auto_approve_unlock_requests,
        }
    }
}

/// PUT request to update organization settings.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateOrganizationSettingsRequest {
    /// Unlock PIN (4-8 digits). Set to null/empty to clear the PIN.
    #[validate(length(min = 4, max = 8, message = "PIN must be 4-8 characters"))]
    #[validate(regex(path = "*PIN_REGEX", message = "PIN must contain only digits"))]
    pub unlock_pin: Option<String>,
    /// Clear the existing PIN (if true, ignores unlock_pin)
    #[serde(default)]
    pub clear_pin: bool,
    /// Default daily screen time limit in minutes (0-1440, 0 = unlimited)
    #[validate(range(min = 0, max = 1440, message = "Daily limit must be 0-1440 minutes"))]
    pub default_daily_limit_minutes: Option<i32>,
    /// Enable push notifications for this organization
    pub notifications_enabled: Option<bool>,
    /// Automatically approve device unlock requests
    pub auto_approve_unlock_requests: Option<bool>,
}

/// POST request to verify unlock PIN.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct VerifyPinRequest {
    /// The PIN to verify
    #[validate(length(min = 1, max = 8, message = "PIN must be provided"))]
    pub pin: String,
}

/// Response for PIN verification.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct VerifyPinResponse {
    /// Whether the PIN is valid
    pub valid: bool,
}

// Regex for PIN validation (digits only)
lazy_static::lazy_static! {
    pub static ref PIN_REGEX: regex::Regex = regex::Regex::new(r"^\d+$").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_response_serialization() {
        let response = OrganizationSettingsResponse {
            has_unlock_pin: true,
            default_daily_limit_minutes: 120,
            notifications_enabled: true,
            auto_approve_unlock_requests: false,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"has_unlock_pin\":true"));
        assert!(json.contains("\"default_daily_limit_minutes\":120"));
    }

    #[test]
    fn test_update_request_deserialization() {
        let json = r#"{"unlock_pin": "1234", "default_daily_limit_minutes": 90}"#;
        let request: UpdateOrganizationSettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.unlock_pin, Some("1234".to_string()));
        assert_eq!(request.default_daily_limit_minutes, Some(90));
    }

    #[test]
    fn test_update_request_validation() {
        let request = UpdateOrganizationSettingsRequest {
            unlock_pin: Some("12".to_string()), // Too short
            clear_pin: false,
            default_daily_limit_minutes: Some(120),
            notifications_enabled: None,
            auto_approve_unlock_requests: None,
        };
        assert!(request.validate().is_err());

        let valid_request = UpdateOrganizationSettingsRequest {
            unlock_pin: Some("1234".to_string()),
            clear_pin: false,
            default_daily_limit_minutes: Some(120),
            notifications_enabled: Some(true),
            auto_approve_unlock_requests: Some(false),
        };
        assert!(valid_request.validate().is_ok());
    }

    #[test]
    fn test_verify_pin_request_deserialization() {
        let json = r#"{"pin": "1234"}"#;
        let request: VerifyPinRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.pin, "1234");
    }

    #[test]
    fn test_pin_regex() {
        assert!(PIN_REGEX.is_match("1234"));
        assert!(PIN_REGEX.is_match("12345678"));
        assert!(!PIN_REGEX.is_match("12a4"));
        assert!(!PIN_REGEX.is_match(""));
    }
}
