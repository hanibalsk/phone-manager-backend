//! Permission listing route handlers.
//!
//! Story AP-1.1: List Permissions

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use persistence::repositories::OrgUserRepository;
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

use domain::models::{
    get_all_permissions, get_permissions_by_category, get_permissions_by_category_filter,
    ListPermissionsQuery, ListPermissionsResponse, OrgUserRole,
};

/// Create permissions routes.
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_permissions))
}

/// List all available permissions.
///
/// GET /api/admin/v1/organizations/{org_id}/permissions
///
/// Returns all available organization-level permissions grouped by category.
/// Supports filtering by category via query parameter.
#[axum::debug_handler]
async fn list_permissions(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListPermissionsQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view permissions)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get permissions based on query
    let data = if let Some(ref category) = query.category {
        get_permissions_by_category_filter(category)
    } else {
        get_all_permissions()
    };

    let response = ListPermissionsResponse {
        data,
        by_category: get_permissions_by_category(),
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
