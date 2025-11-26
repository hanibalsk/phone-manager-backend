//! Location domain model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Represents a location record in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub id: i64,
    pub device_id: Uuid,
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

/// Request payload for single location upload.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UploadLocationRequest {
    pub device_id: Uuid,

    /// Timestamp in milliseconds since epoch
    #[validate(custom(function = "shared::validation::validate_timestamp"))]
    pub timestamp: i64,

    #[validate(custom(function = "shared::validation::validate_latitude"))]
    pub latitude: f64,

    #[validate(custom(function = "shared::validation::validate_longitude"))]
    pub longitude: f64,

    #[validate(custom(function = "shared::validation::validate_accuracy"))]
    pub accuracy: f64,

    pub altitude: Option<f64>,

    #[validate(custom(function = "shared::validation::validate_bearing"))]
    pub bearing: Option<f64>,

    #[validate(custom(function = "shared::validation::validate_speed"))]
    pub speed: Option<f64>,

    pub provider: Option<String>,

    #[validate(custom(function = "shared::validation::validate_battery_level"))]
    pub battery_level: Option<i32>,

    pub network_type: Option<String>,
}

/// Request payload for batch location upload.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BatchUploadRequest {
    pub device_id: Uuid,

    #[validate(length(min = 1, max = 50, message = "Batch must contain 1-50 locations"))]
    pub locations: Vec<LocationData>,
}

/// Individual location data within a batch.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LocationData {
    #[validate(custom(function = "shared::validation::validate_timestamp"))]
    pub timestamp: i64,

    #[validate(custom(function = "shared::validation::validate_latitude"))]
    pub latitude: f64,

    #[validate(custom(function = "shared::validation::validate_longitude"))]
    pub longitude: f64,

    #[validate(custom(function = "shared::validation::validate_accuracy"))]
    pub accuracy: f64,

    pub altitude: Option<f64>,

    #[validate(custom(function = "shared::validation::validate_bearing"))]
    pub bearing: Option<f64>,

    #[validate(custom(function = "shared::validation::validate_speed"))]
    pub speed: Option<f64>,

    pub provider: Option<String>,

    #[validate(custom(function = "shared::validation::validate_battery_level"))]
    pub battery_level: Option<i32>,

    pub network_type: Option<String>,
}

/// Response payload for location upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadLocationResponse {
    pub success: bool,
    pub processed_count: usize,
}

/// Last known location for a device.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: DateTime<Utc>,
    pub accuracy: f64,
}

// ============================================================================
// Location History (GET /api/v1/devices/{device_id}/locations)
// ============================================================================

/// Sort order for location history queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

impl<'de> serde::Deserialize<'de> for SortOrder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "asc" => Ok(SortOrder::Asc),
            "desc" => Ok(SortOrder::Desc),
            _ => Err(serde::de::Error::custom("order must be 'asc' or 'desc'")),
        }
    }
}

/// Query parameters for location history endpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLocationHistoryQuery {
    /// Opaque cursor for pagination (base64-encoded timestamp:id).
    pub cursor: Option<String>,

    /// Number of results per page (1-100, default 50).
    pub limit: Option<i32>,

    /// Start timestamp filter (milliseconds since epoch).
    pub from: Option<i64>,

    /// End timestamp filter (milliseconds since epoch).
    pub to: Option<i64>,

    /// Sort order: "asc" or "desc" (default "desc").
    #[serde(default)]
    pub order: SortOrder,
}

impl GetLocationHistoryQuery {
    /// Default limit for location history queries.
    pub const DEFAULT_LIMIT: i32 = 50;
    /// Maximum limit for location history queries.
    pub const MAX_LIMIT: i32 = 100;
    /// Minimum limit for location history queries.
    pub const MIN_LIMIT: i32 = 1;

    /// Returns the effective limit, clamped to valid range.
    pub fn effective_limit(&self) -> i32 {
        self.limit
            .unwrap_or(Self::DEFAULT_LIMIT)
            .clamp(Self::MIN_LIMIT, Self::MAX_LIMIT)
    }
}

/// Single location item in history response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationHistoryItem {
    pub id: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_type: Option<String>,
    pub captured_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<Location> for LocationHistoryItem {
    fn from(loc: Location) -> Self {
        Self {
            id: loc.id,
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
        }
    }
}

/// Pagination info for cursor-based pagination.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationInfo {
    /// Cursor for fetching the next page.
    pub next_cursor: Option<String>,
    /// Whether there are more results available.
    pub has_more: bool,
}

/// Response payload for location history endpoint.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationHistoryResponse {
    pub locations: Vec<LocationHistoryItem>,
    pub pagination: PaginationInfo,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    /// Returns current timestamp in milliseconds for testing
    fn current_timestamp_millis() -> i64 {
        Utc::now().timestamp_millis()
    }

    fn create_test_location() -> Location {
        Location {
            id: 1,
            device_id: Uuid::new_v4(),
            latitude: 37.7749,
            longitude: -122.4194,
            accuracy: 10.0,
            altitude: Some(100.0),
            bearing: Some(180.0),
            speed: Some(5.5),
            provider: Some("gps".to_string()),
            battery_level: Some(85),
            network_type: Some("wifi".to_string()),
            captured_at: Utc::now(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_location_struct() {
        let location = create_test_location();
        assert_eq!(location.latitude, 37.7749);
        assert_eq!(location.longitude, -122.4194);
        assert_eq!(location.accuracy, 10.0);
    }

    #[test]
    fn test_location_clone() {
        let location = create_test_location();
        let cloned = location.clone();
        assert_eq!(cloned.latitude, location.latitude);
        assert_eq!(cloned.device_id, location.device_id);
    }

    #[test]
    fn test_location_optional_fields() {
        let mut location = create_test_location();
        location.altitude = None;
        location.bearing = None;
        location.speed = None;
        location.provider = None;
        location.battery_level = None;
        location.network_type = None;
        assert!(location.altitude.is_none());
        assert!(location.provider.is_none());
    }

    #[test]
    fn test_upload_location_response() {
        let response = UploadLocationResponse {
            success: true,
            processed_count: 5,
        };
        assert!(response.success);
        assert_eq!(response.processed_count, 5);
    }

    #[test]
    fn test_last_location() {
        let last = LastLocation {
            latitude: 40.7128,
            longitude: -74.0060,
            timestamp: Utc::now(),
            accuracy: 5.0,
        };
        assert_eq!(last.latitude, 40.7128);
        assert_eq!(last.accuracy, 5.0);
    }

    #[test]
    fn test_location_data_valid() {
        let data = LocationData {
            timestamp: current_timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            altitude: None,
            bearing: Some(90.0),
            speed: Some(10.0),
            provider: None,
            battery_level: Some(50),
            network_type: None,
        };
        assert!(data.validate().is_ok());
    }

    #[test]
    fn test_location_data_invalid_latitude() {
        let data = LocationData {
            timestamp: current_timestamp_millis(),
            latitude: 100.0, // Invalid: > 90
            longitude: -120.0,
            accuracy: 10.0,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
        };
        assert!(data.validate().is_err());
    }

    #[test]
    fn test_location_data_invalid_longitude() {
        let data = LocationData {
            timestamp: current_timestamp_millis(),
            latitude: 45.0,
            longitude: -200.0, // Invalid: < -180
            accuracy: 10.0,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
        };
        assert!(data.validate().is_err());
    }

    #[test]
    fn test_location_data_invalid_accuracy() {
        let data = LocationData {
            timestamp: current_timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: -5.0, // Invalid: negative
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
        };
        assert!(data.validate().is_err());
    }

    #[test]
    fn test_location_data_invalid_bearing() {
        let data = LocationData {
            timestamp: current_timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            altitude: None,
            bearing: Some(400.0), // Invalid: > 360
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
        };
        assert!(data.validate().is_err());
    }

    #[test]
    fn test_location_data_invalid_speed() {
        let data = LocationData {
            timestamp: current_timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            altitude: None,
            bearing: None,
            speed: Some(-10.0), // Invalid: negative
            provider: None,
            battery_level: None,
            network_type: None,
        };
        assert!(data.validate().is_err());
    }

    #[test]
    fn test_location_data_invalid_battery() {
        let data = LocationData {
            timestamp: current_timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: Some(150), // Invalid: > 100
            network_type: None,
        };
        assert!(data.validate().is_err());
    }

    #[test]
    fn test_batch_upload_request_valid() {
        let request = BatchUploadRequest {
            device_id: Uuid::new_v4(),
            locations: vec![LocationData {
                timestamp: current_timestamp_millis(),
                latitude: 45.0,
                longitude: -120.0,
                accuracy: 10.0,
                altitude: None,
                bearing: None,
                speed: None,
                provider: None,
                battery_level: None,
                network_type: None,
            }],
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_batch_upload_request_empty() {
        let request = BatchUploadRequest {
            device_id: Uuid::new_v4(),
            locations: vec![], // Invalid: min 1
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_batch_upload_request_too_many() {
        let ts = current_timestamp_millis();
        let locations: Vec<LocationData> = (0..51)
            .map(|_| LocationData {
                timestamp: ts,
                latitude: 45.0,
                longitude: -120.0,
                accuracy: 10.0,
                altitude: None,
                bearing: None,
                speed: None,
                provider: None,
                battery_level: None,
                network_type: None,
            })
            .collect();

        let request = BatchUploadRequest {
            device_id: Uuid::new_v4(),
            locations, // Invalid: > 50
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_location_data_boundary_values() {
        // Test boundary values that should be valid
        let data = LocationData {
            timestamp: current_timestamp_millis(),
            latitude: 90.0,    // Max valid
            longitude: -180.0, // Min valid
            accuracy: 0.0,     // Min valid
            altitude: None,
            bearing: Some(0.0), // Min valid
            speed: Some(0.0),   // Min valid
            provider: None,
            battery_level: Some(0), // Min valid
            network_type: None,
        };
        assert!(data.validate().is_ok());
    }

    #[test]
    fn test_location_data_max_boundary_values() {
        let data = LocationData {
            timestamp: current_timestamp_millis(),
            latitude: -90.0,  // Min valid
            longitude: 180.0, // Max valid
            accuracy: 1000.0, // Any positive is valid
            altitude: None,
            bearing: Some(360.0), // Max valid
            speed: Some(100.0),   // Any positive is valid
            provider: None,
            battery_level: Some(100), // Max valid
            network_type: None,
        };
        assert!(data.validate().is_ok());
    }

    #[test]
    fn test_location_data_invalid_timestamp_future() {
        // Timestamp 1 hour in the future should fail
        let future_ts = Utc::now().timestamp_millis() + 3600000;
        let data = LocationData {
            timestamp: future_ts,
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
        };
        assert!(data.validate().is_err());
    }

    #[test]
    fn test_location_data_invalid_timestamp_old() {
        // Timestamp 10 days ago should fail
        let old_ts = (Utc::now() - chrono::Duration::days(10)).timestamp_millis();
        let data = LocationData {
            timestamp: old_ts,
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
        };
        assert!(data.validate().is_err());
    }
}
