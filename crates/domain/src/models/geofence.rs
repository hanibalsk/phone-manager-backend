//! Geofence domain model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Represents a geofence in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Geofence {
    pub id: i64,
    pub geofence_id: Uuid,
    pub device_id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: f32,
    pub event_types: Vec<GeofenceEventType>,
    pub active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supported geofence event types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GeofenceEventType {
    Enter,
    Exit,
    Dwell,
}

impl GeofenceEventType {
    /// Converts to database string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            GeofenceEventType::Enter => "enter",
            GeofenceEventType::Exit => "exit",
            GeofenceEventType::Dwell => "dwell",
        }
    }

    /// Parses from database string representation.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "enter" => Some(GeofenceEventType::Enter),
            "exit" => Some(GeofenceEventType::Exit),
            "dwell" => Some(GeofenceEventType::Dwell),
            _ => None,
        }
    }
}

/// Default event types for new geofences.
fn default_event_types() -> Vec<GeofenceEventType> {
    vec![GeofenceEventType::Enter, GeofenceEventType::Exit]
}

/// Default active status for new geofences.
fn default_active() -> bool {
    true
}

/// Request payload for creating a geofence.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateGeofenceRequest {
    pub device_id: Uuid,

    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,

    #[validate(custom(function = "shared::validation::validate_latitude"))]
    pub latitude: f64,

    #[validate(custom(function = "shared::validation::validate_longitude"))]
    pub longitude: f64,

    #[validate(range(min = 20.0, max = 50000.0, message = "Radius must be between 20 and 50000 meters"))]
    pub radius_meters: f32,

    #[serde(default = "default_event_types")]
    pub event_types: Vec<GeofenceEventType>,

    #[serde(default = "default_active")]
    pub active: bool,

    pub metadata: Option<serde_json::Value>,
}

/// Request payload for updating a geofence (partial update).
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGeofenceRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: Option<String>,

    #[validate(custom(function = "shared::validation::validate_latitude"))]
    pub latitude: Option<f64>,

    #[validate(custom(function = "shared::validation::validate_longitude"))]
    pub longitude: Option<f64>,

    #[validate(range(min = 20.0, max = 50000.0, message = "Radius must be between 20 and 50000 meters"))]
    pub radius_meters: Option<f32>,

    pub event_types: Option<Vec<GeofenceEventType>>,

    pub active: Option<bool>,

    pub metadata: Option<serde_json::Value>,
}

/// Response payload for geofence operations.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeofenceResponse {
    pub geofence_id: Uuid,
    pub device_id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: f32,
    pub event_types: Vec<GeofenceEventType>,
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Geofence> for GeofenceResponse {
    fn from(g: Geofence) -> Self {
        Self {
            geofence_id: g.geofence_id,
            device_id: g.device_id,
            name: g.name,
            latitude: g.latitude,
            longitude: g.longitude,
            radius_meters: g.radius_meters,
            event_types: g.event_types,
            active: g.active,
            metadata: g.metadata,
            created_at: g.created_at,
            updated_at: g.updated_at,
        }
    }
}

/// Response for listing geofences.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListGeofencesResponse {
    pub geofences: Vec<GeofenceResponse>,
    pub total: usize,
}

/// Query parameters for listing geofences.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListGeofencesQuery {
    pub device_id: Uuid,
    #[serde(default)]
    pub include_inactive: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geofence_event_type_serialization() {
        let enter = GeofenceEventType::Enter;
        let json = serde_json::to_string(&enter).unwrap();
        assert_eq!(json, "\"enter\"");

        let exit = GeofenceEventType::Exit;
        let json = serde_json::to_string(&exit).unwrap();
        assert_eq!(json, "\"exit\"");

        let dwell = GeofenceEventType::Dwell;
        let json = serde_json::to_string(&dwell).unwrap();
        assert_eq!(json, "\"dwell\"");
    }

    #[test]
    fn test_geofence_event_type_deserialization() {
        let enter: GeofenceEventType = serde_json::from_str("\"enter\"").unwrap();
        assert_eq!(enter, GeofenceEventType::Enter);

        let exit: GeofenceEventType = serde_json::from_str("\"exit\"").unwrap();
        assert_eq!(exit, GeofenceEventType::Exit);

        let dwell: GeofenceEventType = serde_json::from_str("\"dwell\"").unwrap();
        assert_eq!(dwell, GeofenceEventType::Dwell);
    }

    #[test]
    fn test_geofence_event_type_as_str() {
        assert_eq!(GeofenceEventType::Enter.as_str(), "enter");
        assert_eq!(GeofenceEventType::Exit.as_str(), "exit");
        assert_eq!(GeofenceEventType::Dwell.as_str(), "dwell");
    }

    #[test]
    fn test_geofence_event_type_from_str() {
        assert_eq!(
            GeofenceEventType::from_str("enter"),
            Some(GeofenceEventType::Enter)
        );
        assert_eq!(
            GeofenceEventType::from_str("exit"),
            Some(GeofenceEventType::Exit)
        );
        assert_eq!(
            GeofenceEventType::from_str("dwell"),
            Some(GeofenceEventType::Dwell)
        );
        assert_eq!(GeofenceEventType::from_str("invalid"), None);
    }

    #[test]
    fn test_default_event_types() {
        let types = default_event_types();
        assert_eq!(types.len(), 2);
        assert!(types.contains(&GeofenceEventType::Enter));
        assert!(types.contains(&GeofenceEventType::Exit));
    }

    #[test]
    fn test_create_geofence_request_deserialization() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Home",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radiusMeters": 100.0
        }"#;

        let request: CreateGeofenceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Home");
        assert_eq!(request.latitude, 37.7749);
        assert_eq!(request.longitude, -122.4194);
        assert_eq!(request.radius_meters, 100.0);
        // Defaults should be applied
        assert_eq!(request.event_types.len(), 2);
        assert!(request.active);
    }

    #[test]
    fn test_create_geofence_request_with_all_fields() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Office",
            "latitude": 40.7128,
            "longitude": -74.0060,
            "radiusMeters": 500.0,
            "eventTypes": ["enter", "exit", "dwell"],
            "active": false,
            "metadata": {"color": "blue", "icon": "work"}
        }"#;

        let request: CreateGeofenceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Office");
        assert_eq!(request.event_types.len(), 3);
        assert!(!request.active);
        assert!(request.metadata.is_some());
    }

    #[test]
    fn test_geofence_response_serialization() {
        let response = GeofenceResponse {
            geofence_id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            name: "Test".to_string(),
            latitude: 45.0,
            longitude: -120.0,
            radius_meters: 100.0,
            event_types: vec![GeofenceEventType::Enter],
            active: true,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"latitude\":45"));
        assert!(json.contains("\"radiusMeters\":100"));
        // metadata should be skipped when None
        assert!(!json.contains("\"metadata\":null"));
    }

    #[test]
    fn test_list_geofences_query_defaults() {
        let json = r#"{"deviceId": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let query: ListGeofencesQuery = serde_json::from_str(json).unwrap();
        assert!(!query.include_inactive);
    }
}
