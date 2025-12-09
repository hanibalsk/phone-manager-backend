//! System role management route handlers.
//!
//! Provides endpoints for managing system-level roles and organization assignments.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use persistence::repositories::{OrganizationRepository, SystemRoleRepository, UserRepository};
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use crate::middleware::system_rbac::SystemRoleAuth;

use domain::models::{
    AddSystemRoleRequest, AddSystemRoleResponse, AssignOrgRequest, AssignOrgResponse,
    ListSystemRolesResponse, RemoveOrgAssignmentResponse, RemoveSystemRoleResponse, SystemRole,
    SystemRoleInfo, UserOrgAssignmentsResponse, UserSystemRolesResponse,
};

/// Create system roles routes.
///
/// These routes require super_admin role for management operations.
pub fn router() -> Router<AppState> {
    Router::new()
        // System roles metadata
        .route("/", get(list_system_roles))
        // User system roles management
        .route("/users/{user_id}/roles", get(get_user_system_roles))
        .route("/users/{user_id}/roles", post(add_system_role))
        .route("/users/{user_id}/roles/{role}", delete(remove_system_role))
        // User org assignments management
        .route("/users/{user_id}/org-assignments", get(get_org_assignments))
        .route("/users/{user_id}/org-assignments", post(assign_org))
        .route(
            "/users/{user_id}/org-assignments/{org_id}",
            delete(unassign_org),
        )
}

/// List available system roles.
///
/// GET /api/admin/v1/system-roles
///
/// Returns all available system roles with their descriptions and requirements.
/// Requires any system role.
#[axum::debug_handler(state = AppState)]
async fn list_system_roles(
    _system_auth: SystemRoleAuth,
) -> Result<impl IntoResponse, ApiError> {
    let roles: Vec<SystemRoleInfo> = SystemRole::all()
        .iter()
        .map(|&role| role.into())
        .collect();

    Ok((StatusCode::OK, Json(ListSystemRolesResponse { data: roles })))
}

/// Get a user's system roles.
///
/// GET /api/admin/v1/system-roles/users/{user_id}/roles
///
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn get_user_system_roles(
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
    system_auth: SystemRoleAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can view other users' system roles
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    let repo = SystemRoleRepository::new(state.pool.clone());

    // Verify target user exists
    let user_repo = UserRepository::new(state.pool.clone());
    if user_repo.find_by_id(target_user_id).await?.is_none() {
        return Err(ApiError::NotFound("User not found".to_string()));
    }

    // Get user's system roles
    let roles = repo.get_user_roles(target_user_id).await?;

    let response = UserSystemRolesResponse {
        user_id: target_user_id,
        roles: roles.into_iter().map(Into::into).collect(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Add a system role to a user.
///
/// POST /api/admin/v1/system-roles/users/{user_id}/roles
///
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn add_system_role(
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
    system_auth: SystemRoleAuth,
    Json(request): Json<AddSystemRoleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can assign system roles
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    let repo = SystemRoleRepository::new(state.pool.clone());

    // Verify target user exists
    let user_repo = UserRepository::new(state.pool.clone());
    if user_repo.find_by_id(target_user_id).await?.is_none() {
        return Err(ApiError::NotFound("User not found".to_string()));
    }

    // Check if user already has this role
    if repo.has_role(target_user_id, request.role).await? {
        return Err(ApiError::Conflict(format!(
            "User already has {} role",
            request.role
        )));
    }

    // Add the role
    let user_role = repo
        .add_role(target_user_id, request.role, Some(system_auth.user_id))
        .await?;

    tracing::info!(
        user_id = %target_user_id,
        role = %request.role,
        granted_by = %system_auth.user_id,
        "System role granted"
    );

    let response: AddSystemRoleResponse = user_role.into();
    Ok((StatusCode::CREATED, Json(response)))
}

/// Remove a system role from a user.
///
/// DELETE /api/admin/v1/system-roles/users/{user_id}/roles/{role}
///
/// Requires super_admin role.
/// Cannot remove the last super_admin from the system.
#[axum::debug_handler(state = AppState)]
async fn remove_system_role(
    State(state): State<AppState>,
    Path((target_user_id, role_str)): Path<(Uuid, String)>,
    system_auth: SystemRoleAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can remove system roles
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    // Parse the role
    let role: SystemRole = role_str
        .parse()
        .map_err(|_| ApiError::Validation(format!("Invalid role: {}", role_str)))?;

    let repo = SystemRoleRepository::new(state.pool.clone());

    // Check if user has this role
    if !repo.has_role(target_user_id, role).await? {
        return Err(ApiError::NotFound("User does not have this role".to_string()));
    }

    // Protection: Cannot remove the last super_admin
    if role == SystemRole::SuperAdmin {
        let super_admin_count = repo.count_super_admins().await?;
        if super_admin_count <= 1 {
            return Err(ApiError::Conflict(
                "Cannot remove the last super admin from the system".to_string(),
            ));
        }
    }

    // If removing org_admin or org_manager, also remove org assignments
    if role.requires_org_assignment() {
        let removed = repo.remove_all_org_assignments(target_user_id).await?;
        if removed > 0 {
            tracing::info!(
                user_id = %target_user_id,
                removed_assignments = removed,
                "Organization assignments removed as part of role removal"
            );
        }
    }

    // Remove the role
    let removed = repo.remove_role(target_user_id, role).await?;

    if !removed {
        return Err(ApiError::NotFound("Role not found".to_string()));
    }

    tracing::info!(
        user_id = %target_user_id,
        role = %role,
        removed_by = %system_auth.user_id,
        "System role removed"
    );

    Ok((
        StatusCode::OK,
        Json(RemoveSystemRoleResponse {
            success: true,
            message: format!("Role {} removed successfully", role),
        }),
    ))
}

/// Get a user's organization assignments.
///
/// GET /api/admin/v1/system-roles/users/{user_id}/org-assignments
///
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn get_org_assignments(
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
    system_auth: SystemRoleAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can view org assignments
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    let repo = SystemRoleRepository::new(state.pool.clone());

    // Verify target user exists
    let user_repo = UserRepository::new(state.pool.clone());
    if user_repo.find_by_id(target_user_id).await?.is_none() {
        return Err(ApiError::NotFound("User not found".to_string()));
    }

    // Get user's org assignments
    let assignments = repo.get_assigned_orgs(target_user_id).await?;

    let response = UserOrgAssignmentsResponse {
        user_id: target_user_id,
        assignments,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Assign an organization to a user.
///
/// POST /api/admin/v1/system-roles/users/{user_id}/org-assignments
///
/// Requires super_admin role.
/// User must have org_admin or org_manager role.
#[axum::debug_handler(state = AppState)]
async fn assign_org(
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
    system_auth: SystemRoleAuth,
    Json(request): Json<AssignOrgRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can assign organizations
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    let repo = SystemRoleRepository::new(state.pool.clone());

    // Verify target user exists
    let user_repo = UserRepository::new(state.pool.clone());
    if user_repo.find_by_id(target_user_id).await?.is_none() {
        return Err(ApiError::NotFound("User not found".to_string()));
    }

    // Verify user has org_admin or org_manager role
    let user_roles = repo.get_user_roles(target_user_id).await?;
    let has_org_role = user_roles
        .iter()
        .any(|r| r.role.requires_org_assignment());

    if !has_org_role {
        return Err(ApiError::Validation(
            "User must have org_admin or org_manager role before assigning organizations"
                .to_string(),
        ));
    }

    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(request.organization_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    // Check if already assigned
    if repo
        .is_assigned_to_org(target_user_id, request.organization_id)
        .await?
    {
        return Err(ApiError::Conflict(
            "User is already assigned to this organization".to_string(),
        ));
    }

    // Assign the organization
    let assignment = repo
        .assign_org(
            target_user_id,
            request.organization_id,
            Some(system_auth.user_id),
        )
        .await?;

    tracing::info!(
        user_id = %target_user_id,
        organization_id = %request.organization_id,
        assigned_by = %system_auth.user_id,
        "Organization assigned to admin user"
    );

    let response: AssignOrgResponse = assignment.into();
    Ok((StatusCode::CREATED, Json(response)))
}

/// Remove an organization assignment from a user.
///
/// DELETE /api/admin/v1/system-roles/users/{user_id}/org-assignments/{org_id}
///
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn unassign_org(
    State(state): State<AppState>,
    Path((target_user_id, org_id)): Path<(Uuid, Uuid)>,
    system_auth: SystemRoleAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can remove org assignments
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    let repo = SystemRoleRepository::new(state.pool.clone());

    // Check if assignment exists
    if !repo.is_assigned_to_org(target_user_id, org_id).await? {
        return Err(ApiError::NotFound(
            "Organization assignment not found".to_string(),
        ));
    }

    // Remove the assignment
    let removed = repo.unassign_org(target_user_id, org_id).await?;

    if !removed {
        return Err(ApiError::NotFound(
            "Organization assignment not found".to_string(),
        ));
    }

    tracing::info!(
        user_id = %target_user_id,
        organization_id = %org_id,
        removed_by = %system_auth.user_id,
        "Organization assignment removed"
    );

    Ok((
        StatusCode::OK,
        Json(RemoveOrgAssignmentResponse {
            success: true,
            message: "Organization assignment removed successfully".to_string(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_system_roles_returns_all_roles() {
        let roles: Vec<SystemRoleInfo> = SystemRole::all()
            .iter()
            .map(|&role| role.into())
            .collect();

        assert_eq!(roles.len(), 5);
        assert!(roles.iter().any(|r| r.role == SystemRole::SuperAdmin));
        assert!(roles.iter().any(|r| r.role == SystemRole::OrgAdmin));
        assert!(roles.iter().any(|r| r.role == SystemRole::OrgManager));
        assert!(roles.iter().any(|r| r.role == SystemRole::Support));
        assert!(roles.iter().any(|r| r.role == SystemRole::Viewer));
    }

    #[test]
    fn test_system_role_info_requires_org_assignment() {
        let org_admin_info: SystemRoleInfo = SystemRole::OrgAdmin.into();
        assert!(org_admin_info.requires_org_assignment);

        let super_admin_info: SystemRoleInfo = SystemRole::SuperAdmin.into();
        assert!(!super_admin_info.requires_org_assignment);
    }

    #[test]
    fn test_parse_role_from_string() {
        assert_eq!(
            "super_admin".parse::<SystemRole>().unwrap(),
            SystemRole::SuperAdmin
        );
        assert_eq!(
            "org_admin".parse::<SystemRole>().unwrap(),
            SystemRole::OrgAdmin
        );
        assert!("invalid_role".parse::<SystemRole>().is_err());
    }
}
