//! Organization admin API routes.
//!
//! Provides administrative endpoints for B2B organization management.
//! These routes require platform admin authentication.

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use domain::models::{
    validate_permissions, AddOrgUserRequest, CreateOrganizationRequest, CreateOrganizationResponse,
    ListOrgUsersQuery, ListOrgUsersResponse, ListOrganizationsQuery, ListOrganizationsResponse,
    OrgUserPagination, OrgUserResponse, OrgUserRole, OrganizationPagination, PlanType,
    SuspendOrganizationRequest, UpdateOrgUserRequest, UpdateOrganizationRequest,
};
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::api_key::ApiKeyAuth;
use persistence::repositories::{OrgUserRepository, OrganizationRepository, UserRepository};

/// POST /api/admin/v1/organizations
///
/// Create a new organization.
pub async fn create_organization(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Json(request): Json<CreateOrganizationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    let repo = OrganizationRepository::new(state.pool.clone());

    // Check if slug already exists
    if repo.slug_exists(&request.slug).await? {
        return Err(ApiError::Conflict(format!(
            "Organization with slug '{}' already exists",
            request.slug
        )));
    }

    // Get plan type and default limits
    let plan_type = request.plan_type.unwrap_or(PlanType::Free);
    let (max_users, max_devices, max_groups) = plan_type.default_limits();

    // Create organization
    let settings = request.settings.unwrap_or(serde_json::json!({}));
    let organization = repo
        .create(
            &request.name,
            &request.slug,
            &request.billing_email,
            plan_type,
            max_users,
            max_devices,
            max_groups,
            &settings,
        )
        .await?;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %organization.id,
        slug = %organization.slug,
        plan_type = %organization.plan_type,
        "Created new organization"
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateOrganizationResponse { organization }),
    ))
}

/// GET /api/admin/v1/organizations
///
/// List organizations with pagination and filtering.
pub async fn list_organizations(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Query(query): Query<ListOrganizationsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = OrganizationRepository::new(state.pool.clone());

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).clamp(1, 100);

    let (organizations, total) = repo.list(&query).await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

    info!(
        admin_key_id = auth.api_key_id,
        count = organizations.len(),
        total = total,
        "Listed organizations"
    );

    Ok(Json(ListOrganizationsResponse {
        data: organizations,
        pagination: OrganizationPagination {
            page,
            per_page,
            total,
            total_pages,
        },
    }))
}

/// GET /api/admin/v1/organizations/:org_id
///
/// Get organization details.
pub async fn get_organization(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = OrganizationRepository::new(state.pool.clone());

    let organization = repo.find_by_id(org_id).await?;

    match organization {
        Some(org) => {
            info!(
                admin_key_id = auth.api_key_id,
                organization_id = %org_id,
                "Fetched organization"
            );
            Ok(Json(org))
        }
        None => {
            warn!(
                admin_key_id = auth.api_key_id,
                organization_id = %org_id,
                "Organization not found"
            );
            Err(ApiError::NotFound("Organization not found".to_string()))
        }
    }
}

/// PUT /api/admin/v1/organizations/:org_id
///
/// Update organization.
pub async fn update_organization(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
    Json(request): Json<UpdateOrganizationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    let repo = OrganizationRepository::new(state.pool.clone());

    // Check if organization exists
    let existing = repo.find_by_id(org_id).await?;
    if existing.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    // Update organization
    let organization = repo
        .update(
            org_id,
            request.name.as_deref(),
            request.billing_email.as_deref(),
            request.plan_type,
            request.max_users,
            request.max_devices,
            request.max_groups,
            request.settings.as_ref(),
        )
        .await?;

    match organization {
        Some(org) => {
            info!(
                admin_key_id = auth.api_key_id,
                organization_id = %org_id,
                "Updated organization"
            );
            Ok(Json(org))
        }
        None => Err(ApiError::NotFound("Organization not found".to_string())),
    }
}

/// DELETE /api/admin/v1/organizations/:org_id
///
/// Soft delete organization (set is_active = false).
pub async fn delete_organization(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = OrganizationRepository::new(state.pool.clone());

    let deleted = repo.soft_delete(org_id).await?;

    if deleted {
        info!(
            admin_key_id = auth.api_key_id,
            organization_id = %org_id,
            "Soft deleted organization"
        );
        Ok(StatusCode::NO_CONTENT)
    } else {
        warn!(
            admin_key_id = auth.api_key_id,
            organization_id = %org_id,
            "Organization not found or already deleted"
        );
        Err(ApiError::NotFound(
            "Organization not found or already inactive".to_string(),
        ))
    }
}

/// GET /api/admin/v1/organizations/:org_id/usage
///
/// Get organization usage statistics.
pub async fn get_organization_usage(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = OrganizationRepository::new(state.pool.clone());

    let usage = repo.get_usage(org_id).await?;

    match usage {
        Some(stats) => {
            info!(
                admin_key_id = auth.api_key_id,
                organization_id = %org_id,
                "Fetched organization usage statistics"
            );
            Ok(Json(stats))
        }
        None => {
            warn!(
                admin_key_id = auth.api_key_id,
                organization_id = %org_id,
                "Organization not found"
            );
            Err(ApiError::NotFound("Organization not found".to_string()))
        }
    }
}

/// POST /api/admin/v1/organizations/:org_id/suspend
///
/// Suspend an organization.
pub async fn suspend_organization(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
    Json(request): Json<SuspendOrganizationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = OrganizationRepository::new(state.pool.clone());

    // Get admin user ID from auth context
    let suspended_by = auth.user_id.ok_or_else(|| {
        ApiError::Unauthorized("Admin user ID required for suspension".to_string())
    })?;

    let result = repo
        .suspend(org_id, suspended_by, request.reason.as_deref())
        .await?;

    match result {
        Some(response) => {
            info!(
                admin_key_id = auth.api_key_id,
                organization_id = %org_id,
                suspended_by = %suspended_by,
                reason = ?request.reason,
                "Suspended organization"
            );
            Ok(Json(response))
        }
        None => {
            warn!(
                admin_key_id = auth.api_key_id,
                organization_id = %org_id,
                "Organization not found for suspension"
            );
            Err(ApiError::NotFound("Organization not found".to_string()))
        }
    }
}

/// POST /api/admin/v1/organizations/:org_id/reactivate
///
/// Reactivate a suspended organization.
pub async fn reactivate_organization(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = OrganizationRepository::new(state.pool.clone());

    let result = repo.reactivate(org_id).await?;

    match result {
        Some(response) => {
            info!(
                admin_key_id = auth.api_key_id,
                organization_id = %org_id,
                "Reactivated organization"
            );
            Ok(Json(response))
        }
        None => {
            warn!(
                admin_key_id = auth.api_key_id,
                organization_id = %org_id,
                "Organization not found for reactivation"
            );
            Err(ApiError::NotFound("Organization not found".to_string()))
        }
    }
}

// =============================================================================
// Organization Users Endpoints (Story 13.2)
// =============================================================================

/// POST /api/admin/v1/organizations/:org_id/users
///
/// Add a user to an organization.
pub async fn add_org_user(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
    Json(request): Json<AddOrgUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    // Validate permissions if provided
    if let Some(ref perms) = request.permissions {
        validate_permissions(perms).map_err(ApiError::Validation)?;
    }

    let org_repo = OrganizationRepository::new(state.pool.clone());
    let user_repo = UserRepository::new(state.pool.clone());
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Check if organization exists
    let org = org_repo.find_by_id(org_id).await?;
    if org.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    // Find user by email
    let user = user_repo.find_by_email(&request.email).await?;
    let user = match user {
        Some(u) => u,
        None => {
            return Err(ApiError::NotFound(format!(
                "User with email '{}' not found. Invite flow not implemented yet.",
                request.email
            )));
        }
    };

    // Check if user is already in organization
    if org_user_repo.exists(org_id, user.id).await? {
        return Err(ApiError::Conflict(
            "User is already a member of this organization".to_string(),
        ));
    }

    // Use provided permissions or default for role
    let permissions = request
        .permissions
        .unwrap_or_else(|| request.role.default_permissions());

    // Create org user
    let org_user = org_user_repo
        .create(org_id, user.id, request.role, &permissions, None)
        .await?;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        user_id = %user.id,
        role = %request.role,
        "Added user to organization"
    );

    Ok((StatusCode::CREATED, Json(OrgUserResponse { org_user })))
}

/// GET /api/admin/v1/organizations/:org_id/users
///
/// List organization users.
pub async fn list_org_users(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListOrgUsersQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let org_repo = OrganizationRepository::new(state.pool.clone());
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Check if organization exists
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).clamp(1, 100);

    let (org_users, total) = org_user_repo.list(org_id, &query).await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        count = org_users.len(),
        total = total,
        "Listed organization users"
    );

    Ok(Json(ListOrgUsersResponse {
        data: org_users,
        pagination: OrgUserPagination {
            page,
            per_page,
            total,
            total_pages,
        },
    }))
}

/// PUT /api/admin/v1/organizations/:org_id/users/:user_id
///
/// Update organization user role/permissions.
pub async fn update_org_user(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateOrgUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    // Validate permissions if provided
    if let Some(ref perms) = request.permissions {
        validate_permissions(perms).map_err(ApiError::Validation)?;
    }

    let org_repo = OrganizationRepository::new(state.pool.clone());
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Check if organization exists
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    // Check if user is in organization
    let existing = org_user_repo.find_by_org_and_user(org_id, user_id).await?;
    let existing = match existing {
        Some(e) => e,
        None => {
            return Err(ApiError::NotFound(
                "User is not a member of this organization".to_string(),
            ));
        }
    };

    // Check if trying to demote the last owner
    if existing.role == OrgUserRole::Owner {
        if let Some(new_role) = request.role {
            if new_role != OrgUserRole::Owner {
                let owner_count = org_user_repo.count_owners(org_id).await?;
                if owner_count <= 1 {
                    return Err(ApiError::Validation(
                        "Cannot demote the last owner of the organization".to_string(),
                    ));
                }
            }
        }
    }

    // Update org user
    let org_user = org_user_repo
        .update(
            org_id,
            user_id,
            request.role,
            request.permissions.as_deref(),
        )
        .await?;

    match org_user {
        Some(user) => {
            info!(
                admin_key_id = auth.api_key_id,
                organization_id = %org_id,
                user_id = %user_id,
                "Updated organization user"
            );
            Ok(Json(OrgUserResponse { org_user: user }))
        }
        None => Err(ApiError::NotFound(
            "User is not a member of this organization".to_string(),
        )),
    }
}

/// DELETE /api/admin/v1/organizations/:org_id/users/:user_id
///
/// Remove user from organization.
pub async fn remove_org_user(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let org_repo = OrganizationRepository::new(state.pool.clone());
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Check if organization exists
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    // Check if user is in organization
    let existing = org_user_repo.find_by_org_and_user(org_id, user_id).await?;
    let existing = match existing {
        Some(e) => e,
        None => {
            return Err(ApiError::NotFound(
                "User is not a member of this organization".to_string(),
            ));
        }
    };

    // Cannot remove the last owner
    if existing.role == OrgUserRole::Owner {
        let owner_count = org_user_repo.count_owners(org_id).await?;
        if owner_count <= 1 {
            return Err(ApiError::Validation(
                "Cannot remove the last owner of the organization".to_string(),
            ));
        }
    }

    let deleted = org_user_repo.delete(org_id, user_id).await?;

    if deleted {
        info!(
            admin_key_id = auth.api_key_id,
            organization_id = %org_id,
            user_id = %user_id,
            "Removed user from organization"
        );
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound(
            "User is not a member of this organization".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_organization_request_deserialization() {
        let json = r#"{
            "name": "Acme Corp",
            "slug": "acme-corp",
            "billing_email": "billing@acme.com",
            "plan_type": "business"
        }"#;
        let request: CreateOrganizationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Acme Corp");
        assert_eq!(request.slug, "acme-corp");
        assert_eq!(request.billing_email, "billing@acme.com");
        assert_eq!(request.plan_type, Some(PlanType::Business));
    }

    #[test]
    fn test_update_organization_request_deserialization() {
        let json = r#"{
            "name": "Updated Name",
            "max_users": 200
        }"#;
        let request: UpdateOrganizationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, Some("Updated Name".to_string()));
        assert_eq!(request.max_users, Some(200));
        assert!(request.billing_email.is_none());
    }

    #[test]
    fn test_list_organizations_query_deserialization() {
        let json = r#"{
            "page": 2,
            "per_page": 25,
            "is_active": true,
            "plan_type": "starter"
        }"#;
        let query: ListOrganizationsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.page, Some(2));
        assert_eq!(query.per_page, Some(25));
        assert_eq!(query.is_active, Some(true));
        assert_eq!(query.plan_type, Some(PlanType::Starter));
    }
}
