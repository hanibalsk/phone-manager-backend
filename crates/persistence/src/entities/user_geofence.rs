//! User geofence entity (database row mapping).
//!
//! User geofences apply to all devices owned by a user.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the user_geofences table.
#[derive(Debug, Clone, FromRow)]
pub struct UserGeofenceEntity {
    pub id: i64,
    pub geofence_id: Uuid,
    pub user_id: Uuid,
    pub created_by: Option<Uuid>,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: f32,
    pub event_types: Vec<String>, // SQLx maps TEXT[] to Vec<String>
    pub active: bool,
    pub color: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User geofence with creator info for listing.
#[derive(Debug, Clone, FromRow)]
pub struct UserGeofenceWithCreatorEntity {
    pub id: i64,
    pub geofence_id: Uuid,
    pub user_id: Uuid,
    pub created_by: Option<Uuid>,
    pub created_by_name: Option<String>,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: f32,
    pub event_types: Vec<String>,
    pub active: bool,
    pub color: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user_geofence_entity() -> UserGeofenceEntity {
        UserGeofenceEntity {
            id: 1,
            geofence_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            created_by: Some(Uuid::new_v4()),
            name: "Home".to_string(),
            latitude: 37.7749,
            longitude: -122.4194,
            radius_meters: 100.0,
            event_types: vec!["enter".to_string(), "exit".to_string()],
            active: true,
            color: Some("#FF5733".to_string()),
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_user_geofence_entity_clone() {
        let entity = create_test_user_geofence_entity();
        let cloned = entity.clone();

        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.name, entity.name);
        assert_eq!(cloned.latitude, entity.latitude);
        assert_eq!(cloned.user_id, entity.user_id);
    }

    #[test]
    fn test_user_geofence_entity_with_metadata() {
        let mut entity = create_test_user_geofence_entity();
        entity.metadata = Some(serde_json::json!({"priority": "high"}));

        assert!(entity.metadata.is_some());
        let meta = entity.metadata.unwrap();
        assert_eq!(meta["priority"], "high");
    }
}
