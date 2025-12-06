//! Device settings route handlers.
//!
//! Handles device configuration settings, locks, and sync operations.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use domain::models::setting::{
    BulkUpdateLocksRequest, BulkUpdateLocksResponse, GetSettingsResponse, ListLocksResponse,
    LockInfo, LockSettingRequest, LockSettingResponse, LockUpdateResult, LockerInfo,
    SettingCategory, SettingChange, SettingDataType, SettingDefinition, SettingValue,
    SkippedLockUpdate, SyncSettingsRequest, SyncSettingsResponse, UnlockSettingResponse,
    UpdateSettingRequest, UpdateSettingsRequest, UpdateSettingsResponse,
};
use domain::models::unlock_request::{
    CreateUnlockRequestRequest, CreateUnlockRequestResponse, DeviceInfo,
    ListUnlockRequestsQuery, ListUnlockRequestsResponse, Pagination, RespondToUnlockRequestRequest,
    RespondToUnlockRequestResponse, UnlockRequestItem, UnlockRequestStatus, UserInfo,
};
use domain::services::{
    NotificationType, SettingChangeAction, SettingChangeNotification, SettingsChangedPayload,
    UnlockRequestResponsePayload,
};
use persistence::entities::UnlockRequestStatusDb;
use persistence::repositories::{
    DeviceRepository, GroupRepository, OrgUserRepository, SettingRepository, UnlockRequestRepository, UserRepository,
};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

/// Query parameters for get settings endpoint.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GetSettingsQuery {
    /// Include setting definitions in response.
    #[serde(default)]
    pub include_definitions: bool,
}

/// Query parameters for update settings endpoints.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateSettingsQuery {
    /// Force update even if setting is locked (admin only).
    #[serde(default)]
    pub force: bool,
}

/// Get all settings for a device.
///
/// GET /api/v1/devices/:device_id/settings
///
/// Requires JWT authentication.
/// Device owner, group admin, or org admin can access.
pub async fn get_device_settings(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(device_id): Path<Uuid>,
    Query(query): Query<GetSettingsQuery>,
) -> Result<Json<GetSettingsResponse>, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());
    let setting_repo = SettingRepository::new(state.pool.clone());
    let group_repo = GroupRepository::new(state.pool.clone());
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Get the device
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Authorization check: must be owner, or admin of device's group, or org admin
    let is_authorized = check_settings_authorization(
        &device_repo,
        &group_repo,
        Some(&org_user_repo),
        &device,
        user_auth.user_id,
    )
    .await?;

    if !is_authorized {
        return Err(ApiError::Forbidden(
            "Not authorized to access this device's settings".to_string(),
        ));
    }

    // Get all setting definitions
    let definitions = setting_repo.get_all_definitions().await?;

    // Get device-specific settings
    let device_settings = setting_repo.get_device_settings(device_id).await?;

    // Build settings map, merging device values with defaults
    let mut settings: HashMap<String, SettingValue> = HashMap::new();

    // First, add all definitions with defaults
    for def in &definitions {
        settings.insert(
            def.key.clone(),
            SettingValue {
                value: def.default_value.clone(),
                is_locked: false,
                locked_by: None,
                locked_at: None,
                lock_reason: None,
                updated_at: def.created_at,
                updated_by: None,
                error: None,
            },
        );
    }

    // Override with device-specific values
    for ds in device_settings {
        settings.insert(
            ds.setting_key.clone(),
            SettingValue {
                value: ds.value,
                is_locked: ds.is_locked,
                locked_by: ds.locked_by,
                locked_at: ds.locked_at,
                lock_reason: ds.lock_reason,
                updated_at: ds.updated_at,
                updated_by: ds.updated_by,
                error: None,
            },
        );
    }

    // Optionally include definitions
    let definitions_response = if query.include_definitions {
        Some(
            definitions
                .into_iter()
                .map(|d| SettingDefinition {
                    key: d.key,
                    display_name: d.display_name,
                    description: d.description,
                    data_type: db_data_type_to_domain(d.data_type),
                    default_value: d.default_value,
                    is_lockable: d.is_lockable,
                    category: db_category_to_domain(d.category),
                    validation_rules: d.validation_rules,
                    sort_order: d.sort_order,
                })
                .collect(),
        )
    } else {
        None
    };

    info!(
        device_id = %device_id,
        user_id = %user_auth.user_id,
        setting_count = %settings.len(),
        "Retrieved device settings"
    );

    Ok(Json(GetSettingsResponse {
        device_id,
        settings,
        last_synced_at: Some(Utc::now()),
        definitions: definitions_response,
    }))
}

/// Update multiple device settings.
///
/// PUT /api/v1/devices/:device_id/settings
///
/// Requires JWT authentication.
/// Device owner, group admin, or org admin can update.
/// Locked settings are skipped unless force=true (admin only).
pub async fn update_device_settings(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(device_id): Path<Uuid>,
    Query(query): Query<UpdateSettingsQuery>,
    Json(request): Json<UpdateSettingsRequest>,
) -> Result<Json<UpdateSettingsResponse>, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());
    let setting_repo = SettingRepository::new(state.pool.clone());
    let group_repo = GroupRepository::new(state.pool.clone());
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Get the device
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Authorization check
    let is_authorized = check_settings_authorization(
        &device_repo,
        &group_repo,
        Some(&org_user_repo),
        &device,
        user_auth.user_id,
    )
    .await?;

    if !is_authorized {
        return Err(ApiError::Forbidden(
            "Not authorized to update this device's settings".to_string(),
        ));
    }

    // Check if user is an admin (can use force)
    let is_admin = check_is_admin(&group_repo, &device, user_auth.user_id).await?;
    let can_force = query.force && is_admin;

    // Get all definitions for validation
    let definitions = setting_repo.get_all_definitions().await?;
    let definitions_map: HashMap<String, _> = definitions
        .into_iter()
        .map(|d| (d.key.clone(), d))
        .collect();

    let mut updated: Vec<String> = Vec::new();
    let mut locked: Vec<String> = Vec::new();
    let mut invalid: Vec<String> = Vec::new();
    let mut settings: HashMap<String, SettingValue> = HashMap::new();

    for (key, value) in request.settings {
        // Check if setting definition exists
        let def = match definitions_map.get(&key) {
            Some(d) => d,
            None => {
                invalid.push(key.clone());
                settings.insert(
                    key.clone(),
                    SettingValue {
                        value,
                        is_locked: false,
                        locked_by: None,
                        locked_at: None,
                        lock_reason: None,
                        updated_at: Utc::now(),
                        updated_by: None,
                        error: Some("Unknown setting key".to_string()),
                    },
                );
                continue;
            }
        };

        // Validate value type
        if !validate_value_type(&value, &db_data_type_to_domain(def.data_type)) {
            invalid.push(key.clone());
            settings.insert(
                key.clone(),
                SettingValue {
                    value,
                    is_locked: false,
                    locked_by: None,
                    locked_at: None,
                    lock_reason: None,
                    updated_at: Utc::now(),
                    updated_by: None,
                    error: Some(format!(
                        "Invalid value type, expected {}",
                        db_data_type_to_domain(def.data_type)
                    )),
                },
            );
            continue;
        }

        // Check if setting is locked
        let is_locked = setting_repo.is_setting_locked(device_id, &key).await?;

        if is_locked && !can_force {
            locked.push(key.clone());
            // Get current setting value
            if let Some(current) = setting_repo.get_device_setting(device_id, &key).await? {
                settings.insert(
                    key.clone(),
                    SettingValue {
                        value: current.value,
                        is_locked: current.is_locked,
                        locked_by: current.locked_by,
                        locked_at: current.locked_at,
                        lock_reason: current.lock_reason,
                        updated_at: current.updated_at,
                        updated_by: current.updated_by,
                        error: Some("Setting is locked by admin".to_string()),
                    },
                );
            }
            continue;
        }

        // Update the setting
        let result = if can_force {
            setting_repo
                .upsert_setting_force(device_id, &key, value.clone(), user_auth.user_id)
                .await?
        } else {
            setting_repo
                .upsert_setting(device_id, &key, value.clone(), Some(user_auth.user_id))
                .await?
        };

        updated.push(key.clone());
        settings.insert(
            key.clone(),
            SettingValue {
                value: result.value,
                is_locked: result.is_locked,
                locked_by: result.locked_by,
                locked_at: result.locked_at,
                lock_reason: result.lock_reason,
                updated_at: result.updated_at,
                updated_by: result.updated_by,
                error: None,
            },
        );
    }

    info!(
        device_id = %device_id,
        user_id = %user_auth.user_id,
        updated_count = updated.len(),
        locked_count = locked.len(),
        invalid_count = invalid.len(),
        force = query.force,
        "Updated device settings"
    );

    Ok(Json(UpdateSettingsResponse {
        updated,
        locked,
        invalid,
        settings,
    }))
}

/// Update a single device setting.
///
/// PUT /api/v1/devices/:device_id/settings/:key
///
/// Requires JWT authentication.
/// Device owner, group admin, or org admin can update.
pub async fn update_device_setting(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path((device_id, key)): Path<(Uuid, String)>,
    Query(query): Query<UpdateSettingsQuery>,
    Json(request): Json<UpdateSettingRequest>,
) -> Result<Json<SettingValue>, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());
    let setting_repo = SettingRepository::new(state.pool.clone());
    let group_repo = GroupRepository::new(state.pool.clone());
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Get the device
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Authorization check
    let is_authorized = check_settings_authorization(
        &device_repo,
        &group_repo,
        Some(&org_user_repo),
        &device,
        user_auth.user_id,
    )
    .await?;

    if !is_authorized {
        return Err(ApiError::Forbidden(
            "Not authorized to update this device's settings".to_string(),
        ));
    }

    // Check if user is an admin (can use force)
    let is_admin = check_is_admin(&group_repo, &device, user_auth.user_id).await?;
    let can_force = query.force && is_admin;

    // Get setting definition
    let def = setting_repo
        .get_definition(&key)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Setting '{}' not found", key)))?;

    // Validate value type
    if !validate_value_type(&request.value, &db_data_type_to_domain(def.data_type)) {
        return Err(ApiError::Validation(format!(
            "Invalid value type, expected {}",
            db_data_type_to_domain(def.data_type)
        )));
    }

    // Check if setting is locked
    let is_locked = setting_repo.is_setting_locked(device_id, &key).await?;

    if is_locked && !can_force {
        // Get current setting value
        if let Some(current) = setting_repo.get_device_setting(device_id, &key).await? {
            return Ok(Json(SettingValue {
                value: current.value,
                is_locked: current.is_locked,
                locked_by: current.locked_by,
                locked_at: current.locked_at,
                lock_reason: current.lock_reason,
                updated_at: current.updated_at,
                updated_by: current.updated_by,
                error: Some("Setting is locked by admin".to_string()),
            }));
        }
        return Err(ApiError::Forbidden("Setting is locked".to_string()));
    }

    // Update the setting
    let result = if can_force {
        setting_repo
            .upsert_setting_force(device_id, &key, request.value.clone(), user_auth.user_id)
            .await?
    } else {
        setting_repo
            .upsert_setting(device_id, &key, request.value.clone(), Some(user_auth.user_id))
            .await?
    };

    info!(
        device_id = %device_id,
        user_id = %user_auth.user_id,
        key = %key,
        force = query.force,
        "Updated device setting"
    );

    Ok(Json(SettingValue {
        value: result.value,
        is_locked: result.is_locked,
        locked_by: result.locked_by,
        locked_at: result.locked_at,
        lock_reason: result.lock_reason,
        updated_at: result.updated_at,
        updated_by: result.updated_by,
        error: None,
    }))
}

/// Get all locks for a device's settings.
///
/// GET /api/v1/devices/:device_id/settings/locks
///
/// Requires JWT authentication.
/// Device owner, group admin, or org admin can access.
pub async fn get_setting_locks(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(device_id): Path<Uuid>,
) -> Result<Json<ListLocksResponse>, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());
    let setting_repo = SettingRepository::new(state.pool.clone());
    let group_repo = GroupRepository::new(state.pool.clone());
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Get the device
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Authorization check
    let is_authorized = check_settings_authorization(
        &device_repo,
        &group_repo,
        Some(&org_user_repo),
        &device,
        user_auth.user_id,
    )
    .await?;

    if !is_authorized {
        return Err(ApiError::Forbidden(
            "Not authorized to access this device's settings".to_string(),
        ));
    }

    // Get all locks for the device
    let locks = setting_repo.get_device_locks(device_id).await?;

    // Get total lockable settings count
    let total_lockable = setting_repo.count_lockable_settings().await?;

    // Build response
    let lock_infos: Vec<LockInfo> = locks
        .into_iter()
        .map(|l| LockInfo {
            key: l.setting_key,
            is_locked: l.is_locked,
            locked_by: l.locked_by.map(|id| LockerInfo {
                id,
                display_name: l.locker_display_name,
            }),
            locked_at: l.locked_at,
            reason: l.lock_reason,
        })
        .collect();

    let locked_count = lock_infos.len() as i64;

    info!(
        device_id = %device_id,
        user_id = %user_auth.user_id,
        locked_count = locked_count,
        "Retrieved device setting locks"
    );

    Ok(Json(ListLocksResponse {
        device_id,
        locks: lock_infos,
        locked_count,
        total_lockable,
    }))
}

/// Lock a device setting.
///
/// POST /api/v1/devices/:device_id/settings/:key/lock
///
/// Requires JWT authentication.
/// Only group admin or owner can lock settings.
pub async fn lock_setting(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path((device_id, key)): Path<(Uuid, String)>,
    Json(request): Json<LockSettingRequest>,
) -> Result<Json<LockSettingResponse>, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());
    let setting_repo = SettingRepository::new(state.pool.clone());
    let group_repo = GroupRepository::new(state.pool.clone());

    // Get the device
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Authorization check - only admin can lock
    let is_admin = check_is_admin(&group_repo, &device, user_auth.user_id).await?;

    if !is_admin {
        return Err(ApiError::Forbidden(
            "Only admins can lock settings".to_string(),
        ));
    }

    // Get setting definition
    let def = setting_repo
        .get_definition(&key)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Setting '{}' not found", key)))?;

    // Check if setting is lockable
    if !def.is_lockable {
        return Err(ApiError::Validation(format!(
            "Setting '{}' is not lockable",
            key
        )));
    }

    // Validate value type if provided
    if let Some(ref value) = request.value {
        if !validate_value_type(value, &db_data_type_to_domain(def.data_type)) {
            return Err(ApiError::Validation(format!(
                "Invalid value type, expected {}",
                db_data_type_to_domain(def.data_type)
            )));
        }
    }

    // Lock the setting
    let result = setting_repo
        .lock_setting(
            device_id,
            &key,
            user_auth.user_id,
            request.reason.as_deref(),
            request.value.clone(),
        )
        .await?;

    info!(
        device_id = %device_id,
        user_id = %user_auth.user_id,
        key = %key,
        reason = ?request.reason,
        "Locked device setting"
    );

    // Send notification if requested
    if request.notify_user {
        let user_repo = UserRepository::new(state.pool.clone());
        let user_name = user_repo
            .find_by_id(user_auth.user_id)
            .await?
            .map(|u| u.display_name.unwrap_or_else(|| "Admin".to_string()))
            .unwrap_or_else(|| "Admin".to_string());

        let changes = vec![SettingChangeNotification {
            key: key.clone(),
            action: SettingChangeAction::Locked,
            new_value: request.value,
        }];

        send_settings_changed_notification(
            &state,
            device_id,
            device.fcm_token.as_deref(),
            changes,
            user_name,
        )
        .await;
    }

    Ok(Json(LockSettingResponse {
        key,
        is_locked: result.is_locked,
        value: result.value,
        locked_by: result.locked_by.unwrap_or(user_auth.user_id),
        locked_at: result.locked_at.unwrap_or_else(Utc::now),
        reason: result.lock_reason,
    }))
}

/// Unlock a device setting.
///
/// DELETE /api/v1/devices/:device_id/settings/:key/lock
///
/// Requires JWT authentication.
/// Only group admin or owner can unlock settings.
pub async fn unlock_setting(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path((device_id, key)): Path<(Uuid, String)>,
) -> Result<Json<UnlockSettingResponse>, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());
    let setting_repo = SettingRepository::new(state.pool.clone());
    let group_repo = GroupRepository::new(state.pool.clone());

    // Get the device
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Authorization check - only admin can unlock
    let is_admin = check_is_admin(&group_repo, &device, user_auth.user_id).await?;

    if !is_admin {
        return Err(ApiError::Forbidden(
            "Only admins can unlock settings".to_string(),
        ));
    }

    // Check if setting definition exists
    let _def = setting_repo
        .get_definition(&key)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Setting '{}' not found", key)))?;

    // Unlock the setting
    let result = setting_repo
        .unlock_setting(device_id, &key, user_auth.user_id)
        .await?;

    if result.is_none() {
        return Err(ApiError::NotFound(format!(
            "Setting '{}' is not locked or doesn't exist for this device",
            key
        )));
    }

    info!(
        device_id = %device_id,
        user_id = %user_auth.user_id,
        key = %key,
        "Unlocked device setting"
    );

    Ok(Json(UnlockSettingResponse {
        key,
        is_locked: false,
        unlocked_by: user_auth.user_id,
        unlocked_at: Utc::now(),
    }))
}

/// Bulk update locks for multiple settings.
///
/// PUT /api/v1/devices/:device_id/settings/locks
///
/// Requires JWT authentication.
/// Only group admin or owner can bulk update locks.
pub async fn bulk_update_locks(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(device_id): Path<Uuid>,
    Json(request): Json<BulkUpdateLocksRequest>,
) -> Result<Json<BulkUpdateLocksResponse>, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());
    let setting_repo = SettingRepository::new(state.pool.clone());
    let group_repo = GroupRepository::new(state.pool.clone());

    // Get the device
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Authorization check - only admin can update locks
    let is_admin = check_is_admin(&group_repo, &device, user_auth.user_id).await?;

    if !is_admin {
        return Err(ApiError::Forbidden(
            "Only admins can update setting locks".to_string(),
        ));
    }

    // Get all setting definitions to check lockability
    let definitions = setting_repo.get_all_definitions().await?;
    let definitions_map: HashMap<String, _> = definitions
        .into_iter()
        .map(|d| (d.key.clone(), d))
        .collect();

    let mut updated: Vec<LockUpdateResult> = Vec::new();
    let mut skipped: Vec<SkippedLockUpdate> = Vec::new();

    for (key, should_lock) in request.locks {
        // Check if setting exists
        let def = match definitions_map.get(&key) {
            Some(d) => d,
            None => {
                skipped.push(SkippedLockUpdate {
                    key: key.clone(),
                    reason: "Setting not found".to_string(),
                });
                continue;
            }
        };

        // Check if setting is lockable
        if !def.is_lockable {
            skipped.push(SkippedLockUpdate {
                key: key.clone(),
                reason: "Setting is not lockable".to_string(),
            });
            continue;
        }

        if should_lock {
            // Lock the setting
            let result = setting_repo
                .lock_setting(
                    device_id,
                    &key,
                    user_auth.user_id,
                    request.reason.as_deref(),
                    None, // No value override for bulk operations
                )
                .await?;

            updated.push(LockUpdateResult {
                key: key.clone(),
                is_locked: result.is_locked,
                locked_at: result.locked_at,
                unlocked_at: None,
            });
        } else {
            // Unlock the setting
            let result = setting_repo
                .unlock_setting(device_id, &key, user_auth.user_id)
                .await?;

            if result.is_some() {
                updated.push(LockUpdateResult {
                    key: key.clone(),
                    is_locked: false,
                    locked_at: None,
                    unlocked_at: Some(Utc::now()),
                });
            } else {
                // Setting wasn't locked, still count as success (idempotent)
                updated.push(LockUpdateResult {
                    key: key.clone(),
                    is_locked: false,
                    locked_at: None,
                    unlocked_at: Some(Utc::now()),
                });
            }
        }
    }

    // Send notification if requested and there were changes
    let notification_sent = if request.notify_user && !updated.is_empty() {
        let user_repo = UserRepository::new(state.pool.clone());
        let user_name = user_repo
            .find_by_id(user_auth.user_id)
            .await?
            .map(|u| u.display_name.unwrap_or_else(|| "Admin".to_string()))
            .unwrap_or_else(|| "Admin".to_string());

        let changes: Vec<SettingChangeNotification> = updated
            .iter()
            .map(|u| SettingChangeNotification {
                key: u.key.clone(),
                action: if u.is_locked {
                    SettingChangeAction::Locked
                } else {
                    SettingChangeAction::Unlocked
                },
                new_value: None,
            })
            .collect();

        send_settings_changed_notification(
            &state,
            device_id,
            device.fcm_token.as_deref(),
            changes,
            user_name,
        )
        .await;
        true
    } else {
        false
    };

    info!(
        device_id = %device_id,
        user_id = %user_auth.user_id,
        updated_count = updated.len(),
        skipped_count = skipped.len(),
        notify_requested = request.notify_user,
        notification_sent = notification_sent,
        "Bulk updated device setting locks"
    );

    Ok(Json(BulkUpdateLocksResponse {
        updated,
        skipped,
        notification_sent,
    }))
}

/// Check if user is an admin of the device's group.
async fn check_is_admin(
    group_repo: &GroupRepository,
    device: &persistence::entities::DeviceEntity,
    user_id: Uuid,
) -> Result<bool, ApiError> {
    // Check if user is the device owner (owner is admin)
    if device.owner_user_id == Some(user_id) {
        return Ok(true);
    }

    // Check if user is an admin/owner of any group
    if !device.group_id.is_empty() {
        let user_groups = group_repo.find_user_groups(user_id, None).await?;
        for group in user_groups {
            let role: domain::models::GroupRole = group.role.into();
            if role.can_manage_members() {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Validate that a value matches the expected data type.
fn validate_value_type(value: &serde_json::Value, expected: &SettingDataType) -> bool {
    match expected {
        SettingDataType::Boolean => value.is_boolean(),
        SettingDataType::Integer => value.is_i64() || value.is_u64(),
        SettingDataType::String => value.is_string(),
        SettingDataType::Float => value.is_f64() || value.is_i64() || value.is_u64(),
        SettingDataType::Json => true, // Any JSON value is valid
    }
}

/// Check if user is authorized to access device settings.
async fn check_settings_authorization(
    _device_repo: &DeviceRepository,
    group_repo: &GroupRepository,
    org_user_repo: Option<&OrgUserRepository>,
    device: &persistence::entities::DeviceEntity,
    user_id: Uuid,
) -> Result<bool, ApiError> {
    // Check if user is the device owner
    if device.owner_user_id == Some(user_id) {
        return Ok(true);
    }

    // Check if user is an admin/owner of the device's group
    // The device.group_id stores the group slug (legacy string field)
    if !device.group_id.is_empty() {
        // Get all groups where user is admin or owner
        let user_groups = group_repo.find_user_groups(user_id, None).await?;
        for group in user_groups {
            let role: domain::models::GroupRole = group.role.into();
            // User must be admin/owner of THIS specific group (matching by slug)
            if role.can_manage_members() && group.slug == device.group_id {
                return Ok(true);
            }
        }
    }

    // Check organization admin for B2B devices
    if let (Some(org_id), Some(repo)) = (device.organization_id, org_user_repo) {
        if let Ok(Some(org_user)) = repo.find_by_org_and_user(org_id, user_id).await {
            // Organization owners and admins can manage device settings
            if org_user.role.has_at_least(domain::models::OrgUserRole::Admin) {
                return Ok(true);
            }
        }
    }

    // Default: only owner can access
    Ok(false)
}

/// Convert DB data type enum to domain enum.
fn db_data_type_to_domain(db: persistence::entities::SettingDataTypeDb) -> SettingDataType {
    match db {
        persistence::entities::SettingDataTypeDb::Boolean => SettingDataType::Boolean,
        persistence::entities::SettingDataTypeDb::Integer => SettingDataType::Integer,
        persistence::entities::SettingDataTypeDb::String => SettingDataType::String,
        persistence::entities::SettingDataTypeDb::Float => SettingDataType::Float,
        persistence::entities::SettingDataTypeDb::Json => SettingDataType::Json,
    }
}

/// Convert DB category enum to domain enum.
fn db_category_to_domain(db: persistence::entities::SettingCategoryDb) -> SettingCategory {
    match db {
        persistence::entities::SettingCategoryDb::Tracking => SettingCategory::Tracking,
        persistence::entities::SettingCategoryDb::Privacy => SettingCategory::Privacy,
        persistence::entities::SettingCategoryDb::Notifications => SettingCategory::Notifications,
        persistence::entities::SettingCategoryDb::Battery => SettingCategory::Battery,
        persistence::entities::SettingCategoryDb::General => SettingCategory::General,
    }
}

/// Convert domain unlock request status to database enum.
fn domain_status_to_db(status: UnlockRequestStatus) -> UnlockRequestStatusDb {
    match status {
        UnlockRequestStatus::Pending => UnlockRequestStatusDb::Pending,
        UnlockRequestStatus::Approved => UnlockRequestStatusDb::Approved,
        UnlockRequestStatus::Denied => UnlockRequestStatusDb::Denied,
        UnlockRequestStatus::Expired => UnlockRequestStatusDb::Expired,
    }
}

/// Convert database unlock request status to domain enum.
fn db_status_to_domain(status: UnlockRequestStatusDb) -> UnlockRequestStatus {
    match status {
        UnlockRequestStatusDb::Pending => UnlockRequestStatus::Pending,
        UnlockRequestStatusDb::Approved => UnlockRequestStatus::Approved,
        UnlockRequestStatusDb::Denied => UnlockRequestStatus::Denied,
        UnlockRequestStatusDb::Expired => UnlockRequestStatus::Expired,
    }
}

/// Helper to send settings changed notification (fire-and-forget).
async fn send_settings_changed_notification(
    state: &AppState,
    device_id: Uuid,
    fcm_token: Option<&str>,
    changes: Vec<SettingChangeNotification>,
    changed_by: String,
) {
    let Some(token) = fcm_token else {
        info!(
            device_id = %device_id,
            "Skipping notification - device has no FCM token"
        );
        return;
    };

    let payload = SettingsChangedPayload {
        notification_type: NotificationType::SettingsChanged,
        device_id,
        changes,
        changed_by,
        timestamp: Utc::now(),
    };

    // Fire and forget - don't await or block on notification result
    let result = state
        .notification_service
        .send_settings_changed(token, payload)
        .await;

    match result {
        domain::services::NotificationResult::Sent => {
            info!(device_id = %device_id, "Settings changed notification sent");
        }
        domain::services::NotificationResult::Failed(err) => {
            warn!(device_id = %device_id, error = %err, "Failed to send notification");
        }
        _ => {}
    }
}

/// Helper to send unlock request response notification (fire-and-forget).
async fn send_unlock_request_response_notification(
    state: &AppState,
    fcm_token: Option<&str>,
    request_id: Uuid,
    setting_key: String,
    status: String,
    note: Option<String>,
    decided_by: String,
) {
    let Some(token) = fcm_token else {
        info!(
            request_id = %request_id,
            "Skipping notification - device has no FCM token"
        );
        return;
    };

    let payload = UnlockRequestResponsePayload {
        notification_type: NotificationType::UnlockRequestResponse,
        request_id,
        setting_key,
        status,
        note,
        decided_by,
        timestamp: Utc::now(),
    };

    let result = state
        .notification_service
        .send_unlock_request_response(token, payload)
        .await;

    match result {
        domain::services::NotificationResult::Sent => {
            info!(request_id = %request_id, "Unlock request response notification sent");
        }
        domain::services::NotificationResult::Failed(err) => {
            warn!(request_id = %request_id, error = %err, "Failed to send notification");
        }
        _ => {}
    }
}

/// Create an unlock request for a locked setting.
///
/// POST /api/v1/devices/:device_id/settings/:key/unlock-request
///
/// Requires JWT authentication.
/// Device owner can request to unlock a locked setting.
pub async fn create_unlock_request(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path((device_id, key)): Path<(Uuid, String)>,
    Json(request): Json<CreateUnlockRequestRequest>,
) -> Result<Json<CreateUnlockRequestResponse>, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());
    let setting_repo = SettingRepository::new(state.pool.clone());
    let unlock_repo = UnlockRequestRepository::new(state.pool.clone());
    let group_repo = GroupRepository::new(state.pool.clone());
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Get the device
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Authorization check: must be device owner or authorized group member
    let is_authorized = check_settings_authorization(
        &device_repo,
        &group_repo,
        Some(&org_user_repo),
        &device,
        user_auth.user_id,
    )
    .await?;

    if !is_authorized {
        return Err(ApiError::Forbidden(
            "Not authorized to access this device's settings".to_string(),
        ));
    }

    // Check if setting definition exists
    let _def = setting_repo
        .get_definition(&key)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Setting '{}' not found", key)))?;

    // Check if setting is actually locked
    let is_locked = setting_repo.is_setting_locked(device_id, &key).await?;
    if !is_locked {
        return Err(ApiError::Validation(format!(
            "Setting '{}' is not locked",
            key
        )));
    }

    // Check for existing pending request
    let existing = unlock_repo
        .find_pending_for_device_setting(device_id, &key)
        .await?;
    if existing.is_some() {
        return Err(ApiError::Conflict(
            "A pending unlock request already exists for this setting".to_string(),
        ));
    }

    // Create the unlock request
    let entity = unlock_repo
        .create(device_id, &key, user_auth.user_id, request.reason.as_deref())
        .await?;

    info!(
        device_id = %device_id,
        user_id = %user_auth.user_id,
        setting_key = %key,
        request_id = %entity.id,
        "Created unlock request"
    );

    Ok(Json(CreateUnlockRequestResponse {
        id: entity.id,
        device_id: entity.device_id,
        setting_key: entity.setting_key,
        status: db_status_to_domain(entity.status),
        reason: entity.reason,
        created_at: entity.created_at,
        expires_at: entity.expires_at,
    }))
}

/// List unlock requests for a group.
///
/// GET /api/v1/groups/:group_id/unlock-requests
///
/// Requires JWT authentication.
/// Only group admins/owners can list unlock requests.
pub async fn list_unlock_requests(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(group_id): Path<Uuid>,
    Query(query): Query<ListUnlockRequestsQuery>,
) -> Result<Json<ListUnlockRequestsResponse>, ApiError> {
    let group_repo = GroupRepository::new(state.pool.clone());
    let unlock_repo = UnlockRequestRepository::new(state.pool.clone());

    // Check if user is a member of the group
    let membership = group_repo
        .get_membership(group_id, user_auth.user_id)
        .await?
        .ok_or_else(|| {
            ApiError::Forbidden("Not a member of this group".to_string())
        })?;

    // Check if user has admin/owner role
    let role: domain::models::GroupRole = membership.role.into();
    if !role.can_manage_members() {
        return Err(ApiError::Forbidden(
            "Only admins can view unlock requests".to_string(),
        ));
    }

    // Parse status filter
    let status_filter = query.status.as_ref().and_then(|s| match s.as_str() {
        "pending" => Some(UnlockRequestStatusDb::Pending),
        "approved" => Some(UnlockRequestStatusDb::Approved),
        "denied" => Some(UnlockRequestStatusDb::Denied),
        "expired" => Some(UnlockRequestStatusDb::Expired),
        _ => None,
    });

    // Calculate pagination
    let limit = query.per_page.clamp(1, 100);
    let offset = (query.page.max(1) - 1) * limit;

    // Get unlock requests for the group
    let requests = unlock_repo
        .list_for_group(group_id, status_filter, limit, offset)
        .await?;

    let total = unlock_repo.count_for_group(group_id, status_filter).await?;

    // Convert to response format
    let items: Vec<UnlockRequestItem> = requests
        .into_iter()
        .map(|r| UnlockRequestItem {
            id: r.id,
            device: DeviceInfo {
                id: r.device_id,
                display_name: r.device_display_name,
            },
            setting_key: r.setting_key,
            setting_display_name: r.setting_display_name,
            status: db_status_to_domain(r.status),
            requested_by: UserInfo {
                id: r.requested_by,
                display_name: r.requester_display_name,
            },
            reason: r.reason,
            responded_by: r.responded_by.map(|id| UserInfo {
                id,
                display_name: r.responder_display_name,
            }),
            response_note: r.response_note,
            created_at: r.created_at,
            expires_at: r.expires_at,
            responded_at: r.responded_at,
        })
        .collect();

    info!(
        group_id = %group_id,
        user_id = %user_auth.user_id,
        count = items.len(),
        total = total,
        "Listed unlock requests"
    );

    Ok(Json(ListUnlockRequestsResponse {
        data: items,
        pagination: Pagination {
            page: query.page,
            per_page: limit,
            total,
        },
    }))
}

/// Respond to an unlock request (approve or deny).
///
/// PUT /api/v1/unlock-requests/:request_id
///
/// Requires JWT authentication.
/// Only group admins/owners can respond to unlock requests.
pub async fn respond_to_unlock_request(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(request_id): Path<Uuid>,
    Json(request): Json<RespondToUnlockRequestRequest>,
) -> Result<Json<RespondToUnlockRequestResponse>, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());
    let group_repo = GroupRepository::new(state.pool.clone());
    let setting_repo = SettingRepository::new(state.pool.clone());
    let unlock_repo = UnlockRequestRepository::new(state.pool.clone());

    // Get the unlock request
    let unlock_request = unlock_repo
        .find_by_id(request_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Unlock request not found".to_string()))?;

    // Check if request is still pending
    if unlock_request.status != UnlockRequestStatusDb::Pending {
        return Err(ApiError::Conflict(format!(
            "Unlock request has already been {}",
            String::from(unlock_request.status)
        )));
    }

    // Check if request has expired
    if unlock_request.expires_at < Utc::now() {
        return Err(ApiError::Conflict(
            "Unlock request has expired".to_string(),
        ));
    }

    // Get the device to check authorization
    let device = device_repo
        .find_by_device_id(unlock_request.device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Check if user is admin of the device's group
    let is_admin = check_is_admin(&group_repo, &device, user_auth.user_id).await?;
    if !is_admin {
        return Err(ApiError::Forbidden(
            "Only admins can respond to unlock requests".to_string(),
        ));
    }

    // Validate the response status
    if request.status != UnlockRequestStatus::Approved && request.status != UnlockRequestStatus::Denied {
        return Err(ApiError::Validation(
            "Status must be 'approved' or 'denied'".to_string(),
        ));
    }

    // Update the unlock request
    let db_status = domain_status_to_db(request.status);
    let updated = unlock_repo
        .respond(request_id, db_status, user_auth.user_id, request.note.as_deref())
        .await?
        .ok_or_else(|| {
            ApiError::Conflict("Failed to update unlock request - it may have been modified".to_string())
        })?;

    // If approved, unlock the setting
    let setting_unlocked = if request.status == UnlockRequestStatus::Approved {
        setting_repo
            .unlock_setting(unlock_request.device_id, &unlock_request.setting_key, user_auth.user_id)
            .await?
            .is_some()
    } else {
        false
    };

    info!(
        request_id = %request_id,
        device_id = %unlock_request.device_id,
        setting_key = %unlock_request.setting_key,
        user_id = %user_auth.user_id,
        status = ?request.status,
        setting_unlocked = setting_unlocked,
        "Responded to unlock request"
    );

    // Send notification to requester's device
    let user_repo = UserRepository::new(state.pool.clone());
    let decided_by = user_repo
        .find_by_id(user_auth.user_id)
        .await?
        .map(|u| u.display_name.unwrap_or_else(|| "Admin".to_string()))
        .unwrap_or_else(|| "Admin".to_string());

    send_unlock_request_response_notification(
        &state,
        device.fcm_token.as_deref(),
        request_id,
        unlock_request.setting_key.clone(),
        request.status.to_string(),
        request.note.clone(),
        decided_by,
    )
    .await;

    Ok(Json(RespondToUnlockRequestResponse {
        id: updated.id,
        status: db_status_to_domain(updated.status),
        responded_by: user_auth.user_id,
        responded_at: updated.responded_at.unwrap_or_else(Utc::now),
        note: updated.response_note,
        setting_unlocked,
    }))
}

/// Sync device settings.
///
/// POST /api/v1/devices/:device_id/settings/sync
///
/// Returns all settings and highlights changes since last sync.
/// Requires JWT authentication.
/// Device owner or group admin can trigger sync.
pub async fn sync_settings(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(device_id): Path<Uuid>,
    Json(request): Json<SyncSettingsRequest>,
) -> Result<Json<SyncSettingsResponse>, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());
    let setting_repo = SettingRepository::new(state.pool.clone());
    let group_repo = GroupRepository::new(state.pool.clone());
    let org_user_repo = OrgUserRepository::new(state.pool.clone());

    // Get the device
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Authorization check: must be owner or admin of device's group, or org admin
    let is_authorized = check_settings_authorization(
        &device_repo,
        &group_repo,
        Some(&org_user_repo),
        &device,
        user_auth.user_id,
    )
    .await?;

    if !is_authorized {
        return Err(ApiError::Forbidden(
            "Not authorized to sync this device's settings".to_string(),
        ));
    }

    let synced_at = Utc::now();

    // Get all setting definitions for defaults
    let definitions = setting_repo.get_all_definitions().await?;

    // Get all device settings
    let device_settings = setting_repo.get_device_settings(device_id).await?;

    // Build settings map with defaults first, then override with device values
    let mut settings: HashMap<String, SettingValue> = HashMap::new();

    // Add all definitions with defaults
    for def in &definitions {
        settings.insert(
            def.key.clone(),
            SettingValue {
                value: def.default_value.clone(),
                is_locked: false,
                locked_by: None,
                locked_at: None,
                lock_reason: None,
                updated_at: def.created_at,
                updated_by: None,
                error: None,
            },
        );
    }

    // Override with device-specific values
    for ds in &device_settings {
        settings.insert(
            ds.setting_key.clone(),
            SettingValue {
                value: ds.value.clone(),
                is_locked: ds.is_locked,
                locked_by: ds.locked_by,
                locked_at: ds.locked_at,
                lock_reason: ds.lock_reason.clone(),
                updated_at: ds.updated_at,
                updated_by: ds.updated_by,
                error: None,
            },
        );
    }

    // Determine changes since last sync
    let changes_applied: Vec<SettingChange> = if let Some(last_sync) = request.last_synced_at {
        // Get settings modified since last sync
        let modified = setting_repo
            .get_settings_modified_since(device_id, last_sync)
            .await?;

        modified
            .into_iter()
            .map(|s| SettingChange {
                key: s.setting_key.clone(),
                old_value: None, // We don't track previous values currently
                new_value: s.value,
                reason: s.lock_reason, // Use lock_reason as change reason if available
            })
            .collect()
    } else {
        // First sync - no changes to report, device should apply all settings
        Vec::new()
    };

    info!(
        device_id = %device_id,
        user_id = %user_auth.user_id,
        settings_count = settings.len(),
        changes_count = changes_applied.len(),
        last_synced_at = ?request.last_synced_at,
        "Settings synced"
    );

    Ok(Json(SyncSettingsResponse {
        synced_at,
        settings,
        changes_applied,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_data_type_to_domain() {
        assert_eq!(
            db_data_type_to_domain(persistence::entities::SettingDataTypeDb::Boolean),
            SettingDataType::Boolean
        );
        assert_eq!(
            db_data_type_to_domain(persistence::entities::SettingDataTypeDb::Integer),
            SettingDataType::Integer
        );
    }

    #[test]
    fn test_db_category_to_domain() {
        assert_eq!(
            db_category_to_domain(persistence::entities::SettingCategoryDb::Tracking),
            SettingCategory::Tracking
        );
        assert_eq!(
            db_category_to_domain(persistence::entities::SettingCategoryDb::Privacy),
            SettingCategory::Privacy
        );
    }

    #[test]
    fn test_get_settings_query_default() {
        let query: GetSettingsQuery = serde_json::from_str("{}").unwrap();
        assert!(!query.include_definitions);
    }

    #[test]
    fn test_get_settings_query_with_definitions() {
        let query: GetSettingsQuery =
            serde_json::from_str(r#"{"include_definitions": true}"#).unwrap();
        assert!(query.include_definitions);
    }

    #[test]
    fn test_update_settings_query_default() {
        let query: UpdateSettingsQuery = serde_json::from_str("{}").unwrap();
        assert!(!query.force);
    }

    #[test]
    fn test_update_settings_query_with_force() {
        let query: UpdateSettingsQuery = serde_json::from_str(r#"{"force": true}"#).unwrap();
        assert!(query.force);
    }

    #[test]
    fn test_validate_value_type_boolean() {
        assert!(validate_value_type(
            &serde_json::json!(true),
            &SettingDataType::Boolean
        ));
        assert!(validate_value_type(
            &serde_json::json!(false),
            &SettingDataType::Boolean
        ));
        assert!(!validate_value_type(
            &serde_json::json!("true"),
            &SettingDataType::Boolean
        ));
        assert!(!validate_value_type(
            &serde_json::json!(1),
            &SettingDataType::Boolean
        ));
    }

    #[test]
    fn test_validate_value_type_integer() {
        assert!(validate_value_type(
            &serde_json::json!(42),
            &SettingDataType::Integer
        ));
        assert!(validate_value_type(
            &serde_json::json!(-10),
            &SettingDataType::Integer
        ));
        assert!(!validate_value_type(
            &serde_json::json!(2.5),
            &SettingDataType::Integer
        ));
        assert!(!validate_value_type(
            &serde_json::json!("42"),
            &SettingDataType::Integer
        ));
    }

    #[test]
    fn test_validate_value_type_string() {
        assert!(validate_value_type(
            &serde_json::json!("hello"),
            &SettingDataType::String
        ));
        assert!(validate_value_type(
            &serde_json::json!(""),
            &SettingDataType::String
        ));
        assert!(!validate_value_type(
            &serde_json::json!(42),
            &SettingDataType::String
        ));
    }

    #[test]
    fn test_validate_value_type_float() {
        assert!(validate_value_type(
            &serde_json::json!(2.5),
            &SettingDataType::Float
        ));
        assert!(validate_value_type(
            &serde_json::json!(42),
            &SettingDataType::Float
        )); // integers are valid floats
        assert!(!validate_value_type(
            &serde_json::json!("3.14"),
            &SettingDataType::Float
        ));
    }

    #[test]
    fn test_validate_value_type_json() {
        // Any JSON value is valid for JSON type
        assert!(validate_value_type(
            &serde_json::json!({"key": "value"}),
            &SettingDataType::Json
        ));
        assert!(validate_value_type(
            &serde_json::json!([1, 2, 3]),
            &SettingDataType::Json
        ));
        assert!(validate_value_type(
            &serde_json::json!(null),
            &SettingDataType::Json
        ));
        assert!(validate_value_type(
            &serde_json::json!(true),
            &SettingDataType::Json
        ));
    }
}
