//! Movement event domain model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Enums
// ============================================================================

/// Transportation mode detected during movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransportationMode {
    Stationary,
    Walking,
    Running,
    Cycling,
    InVehicle,
    Unknown,
}

impl TransportationMode {
    /// Returns the string representation for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            TransportationMode::Stationary => "STATIONARY",
            TransportationMode::Walking => "WALKING",
            TransportationMode::Running => "RUNNING",
            TransportationMode::Cycling => "CYCLING",
            TransportationMode::InVehicle => "IN_VEHICLE",
            TransportationMode::Unknown => "UNKNOWN",
        }
    }
}

impl fmt::Display for TransportationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TransportationMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "STATIONARY" => Ok(TransportationMode::Stationary),
            "WALKING" => Ok(TransportationMode::Walking),
            "RUNNING" => Ok(TransportationMode::Running),
            "CYCLING" => Ok(TransportationMode::Cycling),
            "IN_VEHICLE" => Ok(TransportationMode::InVehicle),
            "UNKNOWN" => Ok(TransportationMode::Unknown),
            _ => Err(format!(
                "Invalid transportation mode: {}. Must be one of: STATIONARY, WALKING, RUNNING, CYCLING, IN_VEHICLE, UNKNOWN",
                s
            )),
        }
    }
}

/// Source of transportation mode detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DetectionSource {
    ActivityRecognition,
    BluetoothCar,
    AndroidAuto,
    Multiple,
    None,
}

impl DetectionSource {
    /// Returns the string representation for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            DetectionSource::ActivityRecognition => "ACTIVITY_RECOGNITION",
            DetectionSource::BluetoothCar => "BLUETOOTH_CAR",
            DetectionSource::AndroidAuto => "ANDROID_AUTO",
            DetectionSource::Multiple => "MULTIPLE",
            DetectionSource::None => "NONE",
        }
    }
}

impl fmt::Display for DetectionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for DetectionSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACTIVITY_RECOGNITION" => Ok(DetectionSource::ActivityRecognition),
            "BLUETOOTH_CAR" => Ok(DetectionSource::BluetoothCar),
            "ANDROID_AUTO" => Ok(DetectionSource::AndroidAuto),
            "MULTIPLE" => Ok(DetectionSource::Multiple),
            "NONE" => Ok(DetectionSource::None),
            _ => Err(format!(
                "Invalid detection source: {}. Must be one of: ACTIVITY_RECOGNITION, BLUETOOTH_CAR, ANDROID_AUTO, MULTIPLE, NONE",
                s
            )),
        }
    }
}

// ============================================================================
// Core Model
// ============================================================================

/// Represents a movement event record in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovementEvent {
    pub id: Uuid,
    pub device_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<Uuid>,
    pub timestamp: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altitude: Option<f64>,
    pub transportation_mode: TransportationMode,
    pub confidence: f64,
    pub detection_source: DetectionSource,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Request DTOs
// ============================================================================

/// Request payload for single movement event upload.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateMovementEventRequest {
    pub device_id: Uuid,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<Uuid>,

    /// Timestamp in milliseconds since epoch
    #[validate(custom(function = "shared::validation::validate_timestamp"))]
    pub timestamp: i64,

    #[validate(custom(function = "shared::validation::validate_latitude"))]
    pub latitude: f64,

    #[validate(custom(function = "shared::validation::validate_longitude"))]
    pub longitude: f64,

    #[validate(custom(function = "shared::validation::validate_accuracy"))]
    pub accuracy: f64,

    #[validate(custom(function = "shared::validation::validate_speed"))]
    pub speed: Option<f64>,

    #[validate(custom(function = "shared::validation::validate_bearing"))]
    pub bearing: Option<f64>,

    pub altitude: Option<f64>,

    pub transportation_mode: TransportationMode,

    #[validate(custom(function = "crate::models::movement_event::validate_confidence"))]
    pub confidence: f64,

    pub detection_source: DetectionSource,
}

/// Validates that confidence is within valid range (0.0 to 1.0).
pub fn validate_confidence(confidence: f64) -> Result<(), validator::ValidationError> {
    if (0.0..=1.0).contains(&confidence) {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("confidence_range");
        err.message = Some("Confidence must be between 0.0 and 1.0".into());
        Err(err)
    }
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Response payload for movement event creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMovementEventResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Response payload for movement event retrieval.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovementEventResponse {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<Uuid>,
    pub timestamp: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altitude: Option<f64>,
    pub transportation_mode: TransportationMode,
    pub confidence: f64,
    pub detection_source: DetectionSource,
    pub created_at: DateTime<Utc>,
}

impl From<MovementEvent> for MovementEventResponse {
    fn from(event: MovementEvent) -> Self {
        Self {
            id: event.id,
            trip_id: event.trip_id,
            timestamp: event.timestamp,
            latitude: event.latitude,
            longitude: event.longitude,
            accuracy: event.accuracy,
            speed: event.speed,
            bearing: event.bearing,
            altitude: event.altitude,
            transportation_mode: event.transportation_mode,
            confidence: event.confidence,
            detection_source: event.detection_source,
            created_at: event.created_at,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    fn current_timestamp_millis() -> i64 {
        Utc::now().timestamp_millis()
    }

    // =========================================================================
    // TransportationMode Tests
    // =========================================================================

    #[test]
    fn test_transportation_mode_as_str() {
        assert_eq!(TransportationMode::Stationary.as_str(), "STATIONARY");
        assert_eq!(TransportationMode::Walking.as_str(), "WALKING");
        assert_eq!(TransportationMode::Running.as_str(), "RUNNING");
        assert_eq!(TransportationMode::Cycling.as_str(), "CYCLING");
        assert_eq!(TransportationMode::InVehicle.as_str(), "IN_VEHICLE");
        assert_eq!(TransportationMode::Unknown.as_str(), "UNKNOWN");
    }

    #[test]
    fn test_transportation_mode_from_str() {
        assert_eq!(
            "STATIONARY".parse::<TransportationMode>().unwrap(),
            TransportationMode::Stationary
        );
        assert_eq!(
            "WALKING".parse::<TransportationMode>().unwrap(),
            TransportationMode::Walking
        );
        assert_eq!(
            "RUNNING".parse::<TransportationMode>().unwrap(),
            TransportationMode::Running
        );
        assert_eq!(
            "CYCLING".parse::<TransportationMode>().unwrap(),
            TransportationMode::Cycling
        );
        assert_eq!(
            "IN_VEHICLE".parse::<TransportationMode>().unwrap(),
            TransportationMode::InVehicle
        );
        assert_eq!(
            "UNKNOWN".parse::<TransportationMode>().unwrap(),
            TransportationMode::Unknown
        );
    }

    #[test]
    fn test_transportation_mode_from_str_invalid() {
        assert!("invalid".parse::<TransportationMode>().is_err());
        assert!("walking".parse::<TransportationMode>().is_err()); // lowercase
    }

    #[test]
    fn test_transportation_mode_display() {
        assert_eq!(format!("{}", TransportationMode::InVehicle), "IN_VEHICLE");
    }

    #[test]
    fn test_transportation_mode_serde() {
        let mode = TransportationMode::Walking;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"WALKING\"");

        let parsed: TransportationMode = serde_json::from_str("\"CYCLING\"").unwrap();
        assert_eq!(parsed, TransportationMode::Cycling);
    }

    // =========================================================================
    // DetectionSource Tests
    // =========================================================================

    #[test]
    fn test_detection_source_as_str() {
        assert_eq!(
            DetectionSource::ActivityRecognition.as_str(),
            "ACTIVITY_RECOGNITION"
        );
        assert_eq!(DetectionSource::BluetoothCar.as_str(), "BLUETOOTH_CAR");
        assert_eq!(DetectionSource::AndroidAuto.as_str(), "ANDROID_AUTO");
        assert_eq!(DetectionSource::Multiple.as_str(), "MULTIPLE");
        assert_eq!(DetectionSource::None.as_str(), "NONE");
    }

    #[test]
    fn test_detection_source_from_str() {
        assert_eq!(
            "ACTIVITY_RECOGNITION".parse::<DetectionSource>().unwrap(),
            DetectionSource::ActivityRecognition
        );
        assert_eq!(
            "BLUETOOTH_CAR".parse::<DetectionSource>().unwrap(),
            DetectionSource::BluetoothCar
        );
        assert_eq!(
            "ANDROID_AUTO".parse::<DetectionSource>().unwrap(),
            DetectionSource::AndroidAuto
        );
        assert_eq!(
            "MULTIPLE".parse::<DetectionSource>().unwrap(),
            DetectionSource::Multiple
        );
        assert_eq!(
            "NONE".parse::<DetectionSource>().unwrap(),
            DetectionSource::None
        );
    }

    #[test]
    fn test_detection_source_from_str_invalid() {
        assert!("invalid".parse::<DetectionSource>().is_err());
    }

    #[test]
    fn test_detection_source_serde() {
        let source = DetectionSource::ActivityRecognition;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"ACTIVITY_RECOGNITION\"");

        let parsed: DetectionSource = serde_json::from_str("\"BLUETOOTH_CAR\"").unwrap();
        assert_eq!(parsed, DetectionSource::BluetoothCar);
    }

    // =========================================================================
    // Confidence Validation Tests
    // =========================================================================

    #[test]
    fn test_validate_confidence_valid() {
        assert!(validate_confidence(0.0).is_ok());
        assert!(validate_confidence(0.5).is_ok());
        assert!(validate_confidence(1.0).is_ok());
        assert!(validate_confidence(0.95).is_ok());
    }

    #[test]
    fn test_validate_confidence_invalid() {
        assert!(validate_confidence(-0.1).is_err());
        assert!(validate_confidence(1.1).is_err());
        assert!(validate_confidence(2.0).is_err());
    }

    #[test]
    fn test_validate_confidence_error_message() {
        let err = validate_confidence(1.5).unwrap_err();
        assert_eq!(
            err.message.unwrap().to_string(),
            "Confidence must be between 0.0 and 1.0"
        );
    }

    // =========================================================================
    // CreateMovementEventRequest Validation Tests
    // =========================================================================

    #[test]
    fn test_create_movement_event_request_valid() {
        let request = CreateMovementEventRequest {
            device_id: Uuid::new_v4(),
            trip_id: None,
            timestamp: current_timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            speed: Some(5.5),
            bearing: Some(180.0),
            altitude: Some(100.0),
            transportation_mode: TransportationMode::Walking,
            confidence: 0.95,
            detection_source: DetectionSource::ActivityRecognition,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_movement_event_request_minimal() {
        let request = CreateMovementEventRequest {
            device_id: Uuid::new_v4(),
            trip_id: None,
            timestamp: current_timestamp_millis(),
            latitude: 0.0,
            longitude: 0.0,
            accuracy: 0.0,
            speed: None,
            bearing: None,
            altitude: None,
            transportation_mode: TransportationMode::Unknown,
            confidence: 0.0,
            detection_source: DetectionSource::None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_movement_event_request_invalid_latitude() {
        let request = CreateMovementEventRequest {
            device_id: Uuid::new_v4(),
            trip_id: None,
            timestamp: current_timestamp_millis(),
            latitude: 100.0, // Invalid
            longitude: -120.0,
            accuracy: 10.0,
            speed: None,
            bearing: None,
            altitude: None,
            transportation_mode: TransportationMode::Walking,
            confidence: 0.95,
            detection_source: DetectionSource::ActivityRecognition,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_movement_event_request_invalid_confidence() {
        let request = CreateMovementEventRequest {
            device_id: Uuid::new_v4(),
            trip_id: None,
            timestamp: current_timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            speed: None,
            bearing: None,
            altitude: None,
            transportation_mode: TransportationMode::Walking,
            confidence: 1.5, // Invalid
            detection_source: DetectionSource::ActivityRecognition,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_movement_event_request_invalid_bearing() {
        let request = CreateMovementEventRequest {
            device_id: Uuid::new_v4(),
            trip_id: None,
            timestamp: current_timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            speed: None,
            bearing: Some(400.0), // Invalid
            altitude: None,
            transportation_mode: TransportationMode::Walking,
            confidence: 0.95,
            detection_source: DetectionSource::ActivityRecognition,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_movement_event_request_with_trip_id() {
        let request = CreateMovementEventRequest {
            device_id: Uuid::new_v4(),
            trip_id: Some(Uuid::new_v4()),
            timestamp: current_timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            speed: None,
            bearing: None,
            altitude: None,
            transportation_mode: TransportationMode::InVehicle,
            confidence: 0.85,
            detection_source: DetectionSource::BluetoothCar,
        };
        assert!(request.validate().is_ok());
    }

    // =========================================================================
    // Boundary Value Tests
    // =========================================================================

    #[test]
    fn test_boundary_coordinates() {
        let request = CreateMovementEventRequest {
            device_id: Uuid::new_v4(),
            trip_id: None,
            timestamp: current_timestamp_millis(),
            latitude: 90.0,   // Max
            longitude: 180.0, // Max
            accuracy: 0.0,    // Min
            speed: Some(0.0), // Min
            bearing: Some(360.0), // Max
            altitude: None,
            transportation_mode: TransportationMode::Stationary,
            confidence: 1.0, // Max
            detection_source: DetectionSource::None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boundary_coordinates_negative() {
        let request = CreateMovementEventRequest {
            device_id: Uuid::new_v4(),
            trip_id: None,
            timestamp: current_timestamp_millis(),
            latitude: -90.0,   // Min
            longitude: -180.0, // Min
            accuracy: 1000.0,
            speed: None,
            bearing: Some(0.0), // Min
            altitude: Some(-400.0), // Below sea level
            transportation_mode: TransportationMode::Walking,
            confidence: 0.0, // Min
            detection_source: DetectionSource::ActivityRecognition,
        };
        assert!(request.validate().is_ok());
    }

    // =========================================================================
    // MovementEventResponse Tests
    // =========================================================================

    #[test]
    fn test_movement_event_to_response() {
        let event = MovementEvent {
            id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            trip_id: Some(Uuid::new_v4()),
            timestamp: current_timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            speed: Some(5.5),
            bearing: Some(180.0),
            altitude: Some(100.0),
            transportation_mode: TransportationMode::Walking,
            confidence: 0.95,
            detection_source: DetectionSource::ActivityRecognition,
            created_at: Utc::now(),
        };

        let response: MovementEventResponse = event.clone().into();

        assert_eq!(response.id, event.id);
        assert_eq!(response.trip_id, event.trip_id);
        assert_eq!(response.timestamp, event.timestamp);
        assert_eq!(response.latitude, event.latitude);
        assert_eq!(response.longitude, event.longitude);
        assert_eq!(response.accuracy, event.accuracy);
        assert_eq!(response.speed, event.speed);
        assert_eq!(response.bearing, event.bearing);
        assert_eq!(response.altitude, event.altitude);
        assert_eq!(response.transportation_mode, event.transportation_mode);
        assert_eq!(response.confidence, event.confidence);
        assert_eq!(response.detection_source, event.detection_source);
    }

    #[test]
    fn test_response_serialization() {
        let response = CreateMovementEventResponse {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("550e8400-e29b-41d4-a716-446655440000"));
        assert!(json.contains("createdAt"));
    }
}
