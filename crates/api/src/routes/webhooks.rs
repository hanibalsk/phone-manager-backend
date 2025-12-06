//! Webhook endpoint handlers.
//!
//! Story 15.1: Webhook Registration and Management API
//! Provides CRUD operations for webhook management aligned with
//! frontend mobile app expectations (phone-manager/WebhookApiService.kt).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use persistence::repositories::{DeviceRepository, WebhookRepository};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::webhook::{
    CreateWebhookRequest, ListWebhooksQuery, ListWebhooksResponse, UpdateWebhookRequest,
    WebhookResponse,
};

/// Maximum number of webhooks allowed per device.
/// Configurable via PM__LIMITS__MAX_WEBHOOKS_PER_DEVICE
const DEFAULT_MAX_WEBHOOKS_PER_DEVICE: i64 = 10;

/// Create a new webhook.
///
/// POST /api/v1/webhooks
///
/// AC 15.1.2: Creates webhook with validation
pub async fn create_webhook(
    State(state): State<AppState>,
    Json(request): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<WebhookResponse>), ApiError> {
    // Validate request
    request.validate().map_err(|e| {
        let errors: Vec<String> = e
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |err| {
                    format!("{}: {}", field, err.message.as_ref().unwrap_or(&"".into()))
                })
            })
            .collect();
        ApiError::Validation(errors.join(", "))
    })?;

    // Verify device exists and is active
    let device_repo = DeviceRepository::new(state.pool.clone());
    let device = device_repo
        .find_by_device_id(request.owner_device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    if !device.active {
        return Err(ApiError::NotFound("Device not found".to_string()));
    }

    let webhook_repo = WebhookRepository::new(state.pool.clone());

    // Check webhook limit per device (AC 15.1.2.7)
    let max_webhooks = state
        .config
        .limits
        .max_webhooks_per_device
        .unwrap_or(DEFAULT_MAX_WEBHOOKS_PER_DEVICE as u32) as i64;
    let count = webhook_repo
        .count_by_owner_device_id(request.owner_device_id)
        .await?;
    if count >= max_webhooks {
        return Err(ApiError::Conflict(format!(
            "Device has reached maximum webhook limit ({})",
            max_webhooks
        )));
    }

    // Check name uniqueness (AC 15.1.2.6)
    let existing = webhook_repo
        .find_by_device_and_name(request.owner_device_id, &request.name)
        .await?;
    if existing.is_some() {
        return Err(ApiError::Conflict(
            "A webhook with this name already exists for this device".to_string(),
        ));
    }

    // Create webhook
    let entity = webhook_repo
        .create(
            request.owner_device_id,
            &request.name,
            &request.target_url,
            &request.secret,
            request.enabled,
        )
        .await?;

    let webhook: domain::models::Webhook = entity.into();
    let response: WebhookResponse = webhook.into();

    info!(
        webhook_id = %response.webhook_id,
        owner_device_id = %response.owner_device_id,
        name = %response.name,
        "Webhook created"
    );

    Ok((StatusCode::CREATED, Json(response)))
}

/// List webhooks for a device.
///
/// GET /api/v1/webhooks?ownerDeviceId=<uuid>
///
/// AC 15.1.3: Returns webhooks for device
pub async fn list_webhooks(
    State(state): State<AppState>,
    Query(query): Query<ListWebhooksQuery>,
) -> Result<Json<ListWebhooksResponse>, ApiError> {
    let webhook_repo = WebhookRepository::new(state.pool.clone());
    let entities = webhook_repo
        .find_by_owner_device_id(query.owner_device_id)
        .await?;

    let webhooks: Vec<WebhookResponse> = entities
        .into_iter()
        .map(|e| {
            let w: domain::models::Webhook = e.into();
            w.into()
        })
        .collect();

    let total = webhooks.len() as i64;

    Ok(Json(ListWebhooksResponse { webhooks, total }))
}

/// Get a single webhook by ID.
///
/// GET /api/v1/webhooks/:webhook_id
///
/// AC 15.1.4: Returns single webhook
pub async fn get_webhook(
    State(state): State<AppState>,
    Path(webhook_id): Path<Uuid>,
) -> Result<Json<WebhookResponse>, ApiError> {
    let webhook_repo = WebhookRepository::new(state.pool.clone());
    let entity = webhook_repo
        .find_by_webhook_id(webhook_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    let webhook: domain::models::Webhook = entity.into();
    Ok(Json(webhook.into()))
}

/// Update a webhook (partial update).
///
/// PUT /api/v1/webhooks/:webhook_id
///
/// AC 15.1.5: Updates webhook fields
pub async fn update_webhook(
    State(state): State<AppState>,
    Path(webhook_id): Path<Uuid>,
    Json(request): Json<UpdateWebhookRequest>,
) -> Result<Json<WebhookResponse>, ApiError> {
    // Validate request
    request.validate().map_err(|e| {
        let errors: Vec<String> = e
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |err| {
                    format!("{}: {}", field, err.message.as_ref().unwrap_or(&"".into()))
                })
            })
            .collect();
        ApiError::Validation(errors.join(", "))
    })?;

    // Validate HTTPS if target_url is provided
    request.validate_https().map_err(ApiError::Validation)?;

    let webhook_repo = WebhookRepository::new(state.pool.clone());

    // Check if webhook exists first (to get device ID for name uniqueness check)
    let existing = webhook_repo
        .find_by_webhook_id(webhook_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    // Check name uniqueness if name is being changed (AC 15.1.5.6)
    if let Some(ref new_name) = request.name {
        if new_name != &existing.name {
            let conflict = webhook_repo
                .find_by_device_and_name(existing.owner_device_id, new_name)
                .await?;
            if conflict.is_some() {
                return Err(ApiError::Conflict(
                    "A webhook with this name already exists for this device".to_string(),
                ));
            }
        }
    }

    let entity = webhook_repo
        .update(
            webhook_id,
            request.name.as_deref(),
            request.target_url.as_deref(),
            request.secret.as_deref(),
            request.enabled,
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    let webhook: domain::models::Webhook = entity.into();
    let response: WebhookResponse = webhook.into();

    info!(webhook_id = %response.webhook_id, "Webhook updated");

    Ok(Json(response))
}

/// Delete a webhook.
///
/// DELETE /api/v1/webhooks/:webhook_id
///
/// AC 15.1.6: Removes webhook (hard delete)
pub async fn delete_webhook(
    State(state): State<AppState>,
    Path(webhook_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let webhook_repo = WebhookRepository::new(state.pool.clone());
    let rows_affected = webhook_repo.delete(webhook_id).await?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    info!(webhook_id = %webhook_id, "Webhook deleted");
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_webhook_request_deserialization() {
        let json = r#"{
            "owner_device_id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/api/webhook/test",
            "secret": "my-secret-key-12345678"
        }"#;

        let request: CreateWebhookRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Home Assistant");
        assert!(request.target_url.starts_with("https://"));
        assert!(request.enabled); // default
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
    fn test_webhook_response_serialization() {
        let response = WebhookResponse {
            webhook_id: Uuid::new_v4(),
            owner_device_id: Uuid::new_v4(),
            name: "Test".to_string(),
            target_url: "https://example.com/webhook".to_string(),
            secret: "test-secret".to_string(),
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"enabled\":true"));
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
}
