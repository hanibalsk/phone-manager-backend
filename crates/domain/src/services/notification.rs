//! Notification service for push notifications.
//!
//! Provides abstractions for sending push notifications to devices.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Notification type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    SettingsChanged,
    UnlockRequestResponse,
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::SettingsChanged => write!(f, "settings_changed"),
            NotificationType::UnlockRequestResponse => write!(f, "unlock_request_response"),
        }
    }
}

/// Action type for setting changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingChangeAction {
    Updated,
    Locked,
    Unlocked,
}

impl std::fmt::Display for SettingChangeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingChangeAction::Updated => write!(f, "updated"),
            SettingChangeAction::Locked => write!(f, "locked"),
            SettingChangeAction::Unlocked => write!(f, "unlocked"),
        }
    }
}

/// A single setting change for notification payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingChangeNotification {
    pub key: String,
    pub action: SettingChangeAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<serde_json::Value>,
}

/// Notification payload for settings changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsChangedPayload {
    #[serde(rename = "type")]
    pub notification_type: NotificationType,
    pub device_id: Uuid,
    pub changes: Vec<SettingChangeNotification>,
    pub changed_by: String,
    pub timestamp: DateTime<Utc>,
}

/// Notification payload for unlock request response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockRequestResponsePayload {
    #[serde(rename = "type")]
    pub notification_type: NotificationType,
    pub request_id: Uuid,
    pub setting_key: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub decided_by: String,
    pub timestamp: DateTime<Utc>,
}

/// Generic notification payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NotificationPayload {
    SettingsChanged(SettingsChangedPayload),
    UnlockRequestResponse(UnlockRequestResponsePayload),
}

/// Result of a notification send attempt.
#[derive(Debug, Clone)]
pub enum NotificationResult {
    /// Notification was sent successfully.
    Sent,
    /// Device has no FCM token registered.
    NoToken,
    /// Notification sending failed (but was non-blocking).
    Failed(String),
    /// Notification was skipped (e.g., notify_user=false).
    Skipped,
}

/// Notification service trait for sending push notifications.
#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    /// Send a settings changed notification to a device.
    async fn send_settings_changed(
        &self,
        fcm_token: &str,
        payload: SettingsChangedPayload,
    ) -> NotificationResult;

    /// Send an unlock request response notification to a device.
    async fn send_unlock_request_response(
        &self,
        fcm_token: &str,
        payload: UnlockRequestResponsePayload,
    ) -> NotificationResult;
}

/// Mock notification service for development and testing.
///
/// Logs notifications but doesn't actually send them.
#[derive(Debug, Clone, Default)]
pub struct MockNotificationService {
    /// Whether to simulate failures for testing.
    pub simulate_failure: bool,
}

impl MockNotificationService {
    /// Create a new mock notification service.
    pub fn new() -> Self {
        Self {
            simulate_failure: false,
        }
    }

    /// Create a mock service that simulates failures.
    pub fn failing() -> Self {
        Self {
            simulate_failure: true,
        }
    }
}

#[async_trait::async_trait]
impl NotificationService for MockNotificationService {
    async fn send_settings_changed(
        &self,
        fcm_token: &str,
        payload: SettingsChangedPayload,
    ) -> NotificationResult {
        if self.simulate_failure {
            tracing::warn!(
                fcm_token = %fcm_token,
                device_id = %payload.device_id,
                "Mock notification service simulating failure"
            );
            return NotificationResult::Failed("Simulated failure".to_string());
        }

        tracing::info!(
            fcm_token = %fcm_token,
            device_id = %payload.device_id,
            changes_count = %payload.changes.len(),
            changed_by = %payload.changed_by,
            "Mock: Would send settings_changed notification"
        );

        NotificationResult::Sent
    }

    async fn send_unlock_request_response(
        &self,
        fcm_token: &str,
        payload: UnlockRequestResponsePayload,
    ) -> NotificationResult {
        if self.simulate_failure {
            tracing::warn!(
                fcm_token = %fcm_token,
                request_id = %payload.request_id,
                "Mock notification service simulating failure"
            );
            return NotificationResult::Failed("Simulated failure".to_string());
        }

        tracing::info!(
            fcm_token = %fcm_token,
            request_id = %payload.request_id,
            setting_key = %payload.setting_key,
            status = %payload.status,
            decided_by = %payload.decided_by,
            "Mock: Would send unlock_request_response notification"
        );

        NotificationResult::Sent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_type_display() {
        assert_eq!(
            NotificationType::SettingsChanged.to_string(),
            "settings_changed"
        );
        assert_eq!(
            NotificationType::UnlockRequestResponse.to_string(),
            "unlock_request_response"
        );
    }

    #[test]
    fn test_setting_change_action_display() {
        assert_eq!(SettingChangeAction::Updated.to_string(), "updated");
        assert_eq!(SettingChangeAction::Locked.to_string(), "locked");
        assert_eq!(SettingChangeAction::Unlocked.to_string(), "unlocked");
    }

    #[test]
    fn test_settings_changed_payload_serialization() {
        let payload = SettingsChangedPayload {
            notification_type: NotificationType::SettingsChanged,
            device_id: Uuid::nil(),
            changes: vec![SettingChangeNotification {
                key: "tracking_interval_minutes".to_string(),
                action: SettingChangeAction::Locked,
                new_value: Some(serde_json::json!(5)),
            }],
            changed_by: "Admin User".to_string(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("settings_changed"));
        assert!(json.contains("tracking_interval_minutes"));
        assert!(json.contains("locked"));
    }

    #[test]
    fn test_unlock_request_response_payload_serialization() {
        let payload = UnlockRequestResponsePayload {
            notification_type: NotificationType::UnlockRequestResponse,
            request_id: Uuid::nil(),
            setting_key: "tracking_interval_minutes".to_string(),
            status: "approved".to_string(),
            note: Some("Approved for battery saving".to_string()),
            decided_by: "Admin User".to_string(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("unlock_request_response"));
        assert!(json.contains("approved"));
    }

    #[tokio::test]
    async fn test_mock_notification_service_send() {
        let service = MockNotificationService::new();

        let payload = SettingsChangedPayload {
            notification_type: NotificationType::SettingsChanged,
            device_id: Uuid::nil(),
            changes: vec![],
            changed_by: "Test".to_string(),
            timestamp: Utc::now(),
        };

        let result = service.send_settings_changed("token123", payload).await;
        assert!(matches!(result, NotificationResult::Sent));
    }

    #[tokio::test]
    async fn test_mock_notification_service_failure() {
        let service = MockNotificationService::failing();

        let payload = SettingsChangedPayload {
            notification_type: NotificationType::SettingsChanged,
            device_id: Uuid::nil(),
            changes: vec![],
            changed_by: "Test".to_string(),
            timestamp: Utc::now(),
        };

        let result = service.send_settings_changed("token123", payload).await;
        assert!(matches!(result, NotificationResult::Failed(_)));
    }
}
