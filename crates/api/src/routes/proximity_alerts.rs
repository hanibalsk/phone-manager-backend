//! Proximity alert endpoint handlers.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::{check_usage_warning, ResponseWithWarnings};
use persistence::repositories::{DeviceRepository, ProximityAlertRepository};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::proximity_alert::{
    CreateProximityAlertRequest, ListProximityAlertsQuery, ListProximityAlertsResponse,
    ProximityAlertResponse, UpdateProximityAlertRequest,
};

/// Maximum number of proximity alerts allowed per source device.
const MAX_ALERTS_PER_DEVICE: i64 = 20;

/// Create a new proximity alert.
///
/// POST /api/v1/proximity-alerts
/// Returns usage warning when proximity alert count approaches configured limit.
pub async fn create_proximity_alert(
    State(state): State<AppState>,
    Json(request): Json<CreateProximityAlertRequest>,
) -> Result<
    (
        StatusCode,
        Json<ResponseWithWarnings<ProximityAlertResponse>>,
    ),
    ApiError,
> {
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

    // Validate source and target are different
    if request.source_device_id == request.target_device_id {
        return Err(ApiError::Validation(
            "Source and target device must be different".to_string(),
        ));
    }

    let device_repo = DeviceRepository::new(state.pool.clone());

    // Verify source device exists and is active
    let source_device = device_repo
        .find_by_device_id(request.source_device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Source device not found".to_string()))?;

    if !source_device.active {
        return Err(ApiError::NotFound("Source device not found".to_string()));
    }

    // Verify target device exists and is active
    let target_device = device_repo
        .find_by_device_id(request.target_device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Target device not found".to_string()))?;

    if !target_device.active {
        return Err(ApiError::NotFound("Target device not found".to_string()));
    }

    // Verify both devices are in the same group
    if source_device.group_id != target_device.group_id {
        return Err(ApiError::Validation(
            "Source and target devices must be in the same group".to_string(),
        ));
    }

    let alert_repo = ProximityAlertRepository::new(state.pool.clone());

    // Check if alert already exists for this device pair
    let exists = alert_repo
        .exists_for_device_pair(request.source_device_id, request.target_device_id)
        .await?;
    if exists {
        return Err(ApiError::Conflict(
            "Proximity alert already exists for this device pair".to_string(),
        ));
    }

    // Check alert limit per device
    // NOTE: This check has a TOCTOU race condition - two concurrent requests could both
    // pass this check before either inserts. For stricter enforcement, use a database
    // constraint or SELECT FOR UPDATE on the device row. Current implementation accepts
    // this trade-off for simplicity; the limit is a soft cap, not a security boundary.
    let count = alert_repo
        .count_by_source_device_id(request.source_device_id)
        .await?;
    if count >= MAX_ALERTS_PER_DEVICE {
        return Err(ApiError::Conflict(format!(
            "Device has reached maximum proximity alert limit ({})",
            MAX_ALERTS_PER_DEVICE
        )));
    }

    // Create proximity alert
    let entity = alert_repo
        .create(
            request.source_device_id,
            request.target_device_id,
            request.name.as_deref(),
            request.radius_meters,
            request.is_active,
            request.metadata,
        )
        .await?;

    let alert: domain::models::ProximityAlert = entity.into();
    let response: ProximityAlertResponse = alert.into();

    info!(
        alert_id = %response.alert_id,
        source_device_id = %response.source_device_id,
        target_device_id = %response.target_device_id,
        "Proximity alert created"
    );

    // Check for usage warning (new count is count + 1 since we just created one)
    let new_count = count + 1;
    let warning_threshold = state.config.limits.warning_threshold_percent;
    let usage_warning = check_usage_warning(
        "proximity_alerts",
        new_count,
        MAX_ALERTS_PER_DEVICE,
        warning_threshold,
    );

    let response_with_warnings = ResponseWithWarnings::maybe_with_warning(response, usage_warning);

    Ok((StatusCode::CREATED, Json(response_with_warnings)))
}

/// List proximity alerts for a source device.
///
/// GET /api/v1/proximity-alerts?sourceDeviceId=<uuid>
pub async fn list_proximity_alerts(
    State(state): State<AppState>,
    Query(query): Query<ListProximityAlertsQuery>,
) -> Result<Json<ListProximityAlertsResponse>, ApiError> {
    let alert_repo = ProximityAlertRepository::new(state.pool.clone());
    let entities = alert_repo
        .find_by_source_device_id(query.source_device_id, query.include_inactive)
        .await?;

    let alerts: Vec<ProximityAlertResponse> = entities
        .into_iter()
        .map(|e| {
            let a: domain::models::ProximityAlert = e.into();
            a.into()
        })
        .collect();

    let total = alerts.len();

    Ok(Json(ListProximityAlertsResponse { alerts, total }))
}

/// Get a single proximity alert by ID.
///
/// GET /api/v1/proximity-alerts/:alert_id
pub async fn get_proximity_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
) -> Result<Json<ProximityAlertResponse>, ApiError> {
    let alert_repo = ProximityAlertRepository::new(state.pool.clone());
    let entity = alert_repo
        .find_by_alert_id(alert_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Proximity alert not found".to_string()))?;

    let alert: domain::models::ProximityAlert = entity.into();
    Ok(Json(alert.into()))
}

/// Update a proximity alert (partial update).
///
/// PATCH /api/v1/proximity-alerts/:alert_id
pub async fn update_proximity_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    Json(request): Json<UpdateProximityAlertRequest>,
) -> Result<Json<ProximityAlertResponse>, ApiError> {
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

    let alert_repo = ProximityAlertRepository::new(state.pool.clone());

    let entity = alert_repo
        .update(
            alert_id,
            request.name.as_deref(),
            request.radius_meters,
            request.is_active,
            request.metadata.clone(),
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("Proximity alert not found".to_string()))?;

    let alert: domain::models::ProximityAlert = entity.into();
    let response: ProximityAlertResponse = alert.into();

    info!(alert_id = %response.alert_id, "Proximity alert updated");

    Ok(Json(response))
}

/// Delete a proximity alert.
///
/// DELETE /api/v1/proximity-alerts/:alert_id
pub async fn delete_proximity_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let alert_repo = ProximityAlertRepository::new(state.pool.clone());
    let rows_affected = alert_repo.delete(alert_id).await?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound("Proximity alert not found".to_string()));
    }

    info!(alert_id = %alert_id, "Proximity alert deleted");
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proximity_alert_request_deserialization() {
        let json = r#"{
            "source_device_id": "550e8400-e29b-41d4-a716-446655440000",
            "target_device_id": "550e8400-e29b-41d4-a716-446655440001",
            "radius_meters": 500
        }"#;

        let request: CreateProximityAlertRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.radius_meters, 500);
        assert!(request.is_active);
    }

    #[test]
    fn test_update_proximity_alert_request_partial() {
        let json = r#"{
            "name": "Updated"
        }"#;

        let request: UpdateProximityAlertRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, Some("Updated".to_string()));
        assert!(request.radius_meters.is_none());
        assert!(request.is_active.is_none());
    }

    #[test]
    fn test_list_proximity_alerts_query_deserialization() {
        let json = r#"{"source_device_id": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let query: ListProximityAlertsQuery = serde_json::from_str(json).unwrap();
        assert!(!query.include_inactive);

        let json_with_inactive = r#"{"source_device_id": "550e8400-e29b-41d4-a716-446655440000", "include_inactive": true}"#;
        let query: ListProximityAlertsQuery = serde_json::from_str(json_with_inactive).unwrap();
        assert!(query.include_inactive);
    }
}
