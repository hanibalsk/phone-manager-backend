//! Managed user entity for admin queries.
//!
//! Represents user data aggregated with device and location information.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Managed user with aggregated data from admin queries.
#[derive(Debug, Clone, FromRow)]
pub struct ManagedUserEntity {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub tracking_enabled: bool,
    pub device_count: i64,
    pub organization_id: Option<Uuid>,
    pub organization_name: Option<String>,
    pub created_at: DateTime<Utc>,
    // Last location fields (nullable - user may have no devices/locations)
    pub last_device_id: Option<Uuid>,
    pub last_device_name: Option<String>,
    pub last_latitude: Option<f64>,
    pub last_longitude: Option<f64>,
    pub last_accuracy: Option<f32>,
    pub last_captured_at: Option<DateTime<Utc>>,
}

/// Row for user location queries (single location result).
#[derive(Debug, Clone, FromRow)]
pub struct UserLocationEntity {
    pub device_id: Uuid,
    pub device_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f32,
    pub captured_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_managed_user_entity() -> ManagedUserEntity {
        ManagedUserEntity {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            tracking_enabled: true,
            device_count: 2,
            organization_id: Some(Uuid::new_v4()),
            organization_name: Some("Test Org".to_string()),
            created_at: Utc::now(),
            last_device_id: Some(Uuid::new_v4()),
            last_device_name: Some("iPhone".to_string()),
            last_latitude: Some(37.7749),
            last_longitude: Some(-122.4194),
            last_accuracy: Some(10.0),
            last_captured_at: Some(Utc::now()),
        }
    }

    #[test]
    fn test_managed_user_entity_clone() {
        let entity = create_test_managed_user_entity();
        let cloned = entity.clone();

        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.email, entity.email);
        assert_eq!(cloned.device_count, entity.device_count);
    }

    #[test]
    fn test_managed_user_entity_no_location() {
        let mut entity = create_test_managed_user_entity();
        entity.last_device_id = None;
        entity.last_device_name = None;
        entity.last_latitude = None;
        entity.last_longitude = None;
        entity.last_accuracy = None;
        entity.last_captured_at = None;

        assert!(entity.last_device_id.is_none());
        assert!(entity.last_latitude.is_none());
    }

    #[test]
    fn test_user_location_entity() {
        let entity = UserLocationEntity {
            device_id: Uuid::new_v4(),
            device_name: "Android".to_string(),
            latitude: 40.7128,
            longitude: -74.0060,
            accuracy: 5.0,
            captured_at: Utc::now(),
        };

        assert_eq!(entity.device_name, "Android");
        assert_eq!(entity.latitude, 40.7128);
    }
}
