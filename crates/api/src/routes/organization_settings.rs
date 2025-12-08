//! Organization settings admin API routes.
//!
//! Provides endpoints for managing per-organization admin settings.
//! These routes require B2B admin authentication.

use axum::{
    extract::{Extension, Path, State},
    response::IntoResponse,
    Json,
};
use domain::models::{
    OrganizationSettings, OrganizationSettingsResponse, UpdateOrganizationSettingsRequest,
    VerifyPinRequest, VerifyPinResponse,
};
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::api_key::ApiKeyAuth;
use persistence::repositories::{OrganizationRepository, OrganizationSettingsRepository};

/// GET /api/admin/v1/organizations/:org_id/settings
///
/// Get organization settings.
/// Returns settings with `has_unlock_pin` flag (PIN is never exposed).
pub async fn get_organization_settings(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let settings_repo = OrganizationSettingsRepository::new(state.pool.clone());

    // Get or create default settings
    let entity = settings_repo.get_or_create(org_id).await?;

    // Convert to domain model
    let settings = OrganizationSettings {
        id: entity.id,
        organization_id: entity.organization_id,
        has_unlock_pin: entity.unlock_pin_hash.is_some(),
        unlock_pin_hash: entity.unlock_pin_hash,
        default_daily_limit_minutes: entity.default_daily_limit_minutes,
        notifications_enabled: entity.notifications_enabled,
        auto_approve_unlock_requests: entity.auto_approve_unlock_requests,
        created_at: entity.created_at,
        updated_at: entity.updated_at,
    };

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        "Fetched organization settings"
    );

    Ok(Json(OrganizationSettingsResponse::from(settings)))
}

/// PUT /api/admin/v1/organizations/:org_id/settings
///
/// Update organization settings.
/// PIN is hashed with Argon2 before storage (irreversible).
pub async fn update_organization_settings(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
    Json(request): Json<UpdateOrganizationSettingsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request.validate().map_err(|e| {
        ApiError::Validation(format!("Validation error: {}", e))
    })?;

    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let settings_repo = OrganizationSettingsRepository::new(state.pool.clone());

    // Get current settings to merge with updates
    let current = settings_repo.get_or_create(org_id).await?;

    // Handle PIN update
    let new_pin_hash = if request.clear_pin {
        // Explicitly clear the PIN
        None
    } else if let Some(ref pin) = request.unlock_pin {
        // Hash new PIN using Argon2
        let hash = shared::password::hash_password(pin)
            .map_err(|e| ApiError::Internal(format!("Failed to hash PIN: {}", e)))?;
        Some(hash)
    } else {
        // Keep existing PIN
        current.unlock_pin_hash.clone()
    };

    // Use provided values or keep existing
    let default_daily_limit_minutes = request
        .default_daily_limit_minutes
        .unwrap_or(current.default_daily_limit_minutes);
    let notifications_enabled = request
        .notifications_enabled
        .unwrap_or(current.notifications_enabled);
    let auto_approve_unlock_requests = request
        .auto_approve_unlock_requests
        .unwrap_or(current.auto_approve_unlock_requests);

    // Update settings
    let entity = settings_repo
        .upsert(
            org_id,
            new_pin_hash,
            default_daily_limit_minutes,
            notifications_enabled,
            auto_approve_unlock_requests,
        )
        .await?;

    // Convert to domain model
    let settings = OrganizationSettings {
        id: entity.id,
        organization_id: entity.organization_id,
        has_unlock_pin: entity.unlock_pin_hash.is_some(),
        unlock_pin_hash: entity.unlock_pin_hash,
        default_daily_limit_minutes: entity.default_daily_limit_minutes,
        notifications_enabled: entity.notifications_enabled,
        auto_approve_unlock_requests: entity.auto_approve_unlock_requests,
        created_at: entity.created_at,
        updated_at: entity.updated_at,
    };

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        pin_changed = request.unlock_pin.is_some() || request.clear_pin,
        "Updated organization settings"
    );

    Ok(Json(OrganizationSettingsResponse::from(settings)))
}

/// POST /api/admin/v1/organizations/:org_id/settings/verify-pin
///
/// Verify unlock PIN.
/// Returns whether the provided PIN matches the stored hash.
pub async fn verify_unlock_pin(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
    Json(request): Json<VerifyPinRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request.validate().map_err(|e| {
        ApiError::Validation(format!("Validation error: {}", e))
    })?;

    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let settings_repo = OrganizationSettingsRepository::new(state.pool.clone());

    // Get settings (no auto-create here - if no settings, no PIN to verify)
    let entity = settings_repo.get_by_organization_id(org_id).await?;

    let valid = match entity {
        Some(settings) => {
            match settings.unlock_pin_hash {
                Some(ref hash) => {
                    // Verify PIN against stored hash
                    shared::password::verify_password(&request.pin, hash)
                        .unwrap_or(false)
                }
                None => {
                    // No PIN set - verification fails
                    false
                }
            }
        }
        None => {
            // No settings exist - no PIN set
            false
        }
    };

    if valid {
        info!(
            admin_key_id = auth.api_key_id,
            organization_id = %org_id,
            "PIN verification successful"
        );
    } else {
        warn!(
            admin_key_id = auth.api_key_id,
            organization_id = %org_id,
            "PIN verification failed"
        );
    }

    Ok(Json(VerifyPinResponse { valid }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_request_with_pin() {
        let json = r#"{"unlock_pin": "1234", "default_daily_limit_minutes": 90}"#;
        let request: UpdateOrganizationSettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.unlock_pin, Some("1234".to_string()));
        assert_eq!(request.default_daily_limit_minutes, Some(90));
        assert!(!request.clear_pin);
    }

    #[test]
    fn test_update_request_clear_pin() {
        let json = r#"{"clear_pin": true}"#;
        let request: UpdateOrganizationSettingsRequest = serde_json::from_str(json).unwrap();
        assert!(request.clear_pin);
        assert!(request.unlock_pin.is_none());
    }

    #[test]
    fn test_verify_pin_request() {
        let json = r#"{"pin": "5678"}"#;
        let request: VerifyPinRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.pin, "5678");
    }

    #[test]
    fn test_settings_response_serialization() {
        let response = OrganizationSettingsResponse {
            has_unlock_pin: true,
            default_daily_limit_minutes: 120,
            notifications_enabled: true,
            auto_approve_unlock_requests: false,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"has_unlock_pin\":true"));
        assert!(json.contains("\"default_daily_limit_minutes\":120"));
    }
}
