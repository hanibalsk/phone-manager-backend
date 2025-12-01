//! Data privacy control API routes.
//!
//! Provides GDPR-compliant data export and deletion endpoints.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{info, warn};
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use persistence::repositories::{DeviceRepository, LocationRepository};

/// Exported location data.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub altitude: Option<f64>,
    pub bearing: Option<f64>,
    pub speed: Option<f64>,
    pub provider: Option<String>,
    pub battery_level: Option<i32>,
    pub network_type: Option<String>,
    pub captured_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Exported device data.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedDevice {
    pub device_id: Uuid,
    pub display_name: String,
    pub group_id: String,
    pub platform: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// Response for data export.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataExportResponse {
    pub device: ExportedDevice,
    pub locations: Vec<ExportedLocation>,
    pub location_count: usize,
    pub export_timestamp: DateTime<Utc>,
}

/// GET /api/v1/devices/:device_id/data-export
///
/// Exports all data for a device including all location history.
/// This endpoint supports GDPR Article 20 (Right to Data Portability).
pub async fn export_device_data(
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());
    let location_repo = LocationRepository::new(state.pool.clone());

    // Find the device
    let device = device_repo.find_by_device_id(device_id).await?;

    let device = match device {
        Some(d) => d,
        None => {
            warn!(device_id = %device_id, "Data export requested for non-existent device");
            return Err(ApiError::NotFound("Device not found".to_string()));
        }
    };

    // Get all locations (this could be a lot of data)
    let locations = location_repo
        .get_all_locations_for_device(device_id)
        .await?;

    let exported_device = ExportedDevice {
        device_id: device.device_id,
        display_name: device.display_name,
        group_id: device.group_id,
        platform: device.platform,
        active: device.active,
        created_at: device.created_at,
        updated_at: device.updated_at,
        last_seen_at: device.last_seen_at,
    };

    let exported_locations: Vec<ExportedLocation> = locations
        .into_iter()
        .map(|loc| ExportedLocation {
            latitude: loc.latitude,
            longitude: loc.longitude,
            accuracy: loc.accuracy,
            altitude: loc.altitude,
            bearing: loc.bearing,
            speed: loc.speed,
            provider: loc.provider,
            battery_level: loc.battery_level,
            network_type: loc.network_type,
            captured_at: loc.captured_at,
            created_at: loc.created_at,
        })
        .collect();

    let location_count = exported_locations.len();

    info!(
        device_id = %device_id,
        location_count = location_count,
        "Data export completed"
    );

    Ok(Json(DataExportResponse {
        device: exported_device,
        locations: exported_locations,
        location_count,
        export_timestamp: Utc::now(),
    }))
}

/// DELETE /api/v1/devices/:device_id/data
///
/// Hard deletes a device and all associated location data.
/// This is irreversible and supports GDPR Article 17 (Right to Erasure).
pub async fn delete_device_data(
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let device_repo = DeviceRepository::new(state.pool.clone());

    // Check if device exists
    let device = device_repo.find_by_device_id(device_id).await?;

    if device.is_none() {
        warn!(device_id = %device_id, "Data deletion requested for non-existent device");
        return Err(ApiError::NotFound("Device not found".to_string()));
    }

    // Hard delete the device (locations are deleted via CASCADE)
    let deleted = device_repo.hard_delete_device(device_id).await?;

    if deleted == 0 {
        return Err(ApiError::NotFound("Device not found".to_string()));
    }

    info!(
        device_id = %device_id,
        "Device and all associated data permanently deleted"
    );

    // Return 204 No Content for successful deletion
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exported_location_serialization() {
        let location = ExportedLocation {
            latitude: 37.7749,
            longitude: -122.4194,
            accuracy: 10.0,
            altitude: Some(100.0),
            bearing: Some(45.0),
            speed: Some(5.0),
            provider: Some("gps".to_string()),
            battery_level: Some(80),
            network_type: Some("wifi".to_string()),
            captured_at: Utc::now(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&location).unwrap();
        assert!(json.contains("\"latitude\":37.7749"));
        assert!(json.contains("\"longitude\":-122.4194"));
        assert!(json.contains("\"accuracy\":10.0"));
        assert!(json.contains("\"batteryLevel\":80"));
    }

    #[test]
    fn test_exported_device_serialization() {
        let device = ExportedDevice {
            device_id: Uuid::nil(),
            display_name: "Test Device".to_string(),
            group_id: "test-group".to_string(),
            platform: "android".to_string(),
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_seen_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("\"displayName\":\"Test Device\""));
        assert!(json.contains("\"groupId\":\"test-group\""));
        assert!(json.contains("\"platform\":\"android\""));
        assert!(json.contains("\"active\":true"));
    }

    #[test]
    fn test_data_export_response_serialization() {
        let response = DataExportResponse {
            device: ExportedDevice {
                device_id: Uuid::nil(),
                display_name: "Test".to_string(),
                group_id: "group".to_string(),
                platform: "ios".to_string(),
                active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_seen_at: None,
            },
            locations: vec![],
            location_count: 0,
            export_timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"device\""));
        assert!(json.contains("\"locations\":[]"));
        assert!(json.contains("\"locationCount\":0"));
        assert!(json.contains("\"exportTimestamp\""));
    }

    #[test]
    fn test_exported_location_without_optional_fields() {
        let location = ExportedLocation {
            latitude: 37.7749,
            longitude: -122.4194,
            accuracy: 10.0,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
            captured_at: Utc::now(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&location).unwrap();
        assert!(json.contains("\"altitude\":null"));
        assert!(json.contains("\"bearing\":null"));
        assert!(json.contains("\"speed\":null"));
    }
}
