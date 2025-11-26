//! Location endpoint handlers.

use axum::{extract::State, Json};

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::location::{BatchUploadRequest, UploadLocationRequest, UploadLocationResponse};

/// Upload a single location.
///
/// POST /api/locations
pub async fn upload_location(
    State(_state): State<AppState>,
    Json(_request): Json<UploadLocationRequest>,
) -> Result<Json<UploadLocationResponse>, ApiError> {
    // Implementation will be completed in Story 3.1
    Err(ApiError::Internal("Not implemented yet".to_string()))
}

/// Upload multiple locations in a batch.
///
/// POST /api/locations/batch
pub async fn upload_batch(
    State(_state): State<AppState>,
    Json(_request): Json<BatchUploadRequest>,
) -> Result<Json<UploadLocationResponse>, ApiError> {
    // Implementation will be completed in Story 3.2
    Err(ApiError::Internal("Not implemented yet".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::location::LocationData;
    use uuid::Uuid;

    #[test]
    fn test_upload_location_request_serialization() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": 1700000000000,
            "latitude": 37.7749,
            "longitude": -122.4194,
            "accuracy": 10.0,
            "altitude": 100.0,
            "bearing": 180.0,
            "speed": 5.5,
            "provider": "gps",
            "batteryLevel": 85,
            "networkType": "wifi"
        }"#;
        let request: UploadLocationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            request.device_id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(request.latitude, 37.7749);
        assert_eq!(request.longitude, -122.4194);
        assert_eq!(request.accuracy, 10.0);
    }

    #[test]
    fn test_upload_location_request_minimal() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": 1700000000000,
            "latitude": 37.7749,
            "longitude": -122.4194,
            "accuracy": 10.0
        }"#;
        let request: UploadLocationRequest = serde_json::from_str(json).unwrap();
        assert!(request.altitude.is_none());
        assert!(request.bearing.is_none());
        assert!(request.speed.is_none());
        assert!(request.provider.is_none());
        assert!(request.battery_level.is_none());
        assert!(request.network_type.is_none());
    }

    #[test]
    fn test_batch_upload_request_serialization() {
        let json = r#"{
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "locations": [
                {
                    "timestamp": 1700000000000,
                    "latitude": 37.7749,
                    "longitude": -122.4194,
                    "accuracy": 10.0
                },
                {
                    "timestamp": 1700000001000,
                    "latitude": 37.7750,
                    "longitude": -122.4195,
                    "accuracy": 15.0
                }
            ]
        }"#;
        let request: BatchUploadRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.locations.len(), 2);
        assert_eq!(request.locations[0].latitude, 37.7749);
        assert_eq!(request.locations[1].latitude, 37.7750);
    }

    #[test]
    fn test_upload_location_response_serialization() {
        let response = UploadLocationResponse {
            success: true,
            processed_count: 5,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"processedCount\":5"));
    }

    #[test]
    fn test_upload_location_response_failed() {
        let response = UploadLocationResponse {
            success: false,
            processed_count: 0,
        };
        assert!(!response.success);
        assert_eq!(response.processed_count, 0);
    }

    #[test]
    fn test_location_data_with_all_fields() {
        let data = LocationData {
            timestamp: 1700000000000,
            latitude: 40.7128,
            longitude: -74.0060,
            accuracy: 5.0,
            altitude: Some(50.0),
            bearing: Some(90.0),
            speed: Some(10.0),
            provider: Some("fused".to_string()),
            battery_level: Some(75),
            network_type: Some("5g".to_string()),
        };
        assert_eq!(data.latitude, 40.7128);
        assert_eq!(data.provider, Some("fused".to_string()));
    }

    #[test]
    fn test_location_data_serialization() {
        let data = LocationData {
            timestamp: 1700000000000,
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
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"latitude\":45"));
        assert!(json.contains("\"longitude\":-120"));
    }
}
