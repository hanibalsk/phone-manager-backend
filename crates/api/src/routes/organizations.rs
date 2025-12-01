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
    CreateOrganizationRequest, CreateOrganizationResponse, ListOrganizationsQuery,
    ListOrganizationsResponse, OrganizationPagination, PlanType, UpdateOrganizationRequest,
};
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::api_key::ApiKeyAuth;
use persistence::repositories::OrganizationRepository;

/// POST /api/admin/v1/organizations
///
/// Create a new organization.
pub async fn create_organization(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Json(request): Json<CreateOrganizationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request.validate().map_err(|e| {
        ApiError::Validation(format!("Validation error: {}", e))
    })?;

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
    request.validate().map_err(|e| {
        ApiError::Validation(format!("Validation error: {}", e))
    })?;

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
        Err(ApiError::NotFound("Organization not found or already inactive".to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_organization_request_deserialization() {
        let json = r#"{
            "name": "Acme Corp",
            "slug": "acme-corp",
            "billingEmail": "billing@acme.com",
            "planType": "business"
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
            "maxUsers": 200
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
            "perPage": 25,
            "isActive": true,
            "planType": "starter"
        }"#;
        let query: ListOrganizationsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.page, Some(2));
        assert_eq!(query.per_page, Some(25));
        assert_eq!(query.is_active, Some(true));
        assert_eq!(query.plan_type, Some(PlanType::Starter));
    }
}
