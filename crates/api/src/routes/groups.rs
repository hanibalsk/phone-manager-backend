//! Group management routes for creating and managing location sharing groups.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::group::{
    generate_slug, CreateGroupRequest, CreateGroupResponse, GroupDetail, GroupSummary,
    ListGroupsQuery, ListGroupsResponse, MembershipInfo, UpdateGroupRequest,
};
use persistence::repositories::GroupRepository;
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

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::group::GroupRole;

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
        assert!(GroupRole::Admin.can_manage_group());
        assert!(!GroupRole::Admin.can_delete_group());
        assert!(!GroupRole::Member.can_manage_group());
        assert!(!GroupRole::Viewer.can_manage_group());
    }
}
