//! Device endpoint handlers.

use axum::{extract::State, Json};
use serde::Deserialize;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::device::{DeviceSummary, RegisterDeviceRequest, RegisterDeviceResponse};

/// Query parameters for device listing.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDevicesQuery {
    pub group_id: Option<String>,
}

/// Response for device listing.
#[derive(Debug, serde::Serialize)]
pub struct GetDevicesResponse {
    pub devices: Vec<DeviceSummary>,
}

/// Register or update a device.
///
/// POST /api/devices/register
pub async fn register_device(
    State(_state): State<AppState>,
    Json(_request): Json<RegisterDeviceRequest>,
) -> Result<Json<RegisterDeviceResponse>, ApiError> {
    // Implementation will be completed in Story 2.1
    Err(ApiError::Internal("Not implemented yet".to_string()))
}

/// Get all active devices in a group.
///
/// GET /api/devices?groupId=<id>
pub async fn get_group_devices(
    State(_state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<GetDevicesQuery>,
) -> Result<Json<GetDevicesResponse>, ApiError> {
    let _group_id = query
        .group_id
        .ok_or_else(|| ApiError::Validation("groupId query parameter is required".to_string()))?;

    // Implementation will be completed in Story 2.5
    Ok(Json(GetDevicesResponse { devices: vec![] }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_get_devices_query_with_group_id() {
        let query = GetDevicesQuery {
            group_id: Some("family-group".to_string()),
        };
        assert_eq!(query.group_id, Some("family-group".to_string()));
    }

    #[test]
    fn test_get_devices_query_without_group_id() {
        let query = GetDevicesQuery { group_id: None };
        assert!(query.group_id.is_none());
    }

    #[test]
    fn test_get_devices_query_debug() {
        let query = GetDevicesQuery {
            group_id: Some("test".to_string()),
        };
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("GetDevicesQuery"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_get_devices_response_empty() {
        let response = GetDevicesResponse { devices: vec![] };
        assert!(response.devices.is_empty());
    }

    #[test]
    fn test_get_devices_response_with_devices() {
        let device1 = DeviceSummary {
            device_id: Uuid::new_v4(),
            display_name: "Phone 1".to_string(),
            last_seen_at: Some(Utc::now()),
        };
        let device2 = DeviceSummary {
            device_id: Uuid::new_v4(),
            display_name: "Phone 2".to_string(),
            last_seen_at: None,
        };
        let response = GetDevicesResponse {
            devices: vec![device1, device2],
        };
        assert_eq!(response.devices.len(), 2);
        assert_eq!(response.devices[0].display_name, "Phone 1");
        assert_eq!(response.devices[1].display_name, "Phone 2");
    }

    #[test]
    fn test_get_devices_response_debug() {
        let response = GetDevicesResponse { devices: vec![] };
        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("GetDevicesResponse"));
    }

    #[test]
    fn test_get_devices_response_serialization() {
        let device = DeviceSummary {
            device_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            display_name: "Test Device".to_string(),
            last_seen_at: None,
        };
        let response = GetDevicesResponse {
            devices: vec![device],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"devices\""));
        assert!(json.contains("Test Device"));
    }

    #[test]
    fn test_get_devices_query_deserialization() {
        let json = r#"{"groupId": "my-group"}"#;
        let query: GetDevicesQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.group_id, Some("my-group".to_string()));
    }

    #[test]
    fn test_get_devices_query_deserialization_missing_group() {
        let json = r#"{}"#;
        let query: GetDevicesQuery = serde_json::from_str(json).unwrap();
        assert!(query.group_id.is_none());
    }
}
