//! Geofence endpoint handlers.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use persistence::repositories::{DeviceRepository, GeofenceRepository};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::geofence::{
    CreateGeofenceRequest, GeofenceResponse, ListGeofencesQuery, ListGeofencesResponse,
    UpdateGeofenceRequest,
};

/// Maximum number of geofences allowed per device.
const MAX_GEOFENCES_PER_DEVICE: i64 = 50;

/// Create a new geofence.
///
/// POST /api/v1/geofences
pub async fn create_geofence(
    State(state): State<AppState>,
    Json(request): Json<CreateGeofenceRequest>,
) -> Result<(StatusCode, Json<GeofenceResponse>), ApiError> {
    // Validate request
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

    // Validate event_types is not empty
    if request.event_types.is_empty() {
        return Err(ApiError::Validation(
            "At least one event type is required".to_string(),
        ));
    }

    // Verify device exists and is active
    let device_repo = DeviceRepository::new(state.pool.clone());
    let device = device_repo
        .find_by_device_id(request.device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    if !device.active {
        return Err(ApiError::NotFound("Device not found".to_string()));
    }

    // Check geofence limit per device
    // NOTE: This check has a TOCTOU race condition - two concurrent requests could both
    // pass this check before either inserts. For stricter enforcement, use a database
    // constraint or SELECT FOR UPDATE on the device row. Current implementation accepts
    // this trade-off for simplicity; the limit is a soft cap, not a security boundary.
    let geofence_repo = GeofenceRepository::new(state.pool.clone());
    let count = geofence_repo.count_by_device_id(request.device_id).await?;
    if count >= MAX_GEOFENCES_PER_DEVICE {
        return Err(ApiError::Conflict(format!(
            "Device has reached maximum geofence limit ({})",
            MAX_GEOFENCES_PER_DEVICE
        )));
    }

    // Convert event types to strings for database
    let event_types: Vec<String> = request
        .event_types
        .iter()
        .map(|e| e.as_str().to_string())
        .collect();

    // Create geofence
    let entity = geofence_repo
        .create(
            request.device_id,
            &request.name,
            request.latitude,
            request.longitude,
            request.radius_meters,
            &event_types,
            request.active,
            request.metadata,
        )
        .await?;

    let geofence: domain::models::Geofence = entity.into();
    let response: GeofenceResponse = geofence.into();

    info!(
        geofence_id = %response.geofence_id,
        device_id = %response.device_id,
        name = %response.name,
        "Geofence created"
    );

    Ok((StatusCode::CREATED, Json(response)))
}

/// List geofences for a device.
///
/// GET /api/v1/geofences?deviceId=<uuid>
pub async fn list_geofences(
    State(state): State<AppState>,
    Query(query): Query<ListGeofencesQuery>,
) -> Result<Json<ListGeofencesResponse>, ApiError> {
    let geofence_repo = GeofenceRepository::new(state.pool.clone());
    let entities = geofence_repo
        .find_by_device_id(query.device_id, query.include_inactive)
        .await?;

    let geofences: Vec<GeofenceResponse> = entities
        .into_iter()
        .map(|e| {
            let g: domain::models::Geofence = e.into();
            g.into()
        })
        .collect();

    let total = geofences.len();

    Ok(Json(ListGeofencesResponse { geofences, total }))
}

/// Get a single geofence by ID.
///
/// GET /api/v1/geofences/:geofence_id
pub async fn get_geofence(
    State(state): State<AppState>,
    Path(geofence_id): Path<Uuid>,
) -> Result<Json<GeofenceResponse>, ApiError> {
    let geofence_repo = GeofenceRepository::new(state.pool.clone());
    let entity = geofence_repo
        .find_by_geofence_id(geofence_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Geofence not found".to_string()))?;

    let geofence: domain::models::Geofence = entity.into();
    Ok(Json(geofence.into()))
}

/// Update a geofence (partial update).
///
/// PATCH /api/v1/geofences/:geofence_id
pub async fn update_geofence(
    State(state): State<AppState>,
    Path(geofence_id): Path<Uuid>,
    Json(request): Json<UpdateGeofenceRequest>,
) -> Result<Json<GeofenceResponse>, ApiError> {
    // Validate request
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

    // Validate event_types if provided
    if let Some(ref event_types) = request.event_types {
        if event_types.is_empty() {
            return Err(ApiError::Validation(
                "At least one event type is required".to_string(),
            ));
        }
    }

    let geofence_repo = GeofenceRepository::new(state.pool.clone());

    // Convert event types if provided
    let event_types: Option<Vec<String>> = request
        .event_types
        .as_ref()
        .map(|types| types.iter().map(|e| e.as_str().to_string()).collect());

    let entity = geofence_repo
        .update(
            geofence_id,
            request.name.as_deref(),
            request.latitude,
            request.longitude,
            request.radius_meters,
            event_types.as_deref(),
            request.active,
            request.metadata.clone(),
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("Geofence not found".to_string()))?;

    let geofence: domain::models::Geofence = entity.into();
    let response: GeofenceResponse = geofence.into();

    info!(geofence_id = %response.geofence_id, "Geofence updated");

    Ok(Json(response))
}

/// Delete a geofence.
///
/// DELETE /api/v1/geofences/:geofence_id
pub async fn delete_geofence(
    State(state): State<AppState>,
    Path(geofence_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let geofence_repo = GeofenceRepository::new(state.pool.clone());
    let rows_affected = geofence_repo.delete(geofence_id).await?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound("Geofence not found".to_string()));
    }

    info!(geofence_id = %geofence_id, "Geofence deleted");
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::geofence::GeofenceEventType;

    #[test]
    fn test_create_geofence_request_deserialization() {
        let json = r#"{
            "device_id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Home",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius_meters": 100.0
        }"#;

        let request: CreateGeofenceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Home");
        assert_eq!(request.latitude, 37.7749);
        assert_eq!(request.radius_meters, 100.0);
    }

    #[test]
    fn test_update_geofence_request_partial() {
        let json = r#"{
            "name": "Work"
        }"#;

        let request: UpdateGeofenceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, Some("Work".to_string()));
        assert!(request.latitude.is_none());
        assert!(request.longitude.is_none());
        assert!(request.radius_meters.is_none());
    }

    #[test]
    fn test_geofence_response_serialization() {
        let response = GeofenceResponse {
            geofence_id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            name: "Test".to_string(),
            latitude: 45.0,
            longitude: -120.0,
            radius_meters: 100.0,
            event_types: vec![GeofenceEventType::Enter, GeofenceEventType::Exit],
            active: true,
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"event_types\":[\"enter\",\"exit\"]"));
    }

    #[test]
    fn test_list_geofences_query_deserialization() {
        let json = r#"{"device_id": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let query: ListGeofencesQuery = serde_json::from_str(json).unwrap();
        assert!(!query.include_inactive);

        let json_with_inactive =
            r#"{"device_id": "550e8400-e29b-41d4-a716-446655440000", "include_inactive": true}"#;
        let query: ListGeofencesQuery = serde_json::from_str(json_with_inactive).unwrap();
        assert!(query.include_inactive);
    }
}
