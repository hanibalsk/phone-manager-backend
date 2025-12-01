//! Movement event endpoint handlers.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use persistence::repositories::{
    DeviceRepository, MovementEventInput, MovementEventQuery, MovementEventRepository,
    TripRepository,
};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::movement_event::{
    BatchMovementEventRequest, BatchMovementEventResponse, CreateMovementEventRequest,
    CreateMovementEventResponse, DetectionSource, GetMovementEventsResponse,
    MovementEventPagination, MovementEventResponse, TransportationMode,
};

/// Query parameters for GET /api/v1/devices/:deviceId/movement-events
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GetMovementEventsQuery {
    /// Pagination cursor (format: "timestamp_id")
    pub cursor: Option<String>,
    /// Number of results (1-100, default 50)
    #[serde(default = "default_limit")]
    pub limit: i32,
    /// Start timestamp filter (milliseconds)
    pub from: Option<i64>,
    /// End timestamp filter (milliseconds)
    pub to: Option<i64>,
    /// Sort order (asc or desc, default desc)
    #[serde(default = "default_order")]
    pub order: String,
}

fn default_limit() -> i32 {
    50
}

fn default_order() -> String {
    "desc".to_string()
}

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
                    format!("{}: {}", field, err.message.as_ref().unwrap_or(&"".into()))
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
        .ok_or_else(|| {
            ApiError::NotFound("Device not found. Please register first.".to_string())
        })?;

    if !device.active {
        return Err(ApiError::NotFound(
            "Device not found. Please register first.".to_string(),
        ));
    }

    // Validate trip_id if provided - must exist and belong to this device
    if let Some(trip_id) = request.trip_id {
        let trip_repo = TripRepository::new(state.pool.clone());
        let trip = trip_repo
            .find_by_id(trip_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Trip {} not found", trip_id)))?;

        if trip.device_id != request.device_id {
            return Err(ApiError::Forbidden(
                "Trip does not belong to this device".to_string(),
            ));
        }
    }

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
                    format!("{}: {}", field, err.message.as_ref().unwrap_or(&"".into()))
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
        .ok_or_else(|| {
            ApiError::NotFound("Device not found. Please register first.".to_string())
        })?;

    if !device.active {
        return Err(ApiError::NotFound(
            "Device not found. Please register first.".to_string(),
        ));
    }

    // Validate all trip_ids in the batch - collect unique ones for efficiency
    let unique_trip_ids: std::collections::HashSet<Uuid> =
        request.events.iter().filter_map(|e| e.trip_id).collect();

    if !unique_trip_ids.is_empty() {
        let trip_repo = TripRepository::new(state.pool.clone());
        for trip_id in unique_trip_ids {
            let trip = trip_repo
                .find_by_id(trip_id)
                .await?
                .ok_or_else(|| ApiError::NotFound(format!("Trip {} not found", trip_id)))?;

            if trip.device_id != request.device_id {
                return Err(ApiError::Forbidden(format!(
                    "Trip {} does not belong to this device",
                    trip_id
                )));
            }
        }
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

/// Get movement events for a device with pagination.
///
/// GET /api/v1/devices/:deviceId/movement-events
pub async fn get_device_movement_events(
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
    Query(query): Query<GetMovementEventsQuery>,
) -> Result<Json<GetMovementEventsResponse>, ApiError> {
    // Validate limit
    let limit = query.limit.clamp(1, 100);

    // Validate order
    let ascending = match query.order.to_lowercase().as_str() {
        "asc" => true,
        "desc" => false,
        _ => {
            return Err(ApiError::Validation(
                "order must be 'asc' or 'desc'".to_string(),
            ))
        }
    };

    // Verify device exists and is active
    let device_repo = DeviceRepository::new(state.pool.clone());
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    if !device.active {
        return Err(ApiError::NotFound("Device not found".to_string()));
    }

    // Parse cursor if provided
    let (cursor_timestamp, cursor_id) = if let Some(cursor) = &query.cursor {
        parse_cursor(cursor)?
    } else {
        (None, None)
    };

    // Build repository query
    let repo_query = MovementEventQuery {
        device_id,
        cursor_timestamp,
        cursor_id,
        from_timestamp: query.from,
        to_timestamp: query.to,
        limit,
        ascending,
    };

    // Fetch events
    let event_repo = MovementEventRepository::new(state.pool.clone());
    let (entities, has_more) = event_repo.get_events_by_device(repo_query).await?;

    // Convert entities to response DTOs
    let events: Vec<MovementEventResponse> = entities
        .iter()
        .map(|e| MovementEventResponse {
            id: e.id,
            trip_id: e.trip_id,
            timestamp: e.timestamp,
            latitude: e.latitude,
            longitude: e.longitude,
            accuracy: e.accuracy as f64,
            speed: e.speed.map(|s| s as f64),
            bearing: e.bearing.map(|b| b as f64),
            altitude: e.altitude,
            transportation_mode: e
                .transportation_mode
                .parse::<TransportationMode>()
                .unwrap_or(TransportationMode::Unknown),
            confidence: e.confidence as f64,
            detection_source: e
                .detection_source
                .parse::<DetectionSource>()
                .unwrap_or(DetectionSource::None),
            created_at: e.created_at,
        })
        .collect();

    // Build next cursor if there are more results
    let next_cursor = if has_more && !events.is_empty() {
        let last = events.last().unwrap();
        Some(format!("{}_{}", last.timestamp, last.id))
    } else {
        None
    };

    info!(
        device_id = %device_id,
        event_count = events.len(),
        has_more = has_more,
        "Movement events retrieved"
    );

    Ok(Json(GetMovementEventsResponse {
        events,
        pagination: MovementEventPagination {
            next_cursor,
            has_more,
        },
    }))
}

/// Parse cursor string into timestamp and UUID components.
fn parse_cursor(cursor: &str) -> Result<(Option<i64>, Option<Uuid>), ApiError> {
    let parts: Vec<&str> = cursor.splitn(2, '_').collect();
    if parts.len() != 2 {
        return Err(ApiError::Validation(
            "Invalid cursor format. Expected 'timestamp_uuid'".to_string(),
        ));
    }

    let timestamp = parts[0]
        .parse::<i64>()
        .map_err(|_| ApiError::Validation("Invalid cursor timestamp".to_string()))?;

    let id = Uuid::parse_str(parts[1])
        .map_err(|_| ApiError::Validation("Invalid cursor UUID".to_string()))?;

    Ok((Some(timestamp), Some(id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::movement_event::{DetectionSource, TransportationMode};
    use uuid::Uuid;

    #[test]
    fn test_create_request_serialization() {
        let json = r#"{
            "device_id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": 1234567890000,
            "latitude": 45.0,
            "longitude": -120.0,
            "accuracy": 10.0,
            "transportation_mode": "WALKING",
            "confidence": 0.95,
            "detection_source": "ACTIVITY_RECOGNITION"
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
            "device_id": "550e8400-e29b-41d4-a716-446655440000",
            "trip_id": "660e8400-e29b-41d4-a716-446655440000",
            "timestamp": 1234567890000,
            "latitude": 45.0,
            "longitude": -120.0,
            "accuracy": 10.0,
            "speed": 5.5,
            "bearing": 180.0,
            "altitude": 100.0,
            "transportation_mode": "IN_VEHICLE",
            "confidence": 0.85,
            "detection_source": "BLUETOOTH_CAR"
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
        assert!(json.contains("created_at"));
    }

    #[test]
    fn test_batch_request_serialization() {
        let json = r#"{
            "device_id": "550e8400-e29b-41d4-a716-446655440000",
            "events": [
                {
                    "timestamp": 1234567890000,
                    "latitude": 45.0,
                    "longitude": -120.0,
                    "accuracy": 10.0,
                    "transportation_mode": "WALKING",
                    "confidence": 0.95,
                    "detection_source": "ACTIVITY_RECOGNITION"
                },
                {
                    "trip_id": "660e8400-e29b-41d4-a716-446655440000",
                    "timestamp": 1234567891000,
                    "latitude": 45.001,
                    "longitude": -120.001,
                    "accuracy": 8.0,
                    "speed": 1.5,
                    "transportation_mode": "WALKING",
                    "confidence": 0.90,
                    "detection_source": "ACTIVITY_RECOGNITION"
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
        assert!(json.contains("\"processed_count\":5"));
    }

    #[test]
    fn test_get_events_query_defaults() {
        let json = r#"{}"#;
        let query: GetMovementEventsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, 50);
        assert_eq!(query.order, "desc");
        assert!(query.cursor.is_none());
        assert!(query.from.is_none());
        assert!(query.to.is_none());
    }

    #[test]
    fn test_get_events_query_with_params() {
        let json = r#"{
            "cursor": "1234567890000_550e8400-e29b-41d4-a716-446655440000",
            "limit": 25,
            "from": 1234567800000,
            "to": 1234567900000,
            "order": "asc"
        }"#;
        let query: GetMovementEventsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, 25);
        assert_eq!(query.order, "asc");
        assert!(query.cursor.is_some());
        assert_eq!(query.from, Some(1234567800000));
        assert_eq!(query.to, Some(1234567900000));
    }

    #[test]
    fn test_parse_cursor_valid() {
        let cursor = "1234567890000_550e8400-e29b-41d4-a716-446655440000";
        let (ts, id) = parse_cursor(cursor).unwrap();
        assert_eq!(ts, Some(1234567890000));
        assert_eq!(
            id,
            Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap())
        );
    }

    #[test]
    fn test_parse_cursor_invalid_format() {
        let result = parse_cursor("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cursor_invalid_timestamp() {
        let result = parse_cursor("abc_550e8400-e29b-41d4-a716-446655440000");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cursor_invalid_uuid() {
        let result = parse_cursor("1234567890000_invalid-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_events_response_serialization() {
        let response = GetMovementEventsResponse {
            events: vec![],
            pagination: MovementEventPagination {
                next_cursor: Some("1234567890000_550e8400-e29b-41d4-a716-446655440000".to_string()),
                has_more: true,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"events\":[]"));
        assert!(json.contains("\"has_more\":true"));
        assert!(json.contains("\"next_cursor\""));
    }
}
