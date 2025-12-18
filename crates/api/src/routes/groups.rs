//! Group management routes for creating and managing location sharing groups.

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use domain::models::device::{DeviceLastLocation, DeviceSummary};
use domain::models::group::{
    generate_slug, CreateGroupRequest, CreateGroupResponse, GroupDetail, GroupRole, GroupSummary,
    LastLocationInfo, ListGroupsQuery, ListGroupsResponse, ListMembersQuery, ListMembersResponse,
    MemberDeviceInfo, MemberResponse, MembershipInfo, Pagination, TransferOwnershipRequest,
    TransferOwnershipResponse, UpdateGroupRequest, UpdateRoleRequest, UpdateRoleResponse,
    UserPublic,
};
use domain::models::invite::{
    JoinGroupInfo, JoinGroupRequest, JoinGroupResponse, JoinMembershipInfo,
};
use persistence::entities::MemberDeviceEntity;
use persistence::repositories::{
    DeviceGroupMembershipRepository, DeviceRepository, GroupRepository, InviteRepository,
    MigrationAuditRepository,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

/// Threshold in minutes for considering a device as online.
/// A device is considered online if it was last seen within this duration.
const DEVICE_ONLINE_THRESHOLD_MINUTES: i64 = 5;

/// Convert a database device entity to the API response format.
fn to_member_device_info(device: MemberDeviceEntity, now: DateTime<Utc>) -> MemberDeviceInfo {
    let online_threshold = chrono::Duration::minutes(DEVICE_ONLINE_THRESHOLD_MINUTES);
    let is_online = device
        .last_seen_at
        .map(|seen| now - seen < online_threshold)
        .unwrap_or(false);

    MemberDeviceInfo {
        device_id: device.device_id,
        name: Some(device.display_name),
        is_online,
        last_location: match (
            device.last_latitude,
            device.last_longitude,
            device.last_location_time,
        ) {
            (Some(lat), Some(lon), Some(time)) => Some(LastLocationInfo {
                latitude: lat,
                longitude: lon,
                timestamp: time,
            }),
            _ => None,
        },
    }
}

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
///
/// Query parameters:
/// - `role`: Optional filter by membership role (e.g., "admin", "member")
/// - `device_id`: Optional device ID to check assignment for.
///   If provided, `has_current_device` will be true only for groups containing this specific device.
///   If not provided, `has_current_device` will be true for groups containing ANY of the user's devices.
pub async fn list_groups(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Query(query): Query<ListGroupsQuery>,
) -> Result<Json<ListGroupsResponse>, ApiError> {
    let repo = GroupRepository::new(state.pool.clone());

    // Fetch groups for the user with device assignment check
    let groups = repo
        .find_user_groups(user_auth.user_id, query.role.as_deref(), query.device_id)
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
            has_current_device: g.has_current_device,
        })
        .collect();

    let count = group_summaries.len();

    info!(
        user_id = %user_auth.user_id,
        group_count = count,
        role_filter = ?query.role,
        device_id = ?query.device_id,
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

    // Fetch group with user's membership (no specific device_id for detail view)
    let group = repo
        .find_group_with_membership(group_id, user_auth.user_id, None)
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
        .find_group_with_membership(group_id, user_auth.user_id, None)
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
/// Enhanced with device count per member (Story UGM-3.6).
pub async fn list_members(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(group_id): Path<Uuid>,
    Query(query): Query<ListMembersQuery>,
) -> Result<Json<ListMembersResponse>, ApiError> {
    let repo = GroupRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());
    let membership_repo = DeviceGroupMembershipRepository::new(state.pool.clone());

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
    let total = repo.count_members(group_id, query.role.as_deref()).await?;

    // Get members
    let members = repo
        .list_members(group_id, query.role.as_deref(), per_page, offset)
        .await?;

    // Collect user IDs and fetch their devices
    let user_ids: Vec<Uuid> = members.iter().map(|m| m.user_id).collect();
    let all_devices = device_repo.find_devices_by_users(&user_ids).await?;

    // Group devices by user_id
    let now = Utc::now();
    let mut devices_by_user: HashMap<Uuid, Vec<MemberDeviceInfo>> = HashMap::new();
    for device in all_devices {
        let owner_id = device.owner_user_id;
        let device_info = to_member_device_info(device, now);
        devices_by_user
            .entry(owner_id)
            .or_default()
            .push(device_info);
    }

    // Get device counts per user in this group (Story UGM-3.6)
    let device_counts = membership_repo
        .count_devices_per_user_in_group(group_id)
        .await?;
    let device_count_map: HashMap<Uuid, i64> = device_counts.into_iter().collect();

    // Transform to response DTOs with devices and device count
    let member_responses: Vec<MemberResponse> = members
        .into_iter()
        .map(|m| {
            let user_devices = devices_by_user.remove(&m.user_id);
            let device_count = device_count_map.get(&m.user_id).copied().unwrap_or(0);
            MemberResponse {
                id: m.id,
                user: UserPublic {
                    id: m.user_id,
                    display_name: m.display_name,
                    avatar_url: m.avatar_url,
                },
                role: m.role.into(),
                joined_at: m.joined_at,
                invited_by: m.invited_by,
                devices: user_devices,
                device_count,
            }
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
/// Enhanced with device count (Story UGM-3.6).
pub async fn get_member(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path((group_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<MemberResponse>, ApiError> {
    let repo = GroupRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());
    let membership_repo = DeviceGroupMembershipRepository::new(state.pool.clone());

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

    // Fetch devices for the member
    let member_devices = device_repo.find_devices_by_users(&[target_user_id]).await?;
    let now = Utc::now();
    let devices: Vec<MemberDeviceInfo> = member_devices
        .into_iter()
        .map(|d| to_member_device_info(d, now))
        .collect();

    // Get device count for this user in this group (Story UGM-3.6)
    let device_counts = membership_repo
        .count_devices_per_user_in_group(group_id)
        .await?;
    let device_count = device_counts
        .into_iter()
        .find(|(user_id, _)| *user_id == target_user_id)
        .map(|(_, count)| count)
        .unwrap_or(0);

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
        devices: if devices.is_empty() {
            None
        } else {
            Some(devices)
        },
        device_count,
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
        return Err(ApiError::Gone(
            "Invite has reached maximum uses".to_string(),
        ));
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

// =============================================================================
// Ownership Transfer (Story 11.6)
// =============================================================================

/// Transfer group ownership to another member.
///
/// POST /api/v1/groups/:group_id/transfer
///
/// Requires JWT authentication. Only the owner can transfer ownership.
/// Target user must be an existing member of the group.
/// The current owner will be demoted to admin.
pub async fn transfer_ownership(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(group_id): Path<Uuid>,
    Json(request): Json<TransferOwnershipRequest>,
) -> Result<Json<TransferOwnershipResponse>, ApiError> {
    let repo = GroupRepository::new(state.pool.clone());

    // Check user is a member of the group and get their role
    let actor_membership = repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    let actor_role: GroupRole = actor_membership.role.into();

    // Only owner can transfer ownership
    if !actor_role.can_transfer_ownership() {
        return Err(ApiError::Forbidden(
            "Only the group owner can transfer ownership".to_string(),
        ));
    }

    // Cannot transfer to yourself
    if request.new_owner_id == user_auth.user_id {
        return Err(ApiError::Validation(
            "Cannot transfer ownership to yourself".to_string(),
        ));
    }

    // Check target user is a member of the group
    let _target_membership = repo
        .get_membership(group_id, request.new_owner_id)
        .await?
        .ok_or_else(|| {
            ApiError::Validation("Target user is not a member of this group".to_string())
        })?;

    // Transfer ownership atomically
    repo.transfer_ownership(group_id, user_auth.user_id, request.new_owner_id)
        .await?;

    let transferred_at = Utc::now();

    info!(
        group_id = %group_id,
        previous_owner_id = %user_auth.user_id,
        new_owner_id = %request.new_owner_id,
        "Group ownership transferred"
    );

    Ok(Json(TransferOwnershipResponse {
        group_id,
        previous_owner_id: user_auth.user_id,
        new_owner_id: request.new_owner_id,
        transferred_at,
    }))
}

// =============================================================================
// Group Devices (Story 12.7 - JWT-authenticated endpoint)
// =============================================================================

/// Response for listing group devices.
#[derive(Debug, serde::Serialize)]
pub struct GroupDevicesResponse {
    pub devices: Vec<DeviceSummary>,
}

// =============================================================================
// Group Migration (Story UGM-2.2)
// =============================================================================

/// Request to migrate a registration group to an authenticated group.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct MigrateGroupRequest {
    /// The registration group ID (string) to migrate from.
    #[validate(length(
        min = 1,
        max = 255,
        message = "Registration group ID must be 1-255 characters"
    ))]
    pub registration_group_id: String,

    /// Optional custom name for the new authenticated group.
    /// If not provided, the registration_group_id will be used as the name.
    #[validate(length(min = 1, max = 100, message = "Group name must be 1-100 characters"))]
    pub group_name: Option<String>,
}

/// Response from a successful group migration.
#[derive(Debug, Clone, Serialize)]
pub struct MigrateGroupResponse {
    /// The migration audit log ID for tracking.
    pub migration_id: Uuid,

    /// The new authenticated group ID.
    pub authenticated_group_id: Uuid,

    /// The name of the new authenticated group.
    pub name: String,

    /// Number of devices migrated.
    pub devices_migrated: i32,

    /// IDs of the devices that were migrated.
    pub device_ids: Vec<Uuid>,
}

/// Migrate a registration group to an authenticated group.
///
/// POST /api/v1/groups/migrate
///
/// Requires JWT authentication. The user must own at least one device in the
/// registration group. Creates a new authenticated group and migrates all
/// devices from the registration group. The operation is atomic.
pub async fn migrate_registration_group(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Json(request): Json<MigrateGroupRequest>,
) -> Result<(StatusCode, Json<MigrateGroupResponse>), ApiError> {
    use crate::middleware::metrics::{record_migration_failure, record_migration_success};
    use std::time::Instant;

    let start_time = Instant::now();

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
        record_migration_failure(start_time.elapsed().as_secs_f64(), "validation_error");
        ApiError::Validation(errors.join(", "))
    })?;

    let group_repo = GroupRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());
    let migration_repo = MigrationAuditRepository::new(state.pool.clone());

    // Check if already migrated
    if migration_repo
        .is_already_migrated(&request.registration_group_id)
        .await?
    {
        // Get the existing migration to provide details
        if let Some(existing) = migration_repo
            .get_migration_for_registration_group(&request.registration_group_id)
            .await?
        {
            warn!(
                registration_group_id = %request.registration_group_id,
                authenticated_group_id = %existing.authenticated_group_id,
                user_id = %user_auth.user_id,
                "Attempt to migrate already-migrated group"
            );
            record_migration_failure(start_time.elapsed().as_secs_f64(), "already_migrated");
            return Err(ApiError::Conflict(format!(
                "Registration group '{}' has already been migrated to authenticated group {}",
                request.registration_group_id, existing.authenticated_group_id
            )));
        }
        record_migration_failure(start_time.elapsed().as_secs_f64(), "already_migrated");
        return Err(ApiError::Conflict(format!(
            "Registration group '{}' has already been migrated",
            request.registration_group_id
        )));
    }

    // Get devices in the registration group
    let devices = device_repo
        .find_devices_by_registration_group(&request.registration_group_id)
        .await?;

    if devices.is_empty() {
        record_migration_failure(start_time.elapsed().as_secs_f64(), "no_devices");
        return Err(ApiError::Validation(format!(
            "Registration group '{}' has no devices to migrate",
            request.registration_group_id
        )));
    }

    // Verify the user owns at least one device in the registration group
    let user_owns_device = devices
        .iter()
        .any(|d| d.owner_user_id == Some(user_auth.user_id));
    if !user_owns_device {
        record_migration_failure(start_time.elapsed().as_secs_f64(), "not_device_owner");
        return Err(ApiError::Forbidden(
            "You must own at least one device in the registration group to migrate it".to_string(),
        ));
    }

    // Determine the group name
    let group_name = request
        .group_name
        .clone()
        .unwrap_or_else(|| request.registration_group_id.clone());

    // Check if group name already exists
    if group_repo
        .find_by_slug(&generate_slug(&group_name))
        .await?
        .is_some()
    {
        record_migration_failure(start_time.elapsed().as_secs_f64(), "group_name_exists");
        return Err(ApiError::Conflict(format!(
            "A group with name '{}' already exists",
            group_name
        )));
    }

    // Generate a unique slug
    let base_slug = generate_slug(&group_name);
    let slug = group_repo.generate_unique_slug(&base_slug).await?;

    // Collect device IDs for the migration
    let device_ids: Vec<Uuid> = devices.iter().map(|d| d.device_id).collect();
    let devices_count = device_ids.len() as i32;

    // Start transaction for atomic migration
    let mut tx = state.pool.begin().await.map_err(|e| {
        error!(error = %e, "Failed to start transaction for migration");
        record_migration_failure(
            start_time.elapsed().as_secs_f64(),
            "transaction_start_error",
        );
        ApiError::Internal("Failed to start migration transaction".to_string())
    })?;

    // Create the new authenticated group (user becomes owner)
    let new_group = sqlx::query_as::<_, persistence::entities::GroupEntity>(
        r#"
        INSERT INTO groups (name, slug, max_devices, created_by, is_active)
        VALUES ($1, $2, $3, $4, true)
        RETURNING id, name, slug, description, icon_emoji, max_devices, is_active, settings, created_by, created_at, updated_at
        "#,
    )
    .bind(&group_name)
    .bind(&slug)
    .bind(20) // Default max devices
    .bind(user_auth.user_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to create group during migration");
        record_migration_failure(start_time.elapsed().as_secs_f64(), "create_group_error");
        ApiError::Internal("Failed to create authenticated group".to_string())
    })?;

    // Add the user as owner of the group
    sqlx::query(
        r#"
        INSERT INTO group_memberships (user_id, group_id, role, invited_by, joined_at)
        VALUES ($1, $2, 'owner', NULL, NOW())
        "#,
    )
    .bind(user_auth.user_id)
    .bind(new_group.id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to add owner membership during migration");
        record_migration_failure(
            start_time.elapsed().as_secs_f64(),
            "create_membership_error",
        );
        ApiError::Internal("Failed to create group membership".to_string())
    })?;

    // Update all devices to point to the new authenticated group (using slug)
    // Note: devices.group_id is a VARCHAR containing the registration group ID or slug
    sqlx::query(
        r#"
        UPDATE devices
        SET group_id = $1, updated_at = NOW()
        WHERE group_id = $2 AND is_active = true
        "#,
    )
    .bind(&slug)
    .bind(&request.registration_group_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to update devices during migration");
        record_migration_failure(start_time.elapsed().as_secs_f64(), "update_devices_error");
        ApiError::Internal("Failed to migrate devices".to_string())
    })?;

    // Create migration audit log
    let audit_log = sqlx::query_as::<_, persistence::entities::MigrationAuditLogEntity>(
        r#"
        INSERT INTO migration_audit_logs (
            user_id,
            registration_group_id,
            authenticated_group_id,
            devices_migrated,
            device_ids,
            status,
            error_message
        )
        VALUES ($1, $2, $3, $4, $5, 'success', NULL)
        RETURNING id, user_id, registration_group_id, authenticated_group_id, devices_migrated, device_ids, status, error_message, created_at
        "#,
    )
    .bind(user_auth.user_id)
    .bind(&request.registration_group_id)
    .bind(new_group.id)
    .bind(devices_count)
    .bind(&device_ids)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to create migration audit log");
        record_migration_failure(start_time.elapsed().as_secs_f64(), "audit_log_error");
        ApiError::Internal("Failed to create migration audit log".to_string())
    })?;

    // Commit the transaction
    tx.commit().await.map_err(|e| {
        error!(error = %e, "Failed to commit migration transaction");
        record_migration_failure(start_time.elapsed().as_secs_f64(), "commit_error");
        ApiError::Internal("Failed to complete migration".to_string())
    })?;

    // Record successful migration metrics
    record_migration_success(start_time.elapsed().as_secs_f64(), devices_count);

    info!(
        migration_id = %audit_log.id,
        user_id = %user_auth.user_id,
        registration_group_id = %request.registration_group_id,
        authenticated_group_id = %new_group.id,
        devices_migrated = devices_count,
        "Registration group migrated successfully"
    );

    Ok((
        StatusCode::CREATED,
        Json(MigrateGroupResponse {
            migration_id: audit_log.id,
            authenticated_group_id: new_group.id,
            name: new_group.name,
            devices_migrated: devices_count,
            device_ids,
        }),
    ))
}

/// Get devices in a group.
///
/// GET /api/v1/groups/:group_id/devices
///
/// Requires JWT authentication. User must be a member of the group.
/// Returns devices with their last location information.
pub async fn get_group_devices(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(group_id): Path<Uuid>,
) -> Result<Json<GroupDevicesResponse>, ApiError> {
    let group_repo = GroupRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());

    // Verify user is a member of the group and get group details
    let group = group_repo
        .find_group_with_membership(group_id, user_auth.user_id, None)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    // Fetch devices using the group's slug (group_id column in devices table is the slug)
    let devices = device_repo
        .find_devices_with_last_location(&group.slug)
        .await?;

    // Transform to DeviceSummary
    let summaries: Vec<DeviceSummary> = devices
        .into_iter()
        .map(|d| {
            let last_location = match (
                d.last_latitude,
                d.last_longitude,
                d.last_location_time,
                d.last_accuracy,
            ) {
                (Some(lat), Some(lon), Some(time), Some(acc)) => Some(DeviceLastLocation {
                    latitude: lat,
                    longitude: lon,
                    timestamp: time,
                    accuracy: acc as f64,
                }),
                _ => None,
            };

            DeviceSummary {
                device_id: d.device_id,
                display_name: d.display_name,
                last_location,
                last_seen_at: d.last_seen_at,
            }
        })
        .collect();

    info!(
        group_id = %group_id,
        user_id = %user_auth.user_id,
        device_count = summaries.len(),
        "Listed group devices"
    );

    Ok(Json(GroupDevicesResponse { devices: summaries }))
}

// =============================================================================
// Device-Group Management (Epic UGM-3)
// =============================================================================

/// Request to add a device to a group.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct AddDeviceToGroupRequest {
    /// The device ID to add to the group.
    pub device_id: Uuid,
}

/// Response after adding a device to a group.
#[derive(Debug, Clone, Serialize)]
pub struct AddDeviceToGroupResponse {
    pub group_id: Uuid,
    pub device_id: Uuid,
    pub added_at: DateTime<Utc>,
}

/// Add a device to an authenticated group.
///
/// POST /api/v1/groups/:group_id/devices/add
///
/// Requires JWT authentication.
/// - User must be a member of the group
/// - User must own the device being added
/// - Device must not already be in the group
///
/// Story UGM-3.2: Add Device to Group
pub async fn add_device_to_group(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(group_id): Path<Uuid>,
    Json(request): Json<AddDeviceToGroupRequest>,
) -> Result<(StatusCode, Json<AddDeviceToGroupResponse>), ApiError> {
    let group_repo = GroupRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());
    let membership_repo = DeviceGroupMembershipRepository::new(state.pool.clone());

    // Check user is a member of the group
    let _membership = group_repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    // Check the device exists and is owned by the user
    let device = device_repo
        .find_by_device_id(request.device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    if device.owner_user_id != Some(user_auth.user_id) {
        return Err(ApiError::Forbidden(
            "You can only add devices you own to a group".to_string(),
        ));
    }

    // Check device is not already in the group
    if membership_repo
        .is_device_in_group(request.device_id, group_id)
        .await?
    {
        return Err(ApiError::Conflict(
            "Device is already in this group".to_string(),
        ));
    }

    // Add the device to the group
    let device_membership = membership_repo
        .add_device_to_group(request.device_id, group_id, user_auth.user_id)
        .await?;

    info!(
        group_id = %group_id,
        device_id = %request.device_id,
        user_id = %user_auth.user_id,
        "Device added to group"
    );

    Ok((
        StatusCode::CREATED,
        Json(AddDeviceToGroupResponse {
            group_id,
            device_id: request.device_id,
            added_at: device_membership.added_at,
        }),
    ))
}

/// Query parameters for listing group devices.
#[derive(Debug, Clone, Deserialize)]
pub struct ListGroupDevicesQuery {
    /// Include last location for each device
    #[serde(default)]
    pub include_location: bool,

    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: i64,

    /// Items per page (1-100)
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

/// A device in a group with its details.
#[derive(Debug, Clone, Serialize)]
pub struct GroupDeviceInfo {
    pub device_id: Uuid,
    pub display_name: String,
    pub owner_user_id: Option<Uuid>,
    pub owner_display_name: Option<String>,
    pub added_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_location: Option<DeviceLocationInfo>,
}

/// Last location information for a device.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceLocationInfo {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f32,
    pub timestamp: DateTime<Utc>,
}

/// Response for listing group devices with pagination.
#[derive(Debug, Clone, Serialize)]
pub struct ListGroupDevicesResponse {
    pub data: Vec<GroupDeviceInfo>,
    pub pagination: Pagination,
}

/// List devices in an authenticated group.
///
/// GET /api/v1/groups/:group_id/devices/members
///
/// Requires JWT authentication.
/// - User must be a member of the group
/// - Optionally include last location with `include_location=true`
/// - Supports pagination
///
/// Story UGM-3.4: List Group Devices
pub async fn list_group_devices(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(group_id): Path<Uuid>,
    Query(query): Query<ListGroupDevicesQuery>,
) -> Result<Json<ListGroupDevicesResponse>, ApiError> {
    let group_repo = GroupRepository::new(state.pool.clone());
    let membership_repo = DeviceGroupMembershipRepository::new(state.pool.clone());

    // Check user is a member of the group
    let _membership = group_repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    // Pagination
    let page = query.page.max(1);
    let per_page = query.per_page.clamp(1, 100);
    let offset = (page - 1) * per_page;

    // Get total count
    let total = membership_repo.count_devices_in_group(group_id).await?;

    // Get devices with or without location
    let devices: Vec<GroupDeviceInfo> = if query.include_location {
        let device_entities = membership_repo
            .list_devices_in_group_with_location(group_id, per_page, offset)
            .await?;

        device_entities
            .into_iter()
            .map(|d| GroupDeviceInfo {
                device_id: d.device_id,
                display_name: d.display_name,
                owner_user_id: d.owner_user_id,
                owner_display_name: d.owner_display_name,
                added_at: d.added_at,
                last_seen_at: d.last_seen_at,
                last_location: match (d.latitude, d.longitude, d.accuracy, d.location_timestamp) {
                    (Some(lat), Some(lon), Some(acc), Some(ts)) => Some(DeviceLocationInfo {
                        latitude: lat,
                        longitude: lon,
                        accuracy: acc,
                        timestamp: ts,
                    }),
                    _ => None,
                },
            })
            .collect()
    } else {
        let device_entities = membership_repo
            .list_devices_in_group(group_id, per_page, offset)
            .await?;

        device_entities
            .into_iter()
            .map(|d| GroupDeviceInfo {
                device_id: d.device_id,
                display_name: d.display_name,
                owner_user_id: d.owner_user_id,
                owner_display_name: d.owner_display_name,
                added_at: d.added_at,
                last_seen_at: d.last_seen_at,
                last_location: None,
            })
            .collect()
    };

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    info!(
        group_id = %group_id,
        user_id = %user_auth.user_id,
        device_count = devices.len(),
        page = page,
        include_location = query.include_location,
        "Listed group devices"
    );

    Ok(Json(ListGroupDevicesResponse {
        data: devices,
        pagination: Pagination {
            page,
            per_page,
            total,
            total_pages,
        },
    }))
}

/// Remove a device from an authenticated group.
///
/// DELETE /api/v1/groups/:group_id/devices/:device_id
///
/// Requires JWT authentication.
/// - Device owner can remove their own device
/// - Group admin/owner can remove any device
///
/// Story UGM-3.3: Remove Device from Group
pub async fn remove_device_from_group(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path((group_id, device_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let group_repo = GroupRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());
    let membership_repo = DeviceGroupMembershipRepository::new(state.pool.clone());

    // Check user is a member of the group and get their role
    let user_membership = group_repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Group not found or you are not a member".to_string()))?;

    let user_role: GroupRole = user_membership.role.into();

    // Check the device exists
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Check device is in the group
    if !membership_repo
        .is_device_in_group(device_id, group_id)
        .await?
    {
        return Err(ApiError::NotFound(
            "Device is not in this group".to_string(),
        ));
    }

    // Authorization: device owner or group admin/owner
    let is_device_owner = device.owner_user_id == Some(user_auth.user_id);
    let is_group_admin = user_role.can_manage_members();

    if !is_device_owner && !is_group_admin {
        return Err(ApiError::Forbidden(
            "You can only remove your own devices or must be a group admin".to_string(),
        ));
    }

    // Remove the device from the group
    let rows_affected = membership_repo
        .remove_device_from_group(device_id, group_id)
        .await?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(
            "Device is not in this group".to_string(),
        ));
    }

    info!(
        group_id = %group_id,
        device_id = %device_id,
        user_id = %user_auth.user_id,
        is_device_owner = is_device_owner,
        is_group_admin = is_group_admin,
        "Device removed from group"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Group info for a device's membership.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceGroupMembershipInfo {
    pub group_id: Uuid,
    pub name: String,
    pub slug: String,
    pub role: GroupRole,
    pub added_at: DateTime<Utc>,
}

/// Query parameters for listing device groups.
#[derive(Debug, Clone, Deserialize)]
pub struct ListDeviceGroupsQuery {
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: i64,

    /// Items per page (1-100)
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

/// Response for listing device's group memberships.
#[derive(Debug, Clone, Serialize)]
pub struct ListDeviceGroupsResponse {
    pub groups: Vec<DeviceGroupMembershipInfo>,
    pub pagination: Pagination,
}

/// List all groups a device belongs to.
///
/// GET /api/v1/devices/:device_id/groups
///
/// Requires JWT authentication.
/// - User must own the device
/// - Supports pagination via `page` and `per_page` query parameters
///
/// Story UGM-3.5: View Device's Group Memberships
pub async fn list_device_groups(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(device_id): Path<Uuid>,
    Query(query): Query<ListDeviceGroupsQuery>,
) -> Result<Json<ListDeviceGroupsResponse>, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());
    let membership_repo = DeviceGroupMembershipRepository::new(state.pool.clone());

    // Check the device exists and is owned by the user
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    if device.owner_user_id != Some(user_auth.user_id) {
        return Err(ApiError::Forbidden(
            "You can only view groups for devices you own".to_string(),
        ));
    }

    // Pagination
    let page = query.page.max(1);
    let per_page = query.per_page.clamp(1, 100);
    let offset = (page - 1) * per_page;

    // Get total count
    let total = membership_repo.count_device_groups(device_id).await?;

    // Get groups with pagination
    let group_infos = membership_repo
        .list_device_groups(device_id, user_auth.user_id, per_page, offset)
        .await?;

    let groups: Vec<DeviceGroupMembershipInfo> = group_infos
        .into_iter()
        .map(|g| DeviceGroupMembershipInfo {
            group_id: g.group_id,
            name: g.group_name,
            slug: g.group_slug,
            role: g.user_role.parse().unwrap_or(GroupRole::Member),
            added_at: g.added_at,
        })
        .collect();

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    info!(
        device_id = %device_id,
        user_id = %user_auth.user_id,
        group_count = groups.len(),
        page = page,
        total = total,
        "Listed device group memberships"
    );

    Ok(Json(ListDeviceGroupsResponse {
        groups,
        pagination: Pagination {
            page,
            per_page,
            total,
            total_pages,
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
