//! Movement event endpoint handlers.

use axum::{extract::State, http::StatusCode, Json};
use persistence::repositories::{DeviceRepository, MovementEventInput, MovementEventRepository};
use tracing::info;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::movement_event::{
    BatchMovementEventRequest, BatchMovementEventResponse, CreateMovementEventRequest,
    CreateMovementEventResponse,
};

/// Create a single movement event.
///
/// POST /api/v1/movement-events
pub async fn create_movement_event(
    State(state): State<AppState>,
    Json(request): Json<CreateMovementEventRequest>,
) -> Result<(StatusCode, Json<CreateMovementEventResponse>), ApiError> {
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

    // Note: trip_id validation will be added when trips table exists (Story 6.1)
    // For now, we accept any trip_id since the FK constraint doesn't exist yet

    // Create input for repository
    let input = MovementEventInput {
        device_id: request.device_id,
        trip_id: request.trip_id,
        timestamp: request.timestamp,
        latitude: request.latitude,
        longitude: request.longitude,
        accuracy: request.accuracy,
        speed: request.speed,
        bearing: request.bearing,
        altitude: request.altitude,
        transportation_mode: request.transportation_mode.as_str().to_string(),
        confidence: request.confidence,
        detection_source: request.detection_source.as_str().to_string(),
    };

    // Insert movement event
    let event_repo = MovementEventRepository::new(state.pool.clone());
    let entity = event_repo.insert_event(input).await?;

    // Build response
    let response = CreateMovementEventResponse {
        id: entity.id,
        created_at: entity.created_at,
    };

    info!(
        device_id = %request.device_id,
        event_id = %entity.id,
        mode = %request.transportation_mode,
        confidence = request.confidence,
        "Movement event created"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Create multiple movement events in batch.
///
/// POST /api/v1/movement-events/batch
pub async fn create_movement_events_batch(
    State(state): State<AppState>,
    Json(request): Json<BatchMovementEventRequest>,
) -> Result<(StatusCode, Json<BatchMovementEventResponse>), ApiError> {
    // Validate the request (including nested events validation)
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

    // Convert batch items to repository inputs
    let inputs: Vec<MovementEventInput> = request
        .events
        .iter()
        .map(|event| MovementEventInput {
            device_id: request.device_id,
            trip_id: event.trip_id,
            timestamp: event.timestamp,
            latitude: event.latitude,
            longitude: event.longitude,
            accuracy: event.accuracy,
            speed: event.speed,
            bearing: event.bearing,
            altitude: event.altitude,
            transportation_mode: event.transportation_mode.as_str().to_string(),
            confidence: event.confidence,
            detection_source: event.detection_source.as_str().to_string(),
        })
        .collect();

    let event_count = inputs.len();

    // Insert all events in a single transaction
    let event_repo = MovementEventRepository::new(state.pool.clone());
    let processed_count = event_repo.insert_events_batch(inputs).await?;

    info!(
        device_id = %request.device_id,
        event_count = event_count,
        processed_count = processed_count,
        "Movement events batch created"
    );

    Ok((
        StatusCode::OK,
        Json(BatchMovementEventResponse {
            success: true,
            processed_count,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::movement_event::{DetectionSource, TransportationMode};
    use uuid::Uuid;

    #[test]
    fn test_create_request_serialization() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": 1234567890000,
            "latitude": 45.0,
            "longitude": -120.0,
            "accuracy": 10.0,
            "transportationMode": "WALKING",
            "confidence": 0.95,
            "detectionSource": "ACTIVITY_RECOGNITION"
        }"#;

        let request: CreateMovementEventRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            request.device_id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(request.latitude, 45.0);
        assert_eq!(request.transportation_mode, TransportationMode::Walking);
        assert_eq!(
            request.detection_source,
            DetectionSource::ActivityRecognition
        );
    }

    #[test]
    fn test_create_request_with_optional_fields() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "tripId": "660e8400-e29b-41d4-a716-446655440000",
            "timestamp": 1234567890000,
            "latitude": 45.0,
            "longitude": -120.0,
            "accuracy": 10.0,
            "speed": 5.5,
            "bearing": 180.0,
            "altitude": 100.0,
            "transportationMode": "IN_VEHICLE",
            "confidence": 0.85,
            "detectionSource": "BLUETOOTH_CAR"
        }"#;

        let request: CreateMovementEventRequest = serde_json::from_str(json).unwrap();
        assert!(request.trip_id.is_some());
        assert_eq!(request.speed, Some(5.5));
        assert_eq!(request.bearing, Some(180.0));
        assert_eq!(request.altitude, Some(100.0));
        assert_eq!(request.transportation_mode, TransportationMode::InVehicle);
        assert_eq!(request.detection_source, DetectionSource::BluetoothCar);
    }

    #[test]
    fn test_response_serialization() {
        let response = CreateMovementEventResponse {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("550e8400-e29b-41d4-a716-446655440000"));
        assert!(json.contains("createdAt"));
    }

    #[test]
    fn test_batch_request_serialization() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "events": [
                {
                    "timestamp": 1234567890000,
                    "latitude": 45.0,
                    "longitude": -120.0,
                    "accuracy": 10.0,
                    "transportationMode": "WALKING",
                    "confidence": 0.95,
                    "detectionSource": "ACTIVITY_RECOGNITION"
                },
                {
                    "tripId": "660e8400-e29b-41d4-a716-446655440000",
                    "timestamp": 1234567891000,
                    "latitude": 45.001,
                    "longitude": -120.001,
                    "accuracy": 8.0,
                    "speed": 1.5,
                    "transportationMode": "WALKING",
                    "confidence": 0.90,
                    "detectionSource": "ACTIVITY_RECOGNITION"
                }
            ]
        }"#;

        let request: BatchMovementEventRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            request.device_id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(request.events.len(), 2);
        assert!(request.events[0].trip_id.is_none());
        assert!(request.events[1].trip_id.is_some());
        assert_eq!(request.events[1].speed, Some(1.5));
    }

    #[test]
    fn test_batch_response_serialization() {
        let response = BatchMovementEventResponse {
            success: true,
            processed_count: 5,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"processedCount\":5"));
    }
}
