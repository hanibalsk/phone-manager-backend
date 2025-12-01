//! Device policy entity for database operations.

use chrono::{DateTime, Utc};
use domain::models::device_policy::{DevicePolicy, DevicePolicyResponse};
use sqlx::FromRow;
use uuid::Uuid;

/// Database entity for device policies.
#[derive(Debug, Clone, FromRow)]
pub struct DevicePolicyEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub settings: serde_json::Value,
    pub locked_settings: Vec<String>,
    pub priority: i32,
    pub device_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DevicePolicyEntity> for DevicePolicy {
    fn from(entity: DevicePolicyEntity) -> Self {
        let settings = serde_json::from_value(entity.settings.clone()).unwrap_or_default();

        DevicePolicy {
            id: entity.id,
            organization_id: entity.organization_id,
            name: entity.name,
            description: entity.description,
            is_default: entity.is_default,
            settings,
            locked_settings: entity.locked_settings,
            priority: entity.priority,
            device_count: entity.device_count,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

impl From<DevicePolicyEntity> for DevicePolicyResponse {
    fn from(entity: DevicePolicyEntity) -> Self {
        let policy: DevicePolicy = entity.into();
        policy.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_policy_entity_to_domain() {
        let now = Utc::now();
        let entity = DevicePolicyEntity {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "Test Policy".to_string(),
            description: Some("A test policy".to_string()),
            is_default: false,
            settings: serde_json::json!({"tracking_enabled": true}),
            locked_settings: vec!["tracking_enabled".to_string()],
            priority: 10,
            device_count: 5,
            created_at: now,
            updated_at: now,
        };

        let policy: DevicePolicy = entity.clone().into();
        assert_eq!(policy.id, entity.id);
        assert_eq!(policy.name, entity.name);
        assert_eq!(policy.priority, 10);
        assert_eq!(policy.device_count, 5);
    }
}
