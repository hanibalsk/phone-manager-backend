//! User-level geofence models (Epic 9).
//!
//! User geofences apply to all devices owned by the user,
//! unlike device geofences which are per-device.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::geofence::GeofenceEventType;

/// User geofence information.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UserGeofence {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: f32,
    pub event_types: Vec<GeofenceEventType>,
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Default event types for new user geofences.
fn default_event_types() -> Vec<GeofenceEventType> {
    vec![GeofenceEventType::Enter, GeofenceEventType::Exit]
}

/// Request to create a user geofence.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateUserGeofenceRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,

    #[validate(custom(function = "shared::validation::validate_latitude"))]
    pub latitude: f64,

    #[validate(custom(function = "shared::validation::validate_longitude"))]
    pub longitude: f64,

    #[validate(range(
        min = 20.0,
        max = 50000.0,
        message = "Radius must be between 20 and 50000 meters"
    ))]
    pub radius_meters: f32,

    #[serde(default = "default_event_types")]
    pub event_types: Vec<GeofenceEventType>,

    #[validate(length(max = 7, message = "Color must be a hex color code like #FF5733"))]
    pub color: Option<String>,

    pub metadata: Option<serde_json::Value>,
}

/// Response for creating a user geofence.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateUserGeofenceResponse {
    pub geofence: UserGeofence,
}

/// Request to update a user geofence.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateUserGeofenceRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: Option<String>,

    #[validate(custom(function = "shared::validation::validate_latitude"))]
    pub latitude: Option<f64>,

    #[validate(custom(function = "shared::validation::validate_longitude"))]
    pub longitude: Option<f64>,

    #[validate(range(
        min = 20.0,
        max = 50000.0,
        message = "Radius must be between 20 and 50000 meters"
    ))]
    pub radius_meters: Option<f32>,

    pub event_types: Option<Vec<GeofenceEventType>>,

    pub active: Option<bool>,

    #[validate(length(max = 7, message = "Color must be a hex color code like #FF5733"))]
    pub color: Option<String>,

    pub metadata: Option<serde_json::Value>,
}

/// Response for updating a user geofence.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateUserGeofenceResponse {
    pub geofence: UserGeofence,
}

/// Response for listing user geofences.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListUserGeofencesResponse {
    pub geofences: Vec<UserGeofence>,
    pub total: i64,
}

/// Response for deleting a user geofence.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DeleteUserGeofenceResponse {
    pub geofence_id: Uuid,
    pub deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_geofence_request_defaults() {
        let json = r#"{
            "name": "Home",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius_meters": 100.0
        }"#;

        let request: CreateUserGeofenceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Home");
        assert_eq!(request.latitude, 37.7749);
        assert_eq!(request.longitude, -122.4194);
        assert_eq!(request.radius_meters, 100.0);
        // Defaults should be applied
        assert_eq!(request.event_types.len(), 2);
        assert!(request.color.is_none());
        assert!(request.metadata.is_none());
    }

    #[test]
    fn test_create_user_geofence_request_with_all_fields() {
        let json = r##"{
            "name": "Office",
            "latitude": 40.7128,
            "longitude": -74.0060,
            "radius_meters": 500.0,
            "event_types": ["enter", "exit", "dwell"],
            "color": "#FF5733",
            "metadata": {"priority": "high"}
        }"##;

        let request: CreateUserGeofenceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Office");
        assert_eq!(request.event_types.len(), 3);
        assert_eq!(request.color, Some("#FF5733".to_string()));
        assert!(request.metadata.is_some());
    }

    #[test]
    fn test_update_user_geofence_request_partial() {
        let json = r#"{"name": "New Name"}"#;
        let request: UpdateUserGeofenceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, Some("New Name".to_string()));
        assert!(request.latitude.is_none());
        assert!(request.longitude.is_none());
        assert!(request.radius_meters.is_none());
        assert!(request.event_types.is_none());
        assert!(request.active.is_none());
    }
}
