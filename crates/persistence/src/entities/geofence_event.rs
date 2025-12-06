//! Geofence event entity (database row mapping).
//!
//! Story 15.2: Webhook Event Delivery
//! Maps to the `geofence_events` table.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the geofence_events table.
#[derive(Debug, Clone, FromRow)]
pub struct GeofenceEventEntity {
    pub id: i64,
    pub event_id: Uuid,
    pub device_id: Uuid,
    pub geofence_id: Uuid,
    pub event_type: String,
    pub timestamp: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub webhook_delivered: bool,
    pub webhook_response_code: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Entity with joined geofence name for response enrichment.
#[derive(Debug, Clone, FromRow)]
pub struct GeofenceEventWithName {
    pub id: i64,
    pub event_id: Uuid,
    pub device_id: Uuid,
    pub geofence_id: Uuid,
    pub geofence_name: Option<String>,
    pub event_type: String,
    pub timestamp: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub webhook_delivered: bool,
    pub webhook_response_code: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entity() -> GeofenceEventEntity {
        GeofenceEventEntity {
            id: 1,
            event_id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            geofence_id: Uuid::new_v4(),
            event_type: "enter".to_string(),
            timestamp: 1701878400000,
            latitude: 37.7749,
            longitude: -122.4194,
            webhook_delivered: false,
            webhook_response_code: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_entity_clone() {
        let entity = create_test_entity();
        let cloned = entity.clone();
        assert_eq!(cloned.event_id, entity.event_id);
        assert_eq!(cloned.event_type, entity.event_type);
    }

    #[test]
    fn test_entity_debug() {
        let entity = create_test_entity();
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("GeofenceEventEntity"));
        assert!(debug_str.contains("enter"));
    }

    #[test]
    fn test_entity_with_webhook_status() {
        let mut entity = create_test_entity();
        entity.webhook_delivered = true;
        entity.webhook_response_code = Some(200);
        assert!(entity.webhook_delivered);
        assert_eq!(entity.webhook_response_code, Some(200));
    }
}
