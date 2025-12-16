//! Admin managed users route handlers.
//!
//! Epic 9: Admin User Management
//!
//! Provides endpoints for admins to manage users:
//! - Org admins can manage users in their organizations
//! - Non-org admins can manage users not in any organization

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use persistence::repositories::{ManagedUserRepository, UserGeofenceRepository};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

use domain::models::geofence::GeofenceEventType;
use domain::models::{
    CreateUserGeofenceRequest, CreateUserGeofenceResponse, DeleteUserGeofenceResponse,
    ListManagedUsersQuery, ListManagedUsersResponse, ListUserGeofencesResponse, ManagedUser,
    ManagedUserPagination, RemoveManagedUserResponse, UpdateTrackingRequest, UpdateTrackingResponse,
    UpdateUserGeofenceRequest, UpdateUserGeofenceResponse, UserGeofence, UserLastLocation,
};

/// Maximum number of geofences per user.
const MAX_GEOFENCES_PER_USER: i64 = 50;

/// Create admin managed users routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_managed_users))
        .route("/{user_id}", delete(remove_managed_user))
        .route("/{user_id}/location", get(get_user_location))
        .route("/{user_id}/geofences", get(list_user_geofences))
        .route("/{user_id}/geofences", post(create_user_geofence))
        .route(
            "/{user_id}/geofences/{geofence_id}",
            put(update_user_geofence),
        )
        .route(
            "/{user_id}/geofences/{geofence_id}",
            delete(delete_user_geofence),
        )
        .route("/{user_id}/tracking", put(update_tracking))
}

/// List managed users.
///
/// GET /api/admin/v1/users
#[axum::debug_handler]
async fn list_managed_users(
    State(state): State<AppState>,
    Query(query): Query<ListManagedUsersQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    query
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let managed_user_repo = ManagedUserRepository::new(state.pool.clone());

    // Get organizations where admin has admin/owner role
    let org_ids = managed_user_repo.get_admin_org_ids(user.user_id).await?;

    let page = query.page;
    let per_page = query.per_page;
    let offset = (page - 1) * per_page;

    let total = managed_user_repo
        .count_managed_users(&org_ids, query.search.as_deref(), query.tracking_enabled)
        .await?;

    let total_pages = if total == 0 {
        0
    } else {
        ((total as f64) / (per_page as f64)).ceil() as u32
    };

    let entities = managed_user_repo
        .list_managed_users(&org_ids, query.search.as_deref(), query.tracking_enabled, per_page, offset)
        .await?;

    let users: Vec<ManagedUser> = entities
        .into_iter()
        .map(|e| {
            let last_location = if let (Some(device_id), Some(lat), Some(lon)) =
                (e.last_device_id, e.last_latitude, e.last_longitude)
            {
                Some(UserLastLocation {
                    device_id,
                    device_name: e.last_device_name.unwrap_or_default(),
                    latitude: lat,
                    longitude: lon,
                    accuracy: e.last_accuracy.unwrap_or(0.0),
                    captured_at: e.last_captured_at.unwrap_or(e.created_at),
                })
            } else {
                None
            };

            ManagedUser {
                id: e.id,
                email: e.email,
                display_name: e.display_name,
                tracking_enabled: e.tracking_enabled,
                device_count: e.device_count,
                last_location,
                organization_id: e.organization_id,
                organization_name: e.organization_name,
                created_at: e.created_at,
            }
        })
        .collect();

    info!(
        admin_id = %user.user_id,
        user_count = users.len(),
        org_count = org_ids.len(),
        "Listed managed users"
    );

    Ok((
        StatusCode::OK,
        Json(ListManagedUsersResponse {
            users,
            pagination: ManagedUserPagination {
                page,
                per_page,
                total,
                total_pages,
            },
        }),
    ))
}

/// Get user's current location.
///
/// GET /api/admin/v1/users/{user_id}/location
#[axum::debug_handler]
async fn get_user_location(
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let managed_user_repo = ManagedUserRepository::new(state.pool.clone());

    // Get organizations where admin has admin/owner role
    let org_ids = managed_user_repo.get_admin_org_ids(user.user_id).await?;

    // Verify admin can manage this user
    if !managed_user_repo
        .can_manage_user(target_user_id, &org_ids)
        .await?
    {
        return Err(ApiError::Forbidden(
            "You cannot manage this user".to_string(),
        ));
    }

    let location = managed_user_repo.get_user_location(target_user_id).await?;

    match location {
        Some(loc) => Ok((
            StatusCode::OK,
            Json(UserLastLocation {
                device_id: loc.device_id,
                device_name: loc.device_name,
                latitude: loc.latitude,
                longitude: loc.longitude,
                accuracy: loc.accuracy,
                captured_at: loc.captured_at,
            }),
        )),
        None => Err(ApiError::NotFound("No location data found".to_string())),
    }
}

/// List geofences for a user.
///
/// GET /api/admin/v1/users/{user_id}/geofences
#[axum::debug_handler]
async fn list_user_geofences(
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let managed_user_repo = ManagedUserRepository::new(state.pool.clone());
    let geofence_repo = UserGeofenceRepository::new(state.pool.clone());

    // Get organizations where admin has admin/owner role
    let org_ids = managed_user_repo.get_admin_org_ids(user.user_id).await?;

    // Verify admin can manage this user
    if !managed_user_repo
        .can_manage_user(target_user_id, &org_ids)
        .await?
    {
        return Err(ApiError::Forbidden(
            "You cannot manage this user".to_string(),
        ));
    }

    let entities = geofence_repo.list_by_user(target_user_id).await?;
    let total = entities.len() as i64;

    let geofences: Vec<UserGeofence> = entities
        .into_iter()
        .map(|e| UserGeofence {
            id: e.geofence_id,
            user_id: e.user_id,
            name: e.name,
            latitude: e.latitude,
            longitude: e.longitude,
            radius_meters: e.radius_meters,
            event_types: e
                .event_types
                .iter()
                .filter_map(|s| GeofenceEventType::parse(s))
                .collect(),
            active: e.active,
            color: e.color,
            metadata: e.metadata,
            created_by: e.created_by,
            created_by_name: e.created_by_name,
            created_at: e.created_at,
            updated_at: e.updated_at,
        })
        .collect();

    info!(
        admin_id = %user.user_id,
        target_user_id = %target_user_id,
        geofence_count = geofences.len(),
        "Listed user geofences"
    );

    Ok((
        StatusCode::OK,
        Json(ListUserGeofencesResponse { geofences, total }),
    ))
}

/// Create a geofence for a user.
///
/// POST /api/admin/v1/users/{user_id}/geofences
#[axum::debug_handler]
async fn create_user_geofence(
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
    user: UserAuth,
    Json(request): Json<CreateUserGeofenceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Validate event types
    for event_type in &request.event_types {
        if !matches!(
            event_type,
            GeofenceEventType::Enter | GeofenceEventType::Exit | GeofenceEventType::Dwell
        ) {
            return Err(ApiError::Validation(
                "Invalid event type. Must be enter, exit, or dwell".to_string(),
            ));
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

    let managed_user_repo = ManagedUserRepository::new(state.pool.clone());
    let geofence_repo = UserGeofenceRepository::new(state.pool.clone());

    // Get organizations where admin has admin/owner role
    let org_ids = managed_user_repo.get_admin_org_ids(user.user_id).await?;

    // Verify admin can manage this user
    if !managed_user_repo
        .can_manage_user(target_user_id, &org_ids)
        .await?
    {
        return Err(ApiError::Forbidden(
            "You cannot manage this user".to_string(),
        ));
    }

    // Check geofence limit
    let count = geofence_repo.count_by_user(target_user_id).await?;
    if count >= MAX_GEOFENCES_PER_USER {
        return Err(ApiError::Conflict(format!(
            "Maximum of {} geofences per user",
            MAX_GEOFENCES_PER_USER
        )));
    }

    // Convert event types to strings for database
    let event_type_strings: Vec<String> =
        request.event_types.iter().map(|e| e.as_str().to_string()).collect();

    let entity = geofence_repo
        .create(
            target_user_id,
            user.user_id,
            &request.name,
            request.latitude,
            request.longitude,
            request.radius_meters,
            &event_type_strings,
            request.color.as_deref(),
            request.metadata.as_ref(),
        )
        .await?;

    info!(
        admin_id = %user.user_id,
        target_user_id = %target_user_id,
        geofence_id = %entity.geofence_id,
        "Created user geofence"
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateUserGeofenceResponse {
            geofence: UserGeofence {
                id: entity.geofence_id,
                user_id: entity.user_id,
                name: entity.name,
                latitude: entity.latitude,
                longitude: entity.longitude,
                radius_meters: entity.radius_meters,
                event_types: entity
                    .event_types
                    .iter()
                    .filter_map(|s| GeofenceEventType::parse(s))
                    .collect(),
                active: entity.active,
                color: entity.color,
                metadata: entity.metadata,
                created_by: entity.created_by,
                created_by_name: entity.created_by_name,
                created_at: entity.created_at,
                updated_at: entity.updated_at,
            },
        }),
    ))
}

/// Update a user geofence.
///
/// PUT /api/admin/v1/users/{user_id}/geofences/{geofence_id}
#[axum::debug_handler]
async fn update_user_geofence(
    State(state): State<AppState>,
    Path((target_user_id, geofence_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
    Json(request): Json<UpdateUserGeofenceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Validate event types if provided
    if let Some(ref event_types) = request.event_types {
        for event_type in event_types {
            if !matches!(
                event_type,
                GeofenceEventType::Enter | GeofenceEventType::Exit | GeofenceEventType::Dwell
            ) {
                return Err(ApiError::Validation(
                    "Invalid event type. Must be enter, exit, or dwell".to_string(),
                ));
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

    let managed_user_repo = ManagedUserRepository::new(state.pool.clone());
    let geofence_repo = UserGeofenceRepository::new(state.pool.clone());

    // Get organizations where admin has admin/owner role
    let org_ids = managed_user_repo.get_admin_org_ids(user.user_id).await?;

    // Verify admin can manage this user
    if !managed_user_repo
        .can_manage_user(target_user_id, &org_ids)
        .await?
    {
        return Err(ApiError::Forbidden(
            "You cannot manage this user".to_string(),
        ));
    }

    // Verify geofence belongs to the user
    let existing = geofence_repo.find_by_id(geofence_id).await?;
    match existing {
        Some(g) if g.user_id != target_user_id => {
            return Err(ApiError::NotFound("Geofence not found".to_string()));
        }
        None => return Err(ApiError::NotFound("Geofence not found".to_string())),
        _ => {}
    }

    // Convert event types to strings for database if provided
    let event_type_strings: Option<Vec<String>> = request.event_types.as_ref().map(|types| {
        types.iter().map(|e| e.as_str().to_string()).collect()
    });

    let entity = geofence_repo
        .update(
            geofence_id,
            request.name.as_deref(),
            request.latitude,
            request.longitude,
            request.radius_meters,
            event_type_strings.as_deref(),
            request.active,
            request.color.as_deref(),
            request.metadata.as_ref(),
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("Geofence not found".to_string()))?;

    info!(
        admin_id = %user.user_id,
        target_user_id = %target_user_id,
        geofence_id = %geofence_id,
        "Updated user geofence"
    );

    Ok((
        StatusCode::OK,
        Json(UpdateUserGeofenceResponse {
            geofence: UserGeofence {
                id: entity.geofence_id,
                user_id: entity.user_id,
                name: entity.name,
                latitude: entity.latitude,
                longitude: entity.longitude,
                radius_meters: entity.radius_meters,
                event_types: entity
                    .event_types
                    .iter()
                    .filter_map(|s| GeofenceEventType::parse(s))
                    .collect(),
                active: entity.active,
                color: entity.color,
                metadata: entity.metadata,
                created_by: entity.created_by,
                created_by_name: entity.created_by_name,
                created_at: entity.created_at,
                updated_at: entity.updated_at,
            },
        }),
    ))
}

/// Delete a user geofence.
///
/// DELETE /api/admin/v1/users/{user_id}/geofences/{geofence_id}
#[axum::debug_handler]
async fn delete_user_geofence(
    State(state): State<AppState>,
    Path((target_user_id, geofence_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let managed_user_repo = ManagedUserRepository::new(state.pool.clone());
    let geofence_repo = UserGeofenceRepository::new(state.pool.clone());

    // Get organizations where admin has admin/owner role
    let org_ids = managed_user_repo.get_admin_org_ids(user.user_id).await?;

    // Verify admin can manage this user
    if !managed_user_repo
        .can_manage_user(target_user_id, &org_ids)
        .await?
    {
        return Err(ApiError::Forbidden(
            "You cannot manage this user".to_string(),
        ));
    }

    // Verify geofence belongs to the user
    let existing = geofence_repo.find_by_id(geofence_id).await?;
    match existing {
        Some(g) if g.user_id != target_user_id => {
            return Err(ApiError::NotFound("Geofence not found".to_string()));
        }
        None => return Err(ApiError::NotFound("Geofence not found".to_string())),
        _ => {}
    }

    let deleted = geofence_repo.delete(geofence_id).await?;

    info!(
        admin_id = %user.user_id,
        target_user_id = %target_user_id,
        geofence_id = %geofence_id,
        deleted = deleted,
        "Deleted user geofence"
    );

    Ok((
        StatusCode::OK,
        Json(DeleteUserGeofenceResponse {
            geofence_id,
            deleted,
        }),
    ))
}

/// Update user tracking status.
///
/// PUT /api/admin/v1/users/{user_id}/tracking
#[axum::debug_handler]
async fn update_tracking(
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
    user: UserAuth,
    Json(request): Json<UpdateTrackingRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let managed_user_repo = ManagedUserRepository::new(state.pool.clone());

    // Get organizations where admin has admin/owner role
    let org_ids = managed_user_repo.get_admin_org_ids(user.user_id).await?;

    // Verify admin can manage this user
    if !managed_user_repo
        .can_manage_user(target_user_id, &org_ids)
        .await?
    {
        return Err(ApiError::Forbidden(
            "You cannot manage this user".to_string(),
        ));
    }

    managed_user_repo
        .update_tracking(target_user_id, request.enabled)
        .await?;

    info!(
        admin_id = %user.user_id,
        target_user_id = %target_user_id,
        tracking_enabled = request.enabled,
        "Updated user tracking status"
    );

    Ok((
        StatusCode::OK,
        Json(UpdateTrackingResponse {
            user_id: target_user_id,
            tracking_enabled: request.enabled,
            updated_at: chrono::Utc::now(),
        }),
    ))
}

/// Remove a user from the managed list.
///
/// For org admins: removes user from their organization(s)
/// For non-org admins: deactivates the user account
///
/// DELETE /api/admin/v1/users/{user_id}
#[axum::debug_handler]
async fn remove_managed_user(
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let managed_user_repo = ManagedUserRepository::new(state.pool.clone());

    // Get organizations where admin has admin/owner role
    let org_ids = managed_user_repo.get_admin_org_ids(user.user_id).await?;

    // Verify admin can manage this user
    if !managed_user_repo
        .can_manage_user(target_user_id, &org_ids)
        .await?
    {
        return Err(ApiError::Forbidden(
            "You cannot manage this user".to_string(),
        ));
    }

    // For org admins: remove user from their organizations
    // For non-org admins: deactivate the user account
    let message = if !org_ids.is_empty() {
        // Remove from all admin's organizations
        let removed = managed_user_repo
            .remove_from_organizations(target_user_id, &org_ids)
            .await?;
        format!("User removed from {} organization(s)", removed)
    } else {
        // Deactivate user account
        managed_user_repo.deactivate_user(target_user_id).await?;
        "User account deactivated".to_string()
    };

    info!(
        admin_id = %user.user_id,
        target_user_id = %target_user_id,
        org_admin = !org_ids.is_empty(),
        "Removed managed user"
    );

    Ok((
        StatusCode::OK,
        Json(RemoveManagedUserResponse {
            user_id: target_user_id,
            removed: true,
            message,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let _router: Router<AppState> = router();
    }
}
