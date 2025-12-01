//! Invite routes for managing group invitations.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{Duration, Utc};
use domain::models::group::GroupRole;
use domain::models::invite::{
    generate_invite_code, CreateInviteRequest, CreateInviteResponse, CreatorInfo, InviteSummary,
    ListInvitesResponse, PublicGroupInfo, PublicInviteInfo,
};
use persistence::entities::GroupRoleDb;
use persistence::repositories::{GroupRepository, InviteRepository};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

/// Create a new invite for a group.
///
/// POST /api/v1/groups/:group_id/invites
///
/// Requires JWT authentication. Only admins and owners can create invites.
pub async fn create_invite(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(group_id): Path<Uuid>,
    Json(request): Json<CreateInviteRequest>,
) -> Result<(StatusCode, Json<CreateInviteResponse>), ApiError> {
    // Validate request
    request.validate().map_err(|e| {
        let errors: Vec<String> = e
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |err| {
                    format!("{}: {}", field, err.message.as_ref().unwrap_or(&"".into()))
                })
            })
            .collect();
        ApiError::Validation(errors.join(", "))
    })?;

    let group_repo = GroupRepository::new(state.pool.clone());
    let invite_repo = InviteRepository::new(state.pool.clone());

    // Check user is a member of the group with admin/owner role
    let membership = group_repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    let role: GroupRole = membership.role.into();
    if !role.can_manage_members() {
        return Err(ApiError::Forbidden(
            "Only admins and owners can create invites".to_string(),
        ));
    }

    // Check preset role is not owner
    let preset_role = request.preset_role.unwrap_or(GroupRole::Member);
    if preset_role == GroupRole::Owner {
        return Err(ApiError::Validation(
            "Cannot create invite with owner role".to_string(),
        ));
    }

    // Generate unique code
    let code = invite_repo
        .generate_unique_code(generate_invite_code)
        .await?;

    // Calculate expiration
    let expires_in_hours = request.expires_in_hours.unwrap_or(24);
    let expires_at = Utc::now() + Duration::hours(expires_in_hours as i64);

    // Create invite
    let max_uses = request.max_uses.unwrap_or(1);
    let preset_role_db: GroupRoleDb = preset_role.into();

    let invite = invite_repo
        .create_invite(
            group_id,
            &code,
            preset_role_db,
            max_uses,
            expires_at,
            user_auth.user_id,
        )
        .await?;

    info!(
        group_id = %group_id,
        invite_id = %invite.id,
        code = %code,
        user_id = %user_auth.user_id,
        "Invite created"
    );

    // Generate invite URL (using a placeholder domain)
    let invite_url = format!("https://phonemanager.com/join/{}", code);

    Ok((
        StatusCode::CREATED,
        Json(CreateInviteResponse {
            id: invite.id,
            group_id: invite.group_id,
            code: invite.code,
            preset_role: invite.preset_role.into(),
            max_uses: invite.max_uses,
            current_uses: invite.current_uses,
            expires_at: invite.expires_at,
            created_by: invite.created_by,
            created_at: invite.created_at,
            invite_url,
        }),
    ))
}

/// List active invites for a group.
///
/// GET /api/v1/groups/:group_id/invites
///
/// Requires JWT authentication. Only admins and owners can view invites.
pub async fn list_invites(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(group_id): Path<Uuid>,
) -> Result<Json<ListInvitesResponse>, ApiError> {
    let group_repo = GroupRepository::new(state.pool.clone());
    let invite_repo = InviteRepository::new(state.pool.clone());

    // Check user is a member of the group with admin/owner role
    let membership = group_repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    let role: GroupRole = membership.role.into();
    if !role.can_manage_members() {
        return Err(ApiError::Forbidden(
            "Only admins and owners can view invites".to_string(),
        ));
    }

    // Get active invites
    let invites = invite_repo.list_active_invites(group_id).await?;

    let invite_summaries: Vec<InviteSummary> = invites
        .into_iter()
        .map(|i| InviteSummary {
            id: i.id,
            code: i.code,
            preset_role: i.preset_role.into(),
            max_uses: i.max_uses,
            current_uses: i.current_uses,
            expires_at: i.expires_at,
            created_by: CreatorInfo {
                id: i.created_by,
                display_name: i.creator_display_name,
            },
            created_at: i.created_at,
        })
        .collect();

    info!(
        group_id = %group_id,
        user_id = %user_auth.user_id,
        invite_count = invite_summaries.len(),
        "Listed invites"
    );

    Ok(Json(ListInvitesResponse {
        data: invite_summaries,
    }))
}

/// Revoke an invite.
///
/// DELETE /api/v1/groups/:group_id/invites/:invite_id
///
/// Requires JWT authentication. Only admins and owners can revoke invites.
pub async fn revoke_invite(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path((group_id, invite_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let group_repo = GroupRepository::new(state.pool.clone());
    let invite_repo = InviteRepository::new(state.pool.clone());

    // Check user is a member of the group with admin/owner role
    let membership = group_repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    let role: GroupRole = membership.role.into();
    if !role.can_manage_members() {
        return Err(ApiError::Forbidden(
            "Only admins and owners can revoke invites".to_string(),
        ));
    }

    // Check invite exists and belongs to this group
    let invite = invite_repo
        .find_by_id(invite_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Invite not found".to_string()))?;

    if invite.group_id != group_id {
        return Err(ApiError::NotFound("Invite not found".to_string()));
    }

    // Revoke the invite
    let rows_affected = invite_repo.revoke_invite(invite_id).await?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound("Invite not found".to_string()));
    }

    info!(
        group_id = %group_id,
        invite_id = %invite_id,
        user_id = %user_auth.user_id,
        "Invite revoked"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Get invite info by code (public, no auth required).
///
/// GET /api/v1/invites/:code
///
/// Returns limited group info for invite preview.
pub async fn get_invite_info(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<PublicInviteInfo>, ApiError> {
    let invite_repo = InviteRepository::new(state.pool.clone());

    // Find invite with group info
    let invite = invite_repo
        .find_by_code_with_group(&code)
        .await?
        .ok_or_else(|| ApiError::NotFound("Invite not found".to_string()))?;

    // Check if invite is valid
    let now = Utc::now();
    let is_valid =
        invite.is_active && invite.expires_at > now && invite.current_uses < invite.max_uses;

    Ok(Json(PublicInviteInfo {
        group: PublicGroupInfo {
            name: invite.group_name,
            icon_emoji: invite.group_icon_emoji,
            member_count: invite.member_count,
        },
        preset_role: invite.preset_role.into(),
        expires_at: invite.expires_at,
        is_valid,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invite_code_generation() {
        let code = generate_invite_code();
        assert_eq!(code.len(), 11);
        assert_eq!(&code[3..4], "-");
        assert_eq!(&code[7..8], "-");
    }
}
