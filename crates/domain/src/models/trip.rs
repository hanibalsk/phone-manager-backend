//! Trip domain model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use validator::Validate;

use super::movement_event::{DetectionSource, TransportationMode};

// ============================================================================
// Trip State Enum
// ============================================================================

/// State of a trip in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TripState {
    Active,
    Completed,
    Cancelled,
}

impl TripState {
    /// Returns the string representation for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            TripState::Active => "ACTIVE",
            TripState::Completed => "COMPLETED",
            TripState::Cancelled => "CANCELLED",
        }
    }

    /// Check if transition to target state is valid.
    pub fn can_transition_to(&self, target: TripState) -> bool {
        match (self, target) {
            // ACTIVE can transition to COMPLETED or CANCELLED
            (TripState::Active, TripState::Completed) => true,
            (TripState::Active, TripState::Cancelled) => true,
            // No other transitions allowed
            _ => false,
        }
    }
}

impl fmt::Display for TripState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TripState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACTIVE" => Ok(TripState::Active),
            "COMPLETED" => Ok(TripState::Completed),
            "CANCELLED" => Ok(TripState::Cancelled),
            _ => Err(format!(
                "Invalid trip state: {}. Must be one of: ACTIVE, COMPLETED, CANCELLED",
                s
            )),
        }
    }
}

// ============================================================================
// Core Model
// ============================================================================

/// Represents a trip record in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trip {
    pub id: Uuid,
    pub device_id: Uuid,
    pub local_trip_id: String,
    pub state: TripState,
    pub start_timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_timestamp: Option<i64>,
    pub start_latitude: f64,
    pub start_longitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_longitude: Option<f64>,
    pub transportation_mode: TransportationMode,
    pub detection_source: DetectionSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Request DTOs
// ============================================================================

/// Request payload for creating a trip.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateTripRequest {
    pub device_id: Uuid,

    #[validate(length(min = 1, max = 100, message = "localTripId must be 1-100 characters"))]
    pub local_trip_id: String,

    #[validate(custom(function = "shared::validation::validate_timestamp"))]
    pub start_timestamp: i64,

    #[validate(custom(function = "shared::validation::validate_latitude"))]
    pub start_latitude: f64,

    #[validate(custom(function = "shared::validation::validate_longitude"))]
    pub start_longitude: f64,

    pub transportation_mode: TransportationMode,

    pub detection_source: DetectionSource,
}

/// Request payload for updating a trip.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTripRequest {
    pub state: TripState,

    #[validate(custom(function = "crate::models::trip::validate_optional_timestamp"))]
    pub end_timestamp: Option<i64>,

    #[validate(custom(function = "crate::models::trip::validate_optional_latitude"))]
    pub end_latitude: Option<f64>,

    #[validate(custom(function = "crate::models::trip::validate_optional_longitude"))]
    pub end_longitude: Option<f64>,
}

/// Validates optional timestamp.
pub fn validate_optional_timestamp(timestamp: i64) -> Result<(), validator::ValidationError> {
    shared::validation::validate_timestamp(timestamp)
}

/// Validates optional latitude.
pub fn validate_optional_latitude(lat: f64) -> Result<(), validator::ValidationError> {
    shared::validation::validate_latitude(lat)
}

/// Validates optional longitude.
pub fn validate_optional_longitude(lon: f64) -> Result<(), validator::ValidationError> {
    shared::validation::validate_longitude(lon)
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Response payload for trip creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTripResponse {
    pub id: Uuid,
    pub local_trip_id: String,
    pub state: TripState,
    pub start_timestamp: i64,
    pub created_at: DateTime<Utc>,
}

/// Response payload for trip retrieval.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TripResponse {
    pub id: Uuid,
    pub local_trip_id: String,
    pub state: TripState,
    pub start_timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_timestamp: Option<i64>,
    pub start_latitude: f64,
    pub start_longitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_longitude: Option<f64>,
    pub transportation_mode: TransportationMode,
    pub detection_source: DetectionSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl From<Trip> for TripResponse {
    fn from(trip: Trip) -> Self {
        Self {
            id: trip.id,
            local_trip_id: trip.local_trip_id,
            state: trip.state,
            start_timestamp: trip.start_timestamp,
            end_timestamp: trip.end_timestamp,
            start_latitude: trip.start_latitude,
            start_longitude: trip.start_longitude,
            end_latitude: trip.end_latitude,
            end_longitude: trip.end_longitude,
            transportation_mode: trip.transportation_mode,
            detection_source: trip.detection_source,
            distance_meters: trip.distance_meters,
            duration_seconds: trip.duration_seconds,
            created_at: trip.created_at,
        }
    }
}

/// Pagination info for trips list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripPagination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Response for GET /api/v1/devices/:deviceId/trips
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTripsResponse {
    pub trips: Vec<TripResponse>,
    pub pagination: TripPagination,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn current_timestamp_millis() -> i64 {
        Utc::now().timestamp_millis()
    }

    // =========================================================================
    // TripState Tests
    // =========================================================================

    #[test]
    fn test_trip_state_as_str() {
        assert_eq!(TripState::Active.as_str(), "ACTIVE");
        assert_eq!(TripState::Completed.as_str(), "COMPLETED");
        assert_eq!(TripState::Cancelled.as_str(), "CANCELLED");
    }

    #[test]
    fn test_trip_state_from_str() {
        assert_eq!("ACTIVE".parse::<TripState>().unwrap(), TripState::Active);
        assert_eq!("COMPLETED".parse::<TripState>().unwrap(), TripState::Completed);
        assert_eq!("CANCELLED".parse::<TripState>().unwrap(), TripState::Cancelled);
    }

    #[test]
    fn test_trip_state_from_str_invalid() {
        assert!("invalid".parse::<TripState>().is_err());
        assert!("active".parse::<TripState>().is_err()); // lowercase
    }

    #[test]
    fn test_trip_state_display() {
        assert_eq!(format!("{}", TripState::Active), "ACTIVE");
        assert_eq!(format!("{}", TripState::Completed), "COMPLETED");
    }

    #[test]
    fn test_trip_state_serde() {
        let state = TripState::Completed;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"COMPLETED\"");

        let parsed: TripState = serde_json::from_str("\"CANCELLED\"").unwrap();
        assert_eq!(parsed, TripState::Cancelled);
    }

    #[test]
    fn test_trip_state_transitions() {
        // Valid transitions
        assert!(TripState::Active.can_transition_to(TripState::Completed));
        assert!(TripState::Active.can_transition_to(TripState::Cancelled));

        // Invalid transitions
        assert!(!TripState::Active.can_transition_to(TripState::Active));
        assert!(!TripState::Completed.can_transition_to(TripState::Active));
        assert!(!TripState::Completed.can_transition_to(TripState::Cancelled));
        assert!(!TripState::Cancelled.can_transition_to(TripState::Active));
        assert!(!TripState::Cancelled.can_transition_to(TripState::Completed));
    }

    // =========================================================================
    // CreateTripRequest Validation Tests
    // =========================================================================

    #[test]
    fn test_create_trip_request_valid() {
        let request = CreateTripRequest {
            device_id: Uuid::new_v4(),
            local_trip_id: "trip-123".to_string(),
            start_timestamp: current_timestamp_millis(),
            start_latitude: 45.0,
            start_longitude: -120.0,
            transportation_mode: TransportationMode::Walking,
            detection_source: DetectionSource::ActivityRecognition,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_trip_request_empty_local_trip_id() {
        let request = CreateTripRequest {
            device_id: Uuid::new_v4(),
            local_trip_id: "".to_string(),
            start_timestamp: current_timestamp_millis(),
            start_latitude: 45.0,
            start_longitude: -120.0,
            transportation_mode: TransportationMode::Walking,
            detection_source: DetectionSource::ActivityRecognition,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_trip_request_invalid_latitude() {
        let request = CreateTripRequest {
            device_id: Uuid::new_v4(),
            local_trip_id: "trip-123".to_string(),
            start_timestamp: current_timestamp_millis(),
            start_latitude: 100.0, // Invalid
            start_longitude: -120.0,
            transportation_mode: TransportationMode::Walking,
            detection_source: DetectionSource::ActivityRecognition,
        };
        assert!(request.validate().is_err());
    }

    // =========================================================================
    // UpdateTripRequest Validation Tests
    // =========================================================================

    #[test]
    fn test_update_trip_request_valid_completion() {
        let request = UpdateTripRequest {
            state: TripState::Completed,
            end_timestamp: Some(current_timestamp_millis()),
            end_latitude: Some(45.5),
            end_longitude: Some(-120.5),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_update_trip_request_valid_cancellation() {
        let request = UpdateTripRequest {
            state: TripState::Cancelled,
            end_timestamp: None,
            end_latitude: None,
            end_longitude: None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_update_trip_request_invalid_latitude() {
        let request = UpdateTripRequest {
            state: TripState::Completed,
            end_timestamp: Some(current_timestamp_millis()),
            end_latitude: Some(100.0), // Invalid
            end_longitude: Some(-120.5),
        };
        assert!(request.validate().is_err());
    }

    // =========================================================================
    // Response Tests
    // =========================================================================

    #[test]
    fn test_create_trip_response_serialization() {
        let response = CreateTripResponse {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            local_trip_id: "trip-123".to_string(),
            state: TripState::Active,
            start_timestamp: 1234567890000,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("550e8400-e29b-41d4-a716-446655440000"));
        assert!(json.contains("trip-123"));
        assert!(json.contains("ACTIVE"));
    }

    #[test]
    fn test_trip_response_from_trip() {
        let trip = Trip {
            id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            local_trip_id: "trip-123".to_string(),
            state: TripState::Active,
            start_timestamp: current_timestamp_millis(),
            end_timestamp: None,
            start_latitude: 45.0,
            start_longitude: -120.0,
            end_latitude: None,
            end_longitude: None,
            transportation_mode: TransportationMode::Walking,
            detection_source: DetectionSource::ActivityRecognition,
            distance_meters: None,
            duration_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response: TripResponse = trip.clone().into();
        assert_eq!(response.id, trip.id);
        assert_eq!(response.local_trip_id, trip.local_trip_id);
        assert_eq!(response.state, trip.state);
    }
}
