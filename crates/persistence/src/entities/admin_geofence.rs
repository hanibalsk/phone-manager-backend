//! Admin geofence entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the admin_geofences table.
#[derive(Debug, Clone, FromRow)]
pub struct AdminGeofenceEntity {
    pub id: i64,
    pub geofence_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: f32,
    pub event_types: Vec<String>,
    pub active: bool,
    pub color: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Admin geofence with creator info for listing.
#[derive(Debug, Clone, FromRow)]
pub struct AdminGeofenceWithCreatorEntity {
    pub id: i64,
    pub geofence_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: f32,
    pub event_types: Vec<String>,
    pub active: bool,
    pub color: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub creator_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Admin geofence event with device and geofence info for organization-level queries.
#[derive(Debug, Clone, FromRow)]
pub struct AdminGeofenceEventEntity {
    pub id: i64,
    pub event_id: Uuid,
    pub device_id: Uuid,
    pub device_name: Option<String>,
    pub geofence_id: Uuid,
    pub geofence_name: Option<String>,
    pub event_type: String,
    pub timestamp: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub created_at: DateTime<Utc>,
}

/// Location analytics summary for an organization.
#[derive(Debug, Clone, FromRow)]
pub struct LocationAnalyticsEntity {
    pub total_devices: i64,
    pub devices_with_location: i64,
    pub total_locations_today: i64,
    pub total_geofences: i64,
    pub total_geofence_events_today: i64,
}

/// Geofence visit count for analytics.
#[derive(Debug, Clone, FromRow)]
pub struct GeofenceVisitCountEntity {
    pub geofence_id: Uuid,
    pub geofence_name: String,
    pub visit_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_admin_geofence_entity() -> AdminGeofenceEntity {
        AdminGeofenceEntity {
            id: 1,
            geofence_id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "Office Zone".to_string(),
            description: Some("Main office building perimeter".to_string()),
            latitude: 37.7749,
            longitude: -122.4194,
            radius_meters: 500.0,
            event_types: vec!["enter".to_string(), "exit".to_string()],
            active: true,
            color: Some("#FF5733".to_string()),
            metadata: None,
            created_by: Some(Uuid::new_v4()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_admin_geofence_entity_clone() {
        let entity = create_test_admin_geofence_entity();
        let cloned = entity.clone();

        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.name, entity.name);
        assert_eq!(cloned.latitude, entity.latitude);
        assert_eq!(cloned.organization_id, entity.organization_id);
    }

    #[test]
    fn test_admin_geofence_entity_with_metadata() {
        let mut entity = create_test_admin_geofence_entity();
        entity.metadata = Some(serde_json::json!({"category": "work", "priority": "high"}));

        assert!(entity.metadata.is_some());
        let meta = entity.metadata.unwrap();
        assert_eq!(meta["category"], "work");
    }
}
