//! User profile routes for viewing and updating user information.
//! Also includes device binding endpoints for linking/unlinking devices to users.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use persistence::repositories::DeviceRepository;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

/// User profile response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileResponse {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub email_verified: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Database row for user profile query.
#[derive(Debug, sqlx::FromRow)]
struct UserProfileRow {
    id: Uuid,
    email: String,
    display_name: String,
    avatar_url: Option<String>,
    email_verified: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Get current user profile.
///
/// GET /api/v1/users/me
///
/// Requires JWT authentication.
pub async fn get_current_user(
    State(state): State<AppState>,
    user_auth: UserAuth,
) -> Result<Json<ProfileResponse>, ApiError> {
    // Fetch user from database
    let user: Option<UserProfileRow> = sqlx::query_as(
        r#"
        SELECT id, email, COALESCE(display_name, '') as display_name, avatar_url, email_verified, created_at, updated_at
        FROM users
        WHERE id = $1 AND is_active = true
        "#,
    )
    .bind(user_auth.user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(ApiError::from)?;

    let user = user.ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(Json(ProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        email_verified: user.email_verified,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
    }))
}

/// Request body for updating user profile.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    /// User's display name (1-100 characters)
    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    pub display_name: Option<String>,

    /// User's avatar URL (optional, must be valid URL if provided)
    #[validate(url(message = "Invalid avatar URL format"))]
    pub avatar_url: Option<String>,
}

/// Update current user profile.
///
/// PUT /api/v1/users/me
///
/// Requires JWT authentication.
pub async fn update_current_user(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Check if user exists
    let user_exists: Option<(bool,)> =
        sqlx::query_as("SELECT is_active FROM users WHERE id = $1")
            .bind(user_auth.user_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(ApiError::from)?;

    match user_exists {
        Some((is_active,)) if !is_active => {
            return Err(ApiError::Forbidden("User account is disabled".to_string()))
        }
        None => return Err(ApiError::NotFound("User not found".to_string())),
        _ => {}
    }

    // Build dynamic update query based on provided fields
    let now = Utc::now();

    if request.display_name.is_none() && request.avatar_url.is_none() {
        // No fields to update, just return current profile
        return get_current_user(State(state), user_auth).await;
    }

    // Build update query
    let mut query_parts = vec!["updated_at = $1".to_string()];
    let mut param_idx = 2;

    if request.display_name.is_some() {
        query_parts.push(format!("display_name = ${}", param_idx));
        param_idx += 1;
    }

    if request.avatar_url.is_some() {
        query_parts.push(format!("avatar_url = ${}", param_idx));
        param_idx += 1;
    }

    let query = format!(
        "UPDATE users SET {} WHERE id = ${} RETURNING id, email, COALESCE(display_name, '') as display_name, avatar_url, email_verified, created_at, updated_at",
        query_parts.join(", "),
        param_idx
    );

    // Execute query with dynamic binding
    let user: UserProfileRow = match (&request.display_name, &request.avatar_url) {
        (Some(display_name), Some(avatar_url)) => {
            sqlx::query_as(&query)
                .bind(now)
                .bind(display_name)
                .bind(avatar_url)
                .bind(user_auth.user_id)
                .fetch_one(&state.pool)
                .await
                .map_err(ApiError::from)?
        }
        (Some(display_name), None) => {
            sqlx::query_as(&query)
                .bind(now)
                .bind(display_name)
                .bind(user_auth.user_id)
                .fetch_one(&state.pool)
                .await
                .map_err(ApiError::from)?
        }
        (None, Some(avatar_url)) => {
            sqlx::query_as(&query)
                .bind(now)
                .bind(avatar_url)
                .bind(user_auth.user_id)
                .fetch_one(&state.pool)
                .await
                .map_err(ApiError::from)?
        }
        (None, None) => {
            // Already handled above, but satisfy the match
            unreachable!()
        }
    };

    Ok(Json(ProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        email_verified: user.email_verified,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
    }))
}

// ============================================================================
// Device Binding Endpoints
// ============================================================================

/// Path parameters for device binding endpoints.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceBindingPath {
    pub user_id: Uuid,
    pub device_id: Uuid,
}

/// Request body for linking a device to a user.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LinkDeviceRequest {
    /// Override device display name (optional)
    #[validate(length(min = 1, max = 50, message = "Display name must be 1-50 characters"))]
    pub display_name: Option<String>,

    /// Set as primary device
    #[serde(default)]
    pub is_primary: bool,
}

/// Response for device binding operations.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedDeviceResponse {
    pub device: DeviceInfo,
    pub linked: bool,
}

/// Device information in responses.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub id: i64,
    pub device_uuid: String,
    pub display_name: String,
    pub owner_user_id: String,
    pub is_primary: bool,
    pub linked_at: String,
}

/// Link a device to the authenticated user.
///
/// POST /api/v1/users/:user_id/devices/:device_id/link
///
/// Requires JWT authentication. User can only link to themselves.
pub async fn link_device(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(path): Path<DeviceBindingPath>,
    Json(request): Json<LinkDeviceRequest>,
) -> Result<Json<LinkedDeviceResponse>, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Check that user is linking to themselves
    if path.user_id != user_auth.user_id {
        return Err(ApiError::Forbidden(
            "You can only link devices to your own account".to_string(),
        ));
    }

    let repo = DeviceRepository::new(state.pool.clone());

    // Check if device exists
    let existing_device = repo.find_by_device_id(path.device_id).await?;
    let device = existing_device
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Check if device is already linked to another user
    if let Some(owner_id) = device.owner_user_id {
        if owner_id != user_auth.user_id {
            return Err(ApiError::Conflict(
                "Device is already linked to another user".to_string(),
            ));
        }
    }

    // Link the device
    let updated_device = repo
        .link_device_to_user(
            path.device_id,
            user_auth.user_id,
            request.display_name.as_deref(),
            request.is_primary,
        )
        .await?;

    info!(
        device_id = %path.device_id,
        user_id = %user_auth.user_id,
        is_primary = request.is_primary,
        "Device linked to user"
    );

    Ok(Json(LinkedDeviceResponse {
        device: DeviceInfo {
            id: updated_device.id,
            device_uuid: updated_device.device_id.to_string(),
            display_name: updated_device.display_name,
            owner_user_id: user_auth.user_id.to_string(),
            is_primary: updated_device.is_primary,
            linked_at: updated_device
                .linked_at
                .map(|t| t.to_rfc3339())
                .unwrap_or_default(),
        },
        linked: true,
    }))
}

/// Path parameters for user devices list endpoint.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDevicesPath {
    pub user_id: Uuid,
}

/// Query parameters for listing user devices.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUserDevicesQuery {
    /// Include inactive devices in the list (default: false)
    #[serde(default)]
    pub include_inactive: bool,
}

/// Device information in user's device list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDeviceResponse {
    pub id: i64,
    pub device_uuid: String,
    pub display_name: String,
    pub platform: String,
    pub is_primary: bool,
    pub active: bool,
    pub linked_at: Option<String>,
    pub last_seen_at: Option<String>,
}

/// Response for listing user's devices.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUserDevicesResponse {
    pub devices: Vec<UserDeviceResponse>,
    pub count: usize,
}

/// List devices owned by the authenticated user.
///
/// GET /api/v1/users/:user_id/devices
///
/// Requires JWT authentication. User can only list their own devices.
pub async fn list_user_devices(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(path): Path<UserDevicesPath>,
    Query(query): Query<ListUserDevicesQuery>,
) -> Result<Json<ListUserDevicesResponse>, ApiError> {
    // Check that user is listing their own devices
    if path.user_id != user_auth.user_id {
        return Err(ApiError::Forbidden(
            "You can only list your own devices".to_string(),
        ));
    }

    let repo = DeviceRepository::new(state.pool.clone());

    // Fetch devices for user
    let devices = repo
        .find_devices_by_user(user_auth.user_id, query.include_inactive)
        .await?;

    // Transform to response DTOs
    let device_responses: Vec<UserDeviceResponse> = devices
        .into_iter()
        .map(|d| UserDeviceResponse {
            id: d.id,
            device_uuid: d.device_id.to_string(),
            display_name: d.display_name,
            platform: d.platform,
            is_primary: d.is_primary,
            active: d.active,
            linked_at: d.linked_at.map(|t| t.to_rfc3339()),
            last_seen_at: d.last_seen_at.map(|t| t.to_rfc3339()),
        })
        .collect();

    let count = device_responses.len();

    info!(
        user_id = %user_auth.user_id,
        device_count = count,
        include_inactive = query.include_inactive,
        "Listed user devices"
    );

    Ok(Json(ListUserDevicesResponse {
        devices: device_responses,
        count,
    }))
}

/// Response for unlink device operation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlinkDeviceResponse {
    pub device_uuid: String,
    pub unlinked: bool,
}

/// Unlink a device from the authenticated user.
///
/// DELETE /api/v1/users/:user_id/devices/:device_id/unlink
///
/// Requires JWT authentication. User can only unlink their own devices.
pub async fn unlink_device(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(path): Path<DeviceBindingPath>,
) -> Result<Json<UnlinkDeviceResponse>, ApiError> {
    // Check that user is unlinking from themselves
    if path.user_id != user_auth.user_id {
        return Err(ApiError::Forbidden(
            "You can only unlink devices from your own account".to_string(),
        ));
    }

    let repo = DeviceRepository::new(state.pool.clone());

    // Check if device exists and is owned by the user
    let existing_device = repo.find_by_device_id(path.device_id).await?;
    let device = existing_device
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Check if device is owned by the user
    match device.owner_user_id {
        Some(owner_id) if owner_id == user_auth.user_id => {
            // User owns this device, proceed with unlink
        }
        Some(_) => {
            return Err(ApiError::Forbidden(
                "Device is owned by another user".to_string(),
            ));
        }
        None => {
            return Err(ApiError::Forbidden(
                "Device is not linked to any user".to_string(),
            ));
        }
    }

    // Unlink the device
    repo.unlink_device(path.device_id).await?;

    info!(
        device_id = %path.device_id,
        user_id = %user_auth.user_id,
        "Device unlinked from user"
    );

    Ok(Json(UnlinkDeviceResponse {
        device_uuid: path.device_id.to_string(),
        unlinked: true,
    }))
}

/// Request body for transferring device ownership.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransferDeviceRequest {
    /// UUID of the user to transfer ownership to
    pub new_owner_id: Uuid,
}

/// Response for transfer device operation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferDeviceResponse {
    pub device: DeviceInfo,
    pub previous_owner_id: String,
    pub new_owner_id: String,
    pub transferred: bool,
}

/// Transfer device ownership to another user.
///
/// POST /api/v1/users/:user_id/devices/:device_id/transfer
///
/// Requires JWT authentication. Only the current device owner can transfer.
pub async fn transfer_device(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(path): Path<DeviceBindingPath>,
    Json(request): Json<TransferDeviceRequest>,
) -> Result<Json<TransferDeviceResponse>, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Check that the user is transferring from themselves
    if path.user_id != user_auth.user_id {
        return Err(ApiError::Forbidden(
            "You can only transfer devices from your own account".to_string(),
        ));
    }

    // Cannot transfer to yourself
    if request.new_owner_id == user_auth.user_id {
        return Err(ApiError::Validation(
            "Cannot transfer device to yourself".to_string(),
        ));
    }

    let repo = DeviceRepository::new(state.pool.clone());

    // Check if device exists and is owned by the user
    let existing_device = repo.find_by_device_id(path.device_id).await?;
    let device = existing_device
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Check if device is owned by the user
    match device.owner_user_id {
        Some(owner_id) if owner_id == user_auth.user_id => {
            // User owns this device, proceed with transfer
        }
        Some(_) => {
            return Err(ApiError::Forbidden(
                "Device is owned by another user".to_string(),
            ));
        }
        None => {
            return Err(ApiError::Forbidden(
                "Device is not linked to any user".to_string(),
            ));
        }
    }

    // Check if target user exists
    let target_user_exists: Option<(bool,)> =
        sqlx::query_as("SELECT is_active FROM users WHERE id = $1")
            .bind(request.new_owner_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(ApiError::from)?;

    match target_user_exists {
        Some((is_active,)) if !is_active => {
            return Err(ApiError::Validation(
                "Target user account is disabled".to_string(),
            ))
        }
        None => return Err(ApiError::NotFound("Target user not found".to_string())),
        _ => {}
    }

    // Transfer the device
    let updated_device = repo
        .transfer_device_ownership(path.device_id, request.new_owner_id)
        .await?;

    info!(
        device_id = %path.device_id,
        previous_owner = %user_auth.user_id,
        new_owner = %request.new_owner_id,
        "Device ownership transferred"
    );

    Ok(Json(TransferDeviceResponse {
        device: DeviceInfo {
            id: updated_device.id,
            device_uuid: updated_device.device_id.to_string(),
            display_name: updated_device.display_name,
            owner_user_id: request.new_owner_id.to_string(),
            is_primary: updated_device.is_primary,
            linked_at: updated_device
                .linked_at
                .map(|t| t.to_rfc3339())
                .unwrap_or_default(),
        },
        previous_owner_id: user_auth.user_id.to_string(),
        new_owner_id: request.new_owner_id.to_string(),
        transferred: true,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_profile_request_validation() {
        let request = UpdateProfileRequest {
            display_name: Some("Test User".to_string()),
            avatar_url: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_update_profile_request_display_name_too_long() {
        let request = UpdateProfileRequest {
            display_name: Some("A".repeat(101)),
            avatar_url: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_update_profile_request_display_name_empty() {
        let request = UpdateProfileRequest {
            display_name: Some("".to_string()),
            avatar_url: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_update_profile_request_valid_avatar_url() {
        let request = UpdateProfileRequest {
            display_name: None,
            avatar_url: Some("https://example.com/avatar.png".to_string()),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_update_profile_request_invalid_avatar_url() {
        let request = UpdateProfileRequest {
            display_name: None,
            avatar_url: Some("not-a-url".to_string()),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_update_profile_request_empty() {
        let request = UpdateProfileRequest {
            display_name: None,
            avatar_url: None,
        };

        // Empty request should be valid (no-op)
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_profile_response_serialization() {
        let response = ProfileResponse {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            email_verified: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("displayName"));
        assert!(json.contains("avatarUrl"));
        assert!(json.contains("emailVerified"));
        assert!(json.contains("createdAt"));
        assert!(json.contains("updatedAt"));
    }

    #[test]
    fn test_profile_response_serialization_no_avatar() {
        let response = ProfileResponse {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            avatar_url: None,
            email_verified: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"avatarUrl\":null"));
    }
}
