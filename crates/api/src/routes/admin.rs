//! Admin operations API routes.
//!
//! Provides administrative endpoints for system maintenance operations.
//! These routes require admin API key authentication.

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::api_key::ApiKeyAuth;
use persistence::repositories::DeviceRepository;

/// Query parameters for deleting inactive devices.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeleteInactiveDevicesQuery {
    /// Days of inactivity threshold (devices older than this will be deleted)
    pub older_than_days: i32,
}

/// Response for admin operations that affect multiple records.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminOperationResponse {
    pub success: bool,
    pub affected_count: i64,
    pub message: String,
}

/// Response for device reactivation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ReactivateDeviceResponse {
    pub success: bool,
    pub device_id: Uuid,
    pub message: String,
}

/// DELETE /api/v1/admin/devices/inactive
///
/// Deletes devices that have been inactive for longer than the specified threshold.
/// Only soft-deleted (active=false) devices older than the threshold are permanently deleted.
pub async fn delete_inactive_devices(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Query(query): Query<DeleteInactiveDevicesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate threshold
    if query.older_than_days < 1 {
        return Err(ApiError::Validation(
            "Days threshold must be at least 1".to_string(),
        ));
    }

    if query.older_than_days > 365 {
        return Err(ApiError::Validation(
            "Days threshold cannot exceed 365".to_string(),
        ));
    }

    let repo = DeviceRepository::new(state.pool.clone());
    let deleted_count = repo.delete_inactive_devices(query.older_than_days).await?;

    info!(
        admin_key_id = auth.api_key_id,
        threshold_days = query.older_than_days,
        deleted_count = deleted_count,
        "Admin deleted inactive devices"
    );

    Ok(Json(AdminOperationResponse {
        success: true,
        affected_count: deleted_count,
        message: format!(
            "Deleted {} inactive devices older than {} days",
            deleted_count, query.older_than_days
        ),
    }))
}

/// POST /api/v1/admin/devices/:device_id/reactivate
///
/// Reactivates a soft-deleted device, making it active again.
pub async fn reactivate_device(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(device_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = DeviceRepository::new(state.pool.clone());

    // Check if device exists
    let device = repo.find_by_device_id(device_id).await?;

    match device {
        None => {
            warn!(
                admin_key_id = auth.api_key_id,
                device_id = %device_id,
                "Admin attempted to reactivate non-existent device"
            );
            Err(ApiError::NotFound("Device not found".to_string()))
        }
        Some(device) => {
            if device.active {
                return Ok((
                    StatusCode::OK,
                    Json(ReactivateDeviceResponse {
                        success: true,
                        device_id,
                        message: "Device is already active".to_string(),
                    }),
                ));
            }

            // Reactivate the device
            let rows_affected = repo.reactivate_device(device_id).await?;

            if rows_affected == 0 {
                return Err(ApiError::NotFound("Device not found".to_string()));
            }

            info!(
                admin_key_id = auth.api_key_id,
                device_id = %device_id,
                "Admin reactivated device"
            );

            Ok((
                StatusCode::OK,
                Json(ReactivateDeviceResponse {
                    success: true,
                    device_id,
                    message: "Device reactivated successfully".to_string(),
                }),
            ))
        }
    }
}

/// GET /api/v1/admin/stats
///
/// Returns system statistics for admin dashboard.
pub async fn get_admin_stats(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = DeviceRepository::new(state.pool.clone());
    let stats = repo.get_admin_stats().await?;

    info!(
        admin_key_id = auth.api_key_id,
        "Admin fetched system statistics"
    );

    Ok(Json(stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_inactive_query_deserialization() {
        let json = r#"{"older_than_days": 30}"#;
        let query: DeleteInactiveDevicesQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.older_than_days, 30);
    }

    #[test]
    fn test_admin_operation_response_serialization() {
        let response = AdminOperationResponse {
            success: true,
            affected_count: 10,
            message: "Deleted 10 devices".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"affected_count\":10"));
    }

    #[test]
    fn test_reactivate_response_serialization() {
        let response = ReactivateDeviceResponse {
            success: true,
            device_id: Uuid::nil(),
            message: "Device reactivated".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"device_id\""));
    }

    #[test]
    fn test_admin_stats_serialization() {
        use persistence::repositories::AdminStats;
        let stats = AdminStats {
            total_devices: 100,
            active_devices: 80,
            inactive_devices: 20,
            total_locations: 10000,
            total_groups: 25,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_devices\":100"));
        assert!(json.contains("\"active_devices\":80"));
        assert!(json.contains("\"inactive_devices\":20"));
        assert!(json.contains("\"total_locations\":10000"));
        assert!(json.contains("\"total_groups\":25"));
    }
}
