//! Setting change entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database enum for setting_change_type that maps to PostgreSQL enum type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "setting_change_type", rename_all = "snake_case")]
pub enum SettingChangeTypeDb {
    ValueChanged,
    Locked,
    Unlocked,
    Reset,
}

impl std::fmt::Display for SettingChangeTypeDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValueChanged => write!(f, "VALUE_CHANGED"),
            Self::Locked => write!(f, "LOCKED"),
            Self::Unlocked => write!(f, "UNLOCKED"),
            Self::Reset => write!(f, "RESET"),
        }
    }
}

/// Database row mapping for the setting_changes table.
#[derive(Debug, Clone, FromRow)]
pub struct SettingChangeEntity {
    pub id: Uuid,
    pub device_id: Uuid,
    pub setting_key: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    /// User who made the change (null if user was deleted).
    pub changed_by: Option<Uuid>,
    pub changed_at: DateTime<Utc>,
    pub change_type: SettingChangeTypeDb,
}

/// Extended setting change entity with user info from JOIN.
#[derive(Debug, Clone, FromRow)]
pub struct SettingChangeWithUserEntity {
    pub id: Uuid,
    pub device_id: Uuid,
    pub setting_key: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    /// User who made the change (null if user was deleted).
    pub changed_by: Option<Uuid>,
    pub changed_at: DateTime<Utc>,
    pub change_type: SettingChangeTypeDb,
    // User info from JOIN (null if user was deleted)
    pub changed_by_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_change_type_display() {
        assert_eq!(format!("{}", SettingChangeTypeDb::ValueChanged), "VALUE_CHANGED");
        assert_eq!(format!("{}", SettingChangeTypeDb::Locked), "LOCKED");
        assert_eq!(format!("{}", SettingChangeTypeDb::Unlocked), "UNLOCKED");
        assert_eq!(format!("{}", SettingChangeTypeDb::Reset), "RESET");
    }

    #[test]
    fn test_setting_change_type_debug() {
        assert_eq!(format!("{:?}", SettingChangeTypeDb::ValueChanged), "ValueChanged");
        assert_eq!(format!("{:?}", SettingChangeTypeDb::Locked), "Locked");
    }
}
