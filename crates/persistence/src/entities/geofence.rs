//! Geofence entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::geofence::{Geofence, GeofenceEventType};

/// Database row mapping for the geofences table.
#[derive(Debug, Clone, FromRow)]
pub struct GeofenceEntity {
    pub id: i64,
    pub geofence_id: Uuid,
    pub device_id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: f32,
    pub event_types: Vec<String>, // SQLx maps TEXT[] to Vec<String>
    pub active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<GeofenceEntity> for Geofence {
    fn from(entity: GeofenceEntity) -> Self {
        Self {
            id: entity.id,
            geofence_id: entity.geofence_id,
            device_id: entity.device_id,
            name: entity.name,
            latitude: entity.latitude,
            longitude: entity.longitude,
            radius_meters: entity.radius_meters,
            event_types: entity
                .event_types
                .iter()
                .filter_map(|s| GeofenceEventType::parse(s))
                .collect(),
            active: entity.active,
            metadata: entity.metadata,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_geofence_entity() -> GeofenceEntity {
        GeofenceEntity {
            id: 1,
            geofence_id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            name: "Home".to_string(),
            latitude: 37.7749,
            longitude: -122.4194,
            radius_meters: 100.0,
            event_types: vec!["enter".to_string(), "exit".to_string()],
            active: true,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_geofence_entity_to_domain() {
        let entity = create_test_geofence_entity();
        let geofence: Geofence = entity.clone().into();

        assert_eq!(geofence.id, entity.id);
        assert_eq!(geofence.geofence_id, entity.geofence_id);
        assert_eq!(geofence.device_id, entity.device_id);
        assert_eq!(geofence.name, entity.name);
        assert_eq!(geofence.latitude, entity.latitude);
        assert_eq!(geofence.longitude, entity.longitude);
        assert_eq!(geofence.radius_meters, entity.radius_meters);
        assert_eq!(geofence.event_types.len(), 2);
        assert!(geofence.event_types.contains(&GeofenceEventType::Enter));
        assert!(geofence.event_types.contains(&GeofenceEventType::Exit));
        assert_eq!(geofence.active, entity.active);
    }

    #[test]
    fn test_geofence_entity_with_all_event_types() {
        let mut entity = create_test_geofence_entity();
        entity.event_types = vec![
            "enter".to_string(),
            "exit".to_string(),
            "dwell".to_string(),
        ];

        let geofence: Geofence = entity.into();
        assert_eq!(geofence.event_types.len(), 3);
        assert!(geofence.event_types.contains(&GeofenceEventType::Dwell));
    }

    #[test]
    fn test_geofence_entity_filters_invalid_event_types() {
        let mut entity = create_test_geofence_entity();
        entity.event_types = vec![
            "enter".to_string(),
            "invalid".to_string(),
            "exit".to_string(),
        ];

        let geofence: Geofence = entity.into();
        // Invalid event type should be filtered out
        assert_eq!(geofence.event_types.len(), 2);
    }

    #[test]
    fn test_geofence_entity_with_metadata() {
        let mut entity = create_test_geofence_entity();
        entity.metadata = Some(serde_json::json!({"color": "blue", "priority": 1}));

        let geofence: Geofence = entity.into();
        assert!(geofence.metadata.is_some());
        let meta = geofence.metadata.unwrap();
        assert_eq!(meta["color"], "blue");
        assert_eq!(meta["priority"], 1);
    }

    #[test]
    fn test_geofence_entity_clone() {
        let entity = create_test_geofence_entity();
        let cloned = entity.clone();

        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.name, entity.name);
        assert_eq!(cloned.latitude, entity.latitude);
    }
}
