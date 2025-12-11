//! Device policy management routes.
//!
//! Story 13.3: Device Policies Table and CRUD Endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use persistence::repositories::DevicePolicyRepository;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::{
    AppliedToCount, ApplyPolicyRequest, ApplyPolicyResponse, CreateDevicePolicyRequest,
    DevicePolicy, DevicePolicyPagination, DevicePolicyResponse, ListDevicePoliciesQuery,
    ListDevicePoliciesResponse, PolicyTargetType, UnapplyPolicyRequest, UnapplyPolicyResponse,
    UpdateDevicePolicyRequest,
};

/// Create a new device policy.
///
/// POST /api/admin/v1/organizations/:org_id/policies
pub async fn create_policy(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(request): Json<CreateDevicePolicyRequest>,
) -> Result<(StatusCode, Json<DevicePolicyResponse>), ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    let repo = DevicePolicyRepository::new(state.pool.clone());

    // Check if policy with same name exists
    let existing: Option<DevicePolicy> = repo.find_by_org_and_name(org_id, &request.name).await?;
    if existing.is_some() {
        return Err(ApiError::Conflict(format!(
            "Policy with name '{}' already exists in this organization",
            request.name
        )));
    }

    // Convert settings to JSON value
    let settings = serde_json::to_value(&request.settings).unwrap_or_default();

    // Create policy
    let policy = repo
        .create(
            org_id,
            &request.name,
            request.description.as_deref(),
            request.is_default,
            &settings,
            &request.locked_settings,
            request.priority,
        )
        .await?;

    tracing::info!(
        policy_id = %policy.id,
        organization_id = %org_id,
        name = %policy.name,
        "Device policy created"
    );

    Ok((StatusCode::CREATED, Json(policy.into())))
}

/// List device policies for an organization.
///
/// GET /api/admin/v1/organizations/:org_id/policies
pub async fn list_policies(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListDevicePoliciesQuery>,
) -> Result<Json<ListDevicePoliciesResponse>, ApiError> {
    let repo = DevicePolicyRepository::new(state.pool.clone());

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);

    let (policies, total): (Vec<DevicePolicy>, i64) = repo.list(org_id, &query).await?;

    let response = ListDevicePoliciesResponse {
        data: policies.into_iter().map(Into::into).collect(),
        pagination: DevicePolicyPagination {
            page,
            per_page,
            total,
        },
    };

    Ok(Json(response))
}

/// Get a specific device policy.
///
/// GET /api/admin/v1/organizations/:org_id/policies/:policy_id
pub async fn get_policy(
    State(state): State<AppState>,
    Path((org_id, policy_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DevicePolicyResponse>, ApiError> {
    let repo = DevicePolicyRepository::new(state.pool.clone());

    let policy: DevicePolicy = repo
        .find_by_id(policy_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device policy not found".to_string()))?;

    // Verify policy belongs to the organization
    if policy.organization_id != org_id {
        return Err(ApiError::NotFound("Device policy not found".to_string()));
    }

    Ok(Json(policy.into()))
}

/// Update a device policy.
///
/// PUT /api/admin/v1/organizations/:org_id/policies/:policy_id
pub async fn update_policy(
    State(state): State<AppState>,
    Path((org_id, policy_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateDevicePolicyRequest>,
) -> Result<Json<DevicePolicyResponse>, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    let repo = DevicePolicyRepository::new(state.pool.clone());

    // Verify policy exists and belongs to organization
    let existing: DevicePolicy = repo
        .find_by_id(policy_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device policy not found".to_string()))?;

    if existing.organization_id != org_id {
        return Err(ApiError::NotFound("Device policy not found".to_string()));
    }

    // Check for name conflict if name is being changed
    if let Some(ref new_name) = request.name {
        if new_name != &existing.name {
            let existing_with_name: Option<DevicePolicy> =
                repo.find_by_org_and_name(org_id, new_name).await?;
            if existing_with_name.is_some() {
                return Err(ApiError::Conflict(format!(
                    "Policy with name '{}' already exists in this organization",
                    new_name
                )));
            }
        }
    }

    // Convert settings to JSON value if provided
    let settings = request
        .settings
        .as_ref()
        .map(|s| serde_json::to_value(s).unwrap_or_default());

    // Handle description update:
    // - request.description is Some(value) -> set description to Some(value)
    // - request.description is None -> keep existing description (pass None to repo)
    let description: Option<Option<&str>> = request.description.as_ref().map(|d| Some(d.as_str()));

    // Update policy
    let updated: DevicePolicy = repo
        .update(
            policy_id,
            request.name.as_deref(),
            description,
            request.is_default,
            settings.as_ref(),
            request.locked_settings.as_deref(),
            request.priority,
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("Device policy not found".to_string()))?;

    tracing::info!(
        policy_id = %policy_id,
        organization_id = %org_id,
        "Device policy updated"
    );

    Ok(Json(updated.into()))
}

/// Delete a device policy.
///
/// DELETE /api/admin/v1/organizations/:org_id/policies/:policy_id
pub async fn delete_policy(
    State(state): State<AppState>,
    Path((org_id, policy_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let repo = DevicePolicyRepository::new(state.pool.clone());

    // Verify policy exists and belongs to organization
    let policy: DevicePolicy = repo
        .find_by_id(policy_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device policy not found".to_string()))?;

    if policy.organization_id != org_id {
        return Err(ApiError::NotFound("Device policy not found".to_string()));
    }

    // Check if policy has devices assigned
    if repo.has_devices(policy_id).await? {
        return Err(ApiError::Conflict(
            "Cannot delete policy with devices assigned. Remove devices first.".to_string(),
        ));
    }

    // Delete policy
    repo.delete(policy_id).await?;

    tracing::info!(
        policy_id = %policy_id,
        organization_id = %org_id,
        "Device policy deleted"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Apply a policy to devices/groups.
///
/// POST /api/admin/v1/organizations/:org_id/policies/:policy_id/apply
pub async fn apply_policy(
    State(state): State<AppState>,
    Path((org_id, policy_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ApplyPolicyRequest>,
) -> Result<Json<ApplyPolicyResponse>, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    let repo = DevicePolicyRepository::new(state.pool.clone());

    // Verify policy exists and belongs to organization
    let policy: DevicePolicy = repo
        .find_by_id(policy_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device policy not found".to_string()))?;

    if policy.organization_id != org_id {
        return Err(ApiError::NotFound("Device policy not found".to_string()));
    }

    let mut devices_affected: i64 = 0;
    let mut groups_count: i64 = 0;
    let mut device_ids = Vec::new();
    let mut group_ids = Vec::new();

    // Separate targets by type
    for target in &request.targets {
        match target.target_type {
            PolicyTargetType::Device => device_ids.push(target.id),
            PolicyTargetType::Group => group_ids.push(target.id),
        }
    }

    // Apply to individual devices
    if !device_ids.is_empty() {
        devices_affected += repo
            .apply_to_devices(policy_id, &device_ids, org_id, request.replace_existing)
            .await?;
    }

    // Apply to groups
    for group_id in &group_ids {
        let affected = repo
            .apply_to_group(policy_id, *group_id, org_id, request.replace_existing)
            .await?;
        devices_affected += affected;
        groups_count += 1;
    }

    tracing::info!(
        policy_id = %policy_id,
        organization_id = %org_id,
        devices = devices_affected,
        groups = groups_count,
        "Policy applied"
    );

    Ok(Json(ApplyPolicyResponse {
        policy_id,
        applied_to: AppliedToCount {
            devices: device_ids.len() as i64,
            groups: groups_count,
        },
        total_devices_affected: devices_affected,
    }))
}

/// Unapply a policy from devices/groups.
///
/// POST /api/admin/v1/organizations/:org_id/policies/:policy_id/unapply
pub async fn unapply_policy(
    State(state): State<AppState>,
    Path((org_id, policy_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UnapplyPolicyRequest>,
) -> Result<Json<UnapplyPolicyResponse>, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    let repo = DevicePolicyRepository::new(state.pool.clone());

    // Verify policy exists and belongs to organization
    let policy: DevicePolicy = repo
        .find_by_id(policy_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device policy not found".to_string()))?;

    if policy.organization_id != org_id {
        return Err(ApiError::NotFound("Device policy not found".to_string()));
    }

    let mut devices_affected: i64 = 0;
    let mut groups_count: i64 = 0;
    let mut device_ids = Vec::new();
    let mut group_ids = Vec::new();

    // Separate targets by type
    for target in &request.targets {
        match target.target_type {
            PolicyTargetType::Device => device_ids.push(target.id),
            PolicyTargetType::Group => group_ids.push(target.id),
        }
    }

    // Unapply from individual devices
    if !device_ids.is_empty() {
        devices_affected += repo
            .unapply_from_devices(policy_id, &device_ids, org_id)
            .await?;
    }

    // Unapply from groups
    for group_id in &group_ids {
        let affected = repo
            .unapply_from_group(policy_id, *group_id, org_id)
            .await?;
        devices_affected += affected;
        groups_count += 1;
    }

    tracing::info!(
        policy_id = %policy_id,
        organization_id = %org_id,
        devices = devices_affected,
        groups = groups_count,
        "Policy unapplied"
    );

    Ok(Json(UnapplyPolicyResponse {
        policy_id,
        unapplied_from: AppliedToCount {
            devices: device_ids.len() as i64,
            groups: groups_count,
        },
        total_devices_affected: devices_affected,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_policy_request_validation() {
        let request = CreateDevicePolicyRequest {
            name: "Test Policy".to_string(),
            description: Some("A test policy".to_string()),
            is_default: false,
            settings: std::collections::HashMap::new(),
            locked_settings: vec!["tracking_enabled".to_string()],
            priority: 10,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_update_policy_request_validation() {
        let request = UpdateDevicePolicyRequest {
            name: Some("Updated Policy".to_string()),
            description: None,
            is_default: None,
            settings: None,
            locked_settings: None,
            priority: Some(20),
        };
        assert!(request.validate().is_ok());
    }
}
