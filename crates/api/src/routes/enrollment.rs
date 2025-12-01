//! Device enrollment endpoint handler.
//!
//! Story 13.5: Device Enrollment Endpoint

use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use persistence::repositories::{
    DevicePolicyRepository, DeviceRepository, DeviceTokenRepository, EnrollmentTokenRepository,
};
use tracing::info;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::{
    calculate_device_token_expiry, extract_device_token_prefix, generate_device_token,
    DevicePolicy, EnrollDeviceRequest, EnrollDeviceResponse, EnrolledDevice, EnrollmentGroupInfo,
    EnrollmentPolicyInfo, EnrollmentStatus, EnrollmentToken, DEFAULT_TOKEN_EXPIRY_DAYS,
};

/// Enroll a device with an organization using an enrollment token.
///
/// POST /api/v1/devices/enroll
///
/// This endpoint does NOT require authentication - the enrollment token serves as auth.
pub async fn enroll_device(
    State(state): State<AppState>,
    Json(request): Json<EnrollDeviceRequest>,
) -> Result<(StatusCode, Json<EnrollDeviceResponse>), ApiError> {
    // Validate the request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    let enrollment_token_repo = EnrollmentTokenRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());
    let device_token_repo = DeviceTokenRepository::new(state.pool.clone());
    let policy_repo = DevicePolicyRepository::new(state.pool.clone());

    // Find and validate the enrollment token
    let enrollment_token: EnrollmentToken = enrollment_token_repo
        .find_valid_token(&request.enrollment_token)
        .await?
        .ok_or_else(|| ApiError::NotFound("Enrollment token not found or invalid".to_string()))?;

    // Check if token is valid (not expired, not exhausted)
    if !enrollment_token.is_valid() {
        if enrollment_token.is_expired() {
            return Err(ApiError::Gone("Enrollment token has expired".to_string()));
        }
        if enrollment_token.is_exhausted() {
            return Err(ApiError::Gone(
                "Enrollment token has reached maximum uses".to_string(),
            ));
        }
        if enrollment_token.is_revoked() {
            return Err(ApiError::Gone(
                "Enrollment token has been revoked".to_string(),
            ));
        }
        return Err(ApiError::Gone("Enrollment token is no longer valid".to_string()));
    }

    // Check if device already exists
    let existing_device = device_repo.find_by_device_id(request.device_uuid).await?;

    // If device exists and is managed by a different organization, reject
    if let Some(ref existing) = existing_device {
        if let Some(org_id) = existing.organization_id {
            if org_id != enrollment_token.organization_id {
                return Err(ApiError::Conflict(
                    "Device is already enrolled in a different organization".to_string(),
                ));
            }
        }
    }

    // Determine group ID - from token or generate one
    let group_id = enrollment_token
        .group_id
        .clone()
        .unwrap_or_else(|| format!("org_{}", enrollment_token.organization_id));

    // Create or update device
    let device = if let Some(existing) = existing_device {
        // Update existing device to be managed
        device_repo
            .update_enrollment(
                existing.id,
                enrollment_token.organization_id,
                Some(&group_id),
                enrollment_token.policy_id,
                EnrollmentStatus::Enrolled.as_str(),
                Some(enrollment_token.id),
            )
            .await?
    } else {
        // Create new managed device
        device_repo
            .create_managed_device(
                request.device_uuid,
                &request.display_name,
                &group_id,
                &request.platform,
                request.fcm_token.as_deref(),
                enrollment_token.organization_id,
                enrollment_token.policy_id,
                enrollment_token.id,
            )
            .await?
    };

    // Increment enrollment token usage
    enrollment_token_repo
        .increment_usage(enrollment_token.id)
        .await?;

    // Generate device token
    let token = generate_device_token();
    let token_prefix = extract_device_token_prefix(&token);
    let expires_at = calculate_device_token_expiry(DEFAULT_TOKEN_EXPIRY_DAYS);

    let device_token = device_token_repo
        .create(
            device.id,
            enrollment_token.organization_id,
            &token,
            &token_prefix,
            expires_at,
        )
        .await?;

    // Get policy details if applicable
    let policy_info = if let Some(policy_id) = enrollment_token.policy_id {
        let policy: Option<DevicePolicy> = policy_repo.find_by_id(policy_id).await?;
        policy.map(|p| EnrollmentPolicyInfo {
            id: p.id,
            name: p.name,
            settings: p.settings,
            locked_settings: p.locked_settings,
        })
    } else {
        None
    };

    // Build group info
    let group_info = Some(EnrollmentGroupInfo {
        id: group_id.clone(),
        name: Some(group_id), // In a real implementation, we'd look up the group name
    });

    let response = EnrollDeviceResponse {
        device: EnrolledDevice {
            id: device.id,
            device_uuid: device.device_id,
            display_name: device.display_name,
            organization_id: enrollment_token.organization_id,
            is_managed: true,
            enrollment_status: EnrollmentStatus::Enrolled,
            enrolled_at: Utc::now(),
        },
        device_token: device_token.token,
        device_token_expires_at: device_token.expires_at,
        policy: policy_info,
        group: group_info,
    };

    info!(
        device_id = %device.device_id,
        organization_id = %enrollment_token.organization_id,
        token_id = %enrollment_token.id,
        "Device enrolled successfully"
    );

    Ok((StatusCode::CREATED, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enrollment_status_conversion() {
        assert_eq!(EnrollmentStatus::Enrolled.as_str(), "enrolled");
        assert_eq!(EnrollmentStatus::Pending.as_str(), "pending");
    }
}
