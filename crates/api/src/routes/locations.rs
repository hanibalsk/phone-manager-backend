//! Location endpoint handlers.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{TimeZone, Utc};
use persistence::repositories::{
    DeviceRepository, IdempotencyKeyRepository, LocationHistoryQuery, LocationInput,
    LocationRepository,
};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::idempotency_key::OptionalIdempotencyKey;
use domain::models::location::{
    BatchUploadRequest, GetLocationHistoryQuery, LocationHistoryItem, LocationHistoryResponse,
    PaginationInfo, SortOrder, UploadLocationRequest, UploadLocationResponse,
};

/// Upload a single location.
///
/// POST /api/v1/locations
pub async fn upload_location(
    State(state): State<AppState>,
    OptionalIdempotencyKey(idempotency_key): OptionalIdempotencyKey,
    Json(request): Json<UploadLocationRequest>,
) -> Result<Json<UploadLocationResponse>, ApiError> {
    // Check idempotency key if present
    let idempotency_repo = IdempotencyKeyRepository::new(state.pool.clone());
    if let Some(ref key) = idempotency_key {
        if let Some(existing) = idempotency_repo.find_by_hash(&key.hash).await? {
            info!(
                idempotency_key = %key.original,
                "Returning cached response for idempotent request"
            );
            // Return cached response
            let response: UploadLocationResponse = serde_json::from_value(existing.response_body)
                .map_err(|_| ApiError::Internal("Failed to parse cached response".to_string()))?;
            return Ok(Json(response));
        }
    }

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

    // Verify device exists
    let device_repo = DeviceRepository::new(state.pool.clone());
    let device = device_repo
        .find_by_device_id(request.device_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound("Device not found. Please register first.".to_string())
        })?;

    // Check device is active
    if !device.active {
        return Err(ApiError::NotFound(
            "Device not found. Please register first.".to_string(),
        ));
    }

    // Convert millisecond timestamp to DateTime
    let captured_at = Utc
        .timestamp_millis_opt(request.timestamp)
        .single()
        .ok_or_else(|| ApiError::Validation("Invalid timestamp".to_string()))?;

    // Insert location
    let location_repo = LocationRepository::new(state.pool.clone());
    let input = LocationInput {
        device_id: request.device_id,
        latitude: request.latitude,
        longitude: request.longitude,
        accuracy: request.accuracy,
        altitude: request.altitude,
        bearing: request.bearing,
        speed: request.speed,
        provider: request.provider.clone(),
        battery_level: request.battery_level,
        network_type: request.network_type.clone(),
        captured_at,
    };
    location_repo.insert_location(input).await?;

    // Update device last_seen_at (fire-and-forget)
    let pool_clone = state.pool.clone();
    let device_id = request.device_id;
    tokio::spawn(async move {
        let repo = DeviceRepository::new(pool_clone);
        if let Err(e) = repo.update_last_seen_at(device_id, Utc::now()).await {
            tracing::warn!("Failed to update device last_seen_at: {}", e);
        }
    });

    let response = UploadLocationResponse {
        success: true,
        processed_count: 1,
    };

    // Store idempotency key with response if present
    if let Some(ref key) = idempotency_key {
        store_idempotency_key(&idempotency_repo, &key.hash, request.device_id, &response).await;
    }

    info!(
        device_id = %request.device_id,
        latitude = request.latitude,
        longitude = request.longitude,
        "Location uploaded"
    );

    Ok(Json(response))
}

/// Upload multiple locations in a batch.
///
/// POST /api/v1/locations/batch
pub async fn upload_batch(
    State(state): State<AppState>,
    OptionalIdempotencyKey(idempotency_key): OptionalIdempotencyKey,
    Json(request): Json<BatchUploadRequest>,
) -> Result<Json<UploadLocationResponse>, ApiError> {
    // Check idempotency key if present
    let idempotency_repo = IdempotencyKeyRepository::new(state.pool.clone());
    if let Some(ref key) = idempotency_key {
        if let Some(existing) = idempotency_repo.find_by_hash(&key.hash).await? {
            info!(
                idempotency_key = %key.original,
                "Returning cached response for idempotent batch request"
            );
            // Return cached response
            let response: UploadLocationResponse = serde_json::from_value(existing.response_body)
                .map_err(|_| ApiError::Internal("Failed to parse cached response".to_string()))?;
            return Ok(Json(response));
        }
    }

    // Validate the request (batch size)
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

    // Validate each location in the batch
    for (i, loc) in request.locations.iter().enumerate() {
        loc.validate().map_err(|e| {
            let errors: Vec<String> = e
                .field_errors()
                .iter()
                .flat_map(|(field, errors)| {
                    errors.iter().map(move |err| {
                        format!(
                            "locations[{}].{}: {}",
                            i,
                            field,
                            err.message.as_ref().unwrap_or(&"".into())
                        )
                    })
                })
                .collect();
            ApiError::Validation(errors.join(", "))
        })?;
    }

    // Verify device exists
    let device_repo = DeviceRepository::new(state.pool.clone());
    let device = device_repo
        .find_by_device_id(request.device_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound("Device not found. Please register first.".to_string())
        })?;

    // Check device is active
    if !device.active {
        return Err(ApiError::NotFound(
            "Device not found. Please register first.".to_string(),
        ));
    }

    // Convert locations to repository format
    let mut locations_data = Vec::with_capacity(request.locations.len());
    for loc in &request.locations {
        let captured_at = Utc
            .timestamp_millis_opt(loc.timestamp)
            .single()
            .ok_or_else(|| ApiError::Validation("Invalid timestamp".to_string()))?;

        locations_data.push(LocationInput {
            device_id: request.device_id,
            latitude: loc.latitude,
            longitude: loc.longitude,
            accuracy: loc.accuracy,
            altitude: loc.altitude,
            bearing: loc.bearing,
            speed: loc.speed,
            provider: loc.provider.clone(),
            battery_level: loc.battery_level,
            network_type: loc.network_type.clone(),
            captured_at,
        });
    }

    // Insert all locations in a transaction
    let location_repo = LocationRepository::new(state.pool.clone());
    let processed_count = location_repo
        .insert_locations_batch(request.device_id, locations_data)
        .await?;

    // Update device last_seen_at (fire-and-forget)
    let pool_clone = state.pool.clone();
    let device_id = request.device_id;
    tokio::spawn(async move {
        let repo = DeviceRepository::new(pool_clone);
        if let Err(e) = repo.update_last_seen_at(device_id, Utc::now()).await {
            tracing::warn!("Failed to update device last_seen_at: {}", e);
        }
    });

    let response = UploadLocationResponse {
        success: true,
        processed_count,
    };

    // Store idempotency key with response if present
    if let Some(ref key) = idempotency_key {
        store_idempotency_key(&idempotency_repo, &key.hash, request.device_id, &response).await;
    }

    info!(
        device_id = %request.device_id,
        count = processed_count,
        "Batch locations uploaded"
    );

    Ok(Json(response))
}

/// Helper function to store idempotency key (fire-and-forget).
async fn store_idempotency_key(
    repo: &IdempotencyKeyRepository,
    key_hash: &str,
    device_id: Uuid,
    response: &UploadLocationResponse,
) {
    let response_json = serde_json::to_value(response).unwrap_or_default();
    if let Err(e) = repo.store(key_hash, device_id, response_json, 200).await {
        tracing::warn!("Failed to store idempotency key: {}", e);
    }
}

/// Get location history for a device with cursor-based pagination.
///
/// GET /api/v1/devices/:device_id/locations
pub async fn get_location_history(
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
    Query(query): Query<GetLocationHistoryQuery>,
) -> Result<Json<LocationHistoryResponse>, ApiError> {
    // Verify device exists and is active
    let device_repo = DeviceRepository::new(state.pool.clone());
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    if !device.active {
        return Err(ApiError::NotFound("Device not found".to_string()));
    }

    // Decode cursor if present
    let (cursor_timestamp, cursor_id) = match &query.cursor {
        Some(cursor) => {
            let (ts, id) = shared::pagination::decode_cursor(cursor)
                .map_err(|_| ApiError::Validation("Invalid cursor format".to_string()))?;
            (Some(ts), Some(id))
        }
        None => (None, None),
    };

    // Convert timestamp filters from milliseconds to DateTime
    // Return 400 if timestamps are provided but invalid (e.g., overflow)
    let from_timestamp = match query.from {
        Some(ts) => {
            let dt = Utc.timestamp_millis_opt(ts).single()
                .ok_or_else(|| ApiError::Validation(format!("Invalid 'from' timestamp: {}", ts)))?;
            Some(dt)
        }
        None => None,
    };
    let to_timestamp = match query.to {
        Some(ts) => {
            let dt = Utc.timestamp_millis_opt(ts).single()
                .ok_or_else(|| ApiError::Validation(format!("Invalid 'to' timestamp: {}", ts)))?;
            Some(dt)
        }
        None => None,
    };

    // Get effective limit (clamped to valid range)
    let limit = query.effective_limit();

    // Build repository query
    let repo_query = LocationHistoryQuery {
        device_id,
        cursor_timestamp,
        cursor_id,
        from_timestamp,
        to_timestamp,
        limit,
        ascending: query.order == SortOrder::Asc,
    };

    // Execute query
    let location_repo = LocationRepository::new(state.pool.clone());
    let (entities, has_more) = location_repo.get_location_history(repo_query).await?;

    // Build response with next cursor
    let next_cursor = if has_more {
        entities.last().map(|loc| {
            shared::pagination::encode_cursor(loc.captured_at, loc.id)
        })
    } else {
        None
    };

    // Convert entities to response items
    let locations: Vec<LocationHistoryItem> = entities
        .into_iter()
        .map(|e| {
            let loc: domain::models::Location = e.into();
            loc.into()
        })
        .collect();

    info!(
        device_id = %device_id,
        count = locations.len(),
        has_more = has_more,
        "Location history retrieved"
    );

    Ok(Json(LocationHistoryResponse {
        locations,
        pagination: PaginationInfo { next_cursor, has_more },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::location::LocationData;
    use uuid::Uuid;

    #[test]
    fn test_upload_location_request_serialization() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": 1700000000000,
            "latitude": 37.7749,
            "longitude": -122.4194,
            "accuracy": 10.0,
            "altitude": 100.0,
            "bearing": 180.0,
            "speed": 5.5,
            "provider": "gps",
            "batteryLevel": 85,
            "networkType": "wifi"
        }"#;
        let request: UploadLocationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            request.device_id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(request.latitude, 37.7749);
        assert_eq!(request.longitude, -122.4194);
        assert_eq!(request.accuracy, 10.0);
    }

    #[test]
    fn test_upload_location_request_minimal() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": 1700000000000,
            "latitude": 37.7749,
            "longitude": -122.4194,
            "accuracy": 10.0
        }"#;
        let request: UploadLocationRequest = serde_json::from_str(json).unwrap();
        assert!(request.altitude.is_none());
        assert!(request.bearing.is_none());
        assert!(request.speed.is_none());
        assert!(request.provider.is_none());
        assert!(request.battery_level.is_none());
        assert!(request.network_type.is_none());
    }

    #[test]
    fn test_batch_upload_request_serialization() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "locations": [
                {
                    "timestamp": 1700000000000,
                    "latitude": 37.7749,
                    "longitude": -122.4194,
                    "accuracy": 10.0
                },
                {
                    "timestamp": 1700000001000,
                    "latitude": 37.7750,
                    "longitude": -122.4195,
                    "accuracy": 15.0
                }
            ]
        }"#;
        let request: BatchUploadRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.locations.len(), 2);
        assert_eq!(request.locations[0].latitude, 37.7749);
        assert_eq!(request.locations[1].latitude, 37.7750);
    }

    #[test]
    fn test_upload_location_response_serialization() {
        let response = UploadLocationResponse {
            success: true,
            processed_count: 5,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"processedCount\":5"));
    }

    #[test]
    fn test_upload_location_response_failed() {
        let response = UploadLocationResponse {
            success: false,
            processed_count: 0,
        };
        assert!(!response.success);
        assert_eq!(response.processed_count, 0);
    }

    #[test]
    fn test_location_data_with_all_fields() {
        let data = LocationData {
            timestamp: 1700000000000,
            latitude: 40.7128,
            longitude: -74.0060,
            accuracy: 5.0,
            altitude: Some(50.0),
            bearing: Some(90.0),
            speed: Some(10.0),
            provider: Some("fused".to_string()),
            battery_level: Some(75),
            network_type: Some("5g".to_string()),
        };
        assert_eq!(data.latitude, 40.7128);
        assert_eq!(data.provider, Some("fused".to_string()));
    }

    #[test]
    fn test_location_data_serialization() {
        let data = LocationData {
            timestamp: 1700000000000,
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"latitude\":45"));
        assert!(json.contains("\"longitude\":-120"));
    }
}
