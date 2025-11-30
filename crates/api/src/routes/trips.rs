//! Trip endpoint handlers.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use persistence::repositories::{
    DeviceRepository, MovementEventRepository, TripInput, TripQuery, TripRepository,
    TripUpdateInput,
};
use tracing::{error, info};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::movement_event::{DetectionSource, TransportationMode};
use domain::models::trip::{
    CreateTripRequest, CreateTripResponse, GetTripsQuery, GetTripsResponse, TripPagination,
    TripResponse, TripState, UpdateTripRequest,
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

    // Trigger async statistics calculation for COMPLETED trips
    if request.state == TripState::Completed {
        let pool = state.pool.clone();
        let start_ts = updated.start_timestamp;
        let end_ts = updated.end_timestamp;
        tokio::spawn(async move {
            calculate_trip_statistics(pool, trip_id, start_ts, end_ts).await;
        });
    }

    Ok(Json(response))
}

/// Get trips for a device with pagination.
///
/// GET /api/v1/devices/:deviceId/trips
///
/// Supports filtering by state, date range, and cursor-based pagination.
/// Returns 404 if device not found.
pub async fn get_device_trips(
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
    Query(query): Query<GetTripsQuery>,
) -> Result<Json<GetTripsResponse>, ApiError> {
    // Verify device exists and is active
    let device_repo = DeviceRepository::new(state.pool.clone());
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    if !device.active {
        return Err(ApiError::NotFound("Device not found".to_string()));
    }

    // Parse and validate limit (1-50, default 20)
    let limit = query.limit.unwrap_or(20).clamp(1, 50);

    // Parse cursor if provided (format: base64(timestamp:uuid))
    let (cursor_timestamp, cursor_id) = if let Some(ref cursor) = query.cursor {
        parse_cursor(cursor)?
    } else {
        (None, None)
    };

    // Validate state filter if provided
    if let Some(ref state_str) = query.state {
        if state_str.parse::<TripState>().is_err() {
            return Err(ApiError::Validation(format!(
                "Invalid state filter: {}. Must be one of: ACTIVE, COMPLETED, CANCELLED",
                state_str
            )));
        }
    }

    // Build query
    let trip_query = TripQuery {
        device_id,
        cursor_timestamp,
        cursor_id,
        from_timestamp: query.from,
        to_timestamp: query.to,
        state_filter: query.state.clone(),
        limit,
    };

    // Execute query
    let trip_repo = TripRepository::new(state.pool.clone());
    let (trips, has_more) = trip_repo.get_trips_by_device(trip_query).await?;

    // Build next cursor from last result
    let next_cursor = if has_more && !trips.is_empty() {
        let last = trips.last().unwrap();
        Some(encode_cursor(last.start_timestamp, last.id))
    } else {
        None
    };

    // Convert to response format
    let trip_responses: Vec<TripResponse> = trips
        .into_iter()
        .map(|entity| TripResponse {
            id: entity.id,
            local_trip_id: entity.local_trip_id,
            state: entity
                .state
                .parse::<TripState>()
                .unwrap_or(TripState::Active),
            start_timestamp: entity.start_timestamp,
            end_timestamp: entity.end_timestamp,
            start_latitude: entity.start_latitude,
            start_longitude: entity.start_longitude,
            end_latitude: entity.end_latitude,
            end_longitude: entity.end_longitude,
            transportation_mode: entity
                .transportation_mode
                .parse::<TransportationMode>()
                .unwrap_or(TransportationMode::Unknown),
            detection_source: entity
                .detection_source
                .parse::<DetectionSource>()
                .unwrap_or(DetectionSource::None),
            distance_meters: entity.distance_meters,
            duration_seconds: entity.duration_seconds,
            created_at: entity.created_at,
        })
        .collect();

    info!(
        device_id = %device_id,
        count = trip_responses.len(),
        has_more = has_more,
        "Retrieved device trips"
    );

    Ok(Json(GetTripsResponse {
        trips: trip_responses,
        pagination: TripPagination {
            next_cursor,
            has_more,
        },
    }))
}

/// Parse cursor from base64-encoded "timestamp:uuid" format.
fn parse_cursor(cursor: &str) -> Result<(Option<i64>, Option<Uuid>), ApiError> {
    let decoded = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| ApiError::Validation("Invalid cursor format".to_string()))?;

    let cursor_str = String::from_utf8(decoded)
        .map_err(|_| ApiError::Validation("Invalid cursor format".to_string()))?;

    let parts: Vec<&str> = cursor_str.split(':').collect();
    if parts.len() != 2 {
        return Err(ApiError::Validation("Invalid cursor format".to_string()));
    }

    let timestamp = parts[0]
        .parse::<i64>()
        .map_err(|_| ApiError::Validation("Invalid cursor timestamp".to_string()))?;

    let uuid = Uuid::parse_str(parts[1])
        .map_err(|_| ApiError::Validation("Invalid cursor UUID".to_string()))?;

    Ok((Some(timestamp), Some(uuid)))
}

/// Encode cursor as base64("timestamp:uuid").
fn encode_cursor(timestamp: i64, id: Uuid) -> String {
    URL_SAFE_NO_PAD.encode(format!("{}:{}", timestamp, id))
}

/// Calculate and store trip statistics asynchronously.
///
/// Calculates distance using PostGIS ST_Distance on movement events
/// and duration from timestamps. Updates the trips table with results.
/// Errors are logged but don't affect the trip state.
async fn calculate_trip_statistics(
    pool: sqlx::PgPool,
    trip_id: Uuid,
    start_timestamp: i64,
    end_timestamp: Option<i64>,
) {
    info!(trip_id = %trip_id, "Starting trip statistics calculation");

    // Calculate duration (if end_timestamp exists)
    let duration_seconds = end_timestamp.map(|end| {
        // Timestamps are in milliseconds, convert to seconds
        (end - start_timestamp) / 1000
    });

    // Calculate distance using PostGIS
    let event_repo = MovementEventRepository::new(pool.clone());
    let distance_result = event_repo.calculate_trip_distance(trip_id).await;

    match distance_result {
        Ok(distance_meters) => {
            // Update trip with calculated statistics
            let trip_repo = TripRepository::new(pool);
            match trip_repo
                .update_statistics(trip_id, distance_meters, duration_seconds.unwrap_or(0))
                .await
            {
                Ok(()) => {
                    info!(
                        trip_id = %trip_id,
                        distance_meters = distance_meters,
                        duration_seconds = duration_seconds,
                        "Trip statistics calculated and stored"
                    );
                }
                Err(e) => {
                    error!(
                        trip_id = %trip_id,
                        error = %e,
                        "Failed to update trip statistics"
                    );
                }
            }
        }
        Err(e) => {
            error!(
                trip_id = %trip_id,
                error = %e,
                "Failed to calculate trip distance"
            );
        }
    }
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

    #[test]
    fn test_get_trips_query_deserialization() {
        use domain::models::trip::GetTripsQuery;

        let json = r#"{
            "cursor": "MTIzNDU2Nzg5MDAwMDo1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDA",
            "limit": 25,
            "state": "COMPLETED",
            "from": 1234567890000,
            "to": 1234567999000
        }"#;

        let query: GetTripsQuery = serde_json::from_str(json).unwrap();
        assert!(query.cursor.is_some());
        assert_eq!(query.limit, Some(25));
        assert_eq!(query.state, Some("COMPLETED".to_string()));
        assert_eq!(query.from, Some(1234567890000));
        assert_eq!(query.to, Some(1234567999000));
    }

    #[test]
    fn test_get_trips_query_minimal() {
        use domain::models::trip::GetTripsQuery;

        let json = r#"{}"#;
        let query: GetTripsQuery = serde_json::from_str(json).unwrap();
        assert!(query.cursor.is_none());
        assert!(query.limit.is_none());
        assert!(query.state.is_none());
        assert!(query.from.is_none());
        assert!(query.to.is_none());
    }

    #[test]
    fn test_cursor_encode_decode() {
        let timestamp = 1234567890000i64;
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let encoded = encode_cursor(timestamp, id);
        let (parsed_ts, parsed_id) = parse_cursor(&encoded).unwrap();

        assert_eq!(parsed_ts, Some(timestamp));
        assert_eq!(parsed_id, Some(id));
    }

    #[test]
    fn test_cursor_decode_invalid() {
        // Invalid base64
        assert!(parse_cursor("!!!invalid!!!").is_err());

        // Valid base64 but wrong format
        let invalid_format = URL_SAFE_NO_PAD.encode("invalid");
        assert!(parse_cursor(&invalid_format).is_err());

        // Valid format but invalid timestamp
        let invalid_ts = URL_SAFE_NO_PAD.encode("abc:550e8400-e29b-41d4-a716-446655440000");
        assert!(parse_cursor(&invalid_ts).is_err());

        // Valid format but invalid UUID
        let invalid_uuid = URL_SAFE_NO_PAD.encode("1234567890000:invalid-uuid");
        assert!(parse_cursor(&invalid_uuid).is_err());
    }

    #[test]
    fn test_get_trips_response_serialization() {
        use domain::models::trip::{GetTripsResponse, TripPagination};

        let response = GetTripsResponse {
            trips: vec![TripResponse {
                id: Uuid::new_v4(),
                local_trip_id: "trip-1".to_string(),
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
            }],
            pagination: TripPagination {
                next_cursor: Some("cursor123".to_string()),
                has_more: true,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"trips\""));
        assert!(json.contains("\"pagination\""));
        assert!(json.contains("\"nextCursor\""));
        assert!(json.contains("\"hasMore\":true"));
    }
}
