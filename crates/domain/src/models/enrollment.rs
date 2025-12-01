//! Device enrollment domain models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

use super::device_token::EnrollmentStatus;

/// Request to enroll a device with an organization.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct EnrollDeviceRequest {
    /// The enrollment token for authentication
    #[validate(length(min = 10, message = "Invalid enrollment token"))]
    pub enrollment_token: String,

    /// The device's unique identifier
    pub device_uuid: Uuid,

    /// Human-readable device name
    #[validate(length(
        min = 2,
        max = 100,
        message = "Display name must be between 2 and 100 characters"
    ))]
    pub display_name: String,

    /// Optional device information
    #[serde(default)]
    pub device_info: Option<DeviceInfo>,

    /// Optional FCM token for push notifications
    pub fcm_token: Option<String>,

    /// Optional platform identifier
    #[serde(default = "default_platform")]
    pub platform: String,
}

fn default_platform() -> String {
    "android".to_string()
}

/// Device information provided during enrollment.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct DeviceInfo {
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub os_version: Option<String>,
}

/// Enrolled device information in the response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EnrolledDevice {
    pub id: i64,
    pub device_uuid: Uuid,
    pub display_name: String,
    pub organization_id: Uuid,
    pub is_managed: bool,
    pub enrollment_status: EnrollmentStatus,
    pub enrolled_at: DateTime<Utc>,
}

/// Policy information in enrollment response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EnrollmentPolicyInfo {
    pub id: Uuid,
    pub name: String,
    pub settings: HashMap<String, serde_json::Value>,
    pub locked_settings: Vec<String>,
}

/// Group information in enrollment response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EnrollmentGroupInfo {
    pub id: String,
    pub name: Option<String>,
}

/// Response for successful device enrollment.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EnrollDeviceResponse {
    /// The enrolled device information
    pub device: EnrolledDevice,

    /// The device token for future authenticated requests
    pub device_token: String,

    /// When the device token expires
    pub device_token_expires_at: DateTime<Utc>,

    /// Applied policy information (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<EnrollmentPolicyInfo>,

    /// Assigned group information (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<EnrollmentGroupInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enroll_device_request_validation() {
        let request = EnrollDeviceRequest {
            enrollment_token: "enroll_abc123xyz".to_string(),
            device_uuid: Uuid::new_v4(),
            display_name: "Test Device".to_string(),
            device_info: Some(DeviceInfo {
                manufacturer: Some("Samsung".to_string()),
                model: Some("Galaxy Tab".to_string()),
                os_version: Some("Android 14".to_string()),
            }),
            fcm_token: None,
            platform: "android".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_enroll_device_request_invalid_token() {
        let request = EnrollDeviceRequest {
            enrollment_token: "short".to_string(),
            device_uuid: Uuid::new_v4(),
            display_name: "Test Device".to_string(),
            device_info: None,
            fcm_token: None,
            platform: "android".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_enroll_device_request_invalid_display_name() {
        let request = EnrollDeviceRequest {
            enrollment_token: "enroll_abc123xyz".to_string(),
            device_uuid: Uuid::new_v4(),
            display_name: "A".to_string(), // Too short
            device_info: None,
            fcm_token: None,
            platform: "android".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_device_info_default() {
        let info = DeviceInfo::default();
        assert!(info.manufacturer.is_none());
        assert!(info.model.is_none());
        assert!(info.os_version.is_none());
    }

    #[test]
    fn test_enrolled_device_serialization() {
        let device = EnrolledDevice {
            id: 1,
            device_uuid: Uuid::new_v4(),
            display_name: "Test Device".to_string(),
            organization_id: Uuid::new_v4(),
            is_managed: true,
            enrollment_status: EnrollmentStatus::Enrolled,
            enrolled_at: Utc::now(),
        };
        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("\"is_managed\":true"));
        assert!(json.contains("\"enrollment_status\":\"enrolled\""));
    }

    #[test]
    fn test_enroll_device_response_serialization() {
        let response = EnrollDeviceResponse {
            device: EnrolledDevice {
                id: 1,
                device_uuid: Uuid::new_v4(),
                display_name: "Test Device".to_string(),
                organization_id: Uuid::new_v4(),
                is_managed: true,
                enrollment_status: EnrollmentStatus::Enrolled,
                enrolled_at: Utc::now(),
            },
            device_token: "dt_testtoken123".to_string(),
            device_token_expires_at: Utc::now(),
            policy: None,
            group: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"device_token\":\"dt_testtoken123\""));
        // policy and group should be omitted when None
        assert!(!json.contains("\"policy\":null"));
        assert!(!json.contains("\"group\":null"));
    }
}
