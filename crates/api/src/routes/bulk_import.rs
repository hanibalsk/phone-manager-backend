//! Bulk device import route handler.
//!
//! Story 13.8: Bulk Device Import Endpoint

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use persistence::repositories::{
    DevicePolicyRepository, DeviceRepository, GroupRepository, OrgUserRepository, UserRepository,
};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;

use domain::models::{
    BulkDeviceImportRequest, BulkDeviceImportResponse, BulkImportError, BulkImportResult,
    OrgUserRole, MAX_METADATA_SIZE,
};

/// Create bulk import routes.
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(bulk_import_devices))
}

/// Bulk import devices to an organization.
///
/// POST /api/admin/v1/organizations/{org_id}/devices/bulk
#[axum::debug_handler]
async fn bulk_import_devices(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    user: UserAuth,
    Json(request): Json<BulkDeviceImportRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let device_repo = DeviceRepository::new(state.pool.clone());
    let user_repo = UserRepository::new(state.pool.clone());
    let group_repo = GroupRepository::new(state.pool.clone());
    let policy_repo = DevicePolicyRepository::new(state.pool.clone());

    // Verify user has admin access to organization
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    let mut processed = 0u32;
    let mut created = 0u32;
    let mut updated = 0u32;
    let mut skipped = 0u32;
    let mut errors = Vec::new();

    // Process each device
    for (idx, device_item) in request.devices.iter().enumerate() {
        let row = idx + 1;
        processed += 1;

        // Validate metadata size if present
        if let Some(ref metadata) = device_item.metadata {
            let metadata_str = serde_json::to_string(metadata).unwrap_or_default();
            if metadata_str.len() > MAX_METADATA_SIZE {
                errors.push(BulkImportError {
                    row,
                    external_id: device_item.external_id.clone(),
                    error: format!("metadata exceeds {} byte limit", MAX_METADATA_SIZE),
                });
                continue;
            }
        }

        // Validate group_id if provided
        if let Some(group_id) = &device_item.group_id {
            let group = group_repo.find_by_id(*group_id).await;
            match group {
                Ok(Some(_)) => {}
                Ok(None) => {
                    errors.push(BulkImportError {
                        row,
                        external_id: device_item.external_id.clone(),
                        error: format!("Group not found: {}", group_id),
                    });
                    continue;
                }
                Err(e) => {
                    errors.push(BulkImportError {
                        row,
                        external_id: device_item.external_id.clone(),
                        error: format!("Error checking group: {}", e),
                    });
                    continue;
                }
            }
        }

        // Validate policy_id if provided
        if let Some(policy_id) = device_item.policy_id {
            let policy = policy_repo.find_by_id(policy_id).await;
            match policy {
                Ok(Some(p)) => {
                    if p.organization_id != org_id {
                        errors.push(BulkImportError {
                            row,
                            external_id: device_item.external_id.clone(),
                            error: format!("Policy not found in organization: {}", policy_id),
                        });
                        continue;
                    }
                }
                Ok(None) => {
                    errors.push(BulkImportError {
                        row,
                        external_id: device_item.external_id.clone(),
                        error: format!("Policy not found: {}", policy_id),
                    });
                    continue;
                }
                Err(e) => {
                    errors.push(BulkImportError {
                        row,
                        external_id: device_item.external_id.clone(),
                        error: format!("Error checking policy: {}", e),
                    });
                    continue;
                }
            }
        }

        // Look up assigned user by email if provided
        let assigned_user_id = if let Some(ref email) = device_item.assigned_user_email {
            let user_result = user_repo.find_by_email(email).await;
            match user_result {
                Ok(Some(u)) => {
                    // Verify user is in organization
                    let org_user_check = org_user_repo.find_by_org_and_user(org_id, u.id).await;
                    match org_user_check {
                        Ok(Some(_)) => Some(u.id),
                        Ok(None) => {
                            if request.options.create_missing_users {
                                // Would create invite here (deferred)
                                errors.push(BulkImportError {
                                    row,
                                    external_id: device_item.external_id.clone(),
                                    error: format!(
                                        "User {} not in organization (invite creation not yet implemented)",
                                        email
                                    ),
                                });
                                continue;
                            } else {
                                errors.push(BulkImportError {
                                    row,
                                    external_id: device_item.external_id.clone(),
                                    error: format!("User not in organization: {}", email),
                                });
                                continue;
                            }
                        }
                        Err(e) => {
                            errors.push(BulkImportError {
                                row,
                                external_id: device_item.external_id.clone(),
                                error: format!("Error checking user organization: {}", e),
                            });
                            continue;
                        }
                    }
                }
                Ok(None) => {
                    if request.options.create_missing_users {
                        // Would create invite here (deferred)
                        errors.push(BulkImportError {
                            row,
                            external_id: device_item.external_id.clone(),
                            error: format!(
                                "User {} not found (invite creation not yet implemented)",
                                email
                            ),
                        });
                        continue;
                    } else {
                        errors.push(BulkImportError {
                            row,
                            external_id: device_item.external_id.clone(),
                            error: format!("User not found: {}", email),
                        });
                        continue;
                    }
                }
                Err(e) => {
                    errors.push(BulkImportError {
                        row,
                        external_id: device_item.external_id.clone(),
                        error: format!("Error looking up user: {}", e),
                    });
                    continue;
                }
            }
        } else {
            None
        };

        // Check if device exists by external_id
        let existing_device = if let Some(ref ext_id) = device_item.external_id {
            device_repo.find_by_external_id(org_id, ext_id).await?
        } else {
            None
        };

        // Convert group_id to string for storage
        let group_id_str = device_item.group_id.as_ref().map(|g| g.to_string());

        // Process based on existence and options
        let result = match existing_device {
            Some(existing) => {
                if request.options.update_existing {
                    // Update existing device
                    match device_repo
                        .update_bulk_device(
                            existing.id,
                            &device_item.display_name,
                            group_id_str.as_deref(),
                            device_item.policy_id,
                            assigned_user_id,
                            device_item.metadata.as_ref(),
                        )
                        .await
                    {
                        Ok(d) => BulkImportResult::Updated(d.id),
                        Err(e) => BulkImportResult::Error(e.to_string()),
                    }
                } else {
                    BulkImportResult::Skipped
                }
            }
            None => {
                // Create new device
                match device_repo
                    .create_bulk_device(
                        org_id,
                        device_item.external_id.as_deref(),
                        &device_item.display_name,
                        group_id_str.as_deref(),
                        device_item.policy_id,
                        assigned_user_id,
                        device_item.metadata.as_ref(),
                    )
                    .await
                {
                    Ok(d) => BulkImportResult::Created(d.id),
                    Err(e) => BulkImportResult::Error(e.to_string()),
                }
            }
        };

        match result {
            BulkImportResult::Created(_) => created += 1,
            BulkImportResult::Updated(_) => updated += 1,
            BulkImportResult::Skipped => skipped += 1,
            BulkImportResult::Error(e) => {
                errors.push(BulkImportError {
                    row,
                    external_id: device_item.external_id.clone(),
                    error: e,
                });
            }
        }
    }

    let response = BulkDeviceImportResponse {
        processed,
        created,
        updated,
        skipped,
        errors,
    };

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let _router: Router<AppState> = router();
    }
}
