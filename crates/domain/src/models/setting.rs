//! Setting domain models for device configuration management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Data type for setting values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingDataType {
    Boolean,
    Integer,
    String,
    Float,
    Json,
}

impl std::fmt::Display for SettingDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingDataType::Boolean => write!(f, "boolean"),
            SettingDataType::Integer => write!(f, "integer"),
            SettingDataType::String => write!(f, "string"),
            SettingDataType::Float => write!(f, "float"),
            SettingDataType::Json => write!(f, "json"),
        }
    }
}

/// Category for grouping settings in UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingCategory {
    Tracking,
    Privacy,
    Notifications,
    Battery,
    General,
}

impl std::fmt::Display for SettingCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingCategory::Tracking => write!(f, "tracking"),
            SettingCategory::Privacy => write!(f, "privacy"),
            SettingCategory::Notifications => write!(f, "notifications"),
            SettingCategory::Battery => write!(f, "battery"),
            SettingCategory::General => write!(f, "general"),
        }
    }
}

/// Metadata about a setting (from setting_definitions table).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingDefinition {
    pub key: String,
    pub display_name: String,
    pub description: Option<String>,
    pub data_type: SettingDataType,
    pub default_value: serde_json::Value,
    pub is_lockable: bool,
    pub category: SettingCategory,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_rules: Option<serde_json::Value>,
    pub sort_order: i32,
}

/// A device setting value with lock state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSetting {
    pub id: Uuid,
    pub device_id: Uuid,
    pub setting_key: String,
    pub value: serde_json::Value,
    pub is_locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_by: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

/// Setting value response with lock info (for API responses).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingValue {
    pub value: serde_json::Value,
    pub is_locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_by: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_reason: Option<String>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response for getting all device settings.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSettingsResponse {
    pub device_id: Uuid,
    pub settings: std::collections::HashMap<String, SettingValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definitions: Option<Vec<SettingDefinition>>,
}

/// Request to update multiple settings.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsRequest {
    pub settings: std::collections::HashMap<String, serde_json::Value>,
}

/// Response after updating settings.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsResponse {
    pub updated: Vec<String>,
    pub locked: Vec<String>,
    pub invalid: Vec<String>,
    pub settings: std::collections::HashMap<String, SettingValue>,
}

/// Request to update a single setting.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingRequest {
    pub value: serde_json::Value,
}

/// Request to lock a setting.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockSettingRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(default)]
    pub notify_user: bool,
}

/// Response after locking a setting.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockSettingResponse {
    pub key: String,
    pub is_locked: bool,
    pub value: serde_json::Value,
    pub locked_by: Uuid,
    pub locked_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Response after unlocking a setting.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockSettingResponse {
    pub key: String,
    pub is_locked: bool,
    pub unlocked_by: Uuid,
    pub unlocked_at: DateTime<Utc>,
}

/// Lock info for listing locks.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockInfo {
    pub key: String,
    pub is_locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_by: Option<LockerInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// User info for lock display.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockerInfo {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Response for listing locks.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListLocksResponse {
    pub device_id: Uuid,
    pub locks: Vec<LockInfo>,
    pub locked_count: i64,
    pub total_lockable: i64,
}

/// Request to bulk update locks.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkUpdateLocksRequest {
    pub locks: std::collections::HashMap<String, bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default)]
    pub notify_user: bool,
}

/// Individual lock update result.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockUpdateResult {
    pub key: String,
    pub is_locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlocked_at: Option<DateTime<Utc>>,
}

/// Skipped lock update info.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedLockUpdate {
    pub key: String,
    pub reason: String,
}

/// Response for bulk lock update.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkUpdateLocksResponse {
    pub updated: Vec<LockUpdateResult>,
    pub skipped: Vec<SkippedLockUpdate>,
    pub notification_sent: bool,
}

/// Request for settings sync.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSettingsRequest {
    /// Last sync timestamp from device (optional, if not provided all settings are returned)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<DateTime<Utc>>,
}

/// Response for settings sync.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSettingsResponse {
    pub synced_at: DateTime<Utc>,
    pub settings: std::collections::HashMap<String, SettingValue>,
    pub changes_applied: Vec<SettingChange>,
}

/// A setting change entry for sync response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingChange {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_data_type_display() {
        assert_eq!(SettingDataType::Boolean.to_string(), "boolean");
        assert_eq!(SettingDataType::Integer.to_string(), "integer");
        assert_eq!(SettingDataType::String.to_string(), "string");
    }

    #[test]
    fn test_setting_category_display() {
        assert_eq!(SettingCategory::Tracking.to_string(), "tracking");
        assert_eq!(SettingCategory::Privacy.to_string(), "privacy");
    }

    #[test]
    fn test_update_settings_request_deserialize() {
        let json = r#"{"settings":{"tracking_enabled":true,"tracking_interval_minutes":10}}"#;
        let req: UpdateSettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.settings.len(), 2);
        assert_eq!(req.settings.get("tracking_enabled"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn test_lock_setting_request_deserialize() {
        let json = r#"{"reason":"Policy","value":5,"notifyUser":true}"#;
        let req: LockSettingRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.reason, Some("Policy".to_string()));
        assert_eq!(req.value, Some(serde_json::json!(5)));
        assert!(req.notify_user);
    }

    #[test]
    fn test_bulk_update_locks_request_deserialize() {
        let json = r#"{"locks":{"tracking_enabled":true,"secret_mode_enabled":false},"reason":"Security"}"#;
        let req: BulkUpdateLocksRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.locks.len(), 2);
        assert_eq!(req.locks.get("tracking_enabled"), Some(&true));
        assert_eq!(req.reason, Some("Security".to_string()));
    }
}
