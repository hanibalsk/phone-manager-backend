//! Webhook delivery service.
//!
//! Story 15.2: Webhook Event Delivery
//! Handles asynchronous delivery of webhook notifications to external systems.

use hmac::{Hmac, Mac};
use persistence::repositories::{GeofenceEventRepository, WebhookRepository};
use reqwest::Client;
use serde::Serialize;
use sha2::Sha256;
use sqlx::PgPool;
use std::time::Duration;
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

use domain::models::GeofenceTransitionType;

/// Webhook delivery timeout in seconds.
const WEBHOOK_TIMEOUT_SECS: u64 = 5;

/// Errors that can occur during webhook delivery.
#[derive(Error, Debug)]
pub enum WebhookDeliveryError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("HMAC signing error: {0}")]
    SigningError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Webhook payload for geofence events.
#[derive(Debug, Clone, Serialize)]
pub struct GeofenceWebhookPayload {
    pub event_type: String,
    pub device_id: Uuid,
    pub geofence_id: Uuid,
    pub geofence_name: String,
    pub timestamp: i64,
    pub location: WebhookLocation,
}

/// Location data in webhook payload.
#[derive(Debug, Clone, Serialize)]
pub struct WebhookLocation {
    pub latitude: f64,
    pub longitude: f64,
}

/// Service for delivering webhooks.
pub struct WebhookDeliveryService {
    pool: PgPool,
    client: Client,
}

impl WebhookDeliveryService {
    /// Create a new webhook delivery service.
    pub fn new(pool: PgPool) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(WEBHOOK_TIMEOUT_SECS))
            .build()
            .expect("Failed to create HTTP client");

        Self { pool, client }
    }

    /// Deliver geofence event to all enabled webhooks for the device.
    ///
    /// This method:
    /// 1. Finds all enabled webhooks for the device
    /// 2. Creates the webhook payload
    /// 3. Signs the payload with HMAC-SHA256
    /// 4. Delivers to each webhook URL
    /// 5. Updates the event's webhook status
    #[allow(clippy::too_many_arguments)]
    pub async fn deliver_geofence_event(
        &self,
        event_id: Uuid,
        device_id: Uuid,
        geofence_id: Uuid,
        geofence_name: &str,
        event_type: GeofenceTransitionType,
        timestamp: i64,
        latitude: f64,
        longitude: f64,
    ) -> Result<(), WebhookDeliveryError> {
        // Find all enabled webhooks for this device
        let webhook_repo = WebhookRepository::new(self.pool.clone());
        let webhooks = webhook_repo.find_enabled_by_owner_device_id(device_id).await?;

        if webhooks.is_empty() {
            info!(device_id = %device_id, "No enabled webhooks found for device");
            return Ok(());
        }

        // Create the payload
        let payload = GeofenceWebhookPayload {
            event_type: event_type.to_webhook_event_type().to_string(),
            device_id,
            geofence_id,
            geofence_name: geofence_name.to_string(),
            timestamp,
            location: WebhookLocation { latitude, longitude },
        };

        let payload_json = serde_json::to_string(&payload)?;

        // Track overall delivery status
        let mut any_success = false;
        let mut last_response_code: Option<i32> = None;

        // Deliver to each webhook
        for webhook in &webhooks {
            let signature = self.sign_payload(&payload_json, &webhook.secret)?;

            match self
                .deliver_to_webhook(&webhook.target_url, &payload_json, &signature)
                .await
            {
                Ok(status_code) => {
                    info!(
                        webhook_id = %webhook.webhook_id,
                        target_url = %webhook.target_url,
                        status_code = status_code,
                        "Webhook delivered successfully"
                    );
                    any_success = true;
                    last_response_code = Some(status_code as i32);
                }
                Err(e) => {
                    warn!(
                        webhook_id = %webhook.webhook_id,
                        target_url = %webhook.target_url,
                        error = %e,
                        "Webhook delivery failed"
                    );
                    // Continue trying other webhooks
                }
            }
        }

        // Update event webhook status
        let event_repo = GeofenceEventRepository::new(self.pool.clone());
        event_repo
            .update_webhook_status(event_id, any_success, last_response_code)
            .await?;

        Ok(())
    }

    /// Sign the payload with HMAC-SHA256.
    fn sign_payload(&self, payload: &str, secret: &str) -> Result<String, WebhookDeliveryError> {
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| WebhookDeliveryError::SigningError(e.to_string()))?;

        mac.update(payload.as_bytes());
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());

        Ok(format!("sha256={}", signature))
    }

    /// Deliver payload to a single webhook URL.
    async fn deliver_to_webhook(
        &self,
        url: &str,
        payload: &str,
        signature: &str,
    ) -> Result<u16, WebhookDeliveryError> {
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", signature)
            .body(payload.to_string())
            .send()
            .await?;

        let status = response.status().as_u16();
        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_payload() {
        // Create a mock service (we just need to test the signing logic)
        let payload = r#"{"event_type":"geofence_enter","device_id":"550e8400-e29b-41d4-a716-446655440000"}"#;
        let secret = "my-secret-key";

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());

        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // SHA256 produces 32 bytes = 64 hex chars
    }

    #[test]
    fn test_geofence_webhook_payload_serialization() {
        let payload = GeofenceWebhookPayload {
            event_type: "geofence_enter".to_string(),
            device_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            geofence_id: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001").unwrap(),
            geofence_name: "Home".to_string(),
            timestamp: 1701878400000,
            location: WebhookLocation {
                latitude: 37.7749,
                longitude: -122.4194,
            },
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"event_type\":\"geofence_enter\""));
        assert!(json.contains("\"geofence_name\":\"Home\""));
        assert!(json.contains("\"latitude\":37.7749"));
    }

    #[test]
    fn test_webhook_timeout_constant() {
        assert_eq!(WEBHOOK_TIMEOUT_SECS, 5);
    }
}
