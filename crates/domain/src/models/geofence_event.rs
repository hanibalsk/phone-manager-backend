//! Geofence event domain models and DTOs.
//!
//! Story 15.2: Webhook Event Delivery
//! Models for geofence event API aligned with frontend expectations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Geofence event transition type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GeofenceTransitionType {
    Enter,
    Exit,
    Dwell,
}

impl GeofenceTransitionType {
    /// Convert to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Enter => "enter",
            Self::Exit => "exit",
            Self::Dwell => "dwell",
        }
    }

    /// Parse from string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "enter" => Some(Self::Enter),
            "exit" => Some(Self::Exit),
            "dwell" => Some(Self::Dwell),
            _ => None,
        }
    }

    /// Convert to webhook event type string.
    pub fn to_webhook_event_type(&self) -> &'static str {
        match self {
            Self::Enter => "geofence_enter",
            Self::Exit => "geofence_exit",
            Self::Dwell => "geofence_dwell",
        }
    }
}

impl std::fmt::Display for GeofenceTransitionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Domain model for a geofence event.
#[derive(Debug, Clone)]
pub struct GeofenceEvent {
    pub id: i64,
    pub event_id: Uuid,
    pub device_id: Uuid,
    pub geofence_id: Uuid,
    pub geofence_name: Option<String>,
    pub event_type: GeofenceTransitionType,
    pub timestamp: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub webhook_delivered: bool,
    pub webhook_response_code: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Request to create a geofence event.
/// POST /api/v1/geofence-events
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateGeofenceEventRequest {
    pub device_id: Uuid,
    pub geofence_id: Uuid,
    pub event_type: GeofenceTransitionType,
    #[validate(length(min = 1, message = "Timestamp is required"))]
    pub timestamp: String,
    #[validate(range(min = -90.0, max = 90.0, message = "Latitude must be between -90 and 90"))]
    pub latitude: f64,
    #[validate(range(min = -180.0, max = 180.0, message = "Longitude must be between -180 and 180"))]
    pub longitude: f64,
}

impl CreateGeofenceEventRequest {
    /// Parse timestamp from ISO 8601 or milliseconds string.
    pub fn parse_timestamp(&self) -> Result<i64, String> {
        // Try parsing as integer (milliseconds)
        if let Ok(millis) = self.timestamp.parse::<i64>() {
            if millis > 0 {
                return Ok(millis);
            }
            return Err("Timestamp must be positive".to_string());
        }

        // Try parsing as ISO 8601
        if let Ok(dt) = DateTime::parse_from_rfc3339(&self.timestamp) {
            return Ok(dt.timestamp_millis());
        }

        // Try parsing as ISO 8601 without timezone (assume UTC)
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&self.timestamp, "%Y-%m-%dT%H:%M:%S%.fZ") {
            return Ok(dt.and_utc().timestamp_millis());
        }

        Err("Invalid timestamp format. Use milliseconds or ISO 8601".to_string())
    }
}

/// Query parameters for listing geofence events.
/// GET /api/v1/geofence-events?deviceId=<uuid>
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListGeofenceEventsQuery {
    pub device_id: Uuid,
    pub geofence_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

/// Response for a single geofence event.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GeofenceEventResponse {
    pub event_id: Uuid,
    pub device_id: Uuid,
    pub geofence_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geofence_name: Option<String>,
    pub event_type: GeofenceTransitionType,
    pub timestamp: String,
    pub latitude: f64,
    pub longitude: f64,
    pub webhook_delivered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_response_code: Option<i32>,
}

impl From<GeofenceEvent> for GeofenceEventResponse {
    fn from(event: GeofenceEvent) -> Self {
        Self {
            event_id: event.event_id,
            device_id: event.device_id,
            geofence_id: event.geofence_id,
            geofence_name: event.geofence_name,
            event_type: event.event_type,
            timestamp: DateTime::from_timestamp_millis(event.timestamp)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| event.timestamp.to_string()),
            latitude: event.latitude,
            longitude: event.longitude,
            webhook_delivered: event.webhook_delivered,
            webhook_response_code: event.webhook_response_code,
        }
    }
}

/// Response for listing geofence events.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListGeofenceEventsResponse {
    pub events: Vec<GeofenceEventResponse>,
    pub total: i64,
}

impl GeofenceEvent {
    /// Create a GeofenceEvent from raw field values.
    /// Used for converting from persistence entities.
    #[allow(clippy::too_many_arguments)]
    pub fn from_raw(
        id: i64,
        event_id: Uuid,
        device_id: Uuid,
        geofence_id: Uuid,
        geofence_name: Option<String>,
        event_type: &str,
        timestamp: i64,
        latitude: f64,
        longitude: f64,
        webhook_delivered: bool,
        webhook_response_code: Option<i32>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            event_id,
            device_id,
            geofence_id,
            geofence_name,
            event_type: GeofenceTransitionType::parse(event_type)
                .unwrap_or(GeofenceTransitionType::Enter),
            timestamp,
            latitude,
            longitude,
            webhook_delivered,
            webhook_response_code,
            created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geofence_transition_type_as_str() {
        assert_eq!(GeofenceTransitionType::Enter.as_str(), "enter");
        assert_eq!(GeofenceTransitionType::Exit.as_str(), "exit");
        assert_eq!(GeofenceTransitionType::Dwell.as_str(), "dwell");
    }

    #[test]
    fn test_geofence_transition_type_parse() {
        assert_eq!(GeofenceTransitionType::parse("enter"), Some(GeofenceTransitionType::Enter));
        assert_eq!(GeofenceTransitionType::parse("EXIT"), Some(GeofenceTransitionType::Exit));
        assert_eq!(GeofenceTransitionType::parse("Dwell"), Some(GeofenceTransitionType::Dwell));
        assert_eq!(GeofenceTransitionType::parse("invalid"), None);
    }

    #[test]
    fn test_geofence_transition_type_to_webhook_event_type() {
        assert_eq!(GeofenceTransitionType::Enter.to_webhook_event_type(), "geofence_enter");
        assert_eq!(GeofenceTransitionType::Exit.to_webhook_event_type(), "geofence_exit");
        assert_eq!(GeofenceTransitionType::Dwell.to_webhook_event_type(), "geofence_dwell");
    }

    #[test]
    fn test_create_request_deserialization() {
        let json = r#"{
            "device_id": "550e8400-e29b-41d4-a716-446655440000",
            "geofence_id": "660e8400-e29b-41d4-a716-446655440001",
            "event_type": "enter",
            "timestamp": "1701878400000",
            "latitude": 37.7749,
            "longitude": -122.4194
        }"#;

        let request: CreateGeofenceEventRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.event_type, GeofenceTransitionType::Enter);
        assert_eq!(request.latitude, 37.7749);
    }

    #[test]
    fn test_create_request_parse_timestamp_millis() {
        let request = CreateGeofenceEventRequest {
            device_id: Uuid::new_v4(),
            geofence_id: Uuid::new_v4(),
            event_type: GeofenceTransitionType::Enter,
            timestamp: "1701878400000".to_string(),
            latitude: 37.7749,
            longitude: -122.4194,
        };
        assert_eq!(request.parse_timestamp().unwrap(), 1701878400000);
    }

    #[test]
    fn test_create_request_parse_timestamp_iso() {
        let request = CreateGeofenceEventRequest {
            device_id: Uuid::new_v4(),
            geofence_id: Uuid::new_v4(),
            event_type: GeofenceTransitionType::Enter,
            timestamp: "2023-12-06T12:00:00Z".to_string(),
            latitude: 37.7749,
            longitude: -122.4194,
        };
        assert!(request.parse_timestamp().is_ok());
    }

    #[test]
    fn test_list_query_deserialization() {
        let json = r#"{"deviceId": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let query: ListGeofenceEventsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, 50); // default
        assert!(query.geofence_id.is_none());
    }

    #[test]
    fn test_response_serialization() {
        let response = GeofenceEventResponse {
            event_id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            geofence_id: Uuid::new_v4(),
            geofence_name: Some("Home".to_string()),
            event_type: GeofenceTransitionType::Enter,
            timestamp: "2023-12-06T12:00:00Z".to_string(),
            latitude: 37.7749,
            longitude: -122.4194,
            webhook_delivered: true,
            webhook_response_code: Some(200),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"event_type\":\"enter\""));
        assert!(json.contains("\"webhook_delivered\":true"));
    }
}
