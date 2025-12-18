//! Device endpoint handlers.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use domain::models::{check_usage_warning, ResponseWithWarnings};
use persistence::repositories::DeviceRepository;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::{OptionalUserAuth, UserAuth};
use domain::models::device::{
    DeviceLastLocation, DeviceSummary, RegisterDeviceRequest, RegisterDeviceResponse,
};

/// Query parameters for device listing.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
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
/// POST /api/v1/devices/register
///
/// Supports both API key authentication (legacy) and JWT authentication.
/// When JWT authenticated, device is automatically linked to the user.
/// Returns usage warning when device count in group approaches configured limit.
pub async fn register_device(
    State(state): State<AppState>,
    optional_user: OptionalUserAuth,
    Json(request): Json<RegisterDeviceRequest>,
) -> Result<Json<ResponseWithWarnings<RegisterDeviceResponse>>, ApiError> {
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

    let repo = DeviceRepository::new(state.pool.clone());

    // Check if this is a new device or an update
    let existing_device = repo.find_by_device_id(request.device_id).await?;
    let is_new_device = existing_device.is_none();
    let is_changing_group = existing_device
        .as_ref()
        .map(|d| d.group_id != request.group_id)
        .unwrap_or(false);

    // If user is authenticated and device already exists with a different owner, reject
    if let (Some(ref user_auth), Some(ref existing)) = (&optional_user.0, &existing_device) {
        if let Some(owner_id) = existing.owner_user_id {
            if owner_id != user_auth.user_id {
                return Err(ApiError::Conflict(
                    "Device is already linked to another user".to_string(),
                ));
            }
        }
    }

    // Track group count for usage warning (only relevant for new devices or group changes)
    let mut group_count_for_warning: Option<i64> = None;
    let max_devices = state.config.limits.max_devices_per_group as i64;

    // Check group capacity if this is a new device or changing groups
    if is_new_device || is_changing_group {
        let group_count = repo
            .count_active_devices_in_group(&request.group_id)
            .await?;

        if group_count >= max_devices {
            return Err(ApiError::Conflict(format!(
                "Group has reached maximum device limit ({})",
                max_devices
            )));
        }

        // Store for warning calculation
        group_count_for_warning = Some(group_count);
    }

    // Perform upsert
    let device = repo
        .upsert_device(
            request.device_id,
            &request.display_name,
            &request.group_id,
            &request.platform,
            request.fcm_token.as_deref(),
        )
        .await?;

    // If user is authenticated and device doesn't have an owner, link it
    let final_device = if let Some(user_auth) = optional_user.0 {
        if device.owner_user_id.is_none() {
            // Link device to user (first device becomes primary)
            let user_has_other_devices = !repo
                .find_devices_by_user(user_auth.user_id, false)
                .await?
                .is_empty();
            let is_primary = !user_has_other_devices;

            let linked = repo
                .link_device_to_user(request.device_id, user_auth.user_id, None, is_primary)
                .await?;

            info!(
                device_id = %device.device_id,
                group_id = %device.group_id,
                user_id = %user_auth.user_id,
                is_new = is_new_device,
                is_primary = is_primary,
                "Device registered and linked to user"
            );

            domain::models::Device::from(linked)
        } else {
            info!(
                device_id = %device.device_id,
                group_id = %device.group_id,
                user_id = %user_auth.user_id,
                is_new = is_new_device,
                "Device registered (already linked)"
            );
            domain::models::Device::from(device)
        }
    } else {
        info!(
            device_id = %device.device_id,
            group_id = %device.group_id,
            is_new = is_new_device,
            "Device registered (no user auth)"
        );
        domain::models::Device::from(device)
    };

    let response = RegisterDeviceResponse::from(final_device);

    // Check for usage warning (only if we added a device to the group)
    let usage_warning = group_count_for_warning.and_then(|count| {
        let new_count = count + 1;
        let warning_threshold = state.config.limits.warning_threshold_percent;
        check_usage_warning("devices", new_count, max_devices, warning_threshold)
    });

    let response_with_warnings = ResponseWithWarnings::maybe_with_warning(response, usage_warning);

    Ok(Json(response_with_warnings))
}

/// Get all active devices in a group with last location.
///
/// GET /api/v1/devices?groupId=<id>
pub async fn get_group_devices(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<GetDevicesQuery>,
) -> Result<Json<GetDevicesResponse>, ApiError> {
    let group_id = query
        .group_id
        .ok_or_else(|| ApiError::Validation("group_id query parameter is required".to_string()))?;

    let repo = DeviceRepository::new(state.pool.clone());
    let devices = repo.find_devices_with_last_location(&group_id).await?;

    let summaries: Vec<DeviceSummary> = devices
        .into_iter()
        .map(|d| {
            // Build last_location if all location fields are present
            let last_location = match (
                d.last_latitude,
                d.last_longitude,
                d.last_location_time,
                d.last_accuracy,
            ) {
                (Some(lat), Some(lon), Some(time), Some(acc)) => Some(DeviceLastLocation {
                    latitude: lat,
                    longitude: lon,
                    timestamp: time,
                    accuracy: acc as f64,
                }),
                _ => None,
            };

            DeviceSummary {
                device_id: d.device_id,
                display_name: d.display_name,
                last_location,
                last_seen_at: d.last_seen_at,
            }
        })
        .collect();

    Ok(Json(GetDevicesResponse { devices: summaries }))
}

/// Deactivate a device (soft delete).
///
/// DELETE /api/v1/devices/:device_id
pub async fn delete_device(
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let repo = DeviceRepository::new(state.pool.clone());
    let rows_affected = repo.deactivate_device(device_id).await?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound("Device not found".to_string()));
    }

    info!(device_id = %device_id, "Device deactivated");
    Ok(StatusCode::NO_CONTENT)
}

/// Response body for registration group status.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RegistrationGroupStatusResponse {
    /// The device UUID
    pub device_id: Uuid,
    /// Device display name
    pub display_name: String,
    /// The registration group ID
    pub group_id: String,
    /// Whether this is considered a registration group (not an authenticated group)
    pub is_registration_group: bool,
    /// When the device was last seen
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// Get the registration group status for the current user's primary device.
///
/// GET /api/v1/devices/me/registration-group
///
/// Returns information about the user's primary device's registration group.
/// This helps the mobile app determine if migration is needed.
pub async fn get_registration_group_status(
    State(state): State<AppState>,
    user_auth: UserAuth,
) -> Result<Json<RegistrationGroupStatusResponse>, ApiError> {
    let repo = DeviceRepository::new(state.pool.clone());

    // Find user's devices, sorted by primary first
    let devices = repo.find_devices_by_user(user_auth.user_id, false).await?;

    // Get the primary device (first in list due to sort)
    let primary_device = devices
        .first()
        .ok_or_else(|| ApiError::NotFound("No devices linked to this user".to_string()))?;

    // A registration group is one that starts with a pattern like "reg-" or is a UUID-like string
    // that was auto-generated during anonymous device registration.
    // For now, we consider any group that is NOT associated with an authenticated group as a registration group.
    // This is a heuristic - in a full implementation, we would have a groups table to check.
    let is_registration_group = !primary_device.group_id.is_empty();

    let response = RegistrationGroupStatusResponse {
        device_id: primary_device.device_id,
        display_name: primary_device.display_name.clone(),
        group_id: primary_device.group_id.clone(),
        is_registration_group,
        last_seen_at: primary_device.last_seen_at,
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_get_devices_query_with_group_id() {
        let query = GetDevicesQuery {
            group_id: Some("family-group".to_string()),
        };
        assert_eq!(query.group_id, Some("family-group".to_string()));
    }

    #[test]
    fn test_get_devices_query_without_group_id() {
        let query = GetDevicesQuery { group_id: None };
        assert!(query.group_id.is_none());
    }

    #[test]
    fn test_get_devices_query_debug() {
        let query = GetDevicesQuery {
            group_id: Some("test".to_string()),
        };
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("GetDevicesQuery"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_get_devices_response_empty() {
        let response = GetDevicesResponse { devices: vec![] };
        assert!(response.devices.is_empty());
    }

    #[test]
    fn test_get_devices_response_with_devices() {
        let device1 = DeviceSummary {
            device_id: Uuid::new_v4(),
            display_name: "Phone 1".to_string(),
            last_location: Some(DeviceLastLocation {
                latitude: 37.7749,
                longitude: -122.4194,
                timestamp: Utc::now(),
                accuracy: 10.0,
            }),
            last_seen_at: Some(Utc::now()),
        };
        let device2 = DeviceSummary {
            device_id: Uuid::new_v4(),
            display_name: "Phone 2".to_string(),
            last_location: None,
            last_seen_at: None,
        };
        let response = GetDevicesResponse {
            devices: vec![device1, device2],
        };
        assert_eq!(response.devices.len(), 2);
        assert_eq!(response.devices[0].display_name, "Phone 1");
        assert!(response.devices[0].last_location.is_some());
        assert_eq!(response.devices[1].display_name, "Phone 2");
        assert!(response.devices[1].last_location.is_none());
    }

    #[test]
    fn test_get_devices_response_debug() {
        let response = GetDevicesResponse { devices: vec![] };
        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("GetDevicesResponse"));
    }

    #[test]
    fn test_get_devices_response_serialization() {
        let device = DeviceSummary {
            device_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            display_name: "Test Device".to_string(),
            last_location: None,
            last_seen_at: None,
        };
        let response = GetDevicesResponse {
            devices: vec![device],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"devices\""));
        assert!(json.contains("Test Device"));
        assert!(json.contains("\"last_location\":null"));
    }

    #[test]
    fn test_get_devices_response_serialization_with_location() {
        let device = DeviceSummary {
            device_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            display_name: "Test Device".to_string(),
            last_location: Some(DeviceLastLocation {
                latitude: 37.7749,
                longitude: -122.4194,
                timestamp: Utc::now(),
                accuracy: 10.0,
            }),
            last_seen_at: Some(Utc::now()),
        };
        let response = GetDevicesResponse {
            devices: vec![device],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"last_location\""));
        assert!(json.contains("\"latitude\":37.7749"));
        assert!(json.contains("\"longitude\":-122.4194"));
    }

    #[test]
    fn test_get_devices_query_deserialization() {
        let json = r#"{"group_id": "my-group"}"#;
        let query: GetDevicesQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.group_id, Some("my-group".to_string()));
    }

    #[test]
    fn test_get_devices_query_deserialization_missing_group() {
        let json = r#"{}"#;
        let query: GetDevicesQuery = serde_json::from_str(json).unwrap();
        assert!(query.group_id.is_none());
    }
}
