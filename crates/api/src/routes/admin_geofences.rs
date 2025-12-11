//! Admin geofence management route handlers.
//!
//! AP-6: Location & Geofence Administration

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use persistence::repositories::{AdminGeofenceRepository, OrgUserRepository};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

use chrono::{TimeZone, Utc};

use domain::models::{
    AdminGeofenceEventInfo, AdminGeofenceEventsQuery, AdminGeofenceEventsResponse,
    AdminGeofenceInfo, AdminGeofenceListResponse, AdminGeofencePagination, AdminGeofenceQuery,
    AdminLocationAnalyticsResponse, CreateAdminGeofenceRequest, CreateAdminGeofenceResponse,
    DeleteAdminGeofenceResponse, GeofenceVisitCount, LocationAnalyticsSummary, OrgUserRole,
    UpdateAdminGeofenceRequest, UpdateAdminGeofenceResponse,
};

/// Create admin geofence management routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_geofences))
        .route("/", post(create_geofence))
        .route("/{geofence_id}", get(get_geofence))
        .route("/{geofence_id}", put(update_geofence))
        .route("/{geofence_id}", delete(delete_geofence))
}

/// Create geofence events routes.
pub fn geofence_events_router() -> Router<AppState> {
    Router::new().route("/", get(list_geofence_events))
}

/// Create location analytics routes.
pub fn location_analytics_router() -> Router<AppState> {
    Router::new().route("/", get(get_location_analytics))
}

/// List admin geofences in organization.
///
/// GET /api/admin/v1/organizations/{org_id}/geofences
#[axum::debug_handler]
async fn list_geofences(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<AdminGeofenceQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Validate query
    query
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let geofence_repo = AdminGeofenceRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view geofences)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get pagination params
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);
    let offset = (page - 1) * per_page;

    // Get total count
    let total = geofence_repo
        .count_geofences(org_id, query.active, query.search.as_deref())
        .await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    // Get geofences
    let geofences = geofence_repo
        .list_geofences(
            org_id,
            query.active,
            query.search.as_deref(),
            per_page,
            offset,
        )
        .await?;

    // Map to response
    let data: Vec<AdminGeofenceInfo> = geofences
        .into_iter()
        .map(|g| AdminGeofenceInfo {
            id: g.geofence_id,
            organization_id: g.organization_id,
            name: g.name,
            description: g.description,
            latitude: g.latitude,
            longitude: g.longitude,
            radius_meters: g.radius_meters,
            event_types: g.event_types,
            active: g.active,
            color: g.color,
            metadata: g.metadata,
            created_by: g.created_by,
            creator_name: g.creator_name,
            created_at: g.created_at,
            updated_at: g.updated_at,
        })
        .collect();

    info!(
        org_id = %org_id,
        user_id = %user.user_id,
        geofence_count = data.len(),
        "Listed admin geofences"
    );

    let response = AdminGeofenceListResponse {
        data,
        pagination: AdminGeofencePagination {
            page,
            per_page,
            total,
            total_pages,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Create a new admin geofence.
///
/// POST /api/admin/v1/organizations/{org_id}/geofences
#[axum::debug_handler]
async fn create_geofence(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    user: UserAuth,
    Json(request): Json<CreateAdminGeofenceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Validate event_types
    for event_type in &request.event_types {
        if !matches!(event_type.as_str(), "enter" | "exit" | "dwell") {
            return Err(ApiError::Validation(format!(
                "Invalid event type '{}'. Must be enter, exit, or dwell",
                event_type
            )));
        }
    }

    // Validate color format if provided
    if let Some(ref color) = request.color {
        if !color.starts_with('#') || color.len() != 7 {
            return Err(ApiError::Validation(
                "Color must be a hex color code like #FF5733".to_string(),
            ));
        }
    }

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let geofence_repo = AdminGeofenceRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can create geofences)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Create geofence
    let geofence = geofence_repo
        .create_geofence(
            org_id,
            &request.name,
            request.description.as_deref(),
            request.latitude,
            request.longitude,
            request.radius_meters,
            &request.event_types,
            request.color.as_deref(),
            request.metadata.as_ref(),
            user.user_id,
        )
        .await?;

    info!(
        org_id = %org_id,
        geofence_id = %geofence.geofence_id,
        user_id = %user.user_id,
        "Created admin geofence"
    );

    let response = CreateAdminGeofenceResponse {
        geofence: AdminGeofenceInfo {
            id: geofence.geofence_id,
            organization_id: geofence.organization_id,
            name: geofence.name,
            description: geofence.description,
            latitude: geofence.latitude,
            longitude: geofence.longitude,
            radius_meters: geofence.radius_meters,
            event_types: geofence.event_types,
            active: geofence.active,
            color: geofence.color,
            metadata: geofence.metadata,
            created_by: geofence.created_by,
            creator_name: None, // Not available from create
            created_at: geofence.created_at,
            updated_at: geofence.updated_at,
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a single admin geofence.
///
/// GET /api/admin/v1/organizations/{org_id}/geofences/{geofence_id}
#[axum::debug_handler]
async fn get_geofence(
    State(state): State<AppState>,
    Path((org_id, geofence_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let geofence_repo = AdminGeofenceRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view geofences)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get geofence
    let geofence = geofence_repo
        .get_geofence(org_id, geofence_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Geofence not found".to_string()))?;

    let response = AdminGeofenceInfo {
        id: geofence.geofence_id,
        organization_id: geofence.organization_id,
        name: geofence.name,
        description: geofence.description,
        latitude: geofence.latitude,
        longitude: geofence.longitude,
        radius_meters: geofence.radius_meters,
        event_types: geofence.event_types,
        active: geofence.active,
        color: geofence.color,
        metadata: geofence.metadata,
        created_by: geofence.created_by,
        creator_name: geofence.creator_name,
        created_at: geofence.created_at,
        updated_at: geofence.updated_at,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Update an admin geofence.
///
/// PUT /api/admin/v1/organizations/{org_id}/geofences/{geofence_id}
#[axum::debug_handler]
async fn update_geofence(
    State(state): State<AppState>,
    Path((org_id, geofence_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
    Json(request): Json<UpdateAdminGeofenceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Validate event_types if provided
    if let Some(ref event_types) = request.event_types {
        for event_type in event_types {
            if !matches!(event_type.as_str(), "enter" | "exit" | "dwell") {
                return Err(ApiError::Validation(format!(
                    "Invalid event type '{}'. Must be enter, exit, or dwell",
                    event_type
                )));
            }
        }
    }

    // Validate color format if provided
    if let Some(ref color) = request.color {
        if !color.starts_with('#') || color.len() != 7 {
            return Err(ApiError::Validation(
                "Color must be a hex color code like #FF5733".to_string(),
            ));
        }
    }

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let geofence_repo = AdminGeofenceRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can update geofences)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Update geofence
    let geofence = geofence_repo
        .update_geofence(
            org_id,
            geofence_id,
            request.name.as_deref(),
            request.description.as_deref(),
            request.latitude,
            request.longitude,
            request.radius_meters,
            request.event_types.as_deref(),
            request.active,
            request.color.as_deref(),
            request.metadata.as_ref(),
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("Geofence not found".to_string()))?;

    info!(
        org_id = %org_id,
        geofence_id = %geofence_id,
        user_id = %user.user_id,
        "Updated admin geofence"
    );

    let response = UpdateAdminGeofenceResponse {
        geofence: AdminGeofenceInfo {
            id: geofence.geofence_id,
            organization_id: geofence.organization_id,
            name: geofence.name,
            description: geofence.description,
            latitude: geofence.latitude,
            longitude: geofence.longitude,
            radius_meters: geofence.radius_meters,
            event_types: geofence.event_types,
            active: geofence.active,
            color: geofence.color,
            metadata: geofence.metadata,
            created_by: geofence.created_by,
            creator_name: None, // Not available from update
            created_at: geofence.created_at,
            updated_at: geofence.updated_at,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Delete an admin geofence.
///
/// DELETE /api/admin/v1/organizations/{org_id}/geofences/{geofence_id}
#[axum::debug_handler]
async fn delete_geofence(
    State(state): State<AppState>,
    Path((org_id, geofence_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let geofence_repo = AdminGeofenceRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can delete geofences)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Delete geofence
    let deleted = geofence_repo.delete_geofence(org_id, geofence_id).await?;

    if !deleted {
        return Err(ApiError::NotFound("Geofence not found".to_string()));
    }

    info!(
        org_id = %org_id,
        geofence_id = %geofence_id,
        user_id = %user.user_id,
        "Deleted admin geofence"
    );

    let response = DeleteAdminGeofenceResponse {
        deleted: true,
        geofence_id,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// List geofence events for organization.
///
/// GET /api/admin/v1/organizations/{org_id}/geofence-events
#[axum::debug_handler]
async fn list_geofence_events(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<AdminGeofenceEventsQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Validate query
    query
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let geofence_repo = AdminGeofenceRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view geofence events)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get pagination params
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);
    let offset = (page - 1) * per_page;

    // Convert timestamps
    let from_timestamp = query
        .from_timestamp
        .and_then(|ts| Utc.timestamp_millis_opt(ts).single());
    let to_timestamp = query
        .to_timestamp
        .and_then(|ts| Utc.timestamp_millis_opt(ts).single());

    // Get total count
    let total = geofence_repo
        .count_geofence_events(
            org_id,
            query.device_id,
            query.geofence_id,
            query.event_type.as_deref(),
            from_timestamp,
            to_timestamp,
        )
        .await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    // Get events
    let events = geofence_repo
        .list_geofence_events(
            org_id,
            query.device_id,
            query.geofence_id,
            query.event_type.as_deref(),
            from_timestamp,
            to_timestamp,
            per_page,
            offset,
        )
        .await?;

    // Map to response
    let data: Vec<AdminGeofenceEventInfo> = events
        .into_iter()
        .map(|e| AdminGeofenceEventInfo {
            id: e.event_id,
            device_id: e.device_id,
            device_name: e.device_name,
            geofence_id: e.geofence_id,
            geofence_name: e.geofence_name.unwrap_or_default(),
            event_type: e.event_type,
            latitude: e.latitude,
            longitude: e.longitude,
            timestamp: e.timestamp,
            created_at: e.created_at,
        })
        .collect();

    info!(
        org_id = %org_id,
        user_id = %user.user_id,
        event_count = data.len(),
        "Listed geofence events"
    );

    let response = AdminGeofenceEventsResponse {
        events: data,
        pagination: AdminGeofencePagination {
            page,
            per_page,
            total,
            total_pages,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get location analytics for organization.
///
/// GET /api/admin/v1/organizations/{org_id}/location-analytics
#[axum::debug_handler]
async fn get_location_analytics(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let geofence_repo = AdminGeofenceRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view analytics)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get analytics summary
    let analytics = geofence_repo.get_location_analytics(org_id).await?;

    // Get most visited geofences (top 10)
    let most_visited = geofence_repo.get_most_visited_geofences(org_id, 10).await?;

    let most_visited_geofences: Vec<GeofenceVisitCount> = most_visited
        .into_iter()
        .map(|v| GeofenceVisitCount {
            geofence_id: v.geofence_id,
            geofence_name: v.geofence_name,
            visit_count: v.visit_count,
        })
        .collect();

    info!(
        org_id = %org_id,
        user_id = %user.user_id,
        total_devices = analytics.total_devices,
        total_geofences = analytics.total_geofences,
        "Retrieved location analytics"
    );

    let response = AdminLocationAnalyticsResponse {
        summary: LocationAnalyticsSummary {
            total_devices: analytics.total_devices,
            devices_with_location: analytics.devices_with_location,
            total_locations_today: analytics.total_locations_today,
            total_geofences: analytics.total_geofences,
            total_geofence_events_today: analytics.total_geofence_events_today,
            most_visited_geofences,
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
    fn test_geofence_events_router_creation() {
        let _router: Router<AppState> = geofence_events_router();
    }

    #[test]
    fn test_location_analytics_router_creation() {
        let _router: Router<AppState> = location_analytics_router();
    }
}
