//! Device command entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the device_commands table.
#[derive(Debug, Clone, FromRow)]
pub struct DeviceCommandEntity {
    pub id: Uuid,
    pub device_id: i64,
    pub organization_id: Uuid,
    pub command_type: String,
    pub status: String,
    pub payload: Option<serde_json::Value>,
    pub issued_by: Uuid,
    pub issued_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_command_entity_debug() {
        let entity = DeviceCommandEntity {
            id: Uuid::new_v4(),
            device_id: 1,
            organization_id: Uuid::new_v4(),
            command_type: "wipe".to_string(),
            status: "pending".to_string(),
            payload: None,
            issued_by: Uuid::new_v4(),
            issued_at: Utc::now(),
            acknowledged_at: None,
            completed_at: None,
            failed_at: None,
            failure_reason: None,
            expires_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("DeviceCommandEntity"));
        assert!(debug_str.contains("wipe"));
    }

    #[test]
    fn test_device_command_entity_clone() {
        let entity = DeviceCommandEntity {
            id: Uuid::new_v4(),
            device_id: 1,
            organization_id: Uuid::new_v4(),
            command_type: "lock".to_string(),
            status: "pending".to_string(),
            payload: Some(serde_json::json!({"reason": "security"})),
            issued_by: Uuid::new_v4(),
            issued_at: Utc::now(),
            acknowledged_at: None,
            completed_at: None,
            failed_at: None,
            failure_reason: None,
            expires_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let cloned = entity.clone();
        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.command_type, entity.command_type);
    }
}
