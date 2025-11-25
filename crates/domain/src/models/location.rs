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
    pub timestamp: i64,

    #[validate(custom(function = "shared::validation::validate_latitude"))]
    pub latitude: f64,

    #[validate(custom(function = "shared::validation::validate_longitude"))]
    pub longitude: f64,

    #[validate(custom(function = "shared::validation::validate_accuracy"))]
    pub accuracy: f64,

    pub altitude: Option<f64>,

    pub bearing: Option<f64>,

    pub speed: Option<f64>,

    pub provider: Option<String>,

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
    pub timestamp: i64,

    #[validate(range(min = -90.0, max = 90.0, message = "Latitude must be between -90 and 90"))]
    pub latitude: f64,

    #[validate(range(min = -180.0, max = 180.0, message = "Longitude must be between -180 and 180"))]
    pub longitude: f64,

    #[validate(range(min = 0.0, message = "Accuracy must be non-negative"))]
    pub accuracy: f64,

    pub altitude: Option<f64>,

    #[validate(range(min = 0.0, max = 360.0, message = "Bearing must be between 0 and 360"))]
    pub bearing: Option<f64>,

    #[validate(range(min = 0.0, message = "Speed must be non-negative"))]
    pub speed: Option<f64>,

    pub provider: Option<String>,

    #[validate(range(
        min = 0,
        max = 100,
        message = "Battery level must be between 0 and 100"
    ))]
    pub battery_level: Option<i32>,

    pub network_type: Option<String>,
}

/// Response payload for location upload.
#[derive(Debug, Clone, Serialize)]
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
