//! Setting change domain models.

use serde::{Deserialize, Serialize};

/// Response for a single setting change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingChangeResponse {
    /// Unique identifier for this change record.
    pub id: String,
    /// Key of the setting that was changed.
    pub setting_key: String,
    /// Previous value before the change (null for new settings or locks).
    pub old_value: Option<serde_json::Value>,
    /// New value after the change (null for unlocks).
    pub new_value: Option<serde_json::Value>,
    /// User ID who made the change.
    pub changed_by: String,
    /// Display name of the user who made the change.
    pub changed_by_name: String,
    /// ISO 8601 timestamp when the change occurred.
    pub changed_at: String,
    /// Type of change: VALUE_CHANGED, LOCKED, UNLOCKED, RESET.
    pub change_type: String,
}

/// Response for settings history endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsHistoryResponse {
    /// List of setting changes.
    pub changes: Vec<SettingChangeResponse>,
    /// Total number of changes for this device.
    pub total_count: i64,
    /// Whether there are more results available.
    pub has_more: bool,
}

/// Domain enum for setting change types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SettingChangeType {
    ValueChanged,
    Locked,
    Unlocked,
    Reset,
}

impl std::fmt::Display for SettingChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingChangeType::ValueChanged => write!(f, "VALUE_CHANGED"),
            SettingChangeType::Locked => write!(f, "LOCKED"),
            SettingChangeType::Unlocked => write!(f, "UNLOCKED"),
            SettingChangeType::Reset => write!(f, "RESET"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_change_type_serialization() {
        assert_eq!(
            serde_json::to_string(&SettingChangeType::ValueChanged).unwrap(),
            "\"VALUE_CHANGED\""
        );
        assert_eq!(
            serde_json::to_string(&SettingChangeType::Locked).unwrap(),
            "\"LOCKED\""
        );
        assert_eq!(
            serde_json::to_string(&SettingChangeType::Unlocked).unwrap(),
            "\"UNLOCKED\""
        );
        assert_eq!(
            serde_json::to_string(&SettingChangeType::Reset).unwrap(),
            "\"RESET\""
        );
    }

    #[test]
    fn test_setting_change_type_display() {
        assert_eq!(SettingChangeType::ValueChanged.to_string(), "VALUE_CHANGED");
        assert_eq!(SettingChangeType::Locked.to_string(), "LOCKED");
        assert_eq!(SettingChangeType::Unlocked.to_string(), "UNLOCKED");
        assert_eq!(SettingChangeType::Reset.to_string(), "RESET");
    }
}
