//! Location entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the locations table.
#[derive(Debug, Clone, FromRow)]
pub struct LocationEntity {
    pub id: i64,
    pub device_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f32, // REAL (FLOAT4) in PostgreSQL
    pub altitude: Option<f64>,
    pub bearing: Option<f32>, // REAL (FLOAT4) in PostgreSQL
    pub speed: Option<f32>,   // REAL (FLOAT4) in PostgreSQL
    pub provider: Option<String>,
    pub battery_level: Option<i16>, // SMALLINT in PostgreSQL
    pub network_type: Option<String>,
    pub captured_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    // Context fields (Epic 7)
    pub transportation_mode: Option<String>,
    pub detection_source: Option<String>,
    pub trip_id: Option<Uuid>,
}

impl From<LocationEntity> for domain::models::Location {
    fn from(entity: LocationEntity) -> Self {
        Self {
            id: entity.id,
            device_id: entity.device_id,
            latitude: entity.latitude,
            longitude: entity.longitude,
            accuracy: entity.accuracy as f64, // f32 → f64
            altitude: entity.altitude,
            bearing: entity.bearing.map(|b| b as f64), // f32 → f64
            speed: entity.speed.map(|s| s as f64),     // f32 → f64
            provider: entity.provider,
            battery_level: entity.battery_level.map(|b| b as i32), // i16 → i32
            network_type: entity.network_type,
            captured_at: entity.captured_at,
            created_at: entity.created_at,
            transportation_mode: entity.transportation_mode,
            detection_source: entity.detection_source,
            trip_id: entity.trip_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_location_entity() -> LocationEntity {
        LocationEntity {
            id: 1,
            device_id: Uuid::new_v4(),
            latitude: 37.7749,
            longitude: -122.4194,
            accuracy: 10.0_f32,
            altitude: Some(100.0),
            bearing: Some(180.0_f32),
            speed: Some(5.5_f32),
            provider: Some("gps".to_string()),
            battery_level: Some(85_i16),
            network_type: Some("wifi".to_string()),
            captured_at: Utc::now(),
            created_at: Utc::now(),
            transportation_mode: None,
            detection_source: None,
            trip_id: None,
        }
    }

    #[test]
    fn test_location_entity_to_domain() {
        let entity = create_test_location_entity();
        let location: domain::models::Location = entity.clone().into();

        assert_eq!(location.id, entity.id);
        assert_eq!(location.device_id, entity.device_id);
        assert_eq!(location.latitude, entity.latitude);
        assert_eq!(location.longitude, entity.longitude);
        assert_eq!(location.accuracy, entity.accuracy as f64); // f32 → f64
        assert_eq!(location.altitude, entity.altitude);
        assert_eq!(location.bearing, entity.bearing.map(|b| b as f64)); // f32 → f64
        assert_eq!(location.speed, entity.speed.map(|s| s as f64)); // f32 → f64
        assert_eq!(location.provider, entity.provider);
        assert_eq!(
            location.battery_level,
            entity.battery_level.map(|b| b as i32)
        ); // i16 → i32
        assert_eq!(location.network_type, entity.network_type);
    }

    #[test]
    fn test_location_entity_clone() {
        let entity = create_test_location_entity();
        let cloned = entity.clone();
        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.latitude, entity.latitude);
        assert_eq!(cloned.longitude, entity.longitude);
    }

    #[test]
    fn test_location_entity_debug() {
        let entity = create_test_location_entity();
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("LocationEntity"));
        assert!(debug_str.contains("37.7749"));
    }

    #[test]
    fn test_location_entity_optional_fields_none() {
        let entity = LocationEntity {
            id: 1,
            device_id: Uuid::new_v4(),
            latitude: 37.7749,
            longitude: -122.4194,
            accuracy: 10.0_f32,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
            captured_at: Utc::now(),
            created_at: Utc::now(),
            transportation_mode: None,
            detection_source: None,
            trip_id: None,
        };

        let location: domain::models::Location = entity.into();
        assert!(location.altitude.is_none());
        assert!(location.bearing.is_none());
        assert!(location.speed.is_none());
        assert!(location.provider.is_none());
        assert!(location.battery_level.is_none());
        assert!(location.network_type.is_none());
        assert!(location.transportation_mode.is_none());
        assert!(location.detection_source.is_none());
        assert!(location.trip_id.is_none());
    }

    #[test]
    fn test_location_entity_coordinates() {
        let entity = create_test_location_entity();
        // San Francisco coordinates
        assert!((entity.latitude - 37.7749).abs() < 0.0001);
        assert!((entity.longitude - (-122.4194)).abs() < 0.0001);
    }

    #[test]
    fn test_location_entity_timestamps() {
        let entity = create_test_location_entity();
        assert!(entity.captured_at <= Utc::now());
        assert!(entity.created_at <= Utc::now());
    }
}
