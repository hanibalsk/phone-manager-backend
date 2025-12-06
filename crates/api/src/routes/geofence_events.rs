//! Geofence event endpoint handlers.
//!
//! Story 15.2: Webhook Event Delivery
//! Provides endpoints for geofence event management aligned with
//! frontend mobile app expectations (phone-manager/GeofenceEventApiService.kt).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use persistence::repositories::{DeviceRepository, GeofenceRepository, GeofenceEventRepository};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::services::webhook_delivery::WebhookDeliveryService;
use domain::models::geofence_event::{
    CreateGeofenceEventRequest, GeofenceEventResponse, ListGeofenceEventsQuery,
    ListGeofenceEventsResponse, GeofenceEvent,
};

/// Maximum events per query.
const MAX_EVENTS_LIMIT: i64 = 100;

/// Create a new geofence event.
///
/// POST /api/v1/geofence-events
///
/// AC 15.2.2: Creates geofence event and triggers webhook delivery
pub async fn create_geofence_event(
    State(state): State<AppState>,
    Json(request): Json<CreateGeofenceEventRequest>,
) -> Result<(StatusCode, Json<GeofenceEventResponse>), ApiError> {
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

    // Parse timestamp
    let timestamp = request.parse_timestamp().map_err(ApiError::Validation)?;

    // Verify device exists and is active
    let device_repo = DeviceRepository::new(state.pool.clone());
    let device = device_repo
        .find_by_device_id(request.device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    if !device.active {
        return Err(ApiError::NotFound("Device not found".to_string()));
    }

    // Verify geofence exists and belongs to device
    let geofence_repo = GeofenceRepository::new(state.pool.clone());
    let geofence = geofence_repo
        .find_by_geofence_id(request.geofence_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Geofence not found".to_string()))?;

    if geofence.device_id != request.device_id {
        return Err(ApiError::NotFound("Geofence not found".to_string()));
    }

    // Create the event
    let event_repo = GeofenceEventRepository::new(state.pool.clone());
    let entity = event_repo
        .create(
            request.device_id,
            request.geofence_id,
            request.event_type.as_str(),
            timestamp,
            request.latitude,
            request.longitude,
        )
        .await?;

    let event_id = entity.event_id;

    // Convert to domain model
    let event = GeofenceEvent::from_raw(
        entity.id,
        entity.event_id,
        entity.device_id,
        entity.geofence_id,
        Some(geofence.name.clone()),
        &entity.event_type,
        entity.timestamp,
        entity.latitude,
        entity.longitude,
        entity.webhook_delivered,
        entity.webhook_response_code,
        entity.created_at,
    );

    let response: GeofenceEventResponse = event.into();

    info!(
        event_id = %response.event_id,
        device_id = %response.device_id,
        geofence_id = %response.geofence_id,
        event_type = %request.event_type,
        "Geofence event created"
    );

    // Trigger async webhook delivery (AC 15.2.5, 15.2.6)
    let pool = state.pool.clone();
    let geofence_name = geofence.name;
    let event_type = request.event_type;
    let device_id = request.device_id;
    let geofence_id = request.geofence_id;
    let latitude = request.latitude;
    let longitude = request.longitude;

    tokio::spawn(async move {
        let delivery_service = WebhookDeliveryService::new(pool.clone());
        if let Err(e) = delivery_service
            .deliver_geofence_event(
                event_id,
                device_id,
                geofence_id,
                &geofence_name,
                event_type,
                timestamp,
                latitude,
                longitude,
            )
            .await
        {
            tracing::error!(
                event_id = %event_id,
                error = %e,
                "Failed to deliver geofence event webhooks"
            );
        }
    });

    Ok((StatusCode::CREATED, Json(response)))
}

/// List geofence events for a device.
///
/// GET /api/v1/geofence-events?deviceId=<uuid>
///
/// AC 15.2.3: Returns events for device
pub async fn list_geofence_events(
    State(state): State<AppState>,
    Query(query): Query<ListGeofenceEventsQuery>,
) -> Result<Json<ListGeofenceEventsResponse>, ApiError> {
    let limit = query.limit.min(MAX_EVENTS_LIMIT);

    let event_repo = GeofenceEventRepository::new(state.pool.clone());
    let entities = event_repo
        .find_by_device_id(query.device_id, query.geofence_id, limit)
        .await?;

    let total = event_repo
        .count_by_device_id(query.device_id, query.geofence_id)
        .await?;

    let events: Vec<GeofenceEventResponse> = entities
        .into_iter()
        .map(|e| {
            let event = GeofenceEvent::from_raw(
                e.id,
                e.event_id,
                e.device_id,
                e.geofence_id,
                e.geofence_name,
                &e.event_type,
                e.timestamp,
                e.latitude,
                e.longitude,
                e.webhook_delivered,
                e.webhook_response_code,
                e.created_at,
            );
            event.into()
        })
        .collect();

    Ok(Json(ListGeofenceEventsResponse { events, total }))
}

/// Get a single geofence event by ID.
///
/// GET /api/v1/geofence-events/:event_id
///
/// AC 15.2.4: Returns single event
pub async fn get_geofence_event(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<GeofenceEventResponse>, ApiError> {
    let event_repo = GeofenceEventRepository::new(state.pool.clone());
    let entity = event_repo
        .find_by_event_id(event_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Geofence event not found".to_string()))?;

    let event = GeofenceEvent::from_raw(
        entity.id,
        entity.event_id,
        entity.device_id,
        entity.geofence_id,
        entity.geofence_name,
        &entity.event_type,
        entity.timestamp,
        entity.latitude,
        entity.longitude,
        entity.webhook_delivered,
        entity.webhook_response_code,
        entity.created_at,
    );
    Ok(Json(event.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::GeofenceTransitionType;

    #[test]
    fn test_create_geofence_event_request_deserialization() {
        let json = r#"{
            "device_id": "550e8400-e29b-41d4-a716-446655440000",
            "geofence_id": "660e8400-e29b-41d4-a716-446655440001",
            "event_type": "enter",
            "timestamp": "1701878400000",
            "latitude": 37.7749,
            "longitude": -122.4194
        }"#;

        let request: CreateGeofenceEventRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.event_type, GeofenceTransitionType::Enter);
        assert_eq!(request.latitude, 37.7749);
    }

    #[test]
    fn test_list_geofence_events_query_deserialization() {
        // Frontend sends camelCase
        let json = r#"{"deviceId": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let query: ListGeofenceEventsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(
            query.device_id.to_string(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(query.limit, 50); // default
    }

    #[test]
    fn test_list_geofence_events_query_with_geofence_id() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "geofenceId": "660e8400-e29b-41d4-a716-446655440001",
            "limit": 25
        }"#;
        let query: ListGeofenceEventsQuery = serde_json::from_str(json).unwrap();
        assert!(query.geofence_id.is_some());
        assert_eq!(query.limit, 25);
    }

    #[test]
    fn test_geofence_event_response_serialization() {
        let response = GeofenceEventResponse {
            event_id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            geofence_id: Uuid::new_v4(),
            geofence_name: Some("Home".to_string()),
            event_type: GeofenceTransitionType::Enter,
            timestamp: "2023-12-06T12:00:00Z".to_string(),
            latitude: 37.7749,
            longitude: -122.4194,
            webhook_delivered: true,
            webhook_response_code: Some(200),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"event_type\":\"enter\""));
        assert!(json.contains("\"webhook_delivered\":true"));
        assert!(json.contains("\"geofence_name\":\"Home\""));
    }
}
