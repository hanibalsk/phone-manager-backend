//! Organization role management route handlers.
//!
//! Story AP-1.2: Create Custom Role
//! Story AP-1.3: Delete Custom Role

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::Utc;
use persistence::repositories::{
    AuditLogRepository, OrgUserRepository, OrganizationRoleRepository,
};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

use domain::models::{
    is_system_role_name, validate_permissions, AuditAction, CreateAuditLogInput,
    CreateOrganizationRoleRequest, DeleteOrganizationRoleResponse, ListOrganizationRolesQuery,
    ListOrganizationRolesResponse, OrgUserRole, OrganizationRoleResponse,
};

/// Create organization role routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_roles).post(create_role))
        .route("/{role_id}", get(get_role).delete(delete_role))
}

/// List all roles for an organization.
///
/// GET /api/admin/v1/organizations/{org_id}/roles
#[axum::debug_handler]
async fn list_roles(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListOrganizationRolesQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let role_repo = OrganizationRoleRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view roles)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get roles based on query filters
    let all_roles = role_repo
        .list(org_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<OrganizationRoleResponse> = if query.system_only == Some(true) {
        all_roles
            .into_iter()
            .filter(|r| r.is_system_role)
            .map(OrganizationRoleResponse::from)
            .collect()
    } else if query.custom_only == Some(true) {
        all_roles
            .into_iter()
            .filter(|r| !r.is_system_role)
            .map(OrganizationRoleResponse::from)
            .collect()
    } else {
        all_roles
            .iter()
            .map(|r| OrganizationRoleResponse::from(r.clone()))
            .collect()
    };

    let system_roles: Vec<OrganizationRoleResponse> = role_repo
        .list_system_roles(org_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .into_iter()
        .map(OrganizationRoleResponse::from)
        .collect();

    let custom_roles: Vec<OrganizationRoleResponse> = role_repo
        .list_custom_roles(org_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .into_iter()
        .map(OrganizationRoleResponse::from)
        .collect();

    let response = ListOrganizationRolesResponse {
        data,
        system_roles,
        custom_roles,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get a specific role by ID.
///
/// GET /api/admin/v1/organizations/{org_id}/roles/{role_id}
#[axum::debug_handler]
async fn get_role(
    State(state): State<AppState>,
    Path((org_id, role_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let role_repo = OrganizationRoleRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get the role
    let role = role_repo
        .find_by_id(role_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Role not found".to_string()))?;

    // Verify role belongs to the organization
    if role.organization_id != org_id {
        return Err(ApiError::NotFound("Role not found".to_string()));
    }

    Ok((StatusCode::OK, Json(OrganizationRoleResponse::from(role))))
}

/// Create a new custom role.
///
/// POST /api/admin/v1/organizations/{org_id}/roles
#[axum::debug_handler]
async fn create_role(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    user: UserAuth,
    Json(request): Json<CreateOrganizationRoleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Validate permissions
    validate_permissions(&request.permissions).map_err(ApiError::Validation)?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let role_repo = OrganizationRoleRepository::new(state.pool.clone());
    let audit_repo = AuditLogRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Only owners can create roles
    if org_user.role != OrgUserRole::Owner {
        return Err(ApiError::Forbidden(
            "Only organization owners can create roles".to_string(),
        ));
    }

    // Check if role name is a system role name
    if is_system_role_name(&request.name) {
        return Err(ApiError::Conflict(format!(
            "Cannot create role with system role name: {}",
            request.name
        )));
    }

    // Check if role name already exists
    if role_repo
        .find_by_name(org_id, &request.name)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .is_some()
    {
        return Err(ApiError::Conflict(format!(
            "Role with name '{}' already exists",
            request.name
        )));
    }

    // Create the role (priority defaults to 50 for custom roles)
    let role = role_repo
        .create(
            org_id,
            &request.name,
            &request.display_name,
            request.description.as_deref(),
            &request.permissions,
            50, // Default priority for custom roles
            Some(user.user_id),
        )
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Create audit log entry (fire and forget)
    let audit_input = CreateAuditLogInput::new(org_id, AuditAction::RoleCreated, "role")
        .with_user_actor(user.user_id, None)
        .with_resource_id(role.id.to_string())
        .with_resource_name(role.name.clone());

    audit_repo.insert_async(audit_input);

    Ok((
        StatusCode::CREATED,
        Json(OrganizationRoleResponse::from(role)),
    ))
}

/// Delete a custom role.
///
/// DELETE /api/admin/v1/organizations/{org_id}/roles/{role_id}
///
/// Returns 200 OK with deletion details on success.
/// Returns 403 if trying to delete a system role.
/// Returns 409 if the role has assigned users.
#[axum::debug_handler]
async fn delete_role(
    State(state): State<AppState>,
    Path((org_id, role_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let role_repo = OrganizationRoleRepository::new(state.pool.clone());
    let audit_repo = AuditLogRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Only owners can delete roles
    if org_user.role != OrgUserRole::Owner {
        return Err(ApiError::Forbidden(
            "Only organization owners can delete roles".to_string(),
        ));
    }

    // Get the role to check if it exists and is deletable
    let role = role_repo
        .find_by_id(role_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Role not found".to_string()))?;

    // Verify role belongs to the organization
    if role.organization_id != org_id {
        return Err(ApiError::NotFound("Role not found".to_string()));
    }

    // System roles cannot be deleted
    if role.is_system_role {
        return Err(ApiError::Forbidden(
            "System roles cannot be deleted".to_string(),
        ));
    }

    // Check if any users are assigned to this role
    let user_count = role_repo
        .count_users_with_role(org_id, &role.name)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    if user_count > 0 {
        return Err(ApiError::Conflict(format!(
            "Cannot delete role: {} user(s) are currently assigned to this role",
            user_count
        )));
    }

    // Delete the role
    let deleted = role_repo
        .delete(role_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    if !deleted {
        return Err(ApiError::NotFound("Role not found".to_string()));
    }

    // Create audit log entry (fire and forget)
    let audit_input = CreateAuditLogInput::new(org_id, AuditAction::RoleDeleted, "role")
        .with_user_actor(user.user_id, None)
        .with_resource_id(role_id.to_string())
        .with_resource_name(role.name.clone());

    audit_repo.insert_async(audit_input);

    // Return the response with deletion details
    let response = DeleteOrganizationRoleResponse {
        deleted: true,
        role_id,
        deleted_at: Utc::now(),
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
