//! System-Level Role-Based Access Control (RBAC) middleware.
//!
//! Provides middleware for requiring system roles on admin routes.
//! System roles operate across organizations and provide different levels
//! of administrative access.

#![allow(dead_code)] // Middleware functions are public APIs for future route use

use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, State},
    http::{request::Parts, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use domain::models::SystemRole;
use persistence::repositories::SystemRoleRepository;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use crate::middleware::user_auth::UserAuth;

/// System role information passed to handlers via request extensions.
#[derive(Debug, Clone)]
pub struct SystemRoleAuth {
    /// The authenticated user's ID.
    pub user_id: Uuid,
    /// All system roles the user has.
    pub roles: Vec<SystemRole>,
    /// Organization IDs the user is assigned to (for org_admin/org_manager).
    pub assigned_org_ids: Vec<Uuid>,
}

impl SystemRoleAuth {
    /// Check if user has a specific system role.
    pub fn has_role(&self, role: SystemRole) -> bool {
        self.roles.contains(&role)
    }

    /// Check if user has at least the specified role level.
    pub fn has_at_least(&self, required: SystemRole) -> bool {
        self.roles.iter().any(|r| r.has_at_least(required))
    }

    /// Check if user is a super admin.
    pub fn is_super_admin(&self) -> bool {
        self.has_role(SystemRole::SuperAdmin)
    }

    /// Check if user can access the given organization.
    /// SuperAdmin can access all, org_admin/org_manager need assignment,
    /// support/viewer have global read access.
    pub fn can_access_org(&self, org_id: Uuid) -> bool {
        if self.is_super_admin() {
            return true;
        }
        if self.has_role(SystemRole::Support) || self.has_role(SystemRole::Viewer) {
            return true;
        }
        self.assigned_org_ids.contains(&org_id)
    }

    /// Check if user can manage the given organization (not just read).
    pub fn can_manage_org(&self, org_id: Uuid) -> bool {
        if self.is_super_admin() {
            return true;
        }
        if (self.has_role(SystemRole::OrgAdmin) || self.has_role(SystemRole::OrgManager))
            && self.assigned_org_ids.contains(&org_id)
        {
            return true;
        }
        false
    }
}

/// Extractor implementation for SystemRoleAuth.
///
/// This extractor validates that the user has at least one system role.
/// It requires that the user is already authenticated via JWT (UserAuth must be in extensions
/// or the extractor will perform JWT validation itself).
#[async_trait]
impl FromRequestParts<AppState> for SystemRoleAuth {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // First, get UserAuth - either from extensions or by validating JWT
        let user_auth = if let Some(auth) = parts.extensions.get::<UserAuth>() {
            auth.clone()
        } else {
            // Try to validate JWT directly
            let auth_header = parts
                .headers
                .get("Authorization")
                .and_then(|v| v.to_str().ok());

            let token = match auth_header {
                Some(header) if header.starts_with("Bearer ") => &header[7..],
                _ => {
                    return Err(ApiError::Unauthorized(
                        "Missing or invalid Authorization header".to_string(),
                    ));
                }
            };

            let jwt_config = UserAuth::create_jwt_config(&state.config.jwt).map_err(|e| {
                tracing::error!("Failed to create JWT config: {}", e);
                ApiError::Internal("Authentication service unavailable".to_string())
            })?;

            UserAuth::validate(&jwt_config, token).map_err(|e| {
                tracing::debug!("JWT validation failed: {}", e);
                ApiError::Unauthorized("Invalid or expired token".to_string())
            })?
        };

        // Load user's system roles from database
        let repo = SystemRoleRepository::new(state.pool.clone());

        let user_roles = repo.get_user_roles(user_auth.user_id).await.map_err(|e| {
            tracing::error!("Database error fetching system roles: {}", e);
            ApiError::Internal("Failed to verify system roles".to_string())
        })?;

        // Check if user has any system role
        if user_roles.is_empty() {
            return Err(ApiError::Forbidden("No system role assigned".to_string()));
        }

        // Convert to SystemRole enum
        let roles: Vec<SystemRole> = user_roles.iter().map(|r| r.role).collect();

        // Load assigned organizations for org_admin/org_manager
        let assigned_org_ids = if roles.iter().any(|r| r.requires_org_assignment()) {
            repo.get_assigned_org_ids(user_auth.user_id)
                .await
                .map_err(|e| {
                    tracing::error!("Database error fetching org assignments: {}", e);
                    ApiError::Internal("Failed to load organization assignments".to_string())
                })?
        } else {
            vec![]
        };

        Ok(SystemRoleAuth {
            user_id: user_auth.user_id,
            roles,
            assigned_org_ids,
        })
    }
}

/// Middleware that requires the user to be a super admin.
///
/// Requires `UserAuth` to be present in request extensions (use after `require_user_auth`).
pub async fn require_super_admin(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    require_system_role_impl(state, req, next, Some(SystemRole::SuperAdmin)).await
}

/// Middleware that requires the user to have at least org_admin role.
///
/// Requires `UserAuth` to be present in request extensions (use after `require_user_auth`).
pub async fn require_org_admin(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    require_system_role_impl(state, req, next, Some(SystemRole::OrgAdmin)).await
}

/// Middleware that requires the user to have at least org_manager role.
///
/// Requires `UserAuth` to be present in request extensions (use after `require_user_auth`).
pub async fn require_org_manager(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    require_system_role_impl(state, req, next, Some(SystemRole::OrgManager)).await
}

/// Middleware that requires the user to have at least support role.
///
/// Requires `UserAuth` to be present in request extensions (use after `require_user_auth`).
pub async fn require_support(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    require_system_role_impl(state, req, next, Some(SystemRole::Support)).await
}

/// Middleware that requires the user to have at least viewer role.
///
/// Requires `UserAuth` to be present in request extensions (use after `require_user_auth`).
pub async fn require_viewer(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    require_system_role_impl(state, req, next, Some(SystemRole::Viewer)).await
}

/// Middleware that requires the user to have any system role.
///
/// Requires `UserAuth` to be present in request extensions (use after `require_user_auth`).
pub async fn require_any_system_role(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    require_system_role_impl(state, req, next, None).await
}

/// Internal implementation of system role checking middleware.
async fn require_system_role_impl(
    state: AppState,
    mut req: Request<Body>,
    next: Next,
    min_role: Option<SystemRole>,
) -> Response {
    // Get user auth from extensions (must be set by require_user_auth)
    let user_auth = match req.extensions().get::<UserAuth>() {
        Some(auth) => auth.clone(),
        None => {
            tracing::warn!("System RBAC middleware called without UserAuth in extensions");
            return unauthorized_response("Authentication required");
        }
    };

    // Load user's system roles from database
    let repo = SystemRoleRepository::new(state.pool.clone());

    let user_roles = match repo.get_user_roles(user_auth.user_id).await {
        Ok(roles) => roles,
        Err(e) => {
            tracing::error!("Database error fetching system roles: {}", e);
            return internal_error_response("Failed to verify system roles");
        }
    };

    // Check if user has any system role
    if user_roles.is_empty() {
        return forbidden_response("No system role assigned");
    }

    // Convert to SystemRole enum
    let roles: Vec<SystemRole> = user_roles.iter().map(|r| r.role).collect();

    // Check role requirement if specified
    if let Some(required_role) = min_role {
        let has_sufficient = roles.iter().any(|r| r.has_at_least(required_role));
        if !has_sufficient {
            return forbidden_response(&format!(
                "Insufficient permissions. Required role: {} or higher",
                required_role
            ));
        }
    }

    // Load assigned organizations for org_admin/org_manager
    let assigned_org_ids = if roles
        .iter()
        .any(|r| r.requires_org_assignment())
    {
        match repo.get_assigned_org_ids(user_auth.user_id).await {
            Ok(ids) => ids,
            Err(e) => {
                tracing::error!("Database error fetching org assignments: {}", e);
                return internal_error_response("Failed to load organization assignments");
            }
        }
    } else {
        vec![]
    };

    // Store system role info in extensions for handler use
    req.extensions_mut().insert(SystemRoleAuth {
        user_id: user_auth.user_id,
        roles,
        assigned_org_ids,
    });

    next.run(req).await
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
    fn test_system_role_auth_has_role() {
        let auth = SystemRoleAuth {
            user_id: Uuid::new_v4(),
            roles: vec![SystemRole::OrgAdmin, SystemRole::Support],
            assigned_org_ids: vec![],
        };
        assert!(auth.has_role(SystemRole::OrgAdmin));
        assert!(auth.has_role(SystemRole::Support));
        assert!(!auth.has_role(SystemRole::SuperAdmin));
    }

    #[test]
    fn test_system_role_auth_has_at_least() {
        let auth = SystemRoleAuth {
            user_id: Uuid::new_v4(),
            roles: vec![SystemRole::OrgAdmin],
            assigned_org_ids: vec![],
        };
        // OrgAdmin has at least OrgAdmin level
        assert!(auth.has_at_least(SystemRole::OrgAdmin));
        // OrgAdmin has at least OrgManager level (lower)
        assert!(auth.has_at_least(SystemRole::OrgManager));
        // OrgAdmin does NOT have SuperAdmin level
        assert!(!auth.has_at_least(SystemRole::SuperAdmin));
    }

    #[test]
    fn test_system_role_auth_is_super_admin() {
        let super_admin = SystemRoleAuth {
            user_id: Uuid::new_v4(),
            roles: vec![SystemRole::SuperAdmin],
            assigned_org_ids: vec![],
        };
        assert!(super_admin.is_super_admin());

        let org_admin = SystemRoleAuth {
            user_id: Uuid::new_v4(),
            roles: vec![SystemRole::OrgAdmin],
            assigned_org_ids: vec![],
        };
        assert!(!org_admin.is_super_admin());
    }

    #[test]
    fn test_system_role_auth_can_access_org() {
        let org_id = Uuid::new_v4();
        let other_org_id = Uuid::new_v4();

        // SuperAdmin can access all orgs
        let super_admin = SystemRoleAuth {
            user_id: Uuid::new_v4(),
            roles: vec![SystemRole::SuperAdmin],
            assigned_org_ids: vec![],
        };
        assert!(super_admin.can_access_org(org_id));
        assert!(super_admin.can_access_org(other_org_id));

        // Support can access all orgs (global read)
        let support = SystemRoleAuth {
            user_id: Uuid::new_v4(),
            roles: vec![SystemRole::Support],
            assigned_org_ids: vec![],
        };
        assert!(support.can_access_org(org_id));

        // OrgAdmin can only access assigned orgs
        let org_admin = SystemRoleAuth {
            user_id: Uuid::new_v4(),
            roles: vec![SystemRole::OrgAdmin],
            assigned_org_ids: vec![org_id],
        };
        assert!(org_admin.can_access_org(org_id));
        assert!(!org_admin.can_access_org(other_org_id));
    }

    #[test]
    fn test_system_role_auth_can_manage_org() {
        let org_id = Uuid::new_v4();
        let other_org_id = Uuid::new_v4();

        // SuperAdmin can manage all orgs
        let super_admin = SystemRoleAuth {
            user_id: Uuid::new_v4(),
            roles: vec![SystemRole::SuperAdmin],
            assigned_org_ids: vec![],
        };
        assert!(super_admin.can_manage_org(org_id));

        // Support cannot manage (read-only)
        let support = SystemRoleAuth {
            user_id: Uuid::new_v4(),
            roles: vec![SystemRole::Support],
            assigned_org_ids: vec![],
        };
        assert!(!support.can_manage_org(org_id));

        // OrgAdmin can only manage assigned orgs
        let org_admin = SystemRoleAuth {
            user_id: Uuid::new_v4(),
            roles: vec![SystemRole::OrgAdmin],
            assigned_org_ids: vec![org_id],
        };
        assert!(org_admin.can_manage_org(org_id));
        assert!(!org_admin.can_manage_org(other_org_id));
    }

    #[test]
    fn test_forbidden_response() {
        let response = forbidden_response("Test message");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
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
    fn test_system_role_auth_clone() {
        let auth = SystemRoleAuth {
            user_id: Uuid::new_v4(),
            roles: vec![SystemRole::OrgAdmin],
            assigned_org_ids: vec![Uuid::new_v4()],
        };
        let cloned = auth.clone();
        assert_eq!(auth.user_id, cloned.user_id);
        assert_eq!(auth.roles.len(), cloned.roles.len());
        assert_eq!(auth.assigned_org_ids.len(), cloned.assigned_org_ids.len());
    }

    #[test]
    fn test_system_role_auth_debug() {
        let auth = SystemRoleAuth {
            user_id: Uuid::new_v4(),
            roles: vec![SystemRole::SuperAdmin],
            assigned_org_ids: vec![],
        };
        let debug_str = format!("{:?}", auth);
        assert!(debug_str.contains("SystemRoleAuth"));
        assert!(debug_str.contains("user_id"));
    }
}
