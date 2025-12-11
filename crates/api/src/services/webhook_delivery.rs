//! Webhook delivery service.
//!
//! Story 15.2: Webhook Event Delivery
//! Story 15.3: Webhook Delivery Logging and Retry
//! Story 15.3 AC 15.3.6: Circuit Breaker
//! Handles asynchronous delivery of webhook notifications to external systems
//! with full delivery logging, retry support, and circuit breaker protection.

use chrono::{Duration as ChronoDuration, Utc};
use hmac::{Hmac, Mac};
use persistence::entities::WebhookDeliveryEntity;
use persistence::repositories::{
    GeofenceEventRepository, WebhookDeliveryRepository, WebhookRepository,
};
use reqwest::Client;
use serde::Serialize;
use sha2::Sha256;
use sqlx::PgPool;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use domain::models::GeofenceTransitionType;

/// Webhook delivery timeout in seconds.
const WEBHOOK_TIMEOUT_SECS: u64 = 5;

/// Number of consecutive failures before circuit breaker opens.
/// After this many failures, the webhook will be temporarily disabled.
pub const CIRCUIT_BREAKER_THRESHOLD: i32 = 5;

/// Cooldown period in seconds when circuit breaker is open.
/// The webhook will be unavailable for this duration after the circuit opens.
pub const CIRCUIT_BREAKER_COOLDOWN_SECS: i64 = 300; // 5 minutes

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
    /// 3. Logs delivery record for each webhook
    /// 4. Signs the payload with HMAC-SHA256
    /// 5. Delivers to each webhook URL
    /// 6. Updates delivery record with result
    /// 7. Updates the event's webhook status
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
        let webhooks = webhook_repo
            .find_enabled_by_owner_device_id(device_id)
            .await?;

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
            location: WebhookLocation {
                latitude,
                longitude,
            },
        };

        let payload_json = serde_json::to_string(&payload)?;
        let payload_value: serde_json::Value = serde_json::from_str(&payload_json)?;

        // Track overall delivery status
        let mut any_success = false;
        let mut last_response_code: Option<i32> = None;

        let delivery_repo = WebhookDeliveryRepository::new(self.pool.clone());

        // Deliver to each webhook
        for webhook in &webhooks {
            // Create delivery record
            let event_type_str = event_type.to_webhook_event_type();
            let delivery = delivery_repo
                .create(
                    webhook.webhook_id,
                    Some(event_id),
                    event_type_str,
                    &payload_value,
                )
                .await?;

            let signature = self.sign_payload(&payload_json, &webhook.secret)?;

            match self
                .deliver_to_webhook(&webhook.target_url, &payload_json, &signature)
                .await
            {
                Ok(status_code) => {
                    let success = (200..300).contains(&(status_code as i32));

                    // Update delivery record
                    delivery_repo
                        .update_attempt(
                            delivery.delivery_id,
                            success,
                            Some(status_code as i32),
                            None,
                        )
                        .await?;

                    if success {
                        info!(
                            webhook_id = %webhook.webhook_id,
                            delivery_id = %delivery.delivery_id,
                            target_url = %webhook.target_url,
                            status_code = status_code,
                            "Webhook delivered successfully"
                        );
                        any_success = true;
                        last_response_code = Some(status_code as i32);

                        // Reset circuit breaker on success
                        if let Err(e) = webhook_repo
                            .reset_consecutive_failures(webhook.webhook_id)
                            .await
                        {
                            warn!(
                                webhook_id = %webhook.webhook_id,
                                error = %e,
                                "Failed to reset consecutive failures"
                            );
                        }
                    } else {
                        warn!(
                            webhook_id = %webhook.webhook_id,
                            delivery_id = %delivery.delivery_id,
                            target_url = %webhook.target_url,
                            status_code = status_code,
                            "Webhook delivery returned non-2xx status"
                        );

                        // Handle circuit breaker on non-2xx response
                        self.handle_delivery_failure(&webhook_repo, webhook.webhook_id)
                            .await;
                    }
                }
                Err(e) => {
                    // Update delivery record with error
                    delivery_repo
                        .update_attempt(delivery.delivery_id, false, None, Some(&e.to_string()))
                        .await?;

                    warn!(
                        webhook_id = %webhook.webhook_id,
                        delivery_id = %delivery.delivery_id,
                        target_url = %webhook.target_url,
                        error = %e,
                        "Webhook delivery failed"
                    );

                    // Handle circuit breaker on error
                    self.handle_delivery_failure(&webhook_repo, webhook.webhook_id)
                        .await;
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

    /// Process pending webhook delivery retries.
    ///
    /// This method:
    /// 1. Finds deliveries that are due for retry
    /// 2. Looks up the webhook configuration
    /// 3. Attempts delivery
    /// 4. Updates delivery status
    pub async fn process_pending_retries(
        &self,
        batch_size: i64,
    ) -> Result<u32, WebhookDeliveryError> {
        let delivery_repo = WebhookDeliveryRepository::new(self.pool.clone());
        let webhook_repo = WebhookRepository::new(self.pool.clone());

        let pending = delivery_repo.find_pending_retries(batch_size).await?;
        let mut processed = 0u32;

        for delivery in pending {
            match self
                .retry_delivery(&delivery, &webhook_repo, &delivery_repo)
                .await
            {
                Ok(_) => {
                    processed += 1;
                }
                Err(e) => {
                    error!(
                        delivery_id = %delivery.delivery_id,
                        error = %e,
                        "Failed to process retry"
                    );
                }
            }
        }

        if processed > 0 {
            info!(processed = processed, "Processed pending webhook retries");
        }

        Ok(processed)
    }

    /// Retry a single delivery.
    async fn retry_delivery(
        &self,
        delivery: &WebhookDeliveryEntity,
        webhook_repo: &WebhookRepository,
        delivery_repo: &WebhookDeliveryRepository,
    ) -> Result<(), WebhookDeliveryError> {
        // Find the webhook
        let webhook = match webhook_repo.find_by_webhook_id(delivery.webhook_id).await? {
            Some(w) => w,
            None => {
                // Webhook was deleted, mark delivery as failed
                delivery_repo
                    .update_attempt(delivery.delivery_id, false, None, Some("Webhook not found"))
                    .await?;
                return Ok(());
            }
        };

        // Check if webhook is still enabled
        if !webhook.enabled {
            delivery_repo
                .update_attempt(delivery.delivery_id, false, None, Some("Webhook disabled"))
                .await?;
            return Ok(());
        }

        // Check if circuit breaker is open - skip retry if still in cooldown
        if let Some(open_until) = webhook.circuit_open_until {
            if open_until > chrono::Utc::now() {
                // Circuit is still open, postpone this delivery by updating next_retry_at
                let postpone_until = open_until + chrono::Duration::seconds(60);
                delivery_repo
                    .postpone_retry(delivery.delivery_id, postpone_until)
                    .await?;

                debug!(
                    delivery_id = %delivery.delivery_id,
                    webhook_id = %webhook.webhook_id,
                    circuit_open_until = %open_until,
                    "Skipping retry - circuit breaker is open, postponing delivery"
                );
                return Ok(());
            }
        }

        let payload_json = serde_json::to_string(&delivery.payload)?;
        let signature = self.sign_payload(&payload_json, &webhook.secret)?;

        match self
            .deliver_to_webhook(&webhook.target_url, &payload_json, &signature)
            .await
        {
            Ok(status_code) => {
                let success = (200..300).contains(&(status_code as i32));
                delivery_repo
                    .update_attempt(
                        delivery.delivery_id,
                        success,
                        Some(status_code as i32),
                        None,
                    )
                    .await?;

                if success {
                    info!(
                        delivery_id = %delivery.delivery_id,
                        webhook_id = %webhook.webhook_id,
                        status_code = status_code,
                        "Retry delivery succeeded"
                    );

                    // Reset circuit breaker on success
                    if let Err(e) = webhook_repo
                        .reset_consecutive_failures(webhook.webhook_id)
                        .await
                    {
                        warn!(
                            webhook_id = %webhook.webhook_id,
                            error = %e,
                            "Failed to reset consecutive failures after retry"
                        );
                    }

                    // Update geofence event webhook status if this delivery is associated with an event
                    if let Some(event_id) = delivery.event_id {
                        let event_repo = GeofenceEventRepository::new(self.pool.clone());
                        if let Err(e) = event_repo
                            .update_webhook_status(event_id, true, Some(status_code as i32))
                            .await
                        {
                            warn!(
                                event_id = %event_id,
                                error = %e,
                                "Failed to update geofence event webhook status after successful retry"
                            );
                        }
                    }
                } else {
                    warn!(
                        delivery_id = %delivery.delivery_id,
                        webhook_id = %webhook.webhook_id,
                        status_code = status_code,
                        "Retry delivery returned non-2xx status"
                    );

                    // Handle circuit breaker on non-2xx response
                    self.handle_delivery_failure(webhook_repo, webhook.webhook_id)
                        .await;
                }
            }
            Err(e) => {
                delivery_repo
                    .update_attempt(delivery.delivery_id, false, None, Some(&e.to_string()))
                    .await?;

                warn!(
                    delivery_id = %delivery.delivery_id,
                    webhook_id = %webhook.webhook_id,
                    error = %e,
                    "Retry delivery failed"
                );

                // Handle circuit breaker on error
                self.handle_delivery_failure(webhook_repo, webhook.webhook_id)
                    .await;
            }
        }

        Ok(())
    }

    /// Clean up old delivery records.
    pub async fn cleanup_old_deliveries(
        &self,
        retention_days: i32,
    ) -> Result<u64, WebhookDeliveryError> {
        let delivery_repo = WebhookDeliveryRepository::new(self.pool.clone());
        let deleted = delivery_repo.delete_old_deliveries(retention_days).await?;

        if deleted > 0 {
            info!(
                deleted = deleted,
                retention_days = retention_days,
                "Cleaned up old webhook deliveries"
            );
        }

        Ok(deleted)
    }

    /// Handle a delivery failure by updating the circuit breaker state.
    ///
    /// This method:
    /// 1. Increments the consecutive failure counter
    /// 2. If threshold is reached, opens the circuit breaker
    async fn handle_delivery_failure(&self, webhook_repo: &WebhookRepository, webhook_id: Uuid) {
        match webhook_repo
            .increment_consecutive_failures(webhook_id)
            .await
        {
            Ok(failure_count) => {
                if failure_count >= CIRCUIT_BREAKER_THRESHOLD {
                    // Open the circuit breaker
                    let open_until =
                        Utc::now() + ChronoDuration::seconds(CIRCUIT_BREAKER_COOLDOWN_SECS);
                    if let Err(e) = webhook_repo.open_circuit(webhook_id, open_until).await {
                        error!(
                            webhook_id = %webhook_id,
                            error = %e,
                            "Failed to open circuit breaker"
                        );
                    } else {
                        warn!(
                            webhook_id = %webhook_id,
                            failure_count = failure_count,
                            open_until = %open_until,
                            "Circuit breaker opened due to consecutive failures"
                        );
                    }
                }
            }
            Err(e) => {
                warn!(
                    webhook_id = %webhook_id,
                    error = %e,
                    "Failed to increment consecutive failures"
                );
            }
        }
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
        let payload =
            r#"{"event_type":"geofence_enter","device_id":"550e8400-e29b-41d4-a716-446655440000"}"#;
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

    #[test]
    fn test_circuit_breaker_threshold_constant() {
        // Circuit breaker opens after 5 consecutive failures
        assert_eq!(CIRCUIT_BREAKER_THRESHOLD, 5);
    }

    #[test]
    fn test_circuit_breaker_cooldown_constant() {
        // Circuit stays open for 5 minutes (300 seconds)
        assert_eq!(CIRCUIT_BREAKER_COOLDOWN_SECS, 300);
    }
}
