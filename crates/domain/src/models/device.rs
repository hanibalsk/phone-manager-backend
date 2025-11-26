//! Device domain model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Represents a registered device in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: i64,
    pub device_id: Uuid,
    pub display_name: String,
    pub group_id: String,
    pub platform: String,
    pub fcm_token: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// Request payload for device registration.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDeviceRequest {
    pub device_id: Uuid,

    #[validate(length(
        min = 2,
        max = 50,
        message = "Display name must be between 2 and 50 characters"
    ))]
    pub display_name: String,

    #[validate(length(
        min = 2,
        max = 50,
        message = "Group ID must be between 2 and 50 characters"
    ))]
    #[validate(custom(function = "validate_group_id"))]
    pub group_id: String,

    #[serde(default = "default_platform")]
    pub platform: String,

    pub fcm_token: Option<String>,
}

/// Response payload for device registration.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDeviceResponse {
    pub device_id: Uuid,
    pub display_name: String,
    pub group_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Device summary for group listings.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSummary {
    pub device_id: Uuid,
    pub display_name: String,
    pub last_seen_at: Option<DateTime<Utc>>,
}

fn default_platform() -> String {
    "android".to_string()
}

fn validate_group_id(group_id: &str) -> Result<(), validator::ValidationError> {
    if group_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_group_id");
        err.message = Some(
            "Group ID may only contain alphanumeric characters, hyphens, and underscores".into(),
        );
        Err(err)
    }
}

impl From<Device> for RegisterDeviceResponse {
    fn from(device: Device) -> Self {
        Self {
            device_id: device.device_id,
            display_name: device.display_name,
            group_id: device.group_id,
            created_at: device.created_at,
            updated_at: device.updated_at,
        }
    }
}

impl From<Device> for DeviceSummary {
    fn from(device: Device) -> Self {
        Self {
            device_id: device.device_id,
            display_name: device.display_name,
            last_seen_at: device.last_seen_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    fn create_test_device() -> Device {
        Device {
            id: 1,
            device_id: Uuid::new_v4(),
            display_name: "Test Device".to_string(),
            group_id: "test-group".to_string(),
            platform: "android".to_string(),
            fcm_token: Some("token123".to_string()),
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_seen_at: Some(Utc::now()),
        }
    }

    #[test]
    fn test_device_struct() {
        let device = create_test_device();
        assert_eq!(device.display_name, "Test Device");
        assert_eq!(device.group_id, "test-group");
        assert!(device.active);
    }

    #[test]
    fn test_device_clone() {
        let device = create_test_device();
        let cloned = device.clone();
        assert_eq!(cloned.display_name, device.display_name);
        assert_eq!(cloned.device_id, device.device_id);
    }

    #[test]
    fn test_device_to_register_response() {
        let device = create_test_device();
        let response: RegisterDeviceResponse = device.clone().into();
        assert_eq!(response.device_id, device.device_id);
        assert_eq!(response.display_name, device.display_name);
        assert_eq!(response.group_id, device.group_id);
    }

    #[test]
    fn test_device_to_summary() {
        let device = create_test_device();
        let summary: DeviceSummary = device.clone().into();
        assert_eq!(summary.device_id, device.device_id);
        assert_eq!(summary.display_name, device.display_name);
        assert_eq!(summary.last_seen_at, device.last_seen_at);
    }

    #[test]
    fn test_register_device_request_valid() {
        let request = RegisterDeviceRequest {
            device_id: Uuid::new_v4(),
            display_name: "My Phone".to_string(),
            group_id: "family-group".to_string(),
            platform: "android".to_string(),
            fcm_token: None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_register_device_request_display_name_too_short() {
        let request = RegisterDeviceRequest {
            device_id: Uuid::new_v4(),
            display_name: "A".to_string(), // Too short (min 2)
            group_id: "family-group".to_string(),
            platform: "android".to_string(),
            fcm_token: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_register_device_request_display_name_too_long() {
        let request = RegisterDeviceRequest {
            device_id: Uuid::new_v4(),
            display_name: "A".repeat(51), // Too long (max 50)
            group_id: "family-group".to_string(),
            platform: "android".to_string(),
            fcm_token: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_register_device_request_group_id_too_short() {
        let request = RegisterDeviceRequest {
            device_id: Uuid::new_v4(),
            display_name: "My Phone".to_string(),
            group_id: "A".to_string(), // Too short (min 2)
            platform: "android".to_string(),
            fcm_token: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_register_device_request_group_id_invalid_chars() {
        let request = RegisterDeviceRequest {
            device_id: Uuid::new_v4(),
            display_name: "My Phone".to_string(),
            group_id: "invalid group!".to_string(), // Contains space and !
            platform: "android".to_string(),
            fcm_token: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_register_device_request_group_id_valid_chars() {
        let request = RegisterDeviceRequest {
            device_id: Uuid::new_v4(),
            display_name: "My Phone".to_string(),
            group_id: "valid-group_123".to_string(), // Alphanumeric, hyphens, underscores
            platform: "android".to_string(),
            fcm_token: None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_default_platform() {
        assert_eq!(default_platform(), "android");
    }

    #[test]
    fn test_device_summary_without_last_seen() {
        let mut device = create_test_device();
        device.last_seen_at = None;
        let summary: DeviceSummary = device.into();
        assert!(summary.last_seen_at.is_none());
    }

    #[test]
    fn test_register_device_response_fields() {
        let device = create_test_device();
        let response: RegisterDeviceResponse = device.clone().into();
        assert_eq!(response.created_at, device.created_at);
        assert_eq!(response.updated_at, device.updated_at);
    }
}
