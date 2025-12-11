//! Admin group management route handlers.
//!
//! Story 14.4: Group Management Admin Endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use persistence::repositories::{AdminGroupRepository, OrgUserRepository};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

use domain::models::{
    AddGroupMemberRequest, AddGroupMemberResponse, AdminGroupDetailResponse,
    AdminGroupListResponse, AdminGroupPagination, AdminGroupQuery, AdminSortOrder,
    DeactivateGroupResponse, GroupMembersPagination, ListGroupMembersQuery,
    ListGroupMembersResponse, OrgUserRole, RemoveGroupMemberResponse, UpdateAdminGroupRequest,
    UpdateAdminGroupResponse,
};

/// Create admin group management routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_groups))
        .route("/{group_id}", get(get_group_detail))
        .route("/{group_id}", put(update_group))
        .route("/{group_id}", delete(deactivate_group))
        .route("/{group_id}/members", get(list_group_members))
        .route("/{group_id}/members", post(add_group_member))
        .route("/{group_id}/members/{member_id}", delete(remove_group_member))
}

/// List groups in organization.
///
/// GET /api/admin/v1/organizations/{org_id}/groups
#[axum::debug_handler]
async fn list_groups(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<AdminGroupQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Validate query
    query
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let admin_group_repo = AdminGroupRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view groups)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get pagination params
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);

    // Get summary counts
    let summary = admin_group_repo.get_group_summary(org_id).await?;

    // Get total count for pagination
    let total = admin_group_repo
        .count_groups(
            org_id,
            query.active,
            query.has_devices,
            query.search.as_deref(),
        )
        .await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    // Get sort options with defaults
    let sort_field = query.sort.unwrap_or_default();
    let sort_order = query.order.unwrap_or(AdminSortOrder::Desc);

    // Calculate offset
    let offset = (page - 1) * per_page;

    // Fetch groups with filtering, sorting, and pagination
    let data = admin_group_repo
        .list_groups(
            org_id,
            query.active,
            query.has_devices,
            query.search.as_deref(),
            sort_field,
            sort_order,
            per_page,
            offset,
        )
        .await?;

    let response = AdminGroupListResponse {
        data,
        pagination: AdminGroupPagination {
            page,
            per_page,
            total,
            total_pages,
        },
        summary,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get group detail.
///
/// GET /api/admin/v1/organizations/{org_id}/groups/{group_id}
#[axum::debug_handler]
async fn get_group_detail(
    State(state): State<AppState>,
    Path((org_id, group_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let admin_group_repo = AdminGroupRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view group details)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get group profile
    let group = admin_group_repo
        .get_group_profile(org_id, group_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found in organization".to_string()))?;

    // Get group members
    let members = admin_group_repo.get_group_members(group_id).await?;

    // Get group devices (using slug as group_id in devices table)
    let devices = admin_group_repo.get_group_devices(&group.slug).await?;

    let response = AdminGroupDetailResponse {
        group,
        members,
        devices,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Update group settings.
///
/// PUT /api/admin/v1/organizations/{org_id}/groups/{group_id}
#[axum::debug_handler]
async fn update_group(
    State(state): State<AppState>,
    Path((org_id, group_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
    Json(request): Json<UpdateAdminGroupRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let admin_group_repo = AdminGroupRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can update groups)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify group belongs to organization
    if !admin_group_repo
        .group_belongs_to_org(org_id, group_id)
        .await?
    {
        return Err(ApiError::NotFound(
            "Group not found in organization".to_string(),
        ));
    }

    // Update group
    let updated = admin_group_repo
        .update_group(
            group_id,
            request.name.as_deref(),
            request.description.as_deref(),
            request.max_devices,
            request.is_active,
        )
        .await?;

    if !updated {
        return Err(ApiError::NotFound("Group not found".to_string()));
    }

    // Get updated group profile
    let group = admin_group_repo
        .get_group_profile(org_id, group_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found".to_string()))?;

    let response = UpdateAdminGroupResponse {
        id: group.id,
        name: group.name,
        description: group.description,
        max_devices: group.max_devices,
        is_active: group.is_active,
        updated_at: Utc::now(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Deactivate group.
///
/// DELETE /api/admin/v1/organizations/{org_id}/groups/{group_id}
#[axum::debug_handler]
async fn deactivate_group(
    State(state): State<AppState>,
    Path((org_id, group_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let admin_group_repo = AdminGroupRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can deactivate groups)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify group belongs to organization
    if !admin_group_repo
        .group_belongs_to_org(org_id, group_id)
        .await?
    {
        return Err(ApiError::NotFound(
            "Group not found in organization".to_string(),
        ));
    }

    // Deactivate group
    let deactivated = admin_group_repo.deactivate_group(group_id).await?;

    if !deactivated {
        return Err(ApiError::NotFound(
            "Group not found or already deactivated".to_string(),
        ));
    }

    let response = DeactivateGroupResponse {
        deactivated: true,
        group_id,
        deactivated_at: Utc::now(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// List members of a group.
///
/// GET /api/admin/v1/organizations/{org_id}/groups/{group_id}/members
#[axum::debug_handler]
async fn list_group_members(
    State(state): State<AppState>,
    Path((org_id, group_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<ListGroupMembersQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Validate query
    query
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let admin_group_repo = AdminGroupRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view group members)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify group belongs to organization
    if !admin_group_repo
        .group_belongs_to_org(org_id, group_id)
        .await?
    {
        return Err(ApiError::NotFound(
            "Group not found in organization".to_string(),
        ));
    }

    // Get pagination params
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);
    let offset = (page - 1) * per_page;

    // Get total count
    let total = admin_group_repo
        .count_group_members(group_id, query.search.as_deref(), query.role.as_deref())
        .await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    // Get members
    let data = admin_group_repo
        .list_group_members_paginated(
            group_id,
            query.search.as_deref(),
            query.role.as_deref(),
            per_page,
            offset,
        )
        .await?;

    let response = ListGroupMembersResponse {
        data,
        pagination: GroupMembersPagination {
            page,
            per_page,
            total,
            total_pages,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Add a member to a group.
///
/// POST /api/admin/v1/organizations/{org_id}/groups/{group_id}/members
#[axum::debug_handler]
async fn add_group_member(
    State(state): State<AppState>,
    Path((org_id, group_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
    Json(request): Json<AddGroupMemberRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let admin_group_repo = AdminGroupRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can add members)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify group belongs to organization
    if !admin_group_repo
        .group_belongs_to_org(org_id, group_id)
        .await?
    {
        return Err(ApiError::NotFound(
            "Group not found in organization".to_string(),
        ));
    }

    // Verify target user is in the same organization
    if !admin_group_repo.user_in_org(org_id, request.user_id).await? {
        return Err(ApiError::Validation(
            "User must be a member of the organization".to_string(),
        ));
    }

    // Check if user is already a member
    if admin_group_repo
        .is_group_member(group_id, request.user_id)
        .await?
    {
        return Err(ApiError::Conflict(
            "User is already a member of this group".to_string(),
        ));
    }

    // Add member
    let member = admin_group_repo
        .add_group_member(group_id, request.user_id, &request.role)
        .await?;

    let response = AddGroupMemberResponse {
        group_id,
        user_id: member.user_id,
        role: member.role,
        joined_at: member.joined_at,
        message: "Member added successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Remove a member from a group.
///
/// DELETE /api/admin/v1/organizations/{org_id}/groups/{group_id}/members/{member_id}
#[axum::debug_handler]
async fn remove_group_member(
    State(state): State<AppState>,
    Path((org_id, group_id, member_id)): Path<(Uuid, Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let admin_group_repo = AdminGroupRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can remove members)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify group belongs to organization
    if !admin_group_repo
        .group_belongs_to_org(org_id, group_id)
        .await?
    {
        return Err(ApiError::NotFound(
            "Group not found in organization".to_string(),
        ));
    }

    // Check if member exists
    if !admin_group_repo.is_group_member(group_id, member_id).await? {
        return Err(ApiError::NotFound("Member not found in group".to_string()));
    }

    // Prevent removing the owner (unless by another owner or higher-level admin)
    if admin_group_repo.is_group_owner(group_id, member_id).await? {
        // Only allow owner removal if there's another owner or the current user is org owner
        if org_user.role != OrgUserRole::Owner {
            return Err(ApiError::Forbidden(
                "Only organization owners can remove group owners".to_string(),
            ));
        }
    }

    // Remove member
    let removed = admin_group_repo
        .remove_group_member(group_id, member_id)
        .await?;

    let response = RemoveGroupMemberResponse {
        group_id,
        user_id: member_id,
        removed,
        removed_at: Utc::now(),
        message: if removed {
            "Member removed successfully".to_string()
        } else {
            "Member not found".to_string()
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
}
