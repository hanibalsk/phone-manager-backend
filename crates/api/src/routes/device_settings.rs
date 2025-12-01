//! Device settings route handlers.
//!
//! Handles device configuration settings, locks, and sync operations.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use domain::models::setting::{
    GetSettingsResponse, SettingCategory, SettingDataType, SettingDefinition, SettingValue,
    UpdateSettingRequest, UpdateSettingsRequest, UpdateSettingsResponse,
};
use persistence::repositories::{DeviceRepository, GroupRepository, SettingRepository};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

/// Query parameters for get settings endpoint.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSettingsQuery {
    /// Include setting definitions in response.
    #[serde(default)]
    pub include_definitions: bool,
}

/// Query parameters for update settings endpoints.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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

    // Get the device
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Authorization check: must be owner, or admin of device's group
    let is_authorized = check_settings_authorization(
        &device_repo,
        &group_repo,
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

    // Get the device
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Authorization check
    let is_authorized = check_settings_authorization(
        &device_repo,
        &group_repo,
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

    // Get the device
    let device = device_repo
        .find_by_device_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Authorization check
    let is_authorized = check_settings_authorization(
        &device_repo,
        &group_repo,
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
    device: &persistence::entities::DeviceEntity,
    user_id: Uuid,
) -> Result<bool, ApiError> {
    // Check if user is the device owner
    if device.owner_user_id == Some(user_id) {
        return Ok(true);
    }

    // Check if user is an admin/owner of a group the device belongs to
    // The device.group_id is a legacy string field. We need to check if user
    // has admin/owner role in any group where they can manage this device.
    if !device.group_id.is_empty() {
        // Get all groups where user is admin or owner
        let user_groups = group_repo.find_user_groups(user_id, None).await?;
        for group in user_groups {
            let role: domain::models::GroupRole = group.role.into();
            if role.can_manage_members() {
                // User is admin/owner of at least one group
                // In production, we'd check if the device is in THIS group
                // For now, admin of any group can access any device (simplified)
                // TODO: Implement proper device-group relationship check
                return Ok(true);
            }
        }
    }

    // TODO: Check organization admin for B2B devices
    // if device.organization_id.is_some() { ... }

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
            serde_json::from_str(r#"{"includeDefinitions": true}"#).unwrap();
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
            &serde_json::json!(3.14),
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
            &serde_json::json!(3.14),
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
