//! Group management routes for creating and managing location sharing groups.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use domain::models::group::{
    generate_slug, CreateGroupRequest, CreateGroupResponse, GroupDetail, GroupRole, GroupSummary,
    ListGroupsQuery, ListGroupsResponse, ListMembersQuery, ListMembersResponse, MemberResponse,
    MembershipInfo, Pagination, UpdateGroupRequest, UpdateRoleRequest, UpdateRoleResponse,
    UserPublic,
};
use domain::models::invite::{JoinGroupInfo, JoinGroupRequest, JoinGroupResponse, JoinMembershipInfo};
use persistence::repositories::{GroupRepository, InviteRepository};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

/// Create a new group.
///
/// POST /api/v1/groups
///
/// Requires JWT authentication. Creator becomes the group owner.
pub async fn create_group(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Json(request): Json<CreateGroupRequest>,
) -> Result<(StatusCode, Json<CreateGroupResponse>), ApiError> {
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

    let repo = GroupRepository::new(state.pool.clone());

    // Generate a unique slug from the name
    let base_slug = generate_slug(&request.name);
    let slug = repo.generate_unique_slug(&base_slug).await?;

    // Create the group (creator becomes owner)
    let max_devices = request.max_devices.unwrap_or(20);
    let group = repo
        .create_group(
            &request.name,
            &slug,
            request.description.as_deref(),
            request.icon_emoji.as_deref(),
            max_devices,
            user_auth.user_id,
        )
        .await?;

    info!(
        group_id = %group.id,
        group_name = %group.name,
        user_id = %user_auth.user_id,
        "Group created"
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateGroupResponse {
            id: group.id,
            name: group.name,
            slug: group.slug,
            description: group.description,
            icon_emoji: group.icon_emoji,
            max_devices: group.max_devices,
            member_count: 1, // Creator is the first member
            device_count: 0,
            is_active: group.is_active,
            created_by: group.created_by,
            created_at: group.created_at,
            your_role: domain::models::group::GroupRole::Owner,
        }),
    ))
}

/// List groups the current user belongs to.
///
/// GET /api/v1/groups
///
/// Requires JWT authentication.
pub async fn list_groups(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Query(query): Query<ListGroupsQuery>,
) -> Result<Json<ListGroupsResponse>, ApiError> {
    let repo = GroupRepository::new(state.pool.clone());

    // Fetch groups for the user
    let groups = repo
        .find_user_groups(user_auth.user_id, query.role.as_deref())
        .await?;

    // Transform to response DTOs
    let group_summaries: Vec<GroupSummary> = groups
        .into_iter()
        .map(|g| GroupSummary {
            id: g.id,
            name: g.name,
            slug: g.slug,
            icon_emoji: g.icon_emoji,
            member_count: g.member_count,
            device_count: g.device_count,
            your_role: g.role.into(),
            joined_at: g.joined_at,
        })
        .collect();

    let count = group_summaries.len();

    info!(
        user_id = %user_auth.user_id,
        group_count = count,
        role_filter = ?query.role,
        "Listed user groups"
    );

    Ok(Json(ListGroupsResponse {
        data: group_summaries,
        count,
    }))
}

/// Get group details.
///
/// GET /api/v1/groups/:group_id
///
/// Requires JWT authentication. User must be a member of the group.
pub async fn get_group(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(group_id): Path<Uuid>,
) -> Result<Json<GroupDetail>, ApiError> {
    let repo = GroupRepository::new(state.pool.clone());

    // Fetch group with user's membership
    let group = repo
        .find_group_with_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    let role: domain::models::group::GroupRole = group.role.into();

    info!(
        group_id = %group_id,
        user_id = %user_auth.user_id,
        role = %role,
        "Retrieved group details"
    );

    Ok(Json(GroupDetail {
        id: group.id,
        name: group.name,
        slug: group.slug,
        description: group.description,
        icon_emoji: group.icon_emoji,
        max_devices: group.max_devices,
        member_count: group.member_count,
        device_count: group.device_count,
        is_active: group.is_active,
        settings: group.settings,
        created_by: group.created_by,
        created_at: group.created_at,
        updated_at: group.updated_at,
        your_role: role,
        your_membership: MembershipInfo {
            id: group.membership_id,
            role,
            joined_at: group.joined_at,
        },
    }))
}

/// Update group settings.
///
/// PUT /api/v1/groups/:group_id
///
/// Requires JWT authentication. Only admins and owners can update.
pub async fn update_group(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(group_id): Path<Uuid>,
    Json(request): Json<UpdateGroupRequest>,
) -> Result<Json<GroupDetail>, ApiError> {
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

    let repo = GroupRepository::new(state.pool.clone());

    // Check user's membership and role
    let membership = repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    // Check permission
    let membership_role: domain::models::group::GroupRole = membership.role.into();
    if !membership_role.can_manage_group() {
        return Err(ApiError::Forbidden(
            "Only group admins and owners can update group settings".to_string(),
        ));
    }

    // Generate new slug if name is being changed
    let new_slug = if let Some(ref new_name) = request.name {
        let base_slug = generate_slug(new_name);
        Some(repo.generate_unique_slug(&base_slug).await?)
    } else {
        None
    };

    // Update the group
    let updated_group = repo
        .update_group(
            group_id,
            request.name.as_deref(),
            new_slug.as_deref(),
            request.description.as_deref(),
            request.icon_emoji.as_deref(),
            request.max_devices,
        )
        .await?;

    info!(
        group_id = %group_id,
        user_id = %user_auth.user_id,
        "Group updated"
    );

    // Fetch updated group with membership info
    let group = repo
        .find_group_with_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::Internal("Failed to fetch updated group".to_string()))?;

    let role: domain::models::group::GroupRole = group.role.into();

    Ok(Json(GroupDetail {
        id: updated_group.id,
        name: updated_group.name,
        slug: updated_group.slug,
        description: updated_group.description,
        icon_emoji: updated_group.icon_emoji,
        max_devices: updated_group.max_devices,
        member_count: group.member_count,
        device_count: group.device_count,
        is_active: updated_group.is_active,
        settings: updated_group.settings,
        created_by: updated_group.created_by,
        created_at: updated_group.created_at,
        updated_at: updated_group.updated_at,
        your_role: role,
        your_membership: MembershipInfo {
            id: group.membership_id,
            role,
            joined_at: group.joined_at,
        },
    }))
}

/// Delete a group.
///
/// DELETE /api/v1/groups/:group_id
///
/// Requires JWT authentication. Only the owner can delete.
pub async fn delete_group(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(group_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let repo = GroupRepository::new(state.pool.clone());

    // Check user's membership and role
    let membership = repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    // Check permission - only owner can delete
    let membership_role: domain::models::group::GroupRole = membership.role.into();
    if !membership_role.can_delete_group() {
        return Err(ApiError::Forbidden(
            "Only the group owner can delete the group".to_string(),
        ));
    }

    // Delete the group (soft delete)
    let rows_affected = repo.delete_group(group_id).await?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound("Group not found".to_string()));
    }

    info!(
        group_id = %group_id,
        user_id = %user_auth.user_id,
        "Group deleted"
    );

    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// Membership Endpoints (Story 11.2)
// =============================================================================

/// List group members.
///
/// GET /api/v1/groups/:group_id/members
///
/// Requires JWT authentication. User must be a member of the group.
pub async fn list_members(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(group_id): Path<Uuid>,
    Query(query): Query<ListMembersQuery>,
) -> Result<Json<ListMembersResponse>, ApiError> {
    let repo = GroupRepository::new(state.pool.clone());

    // Check user is a member of the group
    let _membership = repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    // Pagination defaults
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    // Get total count
    let total = repo
        .count_members(group_id, query.role.as_deref())
        .await?;

    // Get members
    let members = repo
        .list_members(group_id, query.role.as_deref(), per_page, offset)
        .await?;

    // Transform to response DTOs
    let member_responses: Vec<MemberResponse> = members
        .into_iter()
        .map(|m| MemberResponse {
            id: m.id,
            user: UserPublic {
                id: m.user_id,
                display_name: m.display_name,
                avatar_url: m.avatar_url,
            },
            role: m.role.into(),
            joined_at: m.joined_at,
            invited_by: m.invited_by,
            devices: None, // TODO: implement include_devices in future story
        })
        .collect();

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    info!(
        group_id = %group_id,
        user_id = %user_auth.user_id,
        member_count = member_responses.len(),
        page = page,
        "Listed group members"
    );

    Ok(Json(ListMembersResponse {
        data: member_responses,
        pagination: Pagination {
            page,
            per_page,
            total,
            total_pages,
        },
    }))
}

/// Get member details.
///
/// GET /api/v1/groups/:group_id/members/:user_id
///
/// Requires JWT authentication. User must be a member of the group.
pub async fn get_member(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path((group_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<MemberResponse>, ApiError> {
    let repo = GroupRepository::new(state.pool.clone());

    // Check user is a member of the group
    let _membership = repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    // Get target member
    let member = repo
        .get_member_with_user(group_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Member not found".to_string()))?;

    info!(
        group_id = %group_id,
        user_id = %user_auth.user_id,
        target_user_id = %target_user_id,
        "Retrieved member details"
    );

    Ok(Json(MemberResponse {
        id: member.id,
        user: UserPublic {
            id: member.user_id,
            display_name: member.display_name,
            avatar_url: member.avatar_url,
        },
        role: member.role.into(),
        joined_at: member.joined_at,
        invited_by: member.invited_by,
        devices: None,
    }))
}

/// Remove member from group.
///
/// DELETE /api/v1/groups/:group_id/members/:user_id
///
/// Requires JWT authentication.
/// - Admins/owners can remove other members (but not the owner)
/// - Members can remove themselves (leave group)
/// - Owner cannot leave without transferring ownership first
pub async fn remove_member(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path((group_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let repo = GroupRepository::new(state.pool.clone());

    // Check user is a member of the group
    let actor_membership = repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    let actor_role: GroupRole = actor_membership.role.into();
    let is_self_removal = user_auth.user_id == target_user_id;

    if is_self_removal {
        // Self-removal (leaving the group)
        if actor_role == GroupRole::Owner {
            return Err(ApiError::Forbidden(
                "Owner cannot leave the group. Transfer ownership first.".to_string(),
            ));
        }
        // Allow self-removal for non-owners
    } else {
        // Removing another member
        if !actor_role.can_manage_members() {
            return Err(ApiError::Forbidden(
                "Only admins and owners can remove other members".to_string(),
            ));
        }

        // Check target member exists and their role
        let target_membership = repo
            .get_membership(group_id, target_user_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Member not found".to_string()))?;

        let target_role: GroupRole = target_membership.role.into();

        // Cannot remove the owner
        if target_role == GroupRole::Owner {
            return Err(ApiError::Forbidden(
                "Cannot remove the group owner. Transfer ownership first.".to_string(),
            ));
        }

        // Admins cannot remove other admins (only owner can)
        if actor_role == GroupRole::Admin && target_role == GroupRole::Admin {
            return Err(ApiError::Forbidden(
                "Admins cannot remove other admins".to_string(),
            ));
        }
    }

    // Remove the member
    let rows_affected = repo.remove_member(group_id, target_user_id).await?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound("Member not found".to_string()));
    }

    info!(
        group_id = %group_id,
        actor_user_id = %user_auth.user_id,
        removed_user_id = %target_user_id,
        is_self_removal = is_self_removal,
        "Member removed from group"
    );

    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// Role Management Endpoints (Story 11.3)
// =============================================================================

/// Update member role.
///
/// PUT /api/v1/groups/:group_id/members/:user_id/role
///
/// Requires JWT authentication.
/// - Only admins and owners can change roles
/// - Cannot change owner's role (use transfer endpoint)
/// - Cannot promote to owner (use transfer endpoint)
/// - Admins cannot promote others to admin (only owner can)
pub async fn update_member_role(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path((group_id, target_user_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateRoleRequest>,
) -> Result<Json<UpdateRoleResponse>, ApiError> {
    let repo = GroupRepository::new(state.pool.clone());

    // Check actor is a member of the group
    let actor_membership = repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    let actor_role: GroupRole = actor_membership.role.into();

    // Only admins and owners can change roles
    if !actor_role.can_manage_members() {
        return Err(ApiError::Forbidden(
            "Only admins and owners can change member roles".to_string(),
        ));
    }

    // Cannot promote anyone to owner via this endpoint
    if request.role == GroupRole::Owner {
        return Err(ApiError::Forbidden(
            "Cannot promote to owner. Use the transfer ownership endpoint.".to_string(),
        ));
    }

    // Get target member
    let target_membership = repo
        .get_membership(group_id, target_user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Member not found".to_string()))?;

    let target_role: GroupRole = target_membership.role.into();

    // Cannot change owner's role
    if target_role == GroupRole::Owner {
        return Err(ApiError::Forbidden(
            "Cannot change the owner's role. Use the transfer ownership endpoint.".to_string(),
        ));
    }

    // Admins cannot promote others to admin (only owner can)
    if actor_role == GroupRole::Admin && request.role == GroupRole::Admin {
        return Err(ApiError::Forbidden(
            "Only the owner can promote members to admin".to_string(),
        ));
    }

    // Admins cannot change other admins' roles (only owner can)
    if actor_role == GroupRole::Admin && target_role == GroupRole::Admin {
        return Err(ApiError::Forbidden(
            "Admins cannot change other admins' roles".to_string(),
        ));
    }

    // Update the role
    let updated = repo
        .update_member_role(group_id, target_user_id, request.role)
        .await?;

    info!(
        group_id = %group_id,
        actor_user_id = %user_auth.user_id,
        target_user_id = %target_user_id,
        new_role = %request.role,
        "Member role updated"
    );

    Ok(Json(UpdateRoleResponse {
        id: updated.id,
        user_id: updated.user_id,
        group_id: updated.group_id,
        role: updated.role.into(),
        updated_at: updated.updated_at,
    }))
}

// =============================================================================
// Join Group with Invite Code (Story 11.5)
// =============================================================================

/// Join a group using an invite code.
///
/// POST /api/v1/groups/join
///
/// Requires JWT authentication.
/// Returns 400 for invalid code format.
/// Returns 404 if invite not found.
/// Returns 409 if already a member.
/// Returns 410 if invite expired or fully used.
pub async fn join_group(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Json(request): Json<JoinGroupRequest>,
) -> Result<Json<JoinGroupResponse>, ApiError> {
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

    // Find the invite by code
    let invite = invite_repo
        .find_by_code_with_group(&request.code)
        .await?
        .ok_or_else(|| ApiError::NotFound("Invite not found".to_string()))?;

    // Check if invite is still valid
    let now = Utc::now();
    if !invite.is_active {
        return Err(ApiError::Gone("Invite has been revoked".to_string()));
    }
    if invite.expires_at <= now {
        return Err(ApiError::Gone("Invite has expired".to_string()));
    }
    if invite.current_uses >= invite.max_uses {
        return Err(ApiError::Gone("Invite has reached maximum uses".to_string()));
    }

    // Check if user is already a member
    if group_repo
        .get_membership(invite.group_id, user_auth.user_id)
        .await?
        .is_some()
    {
        return Err(ApiError::Conflict(
            "You are already a member of this group".to_string(),
        ));
    }

    // Convert preset_role from DB enum to domain enum
    let preset_role: GroupRole = invite.preset_role.into();

    // Add user as member with preset role
    let membership = group_repo
        .add_member(invite.group_id, user_auth.user_id, preset_role, None)
        .await?;

    // Increment invite use count
    invite_repo.increment_use_count(invite.id).await?;

    info!(
        group_id = %invite.group_id,
        user_id = %user_auth.user_id,
        invite_code = %request.code,
        role = %preset_role,
        "User joined group via invite"
    );

    Ok(Json(JoinGroupResponse {
        group: JoinGroupInfo {
            id: invite.group_id,
            name: invite.group_name,
            member_count: invite.member_count + 1, // Include newly joined user
        },
        membership: JoinMembershipInfo {
            id: membership.id,
            role: membership.role.into(),
            joined_at: membership.joined_at,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug_simple() {
        assert_eq!(generate_slug("My Family"), "my-family");
    }

    #[test]
    fn test_generate_slug_special_chars() {
        assert_eq!(generate_slug("Family Group!@#"), "family-group");
    }

    #[test]
    fn test_generate_slug_multiple_spaces() {
        assert_eq!(generate_slug("My   Family   Group"), "my-family-group");
    }

    #[test]
    fn test_group_role_permissions() {
        assert!(GroupRole::Owner.can_manage_group());
        assert!(GroupRole::Owner.can_delete_group());
        assert!(GroupRole::Owner.can_manage_members());
        assert!(GroupRole::Admin.can_manage_group());
        assert!(!GroupRole::Admin.can_delete_group());
        assert!(GroupRole::Admin.can_manage_members());
        assert!(!GroupRole::Member.can_manage_group());
        assert!(!GroupRole::Member.can_manage_members());
        assert!(!GroupRole::Viewer.can_manage_group());
        assert!(!GroupRole::Viewer.can_manage_members());
    }
}
