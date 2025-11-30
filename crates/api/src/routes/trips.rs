//! Trip endpoint handlers.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use persistence::repositories::{DeviceRepository, TripInput, TripRepository, TripUpdateInput};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::movement_event::{DetectionSource, TransportationMode};
use domain::models::trip::{
    CreateTripRequest, CreateTripResponse, TripResponse, TripState, UpdateTripRequest,
};

/// Create a new trip with idempotency support.
///
/// POST /api/v1/trips
///
/// Returns 200 if trip already exists (idempotent), 201 if created new.
/// Returns 404 if device not found.
/// Returns 409 if device already has an ACTIVE trip with different localTripId.
pub async fn create_trip(
    State(state): State<AppState>,
    Json(request): Json<CreateTripRequest>,
) -> Result<(StatusCode, Json<CreateTripResponse>), ApiError> {
    // Validate the request
    request.validate().map_err(|e| {
        let errors: Vec<String> = e
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |err| {
                    format!(
                        "{}: {}",
                        field,
                        err.message.as_ref().unwrap_or(&"".into())
                    )
                })
            })
            .collect();
        ApiError::Validation(errors.join(", "))
    })?;

    // Verify device exists and is active
    let device_repo = DeviceRepository::new(state.pool.clone());
    let device = device_repo
        .find_by_device_id(request.device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found. Please register first.".to_string()))?;

    if !device.active {
        return Err(ApiError::NotFound(
            "Device not found. Please register first.".to_string(),
        ));
    }

    let trip_repo = TripRepository::new(state.pool.clone());

    // Check for existing ACTIVE trip with different local_trip_id (conflict)
    if let Some(existing_active) = trip_repo.find_active_for_device(request.device_id).await? {
        if existing_active.local_trip_id != request.local_trip_id {
            return Err(ApiError::Conflict(format!(
                "Device already has an active trip with localTripId: {}. Complete or cancel it first.",
                existing_active.local_trip_id
            )));
        }
    }

    // Create input for repository
    let input = TripInput {
        device_id: request.device_id,
        local_trip_id: request.local_trip_id.clone(),
        start_timestamp: request.start_timestamp,
        start_latitude: request.start_latitude,
        start_longitude: request.start_longitude,
        transportation_mode: request.transportation_mode.as_str().to_string(),
        detection_source: request.detection_source.as_str().to_string(),
    };

    // Insert or retrieve existing trip (idempotent)
    let (entity, was_created) = trip_repo.create_trip(input).await?;

    // Build response
    let response = CreateTripResponse {
        id: entity.id,
        local_trip_id: entity.local_trip_id,
        state: entity
            .state
            .parse::<TripState>()
            .unwrap_or(TripState::Active),
        start_timestamp: entity.start_timestamp,
        created_at: entity.created_at,
    };

    let status = if was_created {
        info!(
            device_id = %request.device_id,
            trip_id = %entity.id,
            local_trip_id = %response.local_trip_id,
            mode = %request.transportation_mode,
            "Trip created"
        );
        StatusCode::CREATED
    } else {
        info!(
            device_id = %request.device_id,
            trip_id = %entity.id,
            local_trip_id = %response.local_trip_id,
            "Trip already exists (idempotent)"
        );
        StatusCode::OK
    };

    Ok((status, Json(response)))
}

/// Update trip state (COMPLETED or CANCELLED).
///
/// PATCH /api/v1/trips/:tripId
///
/// State transitions:
/// - ACTIVE → COMPLETED (requires end location)
/// - ACTIVE → CANCELLED (end location optional)
///
/// Returns 200 with updated trip data.
/// Returns 400 for invalid state transition.
/// Returns 404 if trip not found.
pub async fn update_trip_state(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
    Json(request): Json<UpdateTripRequest>,
) -> Result<Json<TripResponse>, ApiError> {
    // Validate the request
    request.validate().map_err(|e| {
        let errors: Vec<String> = e
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |err| {
                    format!(
                        "{}: {}",
                        field,
                        err.message.as_ref().unwrap_or(&"".into())
                    )
                })
            })
            .collect();
        ApiError::Validation(errors.join(", "))
    })?;

    let trip_repo = TripRepository::new(state.pool.clone());

    // Find the trip
    let trip = trip_repo
        .find_by_id(trip_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Trip not found".to_string()))?;

    // Parse current state
    let current_state = trip
        .state
        .parse::<TripState>()
        .map_err(|_| ApiError::Internal("Invalid trip state in database".to_string()))?;

    // Validate state transition
    if !current_state.can_transition_to(request.state) {
        return Err(ApiError::Validation(format!(
            "Invalid state transition from {} to {}",
            current_state, request.state
        )));
    }

    // For COMPLETED state, require end location
    if request.state == TripState::Completed {
        if request.end_timestamp.is_none() {
            return Err(ApiError::Validation(
                "endTimestamp is required for COMPLETED state".to_string(),
            ));
        }
        if request.end_latitude.is_none() {
            return Err(ApiError::Validation(
                "endLatitude is required for COMPLETED state".to_string(),
            ));
        }
        if request.end_longitude.is_none() {
            return Err(ApiError::Validation(
                "endLongitude is required for COMPLETED state".to_string(),
            ));
        }
    }

    // Create update input
    let update_input = TripUpdateInput {
        state: request.state.as_str().to_string(),
        end_timestamp: request.end_timestamp,
        end_latitude: request.end_latitude,
        end_longitude: request.end_longitude,
    };

    // Update the trip
    let updated = trip_repo
        .update_state(trip_id, update_input)
        .await?
        .ok_or_else(|| ApiError::NotFound("Trip not found".to_string()))?;

    // Build response
    let response = TripResponse {
        id: updated.id,
        local_trip_id: updated.local_trip_id,
        state: updated
            .state
            .parse::<TripState>()
            .unwrap_or(TripState::Active),
        start_timestamp: updated.start_timestamp,
        end_timestamp: updated.end_timestamp,
        start_latitude: updated.start_latitude,
        start_longitude: updated.start_longitude,
        end_latitude: updated.end_latitude,
        end_longitude: updated.end_longitude,
        transportation_mode: updated
            .transportation_mode
            .parse::<TransportationMode>()
            .unwrap_or(TransportationMode::Unknown),
        detection_source: updated
            .detection_source
            .parse::<DetectionSource>()
            .unwrap_or(DetectionSource::None),
        distance_meters: updated.distance_meters,
        duration_seconds: updated.duration_seconds,
        created_at: updated.created_at,
    };

    info!(
        trip_id = %trip_id,
        old_state = %current_state,
        new_state = %request.state,
        "Trip state updated"
    );

    // TODO: Trigger async statistics calculation for COMPLETED trips (Story 6.4)

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::movement_event::{DetectionSource, TransportationMode};
    use uuid::Uuid;

    #[test]
    fn test_create_trip_request_serialization() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "localTripId": "trip-abc-123",
            "startTimestamp": 1234567890000,
            "startLatitude": 45.0,
            "startLongitude": -120.0,
            "transportationMode": "WALKING",
            "detectionSource": "ACTIVITY_RECOGNITION"
        }"#;

        let request: CreateTripRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            request.device_id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(request.local_trip_id, "trip-abc-123");
        assert_eq!(request.start_latitude, 45.0);
        assert_eq!(request.transportation_mode, TransportationMode::Walking);
        assert_eq!(
            request.detection_source,
            DetectionSource::ActivityRecognition
        );
    }

    #[test]
    fn test_create_trip_request_with_in_vehicle() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "localTripId": "drive-2024-001",
            "startTimestamp": 1234567890000,
            "startLatitude": 37.7749,
            "startLongitude": -122.4194,
            "transportationMode": "IN_VEHICLE",
            "detectionSource": "BLUETOOTH_CAR"
        }"#;

        let request: CreateTripRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.transportation_mode, TransportationMode::InVehicle);
        assert_eq!(request.detection_source, DetectionSource::BluetoothCar);
    }

    #[test]
    fn test_create_trip_response_serialization() {
        let response = CreateTripResponse {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            local_trip_id: "trip-123".to_string(),
            state: TripState::Active,
            start_timestamp: 1234567890000,
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("550e8400-e29b-41d4-a716-446655440000"));
        assert!(json.contains("trip-123"));
        assert!(json.contains("ACTIVE"));
        assert!(json.contains("createdAt"));
    }

    #[test]
    fn test_create_trip_response_with_completed_state() {
        let response = CreateTripResponse {
            id: Uuid::new_v4(),
            local_trip_id: "trip-completed".to_string(),
            state: TripState::Completed,
            start_timestamp: 1234567890000,
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("COMPLETED"));
    }

    #[test]
    fn test_update_trip_request_completed() {
        let json = r#"{
            "state": "COMPLETED",
            "endTimestamp": 1234567899000,
            "endLatitude": 45.5,
            "endLongitude": -120.5
        }"#;

        let request: UpdateTripRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.state, TripState::Completed);
        assert_eq!(request.end_timestamp, Some(1234567899000));
        assert_eq!(request.end_latitude, Some(45.5));
        assert_eq!(request.end_longitude, Some(-120.5));
    }

    #[test]
    fn test_update_trip_request_cancelled() {
        let json = r#"{
            "state": "CANCELLED"
        }"#;

        let request: UpdateTripRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.state, TripState::Cancelled);
        assert!(request.end_timestamp.is_none());
        assert!(request.end_latitude.is_none());
        assert!(request.end_longitude.is_none());
    }

    #[test]
    fn test_trip_response_serialization() {
        let response = TripResponse {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            local_trip_id: "trip-123".to_string(),
            state: TripState::Completed,
            start_timestamp: 1234567890000,
            end_timestamp: Some(1234567899000),
            start_latitude: 45.0,
            start_longitude: -120.0,
            end_latitude: Some(45.5),
            end_longitude: Some(-120.5),
            transportation_mode: TransportationMode::Walking,
            detection_source: DetectionSource::ActivityRecognition,
            distance_meters: Some(1500.0),
            duration_seconds: Some(9000),
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("COMPLETED"));
        assert!(json.contains("distanceMeters"));
        assert!(json.contains("durationSeconds"));
    }

    #[test]
    fn test_trip_response_without_optional_fields() {
        let response = TripResponse {
            id: Uuid::new_v4(),
            local_trip_id: "trip-active".to_string(),
            state: TripState::Active,
            start_timestamp: 1234567890000,
            end_timestamp: None,
            start_latitude: 45.0,
            start_longitude: -120.0,
            end_latitude: None,
            end_longitude: None,
            transportation_mode: TransportationMode::InVehicle,
            detection_source: DetectionSource::BluetoothCar,
            distance_meters: None,
            duration_seconds: None,
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("ACTIVE"));
        // Optional fields should not appear when None (skip_serializing_if)
        assert!(!json.contains("endTimestamp"));
        assert!(!json.contains("distanceMeters"));
    }
}
