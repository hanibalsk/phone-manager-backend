//! Device policy domain model for organization-wide device configuration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// Device policy domain model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePolicy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub settings: HashMap<String, serde_json::Value>,
    pub locked_settings: Vec<String>,
    pub priority: i32,
    pub device_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response format for device policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePolicyResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_default: bool,
    pub settings: HashMap<String, serde_json::Value>,
    pub locked_settings: Vec<String>,
    pub priority: i32,
    pub device_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DevicePolicy> for DevicePolicyResponse {
    fn from(policy: DevicePolicy) -> Self {
        Self {
            id: policy.id,
            organization_id: policy.organization_id,
            name: policy.name,
            description: policy.description,
            is_default: policy.is_default,
            settings: policy.settings,
            locked_settings: policy.locked_settings,
            priority: policy.priority,
            device_count: policy.device_count,
            created_at: policy.created_at,
            updated_at: policy.updated_at,
        }
    }
}

/// Request to create a new device policy.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateDevicePolicyRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    pub name: String,
    #[validate(length(max = 2000, message = "Description must be at most 2000 characters"))]
    pub description: Option<String>,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
    #[serde(default)]
    #[validate(custom(function = "validate_locked_settings"))]
    pub locked_settings: Vec<String>,
    #[validate(range(min = -1000, max = 1000, message = "Priority must be between -1000 and 1000"))]
    #[serde(default)]
    pub priority: i32,
}

/// Request to update a device policy.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDevicePolicyRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    pub name: Option<String>,
    #[validate(length(max = 2000, message = "Description must be at most 2000 characters"))]
    pub description: Option<String>,
    pub is_default: Option<bool>,
    pub settings: Option<HashMap<String, serde_json::Value>>,
    #[validate(custom(function = "validate_locked_settings_option"))]
    pub locked_settings: Option<Vec<String>>,
    #[validate(range(min = -1000, max = 1000, message = "Priority must be between -1000 and 1000"))]
    pub priority: Option<i32>,
}

/// Query parameters for listing policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDevicePoliciesQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub is_default: Option<bool>,
}

/// Response for listing device policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDevicePoliciesResponse {
    pub data: Vec<DevicePolicyResponse>,
    pub pagination: DevicePolicyPagination,
}

/// Pagination metadata for device policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePolicyPagination {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
}

/// Target for policy application.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyTarget {
    #[serde(rename = "type")]
    pub target_type: PolicyTargetType,
    pub id: Uuid,
}

/// Type of policy target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyTargetType {
    Device,
    Group,
}

/// Request to apply a policy to targets.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ApplyPolicyRequest {
    #[validate(length(min = 1, max = 100, message = "Must specify 1-100 targets"))]
    pub targets: Vec<PolicyTarget>,
    #[serde(default)]
    pub replace_existing: bool,
}

/// Response for policy application.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyPolicyResponse {
    pub policy_id: Uuid,
    pub applied_to: AppliedToCount,
    pub total_devices_affected: i64,
}

/// Count of applied targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedToCount {
    pub devices: i64,
    pub groups: i64,
}

/// Request to unapply a policy from targets.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UnapplyPolicyRequest {
    #[validate(length(min = 1, max = 100, message = "Must specify 1-100 targets"))]
    pub targets: Vec<PolicyTarget>,
}

/// Response for policy unapplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnapplyPolicyResponse {
    pub policy_id: Uuid,
    pub unapplied_from: AppliedToCount,
    pub total_devices_affected: i64,
}

/// Validate locked settings array.
fn validate_locked_settings(settings: &[String]) -> Result<(), validator::ValidationError> {
    if settings.len() > 100 {
        return Err(validator::ValidationError::new("too_many_locked_settings")
            .with_message(std::borrow::Cow::Borrowed(
                "Cannot lock more than 100 settings",
            )));
    }
    for setting in settings {
        if setting.len() > 100 {
            return Err(validator::ValidationError::new("setting_key_too_long")
                .with_message(std::borrow::Cow::Borrowed(
                    "Setting key must be at most 100 characters",
                )));
        }
    }
    Ok(())
}

/// Validate optional locked settings array.
fn validate_locked_settings_option(
    settings: &[String],
) -> Result<(), validator::ValidationError> {
    validate_locked_settings(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_policy_request_validation() {
        let request = CreateDevicePolicyRequest {
            name: "Test Policy".to_string(),
            description: Some("A test policy".to_string()),
            is_default: false,
            settings: HashMap::new(),
            locked_settings: vec!["tracking_enabled".to_string()],
            priority: 10,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_policy_request_empty_name() {
        let request = CreateDevicePolicyRequest {
            name: "".to_string(),
            description: None,
            is_default: false,
            settings: HashMap::new(),
            locked_settings: vec![],
            priority: 0,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_policy_request_priority_too_high() {
        let request = CreateDevicePolicyRequest {
            name: "Test".to_string(),
            description: None,
            is_default: false,
            settings: HashMap::new(),
            locked_settings: vec![],
            priority: 2000,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_policy_response_serialization() {
        let response = DevicePolicyResponse {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "Test".to_string(),
            description: None,
            is_default: false,
            settings: HashMap::new(),
            locked_settings: vec![],
            priority: 0,
            device_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("description")); // skip_serializing_if = None
    }

    #[test]
    fn test_policy_target_type_serialization() {
        let device = PolicyTargetType::Device;
        let group = PolicyTargetType::Group;
        assert_eq!(
            serde_json::to_string(&device).unwrap(),
            "\"device\""
        );
        assert_eq!(
            serde_json::to_string(&group).unwrap(),
            "\"group\""
        );
    }

    #[test]
    fn test_apply_policy_request_deserialize() {
        let json = r#"{
            "targets": [
                {"type": "device", "id": "550e8400-e29b-41d4-a716-446655440000"},
                {"type": "group", "id": "550e8400-e29b-41d4-a716-446655440001"}
            ],
            "replaceExisting": true
        }"#;
        let request: ApplyPolicyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.targets.len(), 2);
        assert!(request.replace_existing);
    }

    #[test]
    fn test_validate_locked_settings_too_many() {
        let settings: Vec<String> = (0..101).map(|i| format!("setting_{}", i)).collect();
        assert!(validate_locked_settings(&settings).is_err());
    }

    #[test]
    fn test_validate_locked_settings_key_too_long() {
        let settings = vec!["a".repeat(101)];
        assert!(validate_locked_settings(&settings).is_err());
    }

    #[test]
    fn test_validate_locked_settings_valid() {
        let settings = vec!["tracking_enabled".to_string(), "secret_mode".to_string()];
        assert!(validate_locked_settings(&settings).is_ok());
    }
}
