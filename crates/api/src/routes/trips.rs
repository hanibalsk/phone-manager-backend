//! Trip endpoint handlers.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use persistence::repositories::{
    DeviceRepository, MovementEventRepository, TripInput, TripPathCorrectionRepository, TripQuery,
    TripRepository, TripUpdateInput,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::services::PathCorrectionService;
use domain::models::movement_event::{DetectionSource, TransportationMode};
use domain::models::trip::{
    CreateTripRequest, CreateTripResponse, GetTripsQuery, GetTripsResponse, TripPagination,
    TripResponse, TripState, UpdateTripRequest,
};
use domain::models::trip_path_correction::{CorrectPathResponse, CorrectionStatus, TripPathResponse};

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

    // Trigger async statistics calculation and path correction for COMPLETED trips
    if request.state == TripState::Completed {
        let pool = state.pool.clone();
        let config = state.config.clone();
        let start_ts = updated.start_timestamp;
        let end_ts = updated.end_timestamp;
        tokio::spawn(async move {
            calculate_trip_statistics(pool.clone(), trip_id, start_ts, end_ts).await;
            correct_trip_path(pool, &config.map_matching, trip_id).await;
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

/// Get trip path correction data.
///
/// GET /api/v1/trips/:tripId/path
///
/// Returns the original and corrected paths for a trip along with correction status.
/// Returns 404 if trip not found or if no path correction record exists.
pub async fn get_trip_path(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<Json<TripPathResponse>, ApiError> {
    // Verify trip exists
    let trip_repo = TripRepository::new(state.pool.clone());
    let _trip = trip_repo
        .find_by_id(trip_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Trip not found".to_string()))?;

    // Get path correction record
    let correction_repo = TripPathCorrectionRepository::new(state.pool.clone());
    let correction = correction_repo
        .find_by_trip_id(trip_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound("Path correction not found for this trip".to_string())
        })?;

    // Parse original path from GeoJSON
    let original_coords = parse_geojson_linestring(&correction.original_path).map_err(|e| {
        error!(trip_id = %trip_id, error = %e, "Failed to parse original path GeoJSON");
        ApiError::Internal("Failed to parse path data".to_string())
    })?;

    // Parse corrected path if present
    let corrected_coords = if let Some(ref corrected_path) = correction.corrected_path {
        Some(parse_geojson_linestring(corrected_path).map_err(|e| {
            error!(trip_id = %trip_id, error = %e, "Failed to parse corrected path GeoJSON");
            ApiError::Internal("Failed to parse path data".to_string())
        })?)
    } else {
        None
    };

    // Parse correction status
    let status = correction
        .correction_status
        .parse::<CorrectionStatus>()
        .unwrap_or(CorrectionStatus::Pending);

    debug!(
        trip_id = %trip_id,
        status = %status,
        original_points = original_coords.len(),
        corrected_points = ?corrected_coords.as_ref().map(|c| c.len()),
        "Retrieved trip path correction"
    );

    Ok(Json(TripPathResponse {
        original_path: original_coords,
        corrected_path: corrected_coords,
        correction_status: status,
        correction_quality: correction.correction_quality,
    }))
}

/// Trigger on-demand path correction for a trip.
///
/// POST /api/v1/trips/:tripId/correct-path
///
/// Manually triggers path correction for a completed trip.
/// Can be used to retry failed corrections or correct older trips.
/// Returns 404 if trip not found.
/// Returns 400 if trip is not in COMPLETED state.
/// Returns 409 if correction is already in progress.
/// Returns 503 if map-matching service is unavailable.
pub async fn trigger_path_correction(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<Json<CorrectPathResponse>, ApiError> {
    // Verify trip exists
    let trip_repo = TripRepository::new(state.pool.clone());
    let trip = trip_repo
        .find_by_id(trip_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Trip not found".to_string()))?;

    // Verify trip is completed (only completed trips can have path correction)
    let trip_state = trip
        .state
        .parse::<TripState>()
        .map_err(|_| ApiError::Internal("Invalid trip state in database".to_string()))?;

    if trip_state != TripState::Completed {
        return Err(ApiError::Validation(format!(
            "Trip must be in COMPLETED state to trigger path correction. Current state: {}",
            trip_state
        )));
    }

    // Check if correction already exists
    let correction_repo = TripPathCorrectionRepository::new(state.pool.clone());
    if let Some(existing) = correction_repo.find_by_trip_id(trip_id).await? {
        let status = existing
            .correction_status
            .parse::<CorrectionStatus>()
            .unwrap_or(CorrectionStatus::Pending);

        match status {
            CorrectionStatus::Pending => {
                return Err(ApiError::Conflict(
                    "Path correction is already in progress".to_string(),
                ));
            }
            CorrectionStatus::Completed => {
                // Allow re-correction of completed paths (user explicitly requested)
                info!(
                    trip_id = %trip_id,
                    "Deleting existing COMPLETED correction for re-correction"
                );
                correction_repo.delete_by_trip_id(trip_id).await?;
            }
            CorrectionStatus::Failed | CorrectionStatus::Skipped => {
                // Allow retry of failed/skipped corrections
                info!(
                    trip_id = %trip_id,
                    status = %status,
                    "Deleting existing {} correction for retry",
                    status
                );
                correction_repo.delete_by_trip_id(trip_id).await?;
            }
        }
    }

    // Trigger path correction
    let service = PathCorrectionService::new(state.pool.clone(), &state.config.map_matching);

    match service.correct_trip_path(trip_id).await {
        Ok(result) => {
            info!(
                trip_id = %trip_id,
                status = %result.status,
                quality = ?result.quality,
                "On-demand path correction completed"
            );

            let message = match result.status.as_str() {
                "COMPLETED" => format!(
                    "Path correction completed with quality score: {:.2}",
                    result.quality.unwrap_or(0.0)
                ),
                "SKIPPED" => "Path correction skipped (insufficient data or service unavailable)"
                    .to_string(),
                "FAILED" => "Path correction failed (map-matching service error)".to_string(),
                _ => "Path correction processed".to_string(),
            };

            Ok(Json(CorrectPathResponse {
                status: result.status,
                message,
            }))
        }
        Err(e) => {
            error!(trip_id = %trip_id, error = %e, "On-demand path correction failed");

            // Return user-friendly error based on error type
            match e {
                crate::services::path_correction::PathCorrectionError::InsufficientLocations(n) => {
                    Ok(Json(CorrectPathResponse {
                        status: "SKIPPED".to_string(),
                        message: format!(
                            "Not enough locations for path correction (need at least 2, got {})",
                            n
                        ),
                    }))
                }
                crate::services::path_correction::PathCorrectionError::AlreadyExists(_) => {
                    Err(ApiError::Conflict(
                        "Path correction already exists".to_string(),
                    ))
                }
                crate::services::path_correction::PathCorrectionError::MapMatching(map_err) => {
                    // Check if circuit breaker is open
                    if map_err.to_string().contains("circuit breaker") {
                        Err(ApiError::ServiceUnavailable(
                            "Map-matching service temporarily unavailable".to_string(),
                        ))
                    } else {
                        Ok(Json(CorrectPathResponse {
                            status: "FAILED".to_string(),
                            message: format!("Map-matching error: {}", map_err),
                        }))
                    }
                }
                crate::services::path_correction::PathCorrectionError::Database(db_err) => {
                    Err(ApiError::Internal(format!("Database error: {}", db_err)))
                }
            }
        }
    }
}

/// Parse GeoJSON LineString to array of [lat, lon] coordinate pairs.
///
/// GeoJSON stores coordinates as [longitude, latitude], but we return
/// [latitude, longitude] for client convenience (matching common mobile APIs).
fn parse_geojson_linestring(geojson: &str) -> Result<Vec<[f64; 2]>, String> {
    // Parse the GeoJSON
    let value: serde_json::Value =
        serde_json::from_str(geojson).map_err(|e| format!("Invalid GeoJSON: {}", e))?;

    // Extract coordinates array
    let coords = value
        .get("coordinates")
        .ok_or_else(|| "Missing coordinates in GeoJSON".to_string())?
        .as_array()
        .ok_or_else(|| "Coordinates is not an array".to_string())?;

    // Convert [lon, lat] to [lat, lon]
    let mut result = Vec::with_capacity(coords.len());
    for coord in coords {
        let arr = coord
            .as_array()
            .ok_or_else(|| "Coordinate is not an array".to_string())?;
        if arr.len() < 2 {
            return Err("Coordinate must have at least 2 elements".to_string());
        }
        let lon = arr[0]
            .as_f64()
            .ok_or_else(|| "Longitude is not a number".to_string())?;
        let lat = arr[1]
            .as_f64()
            .ok_or_else(|| "Latitude is not a number".to_string())?;
        result.push([lat, lon]); // Return as [lat, lon]
    }

    Ok(result)
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

/// Correct trip path using map-matching service asynchronously.
///
/// Extracts trip locations, calls map-matching service, and stores
/// the corrected path. Errors are logged but don't affect the trip state.
/// If map-matching is disabled, the correction is marked as SKIPPED.
async fn correct_trip_path(
    pool: sqlx::PgPool,
    config: &crate::config::MapMatchingConfig,
    trip_id: Uuid,
) {
    info!(trip_id = %trip_id, "Starting path correction for trip");

    let service = PathCorrectionService::new(pool, config);

    match service.correct_trip_path(trip_id).await {
        Ok(result) => {
            info!(
                trip_id = %trip_id,
                status = %result.status,
                quality = ?result.quality,
                original_points = result.original_points,
                corrected_points = ?result.corrected_points,
                "Path correction completed"
            );
        }
        Err(e) => {
            // Log but don't fail - path correction is best-effort
            warn!(
                trip_id = %trip_id,
                error = %e,
                "Path correction failed"
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

    // =========================================================================
    // GeoJSON Parsing Tests
    // =========================================================================

    #[test]
    fn test_parse_geojson_linestring_valid() {
        let geojson = r#"{"type":"LineString","coordinates":[[-120.0,45.0],[-120.1,45.1],[-120.2,45.2]]}"#;
        let result = parse_geojson_linestring(geojson).unwrap();

        assert_eq!(result.len(), 3);
        // Coordinates should be [lat, lon] (swapped from GeoJSON [lon, lat])
        assert_eq!(result[0], [45.0, -120.0]);
        assert_eq!(result[1], [45.1, -120.1]);
        assert_eq!(result[2], [45.2, -120.2]);
    }

    #[test]
    fn test_parse_geojson_linestring_two_points() {
        let geojson = r#"{"type":"LineString","coordinates":[[-122.4194,37.7749],[-122.4084,37.7899]]}"#;
        let result = parse_geojson_linestring(geojson).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], [37.7749, -122.4194]);
        assert_eq!(result[1], [37.7899, -122.4084]);
    }

    #[test]
    fn test_parse_geojson_linestring_invalid_json() {
        let geojson = "not json";
        assert!(parse_geojson_linestring(geojson).is_err());
    }

    #[test]
    fn test_parse_geojson_linestring_missing_coordinates() {
        let geojson = r#"{"type":"LineString"}"#;
        assert!(parse_geojson_linestring(geojson).is_err());
    }

    #[test]
    fn test_parse_geojson_linestring_invalid_coordinates() {
        let geojson = r#"{"type":"LineString","coordinates":"not an array"}"#;
        assert!(parse_geojson_linestring(geojson).is_err());
    }

    #[test]
    fn test_parse_geojson_linestring_invalid_point() {
        let geojson = r#"{"type":"LineString","coordinates":[[-120.0]]}"#; // Single element
        assert!(parse_geojson_linestring(geojson).is_err());
    }

    #[test]
    fn test_parse_geojson_linestring_with_altitude() {
        // GeoJSON can have optional third element (altitude) - should still work
        let geojson = r#"{"type":"LineString","coordinates":[[-120.0,45.0,100.0],[-120.1,45.1,200.0]]}"#;
        let result = parse_geojson_linestring(geojson).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], [45.0, -120.0]); // Altitude is ignored
    }
}
