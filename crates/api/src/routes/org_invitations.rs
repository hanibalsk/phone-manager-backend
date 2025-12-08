//! Organization member invitation routes.
//!
//! Provides endpoints for creating, listing, and revoking member invitations,
//! as well as accepting invitations (public endpoint).

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use domain::models::{
    CreateInvitationRequest, CreateInvitationResponse, InvitationPagination, InvitationResponse,
    InvitationStatus, InvitationSummary, InvitedByInfo, ListInvitationsQuery,
    ListInvitationsResponse,
};
use persistence::entities::OrgMemberInviteEntity;
use persistence::repositories::{
    calculate_invite_expiration, default_invite_expiration, generate_org_member_invite_token,
    OrgMemberInviteRepository, OrgUserRepository, OrganizationRepository, UserRepository,
};
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::api_key::ApiKeyAuth;

/// POST /api/admin/v1/organizations/:org_id/invitations
///
/// Create a new member invitation.
/// Returns the invitation details including the token (shown only once).
pub async fn create_invitation(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
    Json(request): Json<CreateInvitationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request.validate().map_err(|e| {
        ApiError::Validation(format!("Validation error: {}", e))
    })?;

    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    let _organization = org_repo
        .find_by_id(org_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Organization not found".to_string()))?;

    let invite_repo = OrgMemberInviteRepository::new(state.pool.clone());
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Check if email is already a member
    let user_repo = UserRepository::new(state.pool.clone());
    if let Some(user) = user_repo.find_by_email(&request.email).await? {
        if org_user_repo.find_by_org_and_user(org_id, user.id).await?.is_some() {
            return Err(ApiError::Conflict(
                "User is already a member of this organization".to_string(),
            ));
        }
    }

    // Check if pending invite already exists for this email
    if invite_repo.has_pending_invite(org_id, &request.email).await? {
        return Err(ApiError::Conflict(
            "A pending invitation already exists for this email".to_string(),
        ));
    }

    // Generate token
    let token = generate_org_member_invite_token();

    // Calculate expiration
    let expires_at = match request.expires_in_days {
        Some(days) => calculate_invite_expiration(days),
        None => default_invite_expiration(),
    };

    // Determine role (default to "member")
    let role = request.role.as_deref().unwrap_or("member");

    // Create the invitation
    // Note: invited_by is None for API key authenticated requests
    // (API keys don't have a direct user association in this context)
    let entity = invite_repo
        .create(
            org_id,
            &token,
            &request.email,
            role,
            None, // invited_by - could be enhanced to track admin user
            expires_at,
            request.note.as_deref(),
        )
        .await?;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        invitation_id = %entity.id,
        email = %request.email,
        role = %role,
        "Created organization member invitation"
    );

    // Build invite URL
    let invite_url = format!(
        "{}/invite/{}",
        state.config.server.app_base_url,
        token
    );

    let response = CreateInvitationResponse {
        id: entity.id,
        email: entity.email,
        role: entity.role,
        token,
        invite_url,
        expires_at: entity.expires_at,
        created_at: entity.created_at,
        note: entity.note,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/admin/v1/organizations/:org_id/invitations
///
/// List invitations for an organization with optional status filter.
pub async fn list_invitations(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListInvitationsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let invite_repo = OrgMemberInviteRepository::new(state.pool.clone());
    let user_repo = UserRepository::new(state.pool.clone());

    // Get invitations
    let include_accepted = query.include_accepted();
    let entities = invite_repo
        .list_by_organization(org_id, include_accepted, query.per_page(), query.offset())
        .await?;

    // Get total count for pagination
    let total = invite_repo
        .count_by_organization(org_id, include_accepted)
        .await?;

    // Get summary counts
    let summary_counts = invite_repo.get_summary_counts(org_id).await?;

    // Convert entities to responses
    let mut invitations = Vec::with_capacity(entities.len());
    for entity in entities {
        let invited_by = if let Some(user_id) = entity.invited_by {
            if let Some(user) = user_repo.find_by_id(user_id).await? {
                Some(InvitedByInfo {
                    id: user_id,
                    email: user.email,
                })
            } else {
                None
            }
        } else {
            None
        };

        invitations.push(entity_to_response(entity, invited_by));
    }

    // Filter by status if specified (for expired filtering)
    let invitations = match query.status.as_deref() {
        Some("expired") => invitations
            .into_iter()
            .filter(|i| i.status == InvitationStatus::Expired)
            .collect(),
        Some("pending") => invitations
            .into_iter()
            .filter(|i| i.status == InvitationStatus::Pending)
            .collect(),
        Some("accepted") => invitations
            .into_iter()
            .filter(|i| i.status == InvitationStatus::Accepted)
            .collect(),
        _ => invitations, // "all" or None
    };

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        total_invitations = total,
        "Listed organization member invitations"
    );

    Ok(Json(ListInvitationsResponse {
        invitations,
        pagination: InvitationPagination::new(query.page(), query.per_page(), total),
        summary: InvitationSummary {
            pending: summary_counts.pending,
            accepted: summary_counts.accepted,
            expired: summary_counts.expired,
        },
    }))
}

/// GET /api/admin/v1/organizations/:org_id/invitations/:invite_id
///
/// Get details for a specific invitation.
pub async fn get_invitation(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, invite_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let invite_repo = OrgMemberInviteRepository::new(state.pool.clone());
    let user_repo = UserRepository::new(state.pool.clone());

    // Find the invitation
    let entity = invite_repo
        .find_by_id_and_org(invite_id, org_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Invitation not found".to_string()))?;

    // Get invited_by user info
    let invited_by = if let Some(user_id) = entity.invited_by {
        if let Some(user) = user_repo.find_by_id(user_id).await? {
            Some(InvitedByInfo {
                id: user_id,
                email: user.email,
            })
        } else {
            None
        }
    } else {
        None
    };

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        invitation_id = %invite_id,
        "Fetched organization member invitation"
    );

    Ok(Json(entity_to_response(entity, invited_by)))
}

/// DELETE /api/admin/v1/organizations/:org_id/invitations/:invite_id
///
/// Revoke a pending invitation.
/// Only pending invitations can be revoked.
pub async fn revoke_invitation(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, invite_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let invite_repo = OrgMemberInviteRepository::new(state.pool.clone());

    // Check if invitation exists and get its status
    let entity = invite_repo
        .find_by_id_and_org(invite_id, org_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Invitation not found".to_string()))?;

    // Check if already accepted
    if entity.accepted_at.is_some() {
        return Err(ApiError::Conflict(
            "Cannot revoke an accepted invitation. Remove the member instead.".to_string(),
        ));
    }

    // Delete the invitation
    let deleted = invite_repo.delete(invite_id, org_id).await?;

    if !deleted {
        warn!(
            admin_key_id = auth.api_key_id,
            organization_id = %org_id,
            invitation_id = %invite_id,
            "Attempted to revoke non-existent invitation"
        );
        return Err(ApiError::NotFound("Invitation not found".to_string()));
    }

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        invitation_id = %invite_id,
        "Revoked organization member invitation"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Convert entity to response with status calculation.
fn entity_to_response(
    entity: OrgMemberInviteEntity,
    invited_by: Option<InvitedByInfo>,
) -> InvitationResponse {
    let status = if entity.accepted_at.is_some() {
        InvitationStatus::Accepted
    } else if entity.expires_at < Utc::now() {
        InvitationStatus::Expired
    } else {
        InvitationStatus::Pending
    };

    InvitationResponse {
        id: entity.id,
        email: entity.email,
        role: entity.role,
        status,
        expires_at: entity.expires_at,
        created_at: entity.created_at,
        note: entity.note,
        invited_by,
        accepted_at: entity.accepted_at,
    }
}

// Note: The accept_invitation endpoint is a public endpoint that doesn't require
// admin authentication. It will be implemented separately with user creation logic.
// For now, we're focusing on the admin management endpoints.

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::org_member_invite::{
        CreateInvitationRequest, ListInvitationsQuery,
    };

    #[test]
    fn test_create_invitation_request_validation() {
        let valid = CreateInvitationRequest {
            email: "test@example.com".to_string(),
            role: Some("member".to_string()),
            note: Some("Welcome!".to_string()),
            expires_in_days: Some(7),
        };
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_create_invitation_invalid_email() {
        let invalid = CreateInvitationRequest {
            email: "not-an-email".to_string(),
            role: None,
            note: None,
            expires_in_days: None,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_list_query_defaults() {
        let query = ListInvitationsQuery::default();
        assert_eq!(query.page(), 1);
        assert_eq!(query.per_page(), 50);
        assert!(!query.include_accepted());
    }

    #[test]
    fn test_list_query_include_accepted() {
        let query = ListInvitationsQuery {
            status: Some("all".to_string()),
            page: None,
            per_page: None,
        };
        assert!(query.include_accepted());

        let query2 = ListInvitationsQuery {
            status: Some("accepted".to_string()),
            page: None,
            per_page: None,
        };
        assert!(query2.include_accepted());
    }
}
