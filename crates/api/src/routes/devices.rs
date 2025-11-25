//! Device endpoint handlers.

use axum::{extract::State, Json};
use serde::Deserialize;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::device::{DeviceSummary, RegisterDeviceRequest, RegisterDeviceResponse};

/// Query parameters for device listing.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDevicesQuery {
    pub group_id: Option<String>,
}

/// Response for device listing.
#[derive(Debug, serde::Serialize)]
pub struct GetDevicesResponse {
    pub devices: Vec<DeviceSummary>,
}

/// Register or update a device.
///
/// POST /api/devices/register
pub async fn register_device(
    State(_state): State<AppState>,
    Json(_request): Json<RegisterDeviceRequest>,
) -> Result<Json<RegisterDeviceResponse>, ApiError> {
    // Implementation will be completed in Story 2.1
    Err(ApiError::Internal("Not implemented yet".to_string()))
}

/// Get all active devices in a group.
///
/// GET /api/devices?groupId=<id>
pub async fn get_group_devices(
    State(_state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<GetDevicesQuery>,
) -> Result<Json<GetDevicesResponse>, ApiError> {
    let _group_id = query
        .group_id
        .ok_or_else(|| ApiError::Validation("groupId query parameter is required".to_string()))?;

    // Implementation will be completed in Story 2.5
    Ok(Json(GetDevicesResponse { devices: vec![] }))
}
