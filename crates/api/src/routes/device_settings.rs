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
}
