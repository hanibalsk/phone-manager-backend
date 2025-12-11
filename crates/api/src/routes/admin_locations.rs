//! Admin location management route handlers.
//!
//! AP-6: Location & Geofence Administration - Device Location endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{TimeZone, Utc};
use persistence::repositories::{
    DeviceRepository, LocationHistoryQuery, LocationRepository, OrgUserRepository,
};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

use domain::models::{
    AdminAllDeviceLocationsResponse, AdminDeviceLocation, AdminDeviceLocationResponse,
    AdminGeofencePagination, AdminLocationHistoryQuery, AdminLocationHistoryResponse, OrgUserRole,
};

/// Create admin location management routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/current", get(get_all_device_locations))
        .route("/history", get(get_org_location_history))
}

/// Create device location routes (nested under devices).
pub fn device_location_router() -> Router<AppState> {
    Router::new()
        .route("/{device_id}/location", get(get_device_location))
        .route(
            "/{device_id}/location-history",
            get(get_device_location_history),
        )
}

/// Get current location for a specific device.
///
/// GET /api/admin/v1/organizations/{org_id}/devices/{device_id}/location
#[axum::debug_handler]
async fn get_device_location(
    State(state): State<AppState>,
    Path((org_id, device_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());
    let location_repo = LocationRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view locations)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify device belongs to organization
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    if device.organization_id != Some(org_id) {
        return Err(ApiError::NotFound(
            "Device not found in organization".to_string(),
        ));
    }

    // Get last location from locations table (most recent)
    // Note: LocationHistoryQuery uses device.id (i64) but actually it uses device_id (Uuid) for the query
    let locations = location_repo
        .get_location_history(LocationHistoryQuery {
            device_id: device.device_id,
            cursor_timestamp: None,
            cursor_id: None,
            from_timestamp: None,
            to_timestamp: None,
            limit: 1,
            ascending: false, // Get most recent first
        })
        .await?;

    let location = locations.0.first().map(|loc| AdminDeviceLocation {
        device_id: device.device_id,
        device_name: Some(device.display_name.clone()),
        latitude: loc.latitude,
        longitude: loc.longitude,
        accuracy: Some(loc.accuracy),
        altitude: loc.altitude,
        speed: loc.speed,
        bearing: loc.bearing,
        timestamp: loc.captured_at.timestamp_millis(),
        recorded_at: loc.created_at,
    });

    info!(
        org_id = %org_id,
        device_id = %device_id,
        user_id = %user.user_id,
        has_location = location.is_some(),
        "Retrieved device location"
    );

    let response = AdminDeviceLocationResponse { location };

    Ok((StatusCode::OK, Json(response)))
}

/// Get location history for a specific device.
///
/// GET /api/admin/v1/organizations/{org_id}/devices/{device_id}/location-history
#[axum::debug_handler]
async fn get_device_location_history(
    State(state): State<AppState>,
    Path((org_id, device_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<AdminLocationHistoryQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Validate query
    query
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());
    let location_repo = LocationRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view locations)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify device belongs to organization
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    if device.organization_id != Some(org_id) {
        return Err(ApiError::NotFound(
            "Device not found in organization".to_string(),
        ));
    }

    // Get pagination params
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);

    // Convert timestamps
    let from_timestamp = query
        .from_timestamp
        .and_then(|ts| Utc.timestamp_millis_opt(ts).single());
    let to_timestamp = query
        .to_timestamp
        .and_then(|ts| Utc.timestamp_millis_opt(ts).single());

    // For pagination, we need to calculate offset-based pagination
    // Since cursor-based is used internally, we'll use the full range query
    let all_locations = location_repo
        .get_all_locations_in_range(device.device_id, from_timestamp, to_timestamp)
        .await?;

    let total = all_locations.len() as i64;
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    // Apply pagination
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(all_locations.len());
    let page_locations = if start < all_locations.len() {
        &all_locations[start..end]
    } else {
        &[]
    };

    let locations: Vec<AdminDeviceLocation> = page_locations
        .iter()
        .map(|loc| AdminDeviceLocation {
            device_id: device.device_id,
            device_name: Some(device.display_name.clone()),
            latitude: loc.latitude,
            longitude: loc.longitude,
            accuracy: Some(loc.accuracy),
            altitude: loc.altitude,
            speed: loc.speed,
            bearing: loc.bearing,
            timestamp: loc.captured_at.timestamp_millis(),
            recorded_at: loc.created_at,
        })
        .collect();

    info!(
        org_id = %org_id,
        device_id = %device_id,
        user_id = %user.user_id,
        location_count = locations.len(),
        "Retrieved device location history"
    );

    let response = AdminLocationHistoryResponse {
        device_id,
        locations,
        pagination: AdminGeofencePagination {
            page,
            per_page,
            total,
            total_pages,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get current locations for all devices in organization.
///
/// GET /api/admin/v1/organizations/{org_id}/locations/current
#[axum::debug_handler]
async fn get_all_device_locations(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());
    let location_repo = LocationRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view locations)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get all managed devices in organization
    let devices = device_repo.list_org_managed_devices(org_id).await?;

    let mut device_locations = Vec::new();

    // Get latest location for each device
    for device in &devices {
        let locations = location_repo
            .get_location_history(LocationHistoryQuery {
                device_id: device.device_id,
                cursor_timestamp: None,
                cursor_id: None,
                from_timestamp: None,
                to_timestamp: None,
                limit: 1,
                ascending: false,
            })
            .await?;

        if let Some(loc) = locations.0.first() {
            device_locations.push(AdminDeviceLocation {
                device_id: device.device_id,
                device_name: Some(device.display_name.clone()),
                latitude: loc.latitude,
                longitude: loc.longitude,
                accuracy: Some(loc.accuracy),
                altitude: loc.altitude,
                speed: loc.speed,
                bearing: loc.bearing,
                timestamp: loc.captured_at.timestamp_millis(),
                recorded_at: loc.created_at,
            });
        }
    }

    let total = device_locations.len() as i64;

    info!(
        org_id = %org_id,
        user_id = %user.user_id,
        device_count = total,
        "Retrieved all device locations"
    );

    let response = AdminAllDeviceLocationsResponse {
        devices: device_locations,
        total,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get location history for all devices in organization.
///
/// GET /api/admin/v1/organizations/{org_id}/locations/history
#[axum::debug_handler]
async fn get_org_location_history(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<AdminLocationHistoryQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Validate query
    query
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());
    let location_repo = LocationRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view locations)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get all managed devices in organization
    let devices = device_repo.list_org_managed_devices(org_id).await?;

    // Convert timestamps
    let from_timestamp = query
        .from_timestamp
        .and_then(|ts| Utc.timestamp_millis_opt(ts).single());
    let to_timestamp = query
        .to_timestamp
        .and_then(|ts| Utc.timestamp_millis_opt(ts).single());

    // Collect locations from all devices
    let mut all_locations = Vec::new();
    let device_map: std::collections::HashMap<Uuid, Option<String>> = devices
        .iter()
        .map(|d| (d.device_id, Some(d.display_name.clone())))
        .collect();

    for device in &devices {
        let locations = location_repo
            .get_all_locations_in_range(device.device_id, from_timestamp, to_timestamp)
            .await?;

        for loc in locations {
            all_locations.push(AdminDeviceLocation {
                device_id: device.device_id,
                device_name: device_map.get(&device.device_id).cloned().flatten(),
                latitude: loc.latitude,
                longitude: loc.longitude,
                accuracy: Some(loc.accuracy),
                altitude: loc.altitude,
                speed: loc.speed,
                bearing: loc.bearing,
                timestamp: loc.captured_at.timestamp_millis(),
                recorded_at: loc.created_at,
            });
        }
    }

    // Sort by timestamp descending (most recent first)
    all_locations.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let total = all_locations.len() as i64;

    // Apply pagination
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(all_locations.len());
    let page_locations = if start < all_locations.len() {
        all_locations[start..end].to_vec()
    } else {
        vec![]
    };

    info!(
        org_id = %org_id,
        user_id = %user.user_id,
        location_count = page_locations.len(),
        total = total,
        "Retrieved org location history"
    );

    // Return as a generic location list with pagination
    // Use device_id=Uuid::nil() as placeholder for org-wide query
    let response = AdminLocationHistoryResponse {
        device_id: Uuid::nil(),
        locations: page_locations,
        pagination: AdminGeofencePagination {
            page,
            per_page,
            total,
            total_pages,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let _router: Router<AppState> = router();
    }

    #[test]
    fn test_device_location_router_creation() {
        let _router: Router<AppState> = device_location_router();
    }
}
