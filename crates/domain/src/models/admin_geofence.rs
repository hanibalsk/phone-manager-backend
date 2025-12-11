//! Admin geofence domain models for organization-wide geofence management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Query parameters for listing admin geofences.
#[derive(Debug, Clone, Deserialize, Validate, Default)]
#[serde(rename_all = "snake_case")]
pub struct AdminGeofenceQuery {
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    pub page: Option<u32>,
    #[validate(range(min = 1, max = 100, message = "Per page must be between 1 and 100"))]
    pub per_page: Option<u32>,
    pub active: Option<bool>,
    pub search: Option<String>,
}

/// Pagination for admin geofence list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminGeofencePagination {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

/// Admin geofence info for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminGeofenceInfo {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: f32,
    pub event_types: Vec<String>,
    pub active: bool,
    pub color: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub creator_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for listing admin geofences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminGeofenceListResponse {
    pub data: Vec<AdminGeofenceInfo>,
    pub pagination: AdminGeofencePagination,
}

/// Request for creating an admin geofence.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateAdminGeofenceRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,
    pub description: Option<String>,
    #[validate(range(min = -90.0, max = 90.0, message = "Latitude must be between -90 and 90"))]
    pub latitude: f64,
    #[validate(range(min = -180.0, max = 180.0, message = "Longitude must be between -180 and 180"))]
    pub longitude: f64,
    #[validate(range(
        min = 20.0,
        max = 50000.0,
        message = "Radius must be between 20 and 50000 meters"
    ))]
    pub radius_meters: f32,
    #[validate(length(min = 1, message = "At least one event type required"))]
    pub event_types: Vec<String>,
    pub color: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Response for creating an admin geofence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateAdminGeofenceResponse {
    pub geofence: AdminGeofenceInfo,
}

/// Request for updating an admin geofence.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateAdminGeofenceRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(range(min = -90.0, max = 90.0, message = "Latitude must be between -90 and 90"))]
    pub latitude: Option<f64>,
    #[validate(range(min = -180.0, max = 180.0, message = "Longitude must be between -180 and 180"))]
    pub longitude: Option<f64>,
    #[validate(range(
        min = 20.0,
        max = 50000.0,
        message = "Radius must be between 20 and 50000 meters"
    ))]
    pub radius_meters: Option<f32>,
    pub event_types: Option<Vec<String>>,
    pub active: Option<bool>,
    pub color: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Response for updating an admin geofence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateAdminGeofenceResponse {
    pub geofence: AdminGeofenceInfo,
}

/// Response for deleting an admin geofence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeleteAdminGeofenceResponse {
    pub deleted: bool,
    pub geofence_id: Uuid,
}

/// Device location info for admin API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminDeviceLocation {
    pub device_id: Uuid,
    pub device_name: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: Option<f32>,
    pub altitude: Option<f64>,
    pub speed: Option<f32>,
    pub bearing: Option<f32>,
    pub timestamp: i64,
    pub recorded_at: DateTime<Utc>,
}

/// Query parameters for device location history.
#[derive(Debug, Clone, Deserialize, Validate, Default)]
#[serde(rename_all = "snake_case")]
pub struct AdminLocationHistoryQuery {
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    pub page: Option<u32>,
    #[validate(range(min = 1, max = 100, message = "Per page must be between 1 and 100"))]
    pub per_page: Option<u32>,
    pub from_timestamp: Option<i64>,
    pub to_timestamp: Option<i64>,
}

/// Response for device location history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminLocationHistoryResponse {
    pub device_id: Uuid,
    pub locations: Vec<AdminDeviceLocation>,
    pub pagination: AdminGeofencePagination,
}

/// Response for current device location.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminDeviceLocationResponse {
    pub location: Option<AdminDeviceLocation>,
}

/// Response for all device locations in organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminAllDeviceLocationsResponse {
    pub devices: Vec<AdminDeviceLocation>,
    pub total: i64,
}

/// Admin geofence event info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminGeofenceEventInfo {
    pub id: Uuid,
    pub device_id: Uuid,
    pub device_name: Option<String>,
    pub geofence_id: Uuid,
    pub geofence_name: String,
    pub event_type: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: i64,
    pub created_at: DateTime<Utc>,
}

/// Query parameters for geofence events.
#[derive(Debug, Clone, Deserialize, Validate, Default)]
#[serde(rename_all = "snake_case")]
pub struct AdminGeofenceEventsQuery {
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    pub page: Option<u32>,
    #[validate(range(min = 1, max = 100, message = "Per page must be between 1 and 100"))]
    pub per_page: Option<u32>,
    pub device_id: Option<Uuid>,
    pub geofence_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub from_timestamp: Option<i64>,
    pub to_timestamp: Option<i64>,
}

/// Response for geofence events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminGeofenceEventsResponse {
    pub events: Vec<AdminGeofenceEventInfo>,
    pub pagination: AdminGeofencePagination,
}

/// Location analytics summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LocationAnalyticsSummary {
    pub total_devices: i64,
    pub devices_with_location: i64,
    pub total_locations_today: i64,
    pub total_geofences: i64,
    pub total_geofence_events_today: i64,
    pub most_visited_geofences: Vec<GeofenceVisitCount>,
}

/// Geofence visit count for analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GeofenceVisitCount {
    pub geofence_id: Uuid,
    pub geofence_name: String,
    pub visit_count: i64,
}

/// Response for location analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AdminLocationAnalyticsResponse {
    pub summary: LocationAnalyticsSummary,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_geofence_query_validation() {
        let query = AdminGeofenceQuery {
            page: Some(1),
            per_page: Some(50),
            active: Some(true),
            search: None,
        };
        assert!(query.validate().is_ok());
    }

    #[test]
    fn test_create_admin_geofence_request_validation() {
        let request = CreateAdminGeofenceRequest {
            name: "Office Zone".to_string(),
            description: Some("Main office area".to_string()),
            latitude: 37.7749,
            longitude: -122.4194,
            radius_meters: 500.0,
            event_types: vec!["enter".to_string(), "exit".to_string()],
            color: Some("#FF5733".to_string()),
            metadata: None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_admin_geofence_request_invalid_latitude() {
        let request = CreateAdminGeofenceRequest {
            name: "Test".to_string(),
            description: None,
            latitude: 95.0, // Invalid
            longitude: -122.4194,
            radius_meters: 500.0,
            event_types: vec!["enter".to_string()],
            color: None,
            metadata: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_admin_geofence_request_invalid_radius() {
        let request = CreateAdminGeofenceRequest {
            name: "Test".to_string(),
            description: None,
            latitude: 37.7749,
            longitude: -122.4194,
            radius_meters: 10.0, // Too small
            event_types: vec!["enter".to_string()],
            color: None,
            metadata: None,
        };
        assert!(request.validate().is_err());
    }
}
