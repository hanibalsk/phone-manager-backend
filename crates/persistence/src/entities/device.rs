//! Device entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the devices table.
#[derive(Debug, Clone, FromRow)]
pub struct DeviceEntity {
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

impl From<DeviceEntity> for domain::models::Device {
    fn from(entity: DeviceEntity) -> Self {
        Self {
            id: entity.id,
            device_id: entity.device_id,
            display_name: entity.display_name,
            group_id: entity.group_id,
            platform: entity.platform,
            fcm_token: entity.fcm_token,
            active: entity.active,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            last_seen_at: entity.last_seen_at,
        }
    }
}

/// Database row mapping for the devices_with_last_location view.
#[derive(Debug, Clone, FromRow)]
pub struct DeviceWithLastLocationEntity {
    pub id: i64,
    pub device_id: Uuid,
    pub display_name: String,
    pub group_id: String,
    pub platform: String,
    pub fcm_token: Option<String>,
    pub active: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_latitude: Option<f64>,
    pub last_longitude: Option<f64>,
    pub last_location_time: Option<DateTime<Utc>>,
    pub last_accuracy: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_device_entity() -> DeviceEntity {
        DeviceEntity {
            id: 1,
            device_id: Uuid::new_v4(),
            display_name: "Test Device".to_string(),
            group_id: "test-group".to_string(),
            platform: "android".to_string(),
            fcm_token: Some("fcm-token-123".to_string()),
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_seen_at: Some(Utc::now()),
        }
    }

    #[test]
    fn test_device_entity_to_domain() {
        let entity = create_test_device_entity();
        let device: domain::models::Device = entity.clone().into();

        assert_eq!(device.id, entity.id);
        assert_eq!(device.device_id, entity.device_id);
        assert_eq!(device.display_name, entity.display_name);
        assert_eq!(device.group_id, entity.group_id);
        assert_eq!(device.platform, entity.platform);
        assert_eq!(device.fcm_token, entity.fcm_token);
        assert_eq!(device.active, entity.active);
    }

    #[test]
    fn test_device_entity_clone() {
        let entity = create_test_device_entity();
        let cloned = entity.clone();
        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.device_id, entity.device_id);
    }

    #[test]
    fn test_device_entity_debug() {
        let entity = create_test_device_entity();
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("DeviceEntity"));
        assert!(debug_str.contains("Test Device"));
    }

    #[test]
    fn test_device_entity_optional_fields() {
        let mut entity = create_test_device_entity();
        entity.fcm_token = None;
        entity.last_seen_at = None;

        let device: domain::models::Device = entity.into();
        assert!(device.fcm_token.is_none());
        assert!(device.last_seen_at.is_none());
    }

    #[test]
    fn test_device_with_last_location_entity() {
        let entity = DeviceWithLastLocationEntity {
            id: 1,
            device_id: Uuid::new_v4(),
            display_name: "Test Device".to_string(),
            group_id: "test-group".to_string(),
            platform: "android".to_string(),
            fcm_token: None,
            active: true,
            last_seen_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_latitude: Some(37.7749),
            last_longitude: Some(-122.4194),
            last_location_time: Some(Utc::now()),
            last_accuracy: Some(10.0),
        };

        assert_eq!(entity.last_latitude, Some(37.7749));
        assert_eq!(entity.last_longitude, Some(-122.4194));
        assert_eq!(entity.last_accuracy, Some(10.0));
    }

    #[test]
    fn test_device_with_last_location_no_location() {
        let entity = DeviceWithLastLocationEntity {
            id: 1,
            device_id: Uuid::new_v4(),
            display_name: "Test Device".to_string(),
            group_id: "test-group".to_string(),
            platform: "android".to_string(),
            fcm_token: None,
            active: true,
            last_seen_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_latitude: None,
            last_longitude: None,
            last_location_time: None,
            last_accuracy: None,
        };

        assert!(entity.last_latitude.is_none());
        assert!(entity.last_longitude.is_none());
        assert!(entity.last_location_time.is_none());
    }
}
