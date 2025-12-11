//! Setting entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database enum for setting_data_type that maps to PostgreSQL enum type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "setting_data_type", rename_all = "lowercase")]
pub enum SettingDataTypeDb {
    Boolean,
    Integer,
    String,
    Float,
    Json,
}

/// Database enum for setting_category that maps to PostgreSQL enum type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "setting_category", rename_all = "lowercase")]
pub enum SettingCategoryDb {
    Tracking,
    Privacy,
    Notifications,
    Battery,
    General,
}

/// Database row mapping for the setting_definitions table.
#[derive(Debug, Clone, FromRow)]
pub struct SettingDefinitionEntity {
    pub key: String,
    pub display_name: String,
    pub description: Option<String>,
    pub data_type: SettingDataTypeDb,
    pub default_value: serde_json::Value,
    pub is_lockable: bool,
    pub category: SettingCategoryDb,
    pub validation_rules: Option<serde_json::Value>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database row mapping for the device_settings table.
#[derive(Debug, Clone, FromRow)]
pub struct DeviceSettingEntity {
    pub id: Uuid,
    pub device_id: Uuid,
    pub setting_key: String,
    pub value: serde_json::Value,
    pub is_locked: bool,
    pub locked_by: Option<Uuid>,
    pub locked_at: Option<DateTime<Utc>>,
    pub lock_reason: Option<String>,
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Extended device setting entity with definition info for full response.
#[derive(Debug, Clone, FromRow)]
pub struct DeviceSettingWithDefinitionEntity {
    // Device setting fields
    pub id: Uuid,
    pub device_id: Uuid,
    pub setting_key: String,
    pub value: serde_json::Value,
    pub is_locked: bool,
    pub locked_by: Option<Uuid>,
    pub locked_at: Option<DateTime<Utc>>,
    pub lock_reason: Option<String>,
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    // Definition fields
    pub display_name: String,
    pub description: Option<String>,
    pub data_type: SettingDataTypeDb,
    pub default_value: serde_json::Value,
    pub is_lockable: bool,
    pub category: SettingCategoryDb,
}

/// Lock info entity for listing locked settings.
#[derive(Debug, Clone, FromRow)]
pub struct SettingLockEntity {
    pub setting_key: String,
    pub is_locked: bool,
    pub locked_by: Option<Uuid>,
    pub locked_at: Option<DateTime<Utc>>,
    pub lock_reason: Option<String>,
    // User info for locker
    pub locker_display_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_data_type_variants() {
        assert_eq!(format!("{:?}", SettingDataTypeDb::Boolean), "Boolean");
        assert_eq!(format!("{:?}", SettingDataTypeDb::Integer), "Integer");
    }

    #[test]
    fn test_setting_category_variants() {
        assert_eq!(format!("{:?}", SettingCategoryDb::Tracking), "Tracking");
        assert_eq!(format!("{:?}", SettingCategoryDb::Privacy), "Privacy");
    }
}
