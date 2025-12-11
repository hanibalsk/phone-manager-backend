//! Organization webhook domain models.
//!
//! Request/response DTOs for organization-level webhook management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Maximum webhooks per organization.
pub const MAX_WEBHOOKS_PER_ORG: i64 = 50;

/// Minimum secret length.
pub const MIN_SECRET_LENGTH: usize = 16;

/// Maximum secret length.
pub const MAX_SECRET_LENGTH: usize = 256;

/// Supported organization webhook event types.
pub const SUPPORTED_EVENT_TYPES: &[&str] = &[
    "device.enrolled",
    "device.unenrolled",
    "device.assigned",
    "device.unassigned",
    "member.joined",
    "member.removed",
    "policy.applied",
    "policy.updated",
];

/// Request to create an organization webhook.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOrgWebhookRequest {
    /// Webhook name (1-100 characters).
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    /// Target URL (must be HTTPS).
    #[validate(url, length(max = 2048))]
    pub target_url: String,

    /// Secret for HMAC-SHA256 signature (16-256 characters).
    #[validate(length(min = 16, max = 256))]
    pub secret: String,

    /// Event types to subscribe to.
    #[validate(length(min = 1, message = "At least one event type is required"))]
    pub event_types: Vec<String>,
}

impl CreateOrgWebhookRequest {
    /// Validates that the target URL uses HTTPS.
    pub fn validate_https(&self) -> Result<(), String> {
        if !self.target_url.starts_with("https://") {
            return Err("Target URL must use HTTPS".to_string());
        }
        Ok(())
    }

    /// Validates that all event types are supported.
    pub fn validate_event_types(&self) -> Result<(), String> {
        for event_type in &self.event_types {
            if !SUPPORTED_EVENT_TYPES.contains(&event_type.as_str()) {
                return Err(format!("Unsupported event type: {}", event_type));
            }
        }
        Ok(())
    }
}

/// Request to update an organization webhook.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
pub struct UpdateOrgWebhookRequest {
    /// New webhook name.
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    /// New target URL.
    #[validate(url, length(max = 2048))]
    pub target_url: Option<String>,

    /// New secret.
    #[validate(length(min = 16, max = 256))]
    pub secret: Option<String>,

    /// Enable/disable webhook.
    pub enabled: Option<bool>,

    /// New event types.
    pub event_types: Option<Vec<String>>,
}

impl UpdateOrgWebhookRequest {
    /// Checks if the request has any updates.
    pub fn has_updates(&self) -> bool {
        self.name.is_some()
            || self.target_url.is_some()
            || self.secret.is_some()
            || self.enabled.is_some()
            || self.event_types.is_some()
    }

    /// Validates that the target URL uses HTTPS (if provided).
    pub fn validate_https(&self) -> Result<(), String> {
        if let Some(ref url) = self.target_url {
            if !url.starts_with("https://") {
                return Err("Target URL must use HTTPS".to_string());
            }
        }
        Ok(())
    }

    /// Validates that all event types are supported (if provided).
    pub fn validate_event_types(&self) -> Result<(), String> {
        if let Some(ref event_types) = self.event_types {
            if event_types.is_empty() {
                return Err("At least one event type is required".to_string());
            }
            for event_type in event_types {
                if !SUPPORTED_EVENT_TYPES.contains(&event_type.as_str()) {
                    return Err(format!("Unsupported event type: {}", event_type));
                }
            }
        }
        Ok(())
    }
}

/// Response for an organization webhook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgWebhookResponse {
    /// Webhook UUID.
    pub id: Uuid,

    /// Webhook name.
    pub name: String,

    /// Target URL.
    pub target_url: String,

    /// Whether the webhook is enabled.
    pub enabled: bool,

    /// Subscribed event types.
    pub event_types: Vec<String>,

    /// Consecutive delivery failures.
    pub consecutive_failures: i32,

    /// Circuit breaker open until (if open).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circuit_open_until: Option<DateTime<Utc>>,

    /// When the webhook was created.
    pub created_at: DateTime<Utc>,

    /// When the webhook was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Response for listing organization webhooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOrgWebhooksResponse {
    /// List of webhooks.
    pub webhooks: Vec<OrgWebhookResponse>,
}

/// Request to test an organization webhook.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TestOrgWebhookRequest {
    /// Optional custom event type to test (defaults to "device.enrolled").
    pub event_type: Option<String>,
}

impl TestOrgWebhookRequest {
    /// Validates that the event type is supported (if provided).
    pub fn validate_event_type(&self) -> Result<(), String> {
        if let Some(ref event_type) = self.event_type {
            if !SUPPORTED_EVENT_TYPES.contains(&event_type.as_str()) {
                return Err(format!("Unsupported event type: {}", event_type));
            }
        }
        Ok(())
    }

    /// Get the event type to use for testing.
    pub fn get_event_type(&self) -> &str {
        self.event_type.as_deref().unwrap_or("device.enrolled")
    }
}

/// Response for a test webhook delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOrgWebhookResponse {
    /// Whether the test was successful.
    pub success: bool,

    /// Delivery UUID.
    pub delivery_id: Uuid,

    /// HTTP response status code (if delivery was attempted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_code: Option<i32>,

    /// Error message (if delivery failed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Delivery duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
}

/// Query parameters for listing webhook deliveries.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ListWebhookDeliveriesQuery {
    /// Filter by status (pending, success, failed).
    pub status: Option<String>,

    /// Page number (1-based).
    #[validate(range(min = 1))]
    pub page: Option<u32>,

    /// Items per page.
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u32>,
}

/// Webhook delivery log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDeliveryResponse {
    /// Delivery UUID.
    pub id: Uuid,

    /// Event ID this delivery is for (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<Uuid>,

    /// Event type.
    pub event_type: String,

    /// Delivery status (pending, success, failed).
    pub status: String,

    /// Number of delivery attempts.
    pub attempts: i32,

    /// Last attempt timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_attempt_at: Option<DateTime<Utc>>,

    /// Next retry timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_retry_at: Option<DateTime<Utc>>,

    /// HTTP response code from last attempt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_code: Option<i32>,

    /// Error message from last attempt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// When the delivery was created.
    pub created_at: DateTime<Utc>,
}

/// Response for listing webhook deliveries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWebhookDeliveriesResponse {
    /// List of deliveries.
    pub deliveries: Vec<WebhookDeliveryResponse>,

    /// Pagination info.
    pub pagination: WebhookPagination,
}

/// Pagination info for webhook responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPagination {
    /// Current page number.
    pub page: u32,

    /// Items per page.
    pub per_page: u32,

    /// Total number of items.
    pub total: i64,

    /// Total number of pages.
    pub total_pages: u32,
}

/// Response for retrying a webhook delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryDeliveryResponse {
    /// Whether the retry was queued successfully.
    pub success: bool,

    /// Delivery UUID.
    pub delivery_id: Uuid,

    /// New status of the delivery.
    pub status: String,

    /// Message about the retry.
    pub message: String,
}

/// Webhook delivery statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookStatsResponse {
    /// Webhook UUID.
    pub webhook_id: Uuid,

    /// Total deliveries in the time period.
    pub total_deliveries: i64,

    /// Successful deliveries.
    pub successful_deliveries: i64,

    /// Failed deliveries.
    pub failed_deliveries: i64,

    /// Pending deliveries.
    pub pending_deliveries: i64,

    /// Success rate as a percentage (0-100).
    pub success_rate: f64,

    /// Average response time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_response_time_ms: Option<f64>,

    /// Circuit breaker status.
    pub circuit_breaker_open: bool,

    /// Consecutive failures count.
    pub consecutive_failures: i32,

    /// Time period for these stats (e.g., "24h", "7d").
    pub time_period: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_request_validation() {
        let valid = CreateOrgWebhookRequest {
            name: "Production Events".to_string(),
            target_url: "https://api.example.com/webhooks".to_string(),
            secret: "whsec_testsecretkey123456".to_string(),
            event_types: vec!["device.enrolled".to_string()],
        };
        assert!(valid.validate().is_ok());
        assert!(valid.validate_https().is_ok());
        assert!(valid.validate_event_types().is_ok());
    }

    #[test]
    fn test_create_request_invalid_url() {
        let invalid = CreateOrgWebhookRequest {
            name: "Test".to_string(),
            target_url: "http://insecure.example.com".to_string(),
            secret: "whsec_testsecretkey123456".to_string(),
            event_types: vec!["device.enrolled".to_string()],
        };
        assert!(invalid.validate_https().is_err());
    }

    #[test]
    fn test_create_request_invalid_event_type() {
        let invalid = CreateOrgWebhookRequest {
            name: "Test".to_string(),
            target_url: "https://api.example.com/webhooks".to_string(),
            secret: "whsec_testsecretkey123456".to_string(),
            event_types: vec!["invalid.event.type".to_string()],
        };
        assert!(invalid.validate_event_types().is_err());
    }

    #[test]
    fn test_create_request_short_secret() {
        let invalid = CreateOrgWebhookRequest {
            name: "Test".to_string(),
            target_url: "https://api.example.com/webhooks".to_string(),
            secret: "short".to_string(),
            event_types: vec!["device.enrolled".to_string()],
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_update_request_has_updates() {
        let empty = UpdateOrgWebhookRequest::default();
        assert!(!empty.has_updates());

        let with_name = UpdateOrgWebhookRequest {
            name: Some("New Name".to_string()),
            ..Default::default()
        };
        assert!(with_name.has_updates());

        let with_enabled = UpdateOrgWebhookRequest {
            enabled: Some(false),
            ..Default::default()
        };
        assert!(with_enabled.has_updates());
    }

    #[test]
    fn test_update_request_validate_https() {
        let valid = UpdateOrgWebhookRequest {
            target_url: Some("https://api.example.com".to_string()),
            ..Default::default()
        };
        assert!(valid.validate_https().is_ok());

        let invalid = UpdateOrgWebhookRequest {
            target_url: Some("http://insecure.example.com".to_string()),
            ..Default::default()
        };
        assert!(invalid.validate_https().is_err());

        let none = UpdateOrgWebhookRequest::default();
        assert!(none.validate_https().is_ok());
    }

    #[test]
    fn test_update_request_validate_event_types() {
        let valid = UpdateOrgWebhookRequest {
            event_types: Some(vec![
                "device.enrolled".to_string(),
                "member.joined".to_string(),
            ]),
            ..Default::default()
        };
        assert!(valid.validate_event_types().is_ok());

        let invalid = UpdateOrgWebhookRequest {
            event_types: Some(vec!["invalid.event".to_string()]),
            ..Default::default()
        };
        assert!(invalid.validate_event_types().is_err());

        let empty = UpdateOrgWebhookRequest {
            event_types: Some(vec![]),
            ..Default::default()
        };
        assert!(empty.validate_event_types().is_err());
    }

    #[test]
    fn test_supported_event_types() {
        assert!(SUPPORTED_EVENT_TYPES.contains(&"device.enrolled"));
        assert!(SUPPORTED_EVENT_TYPES.contains(&"member.joined"));
        assert!(SUPPORTED_EVENT_TYPES.contains(&"policy.updated"));
        assert!(!SUPPORTED_EVENT_TYPES.contains(&"invalid.event"));
    }
}
