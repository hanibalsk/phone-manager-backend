//! Trip endpoint handlers.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use persistence::repositories::{DeviceRepository, TripInput, TripRepository};
use tracing::info;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::trip::{CreateTripRequest, CreateTripResponse, TripState};

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
}
