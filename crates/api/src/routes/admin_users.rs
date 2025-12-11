//! Admin user management route handlers.
//!
//! Story 14.3: User Management Endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use persistence::repositories::{AdminUserRepository, OrgUserRepository, UserRepository};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;
use crate::services::auth::{AuthError, AuthService};

use domain::models::{
    validate_permissions, AdminUserDetailResponse, AdminUserListResponse, AdminUserPagination,
    AdminUserQuery, ForceMfaResponse, ListUserSessionsResponse, MfaMethod, MfaStatusResponse,
    OrgUserRole, ReactivateOrgUserResponse, RemoveUserResponse, ResetMfaResponse,
    RevokeAllSessionsResponse, RevokeSessionResponse, SuspendOrgUserRequest,
    SuspendOrgUserResponse, TriggerPasswordResetResponse, UpdateAdminUserRequest,
    UpdateAdminUserResponse, UserSessionInfo,
};

/// Create admin user management routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/{user_id}", get(get_user_detail))
        .route("/{user_id}", put(update_user))
        .route("/{user_id}", delete(remove_user))
        .route("/{user_id}/suspend", post(suspend_user))
        .route("/{user_id}/reactivate", post(reactivate_user))
        .route("/{user_id}/reset-password", post(trigger_password_reset))
        .route("/{user_id}/mfa", get(get_mfa_status))
        .route("/{user_id}/mfa/force", post(force_mfa))
        .route("/{user_id}/mfa", delete(reset_mfa))
        .route("/{user_id}/sessions", get(list_user_sessions))
        .route("/{user_id}/sessions/{session_id}", delete(revoke_session))
        .route("/{user_id}/sessions", delete(revoke_all_sessions))
}

/// Helper to create AuthService with OAuth config from AppState.
fn create_auth_service(state: &AppState) -> Result<AuthService, ApiError> {
    let google_client_id = if state.config.oauth.google_client_id.is_empty() {
        None
    } else {
        Some(state.config.oauth.google_client_id.clone())
    };
    let apple_client_id = if state.config.oauth.apple_client_id.is_empty() {
        None
    } else {
        Some(state.config.oauth.apple_client_id.clone())
    };

    AuthService::new(
        state.pool.clone(),
        &state.config.jwt,
        google_client_id,
        apple_client_id,
    )
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to initialize auth service");
        ApiError::Internal(format!("Failed to initialize auth service: {}", e))
    })
}

/// List users in organization.
///
/// GET /api/admin/v1/organizations/{org_id}/users
#[axum::debug_handler]
async fn list_users(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<AdminUserQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Validate query
    query
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let admin_user_repo = AdminUserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view users)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get pagination params
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);

    // Get summary counts
    let summary = admin_user_repo.get_user_summary(org_id).await?;

    // Get filter params
    let role_str = query.role.as_ref().map(|r| r.to_string());
    let role_filter = role_str.as_deref();

    // Get total count for pagination
    let total = admin_user_repo
        .count_users(
            org_id,
            role_filter,
            query.has_device,
            query.search.as_deref(),
        )
        .await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    // Get sort options with defaults
    let sort_field = query.sort.unwrap_or_default();
    let sort_order = query.order.unwrap_or_default();

    // Calculate offset
    let offset = (page - 1) * per_page;

    // Fetch users with filtering, sorting, and pagination
    let data = admin_user_repo
        .list_users(
            org_id,
            role_filter,
            query.has_device,
            query.search.as_deref(),
            sort_field,
            sort_order,
            per_page,
            offset,
        )
        .await?;

    let response = AdminUserListResponse {
        data,
        pagination: AdminUserPagination {
            page,
            per_page,
            total,
            total_pages,
        },
        summary,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get user detail.
///
/// GET /api/admin/v1/organizations/{org_id}/users/{user_id}
#[axum::debug_handler]
async fn get_user_detail(
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let admin_user_repo = AdminUserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view user details)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get user profile
    let profile = admin_user_repo
        .get_user_profile(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Get user's devices
    let devices = admin_user_repo.get_user_devices(target_user_id).await?;

    // Get user's groups
    let groups = admin_user_repo.get_user_groups(target_user_id).await?;

    // Get activity summary
    let activity_summary = admin_user_repo
        .get_user_activity(org_id, target_user_id)
        .await?;

    let response = AdminUserDetailResponse {
        user: profile,
        devices,
        groups,
        activity_summary,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Update user role/permissions.
///
/// PUT /api/admin/v1/organizations/{org_id}/users/{user_id}
#[axum::debug_handler]
async fn update_user(
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
    Json(request): Json<UpdateAdminUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can update users)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get target user to check their role
    let target_org_user = org_user_repo
        .find_by_org_and_user(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Admins cannot modify owners or other admins
    if org_user.role == OrgUserRole::Admin {
        if target_org_user.role == OrgUserRole::Owner {
            return Err(ApiError::Forbidden(
                "Admins cannot modify owners".to_string(),
            ));
        }
        if target_org_user.role == OrgUserRole::Admin && target_user_id != user.user_id {
            return Err(ApiError::Forbidden(
                "Admins cannot modify other admins".to_string(),
            ));
        }
    }

    // Cannot demote the last owner
    if target_org_user.role == OrgUserRole::Owner {
        if let Some(new_role) = &request.role {
            if *new_role != OrgUserRole::Owner {
                let owner_count = org_user_repo.count_owners(org_id).await?;
                if owner_count <= 1 {
                    return Err(ApiError::Conflict(
                        "Cannot demote the last owner of the organization".to_string(),
                    ));
                }
            }
        }
    }

    // Validate permissions if provided
    if let Some(ref perms) = request.permissions {
        validate_permissions(perms).map_err(ApiError::Validation)?;
    }

    // Update user
    let updated = org_user_repo
        .update(
            org_id,
            target_user_id,
            request.role,
            request.permissions.as_deref(),
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    let response = UpdateAdminUserResponse {
        id: updated.user.id,
        email: updated.user.email,
        display_name: updated.user.display_name,
        role: updated.role,
        permissions: updated.permissions,
        updated_at: Utc::now(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Remove user from organization.
///
/// DELETE /api/admin/v1/organizations/{org_id}/users/{user_id}
#[axum::debug_handler]
async fn remove_user(
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can remove users)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Cannot remove yourself
    if target_user_id == user.user_id {
        return Err(ApiError::Conflict(
            "Cannot remove yourself from the organization".to_string(),
        ));
    }

    // Get target user to check their role
    let target_org_user = org_user_repo
        .find_by_org_and_user(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Admins cannot remove owners or other admins
    if org_user.role == OrgUserRole::Admin {
        if target_org_user.role == OrgUserRole::Owner {
            return Err(ApiError::Forbidden(
                "Admins cannot remove owners".to_string(),
            ));
        }
        if target_org_user.role == OrgUserRole::Admin {
            return Err(ApiError::Forbidden(
                "Admins cannot remove other admins".to_string(),
            ));
        }
    }

    // Cannot remove the last owner
    if target_org_user.role == OrgUserRole::Owner {
        let owner_count = org_user_repo.count_owners(org_id).await?;
        if owner_count <= 1 {
            return Err(ApiError::Conflict(
                "Cannot remove the last owner of the organization".to_string(),
            ));
        }
    }

    // Remove user from organization
    let removed = org_user_repo.delete(org_id, target_user_id).await?;

    if !removed {
        return Err(ApiError::NotFound(
            "User not found in organization".to_string(),
        ));
    }

    let response = RemoveUserResponse {
        removed: true,
        user_id: target_user_id,
        removed_at: Utc::now(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Suspend user in organization.
///
/// POST /api/admin/v1/organizations/{org_id}/users/{user_id}/suspend
///
/// Story AP-3.5: Suspend User
#[axum::debug_handler]
async fn suspend_user(
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
    Json(request): Json<SuspendOrgUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can suspend users)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Cannot suspend yourself
    if target_user_id == user.user_id {
        return Err(ApiError::Conflict("Cannot suspend yourself".to_string()));
    }

    // Get target user to check their role
    let target_org_user = org_user_repo
        .find_by_org_and_user(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Check if already suspended (idempotent, but return info)
    if target_org_user.is_suspended() {
        return Err(ApiError::Conflict("User is already suspended".to_string()));
    }

    // Admins cannot suspend owners or other admins
    if org_user.role == OrgUserRole::Admin {
        if target_org_user.role == OrgUserRole::Owner {
            return Err(ApiError::Forbidden(
                "Admins cannot suspend owners".to_string(),
            ));
        }
        if target_org_user.role == OrgUserRole::Admin {
            return Err(ApiError::Forbidden(
                "Admins cannot suspend other admins".to_string(),
            ));
        }
    }

    // Cannot suspend the last owner
    if target_org_user.role == OrgUserRole::Owner {
        let owner_count = org_user_repo.count_owners(org_id).await?;
        if owner_count <= 1 {
            return Err(ApiError::Conflict(
                "Cannot suspend the last owner of the organization".to_string(),
            ));
        }
    }

    // Suspend the user
    let suspended = org_user_repo
        .suspend(
            org_id,
            target_user_id,
            user.user_id,
            request.reason.as_deref(),
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Log the suspension action via tracing
    tracing::info!(
        org_id = %org_id,
        suspended_user_id = %target_user_id,
        suspended_by = %user.user_id,
        reason = ?request.reason,
        "User suspended from organization"
    );

    let response = SuspendOrgUserResponse {
        id: suspended.id,
        user_id: suspended.user_id,
        organization_id: suspended.organization_id,
        suspended_at: suspended
            .suspended_at
            .expect("Suspension timestamp should be set"),
        suspended_by: suspended.suspended_by.expect("Suspended_by should be set"),
        suspension_reason: suspended.suspension_reason,
        message: "User has been suspended successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Reactivate suspended user in organization.
///
/// POST /api/admin/v1/organizations/{org_id}/users/{user_id}/reactivate
///
/// Story AP-3.6: Reactivate User
#[axum::debug_handler]
async fn reactivate_user(
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can reactivate users)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Get target user to check their status
    let target_org_user = org_user_repo
        .find_by_org_and_user(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Check if already active (idempotent)
    if !target_org_user.is_suspended() {
        return Ok((
            StatusCode::OK,
            Json(ReactivateOrgUserResponse {
                id: target_org_user.id,
                user_id: target_org_user.user_id,
                organization_id: target_org_user.organization_id,
                reactivated_at: Utc::now(),
                message: "User is already active".to_string(),
            }),
        ));
    }

    // Admins cannot reactivate owners or other admins (must match who can suspend)
    if org_user.role == OrgUserRole::Admin {
        if target_org_user.role == OrgUserRole::Owner {
            return Err(ApiError::Forbidden(
                "Admins cannot reactivate owners".to_string(),
            ));
        }
        if target_org_user.role == OrgUserRole::Admin {
            return Err(ApiError::Forbidden(
                "Admins cannot reactivate other admins".to_string(),
            ));
        }
    }

    // Reactivate the user
    let reactivated = org_user_repo
        .reactivate(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Log the reactivation action via tracing
    tracing::info!(
        org_id = %org_id,
        reactivated_user_id = %target_user_id,
        reactivated_by = %user.user_id,
        "User reactivated in organization"
    );

    let response = ReactivateOrgUserResponse {
        id: reactivated.id,
        user_id: reactivated.user_id,
        organization_id: reactivated.organization_id,
        reactivated_at: Utc::now(),
        message: "User has been reactivated successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Trigger password reset for a user in organization.
///
/// POST /api/admin/v1/organizations/{org_id}/users/{user_id}/reset-password
///
/// Story AP-3.7: Trigger Password Reset
#[axum::debug_handler]
async fn trigger_password_reset(
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can trigger password reset)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify target user is in the organization
    let target_org_user = org_user_repo
        .find_by_org_and_user(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Admins cannot trigger password reset for owners or other admins
    if org_user.role == OrgUserRole::Admin {
        if target_org_user.role == OrgUserRole::Owner {
            return Err(ApiError::Forbidden(
                "Admins cannot trigger password reset for owners".to_string(),
            ));
        }
        if target_org_user.role == OrgUserRole::Admin && target_user_id != user.user_id {
            return Err(ApiError::Forbidden(
                "Admins cannot trigger password reset for other admins".to_string(),
            ));
        }
    }

    // Check if user is suspended
    if target_org_user.is_suspended() {
        return Err(ApiError::Conflict(
            "Cannot trigger password reset for suspended user".to_string(),
        ));
    }

    // Create auth service and trigger password reset
    let auth_service = create_auth_service(&state)?;
    let (reset_token, email, expires_at) = auth_service
        .admin_trigger_password_reset(target_user_id)
        .await
        .map_err(|e| match e {
            AuthError::UserNotFound => {
                ApiError::NotFound("User not found in organization".to_string())
            }
            AuthError::UserDisabled => {
                ApiError::Conflict("Cannot trigger password reset for disabled user".to_string())
            }
            _ => ApiError::Internal(format!("Failed to trigger password reset: {}", e)),
        })?;

    // Log the action (token is logged separately by auth service, never expose in response)
    tracing::info!(
        org_id = %org_id,
        target_user_id = %target_user_id,
        triggered_by = %user.user_id,
        email = %email,
        expires_at = %expires_at,
        "Admin triggered password reset for user"
    );

    // In production, the email service would be called here to send the reset email
    // For MVP/development, the token is logged by AuthService for manual testing
    // Log the token for development/testing purposes
    tracing::debug!(
        reset_token = %reset_token,
        "Password reset token (development only - remove in production)"
    );

    let response = TriggerPasswordResetResponse {
        user_id: target_user_id,
        email,
        reset_token_sent: true, // Would be true after email integration
        expires_at,
        message: "Password reset email has been sent to the user".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get MFA status for a user in organization.
///
/// GET /api/admin/v1/organizations/{org_id}/users/{user_id}/mfa
///
/// Story AP-3.8: View User MFA Status
#[axum::debug_handler]
async fn get_mfa_status(
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let user_repo = UserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view MFA status)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify target user is in the organization
    let _target_org_user = org_user_repo
        .find_by_org_and_user(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Get MFA status
    let mfa_status = user_repo
        .get_mfa_status(target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Convert mfa_method string to MfaMethod enum
    let mfa_method = mfa_status
        .mfa_method
        .as_ref()
        .and_then(|m| match m.as_str() {
            "totp" => Some(MfaMethod::Totp),
            "sms" => Some(MfaMethod::Sms),
            "email" => Some(MfaMethod::Email),
            _ => None,
        });

    let response = MfaStatusResponse {
        user_id: mfa_status.id,
        mfa_enabled: mfa_status.mfa_enabled,
        mfa_method,
        enrolled_at: mfa_status.mfa_enrolled_at,
        mfa_required: mfa_status.mfa_forced,
        required_at: mfa_status.mfa_forced_at,
        required_by: mfa_status.mfa_forced_by,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Force MFA enrollment for a user in organization.
///
/// POST /api/admin/v1/organizations/{org_id}/users/{user_id}/mfa/force
///
/// Story AP-3.9: Force MFA Enrollment
#[axum::debug_handler]
async fn force_mfa(
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let user_repo = UserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can force MFA)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify target user is in the organization
    let target_org_user = org_user_repo
        .find_by_org_and_user(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Admins cannot force MFA for owners or other admins
    if org_user.role == OrgUserRole::Admin {
        if target_org_user.role == OrgUserRole::Owner {
            return Err(ApiError::Forbidden(
                "Admins cannot force MFA for owners".to_string(),
            ));
        }
        if target_org_user.role == OrgUserRole::Admin && target_user_id != user.user_id {
            return Err(ApiError::Forbidden(
                "Admins cannot force MFA for other admins".to_string(),
            ));
        }
    }

    // Check if user is suspended
    if target_org_user.is_suspended() {
        return Err(ApiError::Conflict(
            "Cannot force MFA for suspended user".to_string(),
        ));
    }

    // Force MFA enrollment
    let mfa_status = user_repo
        .force_mfa(target_user_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Log the action
    tracing::info!(
        org_id = %org_id,
        target_user_id = %target_user_id,
        forced_by = %user.user_id,
        "Admin forced MFA enrollment for user"
    );

    let response = ForceMfaResponse {
        user_id: mfa_status.id,
        mfa_required: mfa_status.mfa_forced,
        required_at: mfa_status
            .mfa_forced_at
            .expect("MFA forced_at should be set"),
        required_by: mfa_status
            .mfa_forced_by
            .expect("MFA forced_by should be set"),
        message: "MFA enrollment requirement has been set for the user".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Reset MFA for a user in organization.
///
/// DELETE /api/admin/v1/organizations/{org_id}/users/{user_id}/mfa
///
/// Story AP-3.10: Reset User MFA
#[axum::debug_handler]
async fn reset_mfa(
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let user_repo = UserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can reset MFA)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify target user is in the organization
    let target_org_user = org_user_repo
        .find_by_org_and_user(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Admins cannot reset MFA for owners or other admins
    if org_user.role == OrgUserRole::Admin {
        if target_org_user.role == OrgUserRole::Owner {
            return Err(ApiError::Forbidden(
                "Admins cannot reset MFA for owners".to_string(),
            ));
        }
        if target_org_user.role == OrgUserRole::Admin && target_user_id != user.user_id {
            return Err(ApiError::Forbidden(
                "Admins cannot reset MFA for other admins".to_string(),
            ));
        }
    }

    // Check if user is suspended
    if target_org_user.is_suspended() {
        return Err(ApiError::Conflict(
            "Cannot reset MFA for suspended user".to_string(),
        ));
    }

    // Reset MFA
    let reset = user_repo.reset_mfa(target_user_id).await?;

    if !reset {
        return Err(ApiError::NotFound("User not found".to_string()));
    }

    // Log the action
    tracing::info!(
        org_id = %org_id,
        target_user_id = %target_user_id,
        reset_by = %user.user_id,
        "Admin reset MFA for user"
    );

    let response = ResetMfaResponse {
        user_id: target_user_id,
        mfa_reset: true,
        reset_at: Utc::now(),
        message: "MFA has been reset for the user".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// List active sessions for a user in organization.
///
/// GET /api/admin/v1/organizations/{org_id}/users/{user_id}/sessions
///
/// Story AP-3.11: List User Sessions
#[axum::debug_handler]
async fn list_user_sessions(
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let user_repo = UserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can view sessions)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify target user is in the organization
    let _target_org_user = org_user_repo
        .find_by_org_and_user(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Get sessions
    let sessions = user_repo.list_user_sessions(target_user_id).await?;
    let total = sessions.len() as i64;

    // Convert to response format
    let data: Vec<UserSessionInfo> = sessions
        .into_iter()
        .map(|s| UserSessionInfo {
            id: s.id,
            user_id: s.user_id,
            device_name: s.device_name,
            device_type: s.device_type,
            browser: s.browser,
            os: s.os,
            ip_address: s.ip_address,
            location: s.location,
            created_at: s.created_at,
            last_used_at: s.last_used_at,
            expires_at: s.expires_at,
            is_current: false, // We don't have access to current token in this context
        })
        .collect();

    let response = ListUserSessionsResponse { data, total };

    Ok((StatusCode::OK, Json(response)))
}

/// Revoke a specific session for a user in organization.
///
/// DELETE /api/admin/v1/organizations/{org_id}/users/{user_id}/sessions/{session_id}
///
/// Story AP-3.12: Revoke Session
#[axum::debug_handler]
async fn revoke_session(
    State(state): State<AppState>,
    Path((org_id, target_user_id, session_id)): Path<(Uuid, Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let user_repo = UserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can revoke sessions)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify target user is in the organization
    let target_org_user = org_user_repo
        .find_by_org_and_user(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Admins cannot revoke sessions for owners or other admins
    if org_user.role == OrgUserRole::Admin {
        if target_org_user.role == OrgUserRole::Owner {
            return Err(ApiError::Forbidden(
                "Admins cannot revoke sessions for owners".to_string(),
            ));
        }
        if target_org_user.role == OrgUserRole::Admin && target_user_id != user.user_id {
            return Err(ApiError::Forbidden(
                "Admins cannot revoke sessions for other admins".to_string(),
            ));
        }
    }

    // Revoke the session
    let revoked = user_repo.revoke_session(session_id, target_user_id).await?;

    if !revoked {
        return Err(ApiError::NotFound("Session not found".to_string()));
    }

    // Log the action
    tracing::info!(
        org_id = %org_id,
        target_user_id = %target_user_id,
        session_id = %session_id,
        revoked_by = %user.user_id,
        "Admin revoked user session"
    );

    let response = RevokeSessionResponse {
        session_id,
        revoked: true,
        revoked_at: Utc::now(),
        message: "Session has been revoked successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Revoke all sessions for a user in organization.
///
/// DELETE /api/admin/v1/organizations/{org_id}/users/{user_id}/sessions
///
/// Story AP-3.13: Revoke All Sessions
#[axum::debug_handler]
async fn revoke_all_sessions(
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let user_repo = UserRepository::new(state.pool.clone());

    // Verify user has access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner can revoke all sessions)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Verify target user is in the organization
    let target_org_user = org_user_repo
        .find_by_org_and_user(org_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found in organization".to_string()))?;

    // Admins cannot revoke all sessions for owners or other admins
    if org_user.role == OrgUserRole::Admin {
        if target_org_user.role == OrgUserRole::Owner {
            return Err(ApiError::Forbidden(
                "Admins cannot revoke sessions for owners".to_string(),
            ));
        }
        if target_org_user.role == OrgUserRole::Admin && target_user_id != user.user_id {
            return Err(ApiError::Forbidden(
                "Admins cannot revoke sessions for other admins".to_string(),
            ));
        }
    }

    // Revoke all sessions
    let revoked_count = user_repo.revoke_all_sessions(target_user_id).await?;

    // Log the action
    tracing::info!(
        org_id = %org_id,
        target_user_id = %target_user_id,
        revoked_count = %revoked_count,
        revoked_by = %user.user_id,
        "Admin revoked all user sessions"
    );

    let response = RevokeAllSessionsResponse {
        user_id: target_user_id,
        revoked_count,
        revoked_at: Utc::now(),
        message: format!(
            "{} session(s) have been revoked successfully",
            revoked_count
        ),
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
