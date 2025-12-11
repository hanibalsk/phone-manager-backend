//! Admin unlock request management route handlers.
//!
//! AP-8.4-8.8: Unlock Request Management Endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use persistence::entities::UnlockRequestStatusDb;
use persistence::repositories::{OrgUserRepository, UnlockRequestRepository};
use tracing::info;
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

use domain::models::{
    AdminListUnlockRequestsQuery, AdminListUnlockRequestsResponse, AdminUnlockPagination,
    AdminUnlockRequestActionResponse, AdminUnlockRequestItem, AdminUserBrief,
    ApproveUnlockRequestRequest, BulkProcessUnlockRequestsRequest,
    BulkProcessUnlockRequestsResponse, DenyUnlockRequestRequest, OrgUserRole, UnlockRequestStatus,
};

/// Create admin unlock request management routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_unlock_requests))
        .route("/{request_id}", get(get_unlock_request))
        .route("/{request_id}/approve", post(approve_unlock_request))
        .route("/{request_id}/deny", post(deny_unlock_request))
        .route("/bulk-process", post(bulk_process_unlock_requests))
}

/// List unlock requests for an organization.
///
/// GET /api/admin/v1/organizations/{org_id}/unlock-requests
#[axum::debug_handler]
async fn list_unlock_requests(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<AdminListUnlockRequestsQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let unlock_repo = UnlockRequestRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view unlock requests)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Parse status filter
    let status_filter = query.status.as_deref().and_then(|s| match s {
        "pending" => Some(UnlockRequestStatusDb::Pending),
        "approved" => Some(UnlockRequestStatusDb::Approved),
        "denied" => Some(UnlockRequestStatusDb::Denied),
        "expired" => Some(UnlockRequestStatusDb::Expired),
        _ => None,
    });

    // Get pagination params
    let page = query.page;
    let per_page = query.per_page;
    let limit = per_page as i64;
    let offset = ((page - 1) * per_page) as i64;

    // Get total count
    let total = unlock_repo
        .count_for_organization(org_id, status_filter, query.device_id)
        .await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    // Get unlock requests
    let entities = unlock_repo
        .list_for_organization(org_id, status_filter, query.device_id, limit, offset)
        .await?;

    // Map to response format
    let requests: Vec<AdminUnlockRequestItem> = entities
        .into_iter()
        .map(|e| AdminUnlockRequestItem {
            id: e.id,
            device_id: e.device_id,
            device_name: e.device_display_name,
            setting_key: e.setting_key,
            setting_name: e.setting_display_name,
            status: match e.status {
                UnlockRequestStatusDb::Pending => UnlockRequestStatus::Pending,
                UnlockRequestStatusDb::Approved => UnlockRequestStatus::Approved,
                UnlockRequestStatusDb::Denied => UnlockRequestStatus::Denied,
                UnlockRequestStatusDb::Expired => UnlockRequestStatus::Expired,
            },
            requested_by: AdminUserBrief {
                id: e.requested_by,
                display_name: e.requester_display_name,
            },
            reason: e.reason,
            responded_by: e.responded_by.map(|id| AdminUserBrief {
                id,
                display_name: e.responder_display_name,
            }),
            response_note: e.response_note,
            created_at: e.created_at,
            expires_at: e.expires_at,
            responded_at: e.responded_at,
        })
        .collect();

    info!(
        org_id = %org_id,
        user_id = %user.user_id,
        request_count = requests.len(),
        total = total,
        "Listed unlock requests"
    );

    let response = AdminListUnlockRequestsResponse {
        requests,
        pagination: AdminUnlockPagination {
            page,
            per_page,
            total,
            total_pages,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get a single unlock request by ID.
///
/// GET /api/admin/v1/organizations/{org_id}/unlock-requests/{request_id}
#[axum::debug_handler]
async fn get_unlock_request(
    State(state): State<AppState>,
    Path((org_id, request_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let unlock_repo = UnlockRequestRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view unlock requests)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get unlock request
    let entity = unlock_repo
        .find_by_id_for_organization(request_id, org_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Unlock request not found".to_string()))?;

    let response = AdminUnlockRequestItem {
        id: entity.id,
        device_id: entity.device_id,
        device_name: entity.device_display_name,
        setting_key: entity.setting_key,
        setting_name: entity.setting_display_name,
        status: match entity.status {
            UnlockRequestStatusDb::Pending => UnlockRequestStatus::Pending,
            UnlockRequestStatusDb::Approved => UnlockRequestStatus::Approved,
            UnlockRequestStatusDb::Denied => UnlockRequestStatus::Denied,
            UnlockRequestStatusDb::Expired => UnlockRequestStatus::Expired,
        },
        requested_by: AdminUserBrief {
            id: entity.requested_by,
            display_name: entity.requester_display_name,
        },
        reason: entity.reason,
        responded_by: entity.responded_by.map(|id| AdminUserBrief {
            id,
            display_name: entity.responder_display_name,
        }),
        response_note: entity.response_note,
        created_at: entity.created_at,
        expires_at: entity.expires_at,
        responded_at: entity.responded_at,
    };

    info!(
        org_id = %org_id,
        request_id = %request_id,
        user_id = %user.user_id,
        "Retrieved unlock request"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Approve an unlock request.
///
/// POST /api/admin/v1/organizations/{org_id}/unlock-requests/{request_id}/approve
#[axum::debug_handler]
async fn approve_unlock_request(
    State(state): State<AppState>,
    Path((org_id, request_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
    Json(request): Json<ApproveUnlockRequestRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let unlock_repo = UnlockRequestRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can approve unlock requests)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify unlock request exists and belongs to organization
    let existing = unlock_repo
        .find_by_id_for_organization(request_id, org_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Unlock request not found".to_string()))?;

    // Check if already responded
    if existing.status != UnlockRequestStatusDb::Pending {
        return Err(ApiError::Conflict(format!(
            "Request already {:?}",
            existing.status
        )));
    }

    // Approve the request
    let entity = unlock_repo
        .respond(
            request_id,
            UnlockRequestStatusDb::Approved,
            user.user_id,
            request.note.as_deref(),
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("Unlock request not found or already responded".to_string()))?;

    info!(
        org_id = %org_id,
        request_id = %request_id,
        user_id = %user.user_id,
        "Approved unlock request"
    );

    let response = AdminUnlockRequestActionResponse {
        id: entity.id,
        status: UnlockRequestStatus::Approved,
        responded_by: user.user_id,
        responded_at: entity.responded_at.unwrap_or_else(chrono::Utc::now),
        note: entity.response_note,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Deny an unlock request.
///
/// POST /api/admin/v1/organizations/{org_id}/unlock-requests/{request_id}/deny
#[axum::debug_handler]
async fn deny_unlock_request(
    State(state): State<AppState>,
    Path((org_id, request_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
    Json(request): Json<DenyUnlockRequestRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let unlock_repo = UnlockRequestRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can deny unlock requests)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify unlock request exists and belongs to organization
    let existing = unlock_repo
        .find_by_id_for_organization(request_id, org_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Unlock request not found".to_string()))?;

    // Check if already responded
    if existing.status != UnlockRequestStatusDb::Pending {
        return Err(ApiError::Conflict(format!(
            "Request already {:?}",
            existing.status
        )));
    }

    // Deny the request
    let entity = unlock_repo
        .respond(
            request_id,
            UnlockRequestStatusDb::Denied,
            user.user_id,
            request.note.as_deref(),
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("Unlock request not found or already responded".to_string()))?;

    info!(
        org_id = %org_id,
        request_id = %request_id,
        user_id = %user.user_id,
        "Denied unlock request"
    );

    let response = AdminUnlockRequestActionResponse {
        id: entity.id,
        status: UnlockRequestStatus::Denied,
        responded_by: user.user_id,
        responded_at: entity.responded_at.unwrap_or_else(chrono::Utc::now),
        note: entity.response_note,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Bulk process unlock requests.
///
/// POST /api/admin/v1/organizations/{org_id}/unlock-requests/bulk-process
#[axum::debug_handler]
async fn bulk_process_unlock_requests(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    user: UserAuth,
    Json(request): Json<BulkProcessUnlockRequestsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let unlock_repo = UnlockRequestRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can bulk process unlock requests)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Validate action
    let status = match request.action.as_str() {
        "approve" => UnlockRequestStatusDb::Approved,
        "deny" => UnlockRequestStatusDb::Denied,
        _ => {
            return Err(ApiError::Validation(
                "Invalid action. Must be 'approve' or 'deny'".to_string(),
            ));
        }
    };

    // Validate request_ids not empty
    if request.request_ids.is_empty() {
        return Err(ApiError::Validation(
            "request_ids cannot be empty".to_string(),
        ));
    }

    // Limit bulk operations
    const MAX_BULK_REQUESTS: usize = 100;
    if request.request_ids.len() > MAX_BULK_REQUESTS {
        return Err(ApiError::Validation(format!(
            "Cannot process more than {} requests at once",
            MAX_BULK_REQUESTS
        )));
    }

    // Bulk process
    let processed = unlock_repo
        .bulk_respond(
            &request.request_ids,
            org_id,
            status,
            user.user_id,
            request.note.as_deref(),
        )
        .await?;

    info!(
        org_id = %org_id,
        user_id = %user.user_id,
        action = %request.action,
        requested = request.request_ids.len(),
        processed = processed,
        "Bulk processed unlock requests"
    );

    let response = BulkProcessUnlockRequestsResponse {
        processed,
        requested: request.request_ids.len(),
        action: request.action,
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
}
