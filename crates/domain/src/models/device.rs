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
