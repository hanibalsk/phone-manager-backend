//! Movement event entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the movement_events table.
///
/// Note: The location column uses PostGIS GEOGRAPHY type which we read as
/// separate latitude/longitude values via ST_Y and ST_X in queries.
#[derive(Debug, Clone, FromRow)]
pub struct MovementEventEntity {
    pub id: Uuid,
    pub device_id: Uuid,
    pub trip_id: Option<Uuid>,
    pub timestamp: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f32,
    pub speed: Option<f32>,
    pub bearing: Option<f32>,
    pub altitude: Option<f64>,
    pub transportation_mode: String,
    pub confidence: f32,
    pub detection_source: String,
    pub created_at: DateTime<Utc>,
}

impl MovementEventEntity {
    /// Convert to domain model.
    pub fn into_domain(self) -> domain::models::MovementEvent {
        use domain::models::movement_event::{DetectionSource, TransportationMode};

        let transportation_mode = self
            .transportation_mode
            .parse::<TransportationMode>()
            .unwrap_or(TransportationMode::Unknown);

        let detection_source = self
            .detection_source
            .parse::<DetectionSource>()
            .unwrap_or(DetectionSource::None);

        domain::models::MovementEvent {
            id: self.id,
            device_id: self.device_id,
            trip_id: self.trip_id,
            timestamp: self.timestamp,
            latitude: self.latitude,
            longitude: self.longitude,
            accuracy: self.accuracy as f64,
            speed: self.speed.map(|s| s as f64),
            bearing: self.bearing.map(|b| b as f64),
            altitude: self.altitude,
            transportation_mode,
            confidence: self.confidence as f64,
            detection_source,
            created_at: self.created_at,
        }
    }
}

impl From<MovementEventEntity> for domain::models::MovementEvent {
    fn from(entity: MovementEventEntity) -> Self {
        entity.into_domain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::movement_event::{DetectionSource, TransportationMode};

    fn create_test_entity() -> MovementEventEntity {
        MovementEventEntity {
            id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            trip_id: Some(Uuid::new_v4()),
            timestamp: Utc::now().timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            speed: Some(5.5),
            bearing: Some(180.0),
            altitude: Some(100.0),
            transportation_mode: "WALKING".to_string(),
            confidence: 0.95,
            detection_source: "ACTIVITY_RECOGNITION".to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_entity_to_domain() {
        let entity = create_test_entity();
        let event: domain::models::MovementEvent = entity.clone().into();

        assert_eq!(event.id, entity.id);
        assert_eq!(event.device_id, entity.device_id);
        assert_eq!(event.trip_id, entity.trip_id);
        assert_eq!(event.timestamp, entity.timestamp);
        assert_eq!(event.latitude, entity.latitude);
        assert_eq!(event.longitude, entity.longitude);
        assert_eq!(event.accuracy, entity.accuracy as f64);
        assert_eq!(event.speed, Some(5.5));
        assert_eq!(event.bearing, Some(180.0));
        assert_eq!(event.altitude, Some(100.0));
        assert_eq!(event.transportation_mode, TransportationMode::Walking);
        assert_eq!(event.confidence, 0.95_f32 as f64);
        assert_eq!(event.detection_source, DetectionSource::ActivityRecognition);
    }

    #[test]
    fn test_entity_with_unknown_mode() {
        let mut entity = create_test_entity();
        entity.transportation_mode = "INVALID".to_string();

        let event: domain::models::MovementEvent = entity.into();
        assert_eq!(event.transportation_mode, TransportationMode::Unknown);
    }

    #[test]
    fn test_entity_with_unknown_source() {
        let mut entity = create_test_entity();
        entity.detection_source = "INVALID".to_string();

        let event: domain::models::MovementEvent = entity.into();
        assert_eq!(event.detection_source, DetectionSource::None);
    }

    #[test]
    fn test_entity_with_no_optional_fields() {
        let entity = MovementEventEntity {
            id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            trip_id: None,
            timestamp: Utc::now().timestamp_millis(),
            latitude: 0.0,
            longitude: 0.0,
            accuracy: 0.0,
            speed: None,
            bearing: None,
            altitude: None,
            transportation_mode: "STATIONARY".to_string(),
            confidence: 0.0,
            detection_source: "NONE".to_string(),
            created_at: Utc::now(),
        };

        let event: domain::models::MovementEvent = entity.into();
        assert!(event.trip_id.is_none());
        assert!(event.speed.is_none());
        assert!(event.bearing.is_none());
        assert!(event.altitude.is_none());
    }

    #[test]
    fn test_entity_clone() {
        let entity = create_test_entity();
        let cloned = entity.clone();
        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.latitude, entity.latitude);
    }

    #[test]
    fn test_entity_debug() {
        let entity = create_test_entity();
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("MovementEventEntity"));
        assert!(debug_str.contains("WALKING"));
    }
}
