//! Trip entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the trips table.
///
/// Note: The start_location and end_location columns use PostGIS GEOGRAPHY type
/// which we read as separate latitude/longitude values via ST_Y and ST_X in queries.
#[derive(Debug, Clone, FromRow)]
pub struct TripEntity {
    pub id: Uuid,
    pub device_id: Uuid,
    pub local_trip_id: String,
    pub state: String,
    pub start_timestamp: i64,
    pub end_timestamp: Option<i64>,
    pub start_latitude: f64,
    pub start_longitude: f64,
    pub end_latitude: Option<f64>,
    pub end_longitude: Option<f64>,
    pub transportation_mode: String,
    pub detection_source: String,
    pub distance_meters: Option<f64>,
    pub duration_seconds: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TripEntity {
    /// Convert to domain model.
    pub fn into_domain(self) -> domain::models::Trip {
        use domain::models::movement_event::{DetectionSource, TransportationMode};
        use domain::models::trip::TripState;

        let transportation_mode = self
            .transportation_mode
            .parse::<TransportationMode>()
            .unwrap_or(TransportationMode::Unknown);

        let detection_source = self
            .detection_source
            .parse::<DetectionSource>()
            .unwrap_or(DetectionSource::None);

        let state = self
            .state
            .parse::<TripState>()
            .unwrap_or(TripState::Active);

        domain::models::Trip {
            id: self.id,
            device_id: self.device_id,
            local_trip_id: self.local_trip_id,
            state,
            start_timestamp: self.start_timestamp,
            end_timestamp: self.end_timestamp,
            start_latitude: self.start_latitude,
            start_longitude: self.start_longitude,
            end_latitude: self.end_latitude,
            end_longitude: self.end_longitude,
            transportation_mode,
            detection_source,
            distance_meters: self.distance_meters,
            duration_seconds: self.duration_seconds,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl From<TripEntity> for domain::models::Trip {
    fn from(entity: TripEntity) -> Self {
        entity.into_domain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::movement_event::{DetectionSource, TransportationMode};
    use domain::models::trip::TripState;

    fn create_test_entity() -> TripEntity {
        TripEntity {
            id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            local_trip_id: "test-trip-123".to_string(),
            state: "ACTIVE".to_string(),
            start_timestamp: Utc::now().timestamp_millis(),
            end_timestamp: None,
            start_latitude: 45.0,
            start_longitude: -120.0,
            end_latitude: None,
            end_longitude: None,
            transportation_mode: "WALKING".to_string(),
            detection_source: "ACTIVITY_RECOGNITION".to_string(),
            distance_meters: None,
            duration_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_entity_to_domain() {
        let entity = create_test_entity();
        let trip: domain::models::Trip = entity.clone().into();

        assert_eq!(trip.id, entity.id);
        assert_eq!(trip.device_id, entity.device_id);
        assert_eq!(trip.local_trip_id, entity.local_trip_id);
        assert_eq!(trip.state, TripState::Active);
        assert_eq!(trip.start_timestamp, entity.start_timestamp);
        assert_eq!(trip.start_latitude, entity.start_latitude);
        assert_eq!(trip.start_longitude, entity.start_longitude);
        assert_eq!(trip.transportation_mode, TransportationMode::Walking);
        assert_eq!(trip.detection_source, DetectionSource::ActivityRecognition);
    }

    #[test]
    fn test_entity_with_completed_state() {
        let mut entity = create_test_entity();
        entity.state = "COMPLETED".to_string();
        entity.end_timestamp = Some(Utc::now().timestamp_millis());
        entity.end_latitude = Some(45.5);
        entity.end_longitude = Some(-120.5);
        entity.distance_meters = Some(1500.0);
        entity.duration_seconds = Some(3600);

        let trip: domain::models::Trip = entity.into();
        assert_eq!(trip.state, TripState::Completed);
        assert!(trip.end_timestamp.is_some());
        assert!(trip.end_latitude.is_some());
        assert!(trip.distance_meters.is_some());
        assert!(trip.duration_seconds.is_some());
    }

    #[test]
    fn test_entity_with_cancelled_state() {
        let mut entity = create_test_entity();
        entity.state = "CANCELLED".to_string();

        let trip: domain::models::Trip = entity.into();
        assert_eq!(trip.state, TripState::Cancelled);
    }

    #[test]
    fn test_entity_with_unknown_state_defaults_to_active() {
        let mut entity = create_test_entity();
        entity.state = "INVALID".to_string();

        let trip: domain::models::Trip = entity.into();
        assert_eq!(trip.state, TripState::Active);
    }

    #[test]
    fn test_entity_clone() {
        let entity = create_test_entity();
        let cloned = entity.clone();
        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.local_trip_id, entity.local_trip_id);
    }

    #[test]
    fn test_entity_debug() {
        let entity = create_test_entity();
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("TripEntity"));
        assert!(debug_str.contains("ACTIVE"));
    }
}
