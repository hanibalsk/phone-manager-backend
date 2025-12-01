//! Role-Based Access Control (RBAC) middleware for group-based authorization.
//!
//! Provides middleware for requiring group membership and specific roles on routes.

#![allow(dead_code)] // Middleware functions are public APIs for future route use

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use domain::models::group::GroupRole;
use persistence::repositories::GroupRepository;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::middleware::user_auth::UserAuth;

/// Group membership information passed to handlers via request extensions.
#[derive(Debug, Clone)]
pub struct GroupMembership {
    /// The group ID.
    pub group_id: Uuid,
    /// The user's role in the group.
    pub role: GroupRole,
    /// The membership ID.
    pub membership_id: Uuid,
}

/// Middleware that requires the user to be a member of the group.
///
/// This middleware:
/// 1. Extracts the group_id from path parameters
/// 2. Verifies the user is a member of the group
/// 3. Stores the membership info in request extensions
///
/// Requires `UserAuth` to be present in request extensions (use after `require_user_auth`).
pub async fn require_group_member(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    require_group_role_impl(state, req, next, None).await
}

/// Middleware that requires the user to have at least the specified role in the group.
///
/// This middleware:
/// 1. Extracts the group_id from path parameters
/// 2. Verifies the user is a member of the group
/// 3. Checks the user has at least the minimum required role
/// 4. Stores the membership info in request extensions
///
/// Requires `UserAuth` to be present in request extensions (use after `require_user_auth`).
pub async fn require_group_admin(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    require_group_role_impl(state, req, next, Some(GroupRole::Admin)).await
}

/// Middleware that requires the user to be the owner of the group.
///
/// This middleware:
/// 1. Extracts the group_id from path parameters
/// 2. Verifies the user is a member of the group
/// 3. Checks the user is the owner
/// 4. Stores the membership info in request extensions
///
/// Requires `UserAuth` to be present in request extensions (use after `require_user_auth`).
pub async fn require_group_owner(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    require_group_role_impl(state, req, next, Some(GroupRole::Owner)).await
}

/// Internal implementation of role checking middleware.
async fn require_group_role_impl(
    state: AppState,
    mut req: Request<Body>,
    next: Next,
    min_role: Option<GroupRole>,
) -> Response {
    // Get user auth from extensions (must be set by require_user_auth)
    let user_auth = match req.extensions().get::<UserAuth>() {
        Some(auth) => auth.clone(),
        None => {
            tracing::warn!("RBAC middleware called without UserAuth in extensions");
            return unauthorized_response("Authentication required");
        }
    };

    // Extract group_id from path parameters
    let group_id = match extract_group_id_from_path(req.uri().path()) {
        Some(id) => id,
        None => {
            tracing::error!("Could not extract group_id from path: {}", req.uri().path());
            return not_found_response("Group not found");
        }
    };

    // Check membership in database
    let repo = GroupRepository::new(state.pool.clone());
    let membership = match repo.get_membership(group_id, user_auth.user_id).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return not_found_response("Group not found or you are not a member");
        }
        Err(e) => {
            tracing::error!("Database error checking membership: {}", e);
            return internal_error_response("Failed to verify group membership");
        }
    };

    let user_role: GroupRole = membership.role.into();

    // Check role requirement if specified
    if let Some(required_role) = min_role {
        if !has_sufficient_role(&user_role, &required_role) {
            return forbidden_response(&format!(
                "Insufficient permissions. Required role: {} or higher",
                required_role
            ));
        }
    }

    // Store membership info in extensions for handler use
    req.extensions_mut().insert(GroupMembership {
        group_id,
        role: user_role,
        membership_id: membership.id,
    });

    next.run(req).await
}

/// Checks if user_role is at least as privileged as required_role.
fn has_sufficient_role(user_role: &GroupRole, required_role: &GroupRole) -> bool {
    match required_role {
        GroupRole::Owner => *user_role == GroupRole::Owner,
        GroupRole::Admin => matches!(user_role, GroupRole::Owner | GroupRole::Admin),
        GroupRole::Member => matches!(
            user_role,
            GroupRole::Owner | GroupRole::Admin | GroupRole::Member
        ),
        GroupRole::Viewer => true, // Everyone has at least viewer access
    }
}

/// Extract group_id from the request path.
/// Expects paths like /api/v1/groups/:group_id/...
fn extract_group_id_from_path(path: &str) -> Option<Uuid> {
    let segments: Vec<&str> = path.split('/').collect();

    // Find "groups" segment and get the next one
    for (i, segment) in segments.iter().enumerate() {
        if *segment == "groups" {
            if let Some(id_str) = segments.get(i + 1) {
                // Skip "join" which is not a group_id
                if *id_str == "join" {
                    return None;
                }
                return Uuid::parse_str(id_str).ok();
            }
        }
    }

    None
}

/// Helper to create forbidden response.
fn forbidden_response(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "error": "forbidden",
            "message": message
        })),
    )
        .into_response()
}

/// Helper to create not found response.
fn not_found_response(message: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "not_found",
            "message": message
        })),
    )
        .into_response()
}

/// Helper to create unauthorized response.
fn unauthorized_response(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "unauthorized",
            "message": message
        })),
    )
        .into_response()
}

/// Helper to create internal error response.
fn internal_error_response(message: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": "internal_error",
            "message": message
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_sufficient_role_owner() {
        assert!(has_sufficient_role(&GroupRole::Owner, &GroupRole::Owner));
        assert!(has_sufficient_role(&GroupRole::Owner, &GroupRole::Admin));
        assert!(has_sufficient_role(&GroupRole::Owner, &GroupRole::Member));
        assert!(has_sufficient_role(&GroupRole::Owner, &GroupRole::Viewer));
    }

    #[test]
    fn test_has_sufficient_role_admin() {
        assert!(!has_sufficient_role(&GroupRole::Admin, &GroupRole::Owner));
        assert!(has_sufficient_role(&GroupRole::Admin, &GroupRole::Admin));
        assert!(has_sufficient_role(&GroupRole::Admin, &GroupRole::Member));
        assert!(has_sufficient_role(&GroupRole::Admin, &GroupRole::Viewer));
    }

    #[test]
    fn test_has_sufficient_role_member() {
        assert!(!has_sufficient_role(&GroupRole::Member, &GroupRole::Owner));
        assert!(!has_sufficient_role(&GroupRole::Member, &GroupRole::Admin));
        assert!(has_sufficient_role(&GroupRole::Member, &GroupRole::Member));
        assert!(has_sufficient_role(&GroupRole::Member, &GroupRole::Viewer));
    }

    #[test]
    fn test_has_sufficient_role_viewer() {
        assert!(!has_sufficient_role(&GroupRole::Viewer, &GroupRole::Owner));
        assert!(!has_sufficient_role(&GroupRole::Viewer, &GroupRole::Admin));
        assert!(!has_sufficient_role(&GroupRole::Viewer, &GroupRole::Member));
        assert!(has_sufficient_role(&GroupRole::Viewer, &GroupRole::Viewer));
    }

    #[test]
    fn test_extract_group_id_from_path() {
        let uuid = Uuid::new_v4();
        let path = format!("/api/v1/groups/{}/members", uuid);
        assert_eq!(extract_group_id_from_path(&path), Some(uuid));
    }

    #[test]
    fn test_extract_group_id_from_path_invites() {
        let uuid = Uuid::new_v4();
        let path = format!("/api/v1/groups/{}/invites", uuid);
        assert_eq!(extract_group_id_from_path(&path), Some(uuid));
    }

    #[test]
    fn test_extract_group_id_from_path_nested() {
        let uuid = Uuid::new_v4();
        let path = format!("/api/v1/groups/{}/members/some-user-id/role", uuid);
        assert_eq!(extract_group_id_from_path(&path), Some(uuid));
    }

    #[test]
    fn test_extract_group_id_from_path_no_id() {
        assert_eq!(extract_group_id_from_path("/api/v1/groups"), None);
    }

    #[test]
    fn test_extract_group_id_from_path_invalid_uuid() {
        assert_eq!(
            extract_group_id_from_path("/api/v1/groups/not-a-uuid/members"),
            None
        );
    }

    #[test]
    fn test_extract_group_id_from_path_join() {
        assert_eq!(extract_group_id_from_path("/api/v1/groups/join"), None);
    }

    #[test]
    fn test_forbidden_response() {
        let response = forbidden_response("Test message");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_not_found_response() {
        let response = not_found_response("Test message");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_unauthorized_response() {
        let response = unauthorized_response("Test message");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_internal_error_response() {
        let response = internal_error_response("Test message");
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_group_membership_struct() {
        let membership = GroupMembership {
            group_id: Uuid::new_v4(),
            role: GroupRole::Admin,
            membership_id: Uuid::new_v4(),
        };
        assert_eq!(membership.role, GroupRole::Admin);
    }

    #[test]
    fn test_group_membership_clone() {
        let membership = GroupMembership {
            group_id: Uuid::new_v4(),
            role: GroupRole::Member,
            membership_id: Uuid::new_v4(),
        };
        let cloned = membership.clone();
        assert_eq!(membership.group_id, cloned.group_id);
        assert_eq!(membership.role, cloned.role);
        assert_eq!(membership.membership_id, cloned.membership_id);
    }
}
