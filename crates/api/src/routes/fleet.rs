//! Fleet management route handlers.
//!
//! Story 13.7: Fleet Management Endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use persistence::repositories::{
    DeviceCommandRepository, DeviceRepository, OrgUserRepository, UserRepository,
};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

use domain::models::{
    AssignDeviceRequest, AssignDeviceResponse, AssignedUserInfo, DeviceCommandStatus,
    DeviceCommandType, DeviceStatusChangeResponse, EnrollmentStatus, FleetDeviceListResponse,
    FleetDeviceQuery, FleetPagination, FleetSummary, IssueCommandRequest, IssueCommandResponse,
    OrgUserRole, UnassignDeviceResponse,
};

/// Create fleet management routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_fleet_devices))
        .route("/{device_id}/assign", post(assign_device))
        .route("/{device_id}/unassign", post(unassign_device))
        .route("/{device_id}/suspend", post(suspend_device))
        .route("/{device_id}/retire", post(retire_device))
        .route("/{device_id}/wipe", post(wipe_device))
}

/// List all devices in organization fleet.
///
/// GET /api/admin/v1/organizations/{org_id}/devices
#[axum::debug_handler]
async fn list_fleet_devices(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<FleetDeviceQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Validate query
    query
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view fleet)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get pagination params
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);

    // Get summary counts
    let summary_counts = device_repo.get_fleet_summary(org_id).await?;
    let summary = FleetSummary {
        enrolled: summary_counts.enrolled,
        pending: summary_counts.pending,
        suspended: summary_counts.suspended,
        retired: summary_counts.retired,
        assigned: summary_counts.assigned,
        unassigned: summary_counts.unassigned,
    };

    // Get total count for pagination
    let status_str = query.status.as_ref().map(|s| s.as_str());
    let total = device_repo
        .count_fleet_devices(
            org_id,
            status_str,
            query.group_id.as_deref(),
            query.policy_id,
            query.assigned,
            query.search.as_deref(),
        )
        .await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    // Get sort options with defaults
    let sort_field = query.sort.unwrap_or_default();
    let sort_order = query.order.unwrap_or_default();

    // Calculate offset
    let offset = (page - 1) * per_page;

    // Fetch devices with filtering, sorting, and pagination
    let data = device_repo
        .list_fleet_devices(
            org_id,
            status_str,
            query.group_id.as_deref(),
            query.policy_id,
            query.assigned,
            query.search.as_deref(),
            sort_field,
            sort_order,
            per_page,
            offset,
        )
        .await?;

    let response = FleetDeviceListResponse {
        data,
        pagination: FleetPagination {
            page,
            per_page,
            total,
            total_pages,
        },
        summary,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Assign a user to a device.
///
/// POST /api/admin/v1/organizations/{org_id}/devices/{device_id}/assign
#[axum::debug_handler]
async fn assign_device(
    State(state): State<AppState>,
    Path((org_id, device_id)): Path<(Uuid, i64)>,
    user: UserAuth,
    Json(request): Json<AssignDeviceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());
    let user_repo = UserRepository::new(state.pool.clone());

    // Verify user has admin access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify target user is in the organization
    let _target_org_user = org_user_repo
        .find_by_org_and_user(org_id, request.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Assign user to device
    let _updated_device = device_repo.assign_user(device_id, request.user_id).await?;

    // Get user details
    let user_entity = user_repo
        .find_by_id(request.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let response = AssignDeviceResponse {
        device_id,
        assigned_user: AssignedUserInfo {
            id: request.user_id,
            email: user_entity.email,
            display_name: user_entity.display_name,
        },
        assigned_at: Utc::now(),
        notification_sent: false, // Notification sending deferred
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Unassign user from a device.
///
/// POST /api/admin/v1/organizations/{org_id}/devices/{device_id}/unassign
#[axum::debug_handler]
async fn unassign_device(
    State(state): State<AppState>,
    Path((org_id, device_id)): Path<(Uuid, i64)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());

    // Verify user has admin access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Unassign user
    let _updated_device = device_repo.unassign_user(device_id).await?;

    let response = UnassignDeviceResponse {
        device_id,
        unassigned_at: Utc::now(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Suspend a device.
///
/// POST /api/admin/v1/organizations/{org_id}/devices/{device_id}/suspend
#[axum::debug_handler]
async fn suspend_device(
    State(state): State<AppState>,
    Path((org_id, device_id)): Path<(Uuid, i64)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());

    // Verify user has admin access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get current status
    let current_status = device_repo.get_enrollment_status(device_id).await?;
    let previous_status = current_status.as_deref().and_then(|s| s.parse().ok());

    // Check if device can be suspended (not retired)
    if current_status.as_deref() == Some("retired") {
        return Err(ApiError::Conflict(
            "Cannot suspend a retired device".to_string(),
        ));
    }

    // Update status to suspended
    let _updated_device = device_repo
        .update_enrollment_status(device_id, "suspended")
        .await?;

    let response = DeviceStatusChangeResponse {
        device_id,
        previous_status,
        new_status: EnrollmentStatus::Suspended,
        changed_at: Utc::now(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Retire a device (permanent).
///
/// POST /api/admin/v1/organizations/{org_id}/devices/{device_id}/retire
#[axum::debug_handler]
async fn retire_device(
    State(state): State<AppState>,
    Path((org_id, device_id)): Path<(Uuid, i64)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());

    // Verify user has admin access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get current status
    let current_status = device_repo.get_enrollment_status(device_id).await?;
    let previous_status = current_status.as_deref().and_then(|s| s.parse().ok());

    // Check if already retired
    if current_status.as_deref() == Some("retired") {
        return Err(ApiError::Conflict("Device is already retired".to_string()));
    }

    // Update status to retired
    let _updated_device = device_repo
        .update_enrollment_status(device_id, "retired")
        .await?;

    let response = DeviceStatusChangeResponse {
        device_id,
        previous_status,
        new_status: EnrollmentStatus::Retired,
        changed_at: Utc::now(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Issue wipe command to a device.
///
/// POST /api/admin/v1/organizations/{org_id}/devices/{device_id}/wipe
#[axum::debug_handler]
async fn wipe_device(
    State(state): State<AppState>,
    Path((org_id, device_id)): Path<(Uuid, i64)>,
    user: UserAuth,
    Json(request): Json<Option<IssueCommandRequest>>,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let device_command_repo = DeviceCommandRepository::new(state.pool.clone());

    // Verify user has admin access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    let req = request.unwrap_or(IssueCommandRequest {
        payload: None,
        expires_in_hours: None,
    });

    let expires_in_hours = req.expires_in_hours.unwrap_or(24);

    // Create wipe command
    let command = device_command_repo
        .create(
            device_id,
            org_id,
            DeviceCommandType::Wipe.as_str(),
            req.payload.as_ref(),
            user.user_id,
            expires_in_hours,
        )
        .await?;

    let expires_at = Utc::now() + chrono::Duration::hours(expires_in_hours as i64);

    let response = IssueCommandResponse {
        command_id: command.id,
        device_id,
        command_type: DeviceCommandType::Wipe,
        status: DeviceCommandStatus::Pending,
        issued_at: command.issued_at,
        expires_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let _router: Router<AppState> = router();
    }
}
