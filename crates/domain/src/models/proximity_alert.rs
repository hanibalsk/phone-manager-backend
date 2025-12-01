//! Proximity alert domain model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Represents a proximity alert between two devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProximityAlert {
    pub id: i64,
    pub alert_id: Uuid,
    pub source_device_id: Uuid,
    pub target_device_id: Uuid,
    pub name: Option<String>,
    pub radius_meters: i32,
    pub is_active: bool,
    pub is_triggered: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a proximity alert.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProximityAlertRequest {
    pub source_device_id: Uuid,
    pub target_device_id: Uuid,

    #[validate(length(max = 100, message = "Name must be at most 100 characters"))]
    pub name: Option<String>,

    #[validate(range(
        min = 50,
        max = 100000,
        message = "Radius must be between 50 and 100000 meters"
    ))]
    pub radius_meters: i32,

    #[serde(default = "default_active")]
    pub is_active: bool,

    pub metadata: Option<serde_json::Value>,
}

/// Default active status for new proximity alerts.
fn default_active() -> bool {
    true
}

/// Request payload for updating a proximity alert (partial update).
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProximityAlertRequest {
    #[validate(length(max = 100, message = "Name must be at most 100 characters"))]
    pub name: Option<String>,

    #[validate(range(
        min = 50,
        max = 100000,
        message = "Radius must be between 50 and 100000 meters"
    ))]
    pub radius_meters: Option<i32>,

    pub is_active: Option<bool>,

    pub metadata: Option<serde_json::Value>,
}

/// Response payload for proximity alert operations.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProximityAlertResponse {
    pub alert_id: Uuid,
    pub source_device_id: Uuid,
    pub target_device_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub radius_meters: i32,
    pub is_active: bool,
    pub is_triggered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_triggered_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProximityAlert> for ProximityAlertResponse {
    fn from(a: ProximityAlert) -> Self {
        Self {
            alert_id: a.alert_id,
            source_device_id: a.source_device_id,
            target_device_id: a.target_device_id,
            name: a.name,
            radius_meters: a.radius_meters,
            is_active: a.is_active,
            is_triggered: a.is_triggered,
            last_triggered_at: a.last_triggered_at,
            metadata: a.metadata,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

/// Response for listing proximity alerts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProximityAlertsResponse {
    pub alerts: Vec<ProximityAlertResponse>,
    pub total: usize,
}

/// Query parameters for listing proximity alerts.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProximityAlertsQuery {
    /// Filter by source device ID
    pub source_device_id: Uuid,
    /// Include inactive alerts (default: false)
    #[serde(default)]
    pub include_inactive: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proximity_alert_request_deserialization() {
        let json = r#"{
            "sourceDeviceId": "550e8400-e29b-41d4-a716-446655440000",
            "targetDeviceId": "550e8400-e29b-41d4-a716-446655440001",
            "radiusMeters": 500
        }"#;

        let request: CreateProximityAlertRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.radius_meters, 500);
        assert!(request.is_active); // default
        assert!(request.name.is_none());
    }

    #[test]
    fn test_create_proximity_alert_request_with_all_fields() {
        let json = r#"{
            "sourceDeviceId": "550e8400-e29b-41d4-a716-446655440000",
            "targetDeviceId": "550e8400-e29b-41d4-a716-446655440001",
            "name": "Near Mom",
            "radiusMeters": 1000,
            "isActive": false,
            "metadata": {"color": "red"}
        }"#;

        let request: CreateProximityAlertRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, Some("Near Mom".to_string()));
        assert_eq!(request.radius_meters, 1000);
        assert!(!request.is_active);
        assert!(request.metadata.is_some());
    }

    #[test]
    fn test_update_proximity_alert_request_partial() {
        let json = r#"{
            "name": "Updated Name"
        }"#;

        let request: UpdateProximityAlertRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, Some("Updated Name".to_string()));
        assert!(request.radius_meters.is_none());
        assert!(request.is_active.is_none());
    }

    #[test]
    fn test_proximity_alert_response_serialization() {
        let response = ProximityAlertResponse {
            alert_id: Uuid::new_v4(),
            source_device_id: Uuid::new_v4(),
            target_device_id: Uuid::new_v4(),
            name: Some("Test Alert".to_string()),
            radius_meters: 500,
            is_active: true,
            is_triggered: false,
            last_triggered_at: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"name\":\"Test Alert\""));
        assert!(json.contains("\"radiusMeters\":500"));
        assert!(json.contains("\"isActive\":true"));
        assert!(json.contains("\"isTriggered\":false"));
        // Should skip None fields
        assert!(!json.contains("\"lastTriggeredAt\":null"));
        assert!(!json.contains("\"metadata\":null"));
    }

    #[test]
    fn test_list_proximity_alerts_query_defaults() {
        let json = r#"{"sourceDeviceId": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let query: ListProximityAlertsQuery = serde_json::from_str(json).unwrap();
        assert!(!query.include_inactive);
    }
}
