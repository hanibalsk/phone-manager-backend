//! Webhook domain model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Represents a webhook in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Webhook {
    pub id: i64,
    pub webhook_id: Uuid,
    pub owner_device_id: Uuid,
    pub name: String,
    pub target_url: String,
    pub secret: String,
    pub enabled: bool,
    /// Number of consecutive delivery failures since last success
    pub consecutive_failures: i32,
    /// When circuit breaker is open, this is when it will auto-close
    pub circuit_open_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Webhook {
    /// Check if the circuit breaker is currently open.
    pub fn is_circuit_open(&self) -> bool {
        if let Some(open_until) = self.circuit_open_until {
            Utc::now() < open_until
        } else {
            false
        }
    }

    /// Check if webhook is available for delivery (enabled and circuit closed).
    pub fn is_available(&self) -> bool {
        self.enabled && !self.is_circuit_open()
    }
}

/// Default enabled status for new webhooks.
fn default_enabled() -> bool {
    true
}

/// Request payload for creating a webhook.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateWebhookRequest {
    pub owner_device_id: Uuid,

    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,

    #[validate(
        url(message = "Invalid URL format"),
        length(max = 2048, message = "URL must be at most 2048 characters")
    )]
    #[validate(custom(function = "validate_https_url"))]
    pub target_url: String,

    #[validate(length(min = 8, max = 256, message = "Secret must be 8-256 characters"))]
    pub secret: String,

    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// Custom validator for HTTPS URLs.
fn validate_https_url(url: &str) -> Result<(), validator::ValidationError> {
    if url.starts_with("https://") {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("https_required");
        err.message = Some("URL must use HTTPS protocol".into());
        Err(err)
    }
}

/// Request payload for updating a webhook (partial update).
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateWebhookRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: Option<String>,

    #[validate(
        url(message = "Invalid URL format"),
        length(max = 2048, message = "URL must be at most 2048 characters")
    )]
    pub target_url: Option<String>,

    #[validate(length(min = 8, max = 256, message = "Secret must be 8-256 characters"))]
    pub secret: Option<String>,

    pub enabled: Option<bool>,
}

impl UpdateWebhookRequest {
    /// Validate that target_url is HTTPS if provided.
    pub fn validate_https(&self) -> Result<(), String> {
        if let Some(ref url) = self.target_url {
            if !url.starts_with("https://") {
                return Err("URL must use HTTPS protocol".to_string());
            }
        }
        Ok(())
    }
}

/// Response payload for webhook operations.
/// Matches frontend WebhookDto expectations from WebhookModels.kt
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WebhookResponse {
    pub webhook_id: Uuid,
    pub owner_device_id: Uuid,
    pub name: String,
    pub target_url: String,
    pub secret: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Webhook> for WebhookResponse {
    fn from(w: Webhook) -> Self {
        Self {
            webhook_id: w.webhook_id,
            owner_device_id: w.owner_device_id,
            name: w.name,
            target_url: w.target_url,
            secret: w.secret,
            enabled: w.enabled,
            created_at: w.created_at,
            updated_at: w.updated_at,
        }
    }
}

/// Response for listing webhooks.
/// Matches frontend ListWebhooksResponse expectations.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListWebhooksResponse {
    pub webhooks: Vec<WebhookResponse>,
    pub total: i64,
}

/// Query parameters for listing webhooks.
/// Uses ownerDeviceId to match frontend expectations (camelCase query param).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWebhooksQuery {
    pub owner_device_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_response_serialization() {
        let response = WebhookResponse {
            webhook_id: Uuid::new_v4(),
            owner_device_id: Uuid::new_v4(),
            name: "Test Webhook".to_string(),
            target_url: "https://example.com/webhook".to_string(),
            secret: "test-secret-key".to_string(),
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"webhook_id\":"));
        assert!(json.contains("\"owner_device_id\":"));
        assert!(json.contains("\"name\":\"Test Webhook\""));
        assert!(json.contains("\"target_url\":\"https://example.com/webhook\""));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_create_webhook_request_deserialization() {
        let json = r#"{
            "owner_device_id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/api/webhook/abc123",
            "secret": "my-secret-key-12345678"
        }"#;

        let request: CreateWebhookRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Home Assistant");
        assert!(request.target_url.starts_with("https://"));
        // Default should be applied
        assert!(request.enabled);
    }

    #[test]
    fn test_create_webhook_request_with_enabled_false() {
        let json = r#"{
            "owner_device_id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Disabled Webhook",
            "target_url": "https://example.com/webhook",
            "secret": "my-secret-key-12345678",
            "enabled": false
        }"#;

        let request: CreateWebhookRequest = serde_json::from_str(json).unwrap();
        assert!(!request.enabled);
    }

    #[test]
    fn test_update_webhook_request_partial() {
        let json = r#"{
            "name": "Updated Name"
        }"#;

        let request: UpdateWebhookRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, Some("Updated Name".to_string()));
        assert!(request.target_url.is_none());
        assert!(request.secret.is_none());
        assert!(request.enabled.is_none());
    }

    #[test]
    fn test_update_webhook_request_validate_https() {
        let request = UpdateWebhookRequest {
            name: None,
            target_url: Some("http://example.com".to_string()),
            secret: None,
            enabled: None,
        };

        let result = request.validate_https();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "URL must use HTTPS protocol");
    }

    #[test]
    fn test_update_webhook_request_validate_https_valid() {
        let request = UpdateWebhookRequest {
            name: None,
            target_url: Some("https://example.com".to_string()),
            secret: None,
            enabled: None,
        };

        let result = request.validate_https();
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_webhooks_query_deserialization() {
        // Frontend sends camelCase
        let json = r#"{"ownerDeviceId": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let query: ListWebhooksQuery = serde_json::from_str(json).unwrap();
        assert_eq!(
            query.owner_device_id.to_string(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn test_list_webhooks_response_serialization() {
        let response = ListWebhooksResponse {
            webhooks: vec![],
            total: 0,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"webhooks\":[]"));
        assert!(json.contains("\"total\":0"));
    }

    #[test]
    fn test_validate_https_url_valid() {
        assert!(validate_https_url("https://example.com").is_ok());
        assert!(validate_https_url("https://sub.example.com/path").is_ok());
    }

    #[test]
    fn test_validate_https_url_invalid() {
        assert!(validate_https_url("http://example.com").is_err());
        assert!(validate_https_url("ftp://example.com").is_err());
        assert!(validate_https_url("example.com").is_err());
    }

    fn create_test_webhook(enabled: bool, circuit_open_until: Option<DateTime<Utc>>) -> Webhook {
        Webhook {
            id: 1,
            webhook_id: Uuid::new_v4(),
            owner_device_id: Uuid::new_v4(),
            name: "Test Webhook".to_string(),
            target_url: "https://example.com/webhook".to_string(),
            secret: "test-secret-key".to_string(),
            enabled,
            consecutive_failures: 0,
            circuit_open_until,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_webhook_is_circuit_open_none() {
        let webhook = create_test_webhook(true, None);
        assert!(!webhook.is_circuit_open());
    }

    #[test]
    fn test_webhook_is_circuit_open_future() {
        use chrono::Duration;
        let future = Utc::now() + Duration::hours(1);
        let webhook = create_test_webhook(true, Some(future));
        assert!(webhook.is_circuit_open());
    }

    #[test]
    fn test_webhook_is_circuit_open_past() {
        use chrono::Duration;
        let past = Utc::now() - Duration::hours(1);
        let webhook = create_test_webhook(true, Some(past));
        assert!(!webhook.is_circuit_open());
    }

    #[test]
    fn test_webhook_is_available_enabled_circuit_closed() {
        let webhook = create_test_webhook(true, None);
        assert!(webhook.is_available());
    }

    #[test]
    fn test_webhook_is_available_disabled() {
        let webhook = create_test_webhook(false, None);
        assert!(!webhook.is_available());
    }

    #[test]
    fn test_webhook_is_available_circuit_open() {
        use chrono::Duration;
        let future = Utc::now() + Duration::hours(1);
        let webhook = create_test_webhook(true, Some(future));
        assert!(!webhook.is_available());
    }

    #[test]
    fn test_webhook_is_available_disabled_and_circuit_open() {
        use chrono::Duration;
        let future = Utc::now() + Duration::hours(1);
        let webhook = create_test_webhook(false, Some(future));
        assert!(!webhook.is_available());
    }
}
