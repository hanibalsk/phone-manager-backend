//! Organization webhook routes.
//!
//! Provides endpoints for managing organization-level webhooks.

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use domain::models::{
    CreateOrgWebhookRequest, ListOrgWebhooksResponse, ListWebhookDeliveriesQuery,
    ListWebhookDeliveriesResponse, OrgWebhookResponse, RetryDeliveryResponse,
    TestOrgWebhookRequest, TestOrgWebhookResponse, UpdateOrgWebhookRequest,
    WebhookDeliveryResponse, WebhookPagination, WebhookStatsResponse, MAX_WEBHOOKS_PER_ORG,
};
use hmac::{Hmac, Mac};
use persistence::entities::{OrgWebhookEntity, WebhookDeliveryEntity};
use persistence::repositories::{
    OrgWebhookRepository, OrganizationRepository, WebhookDeliveryRepository,
};
use reqwest::Client;
use serde_json::json;
use sha2::Sha256;
use std::time::{Duration, Instant};
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::api_key::ApiKeyAuth;

/// POST /api/admin/v1/organizations/:org_id/webhooks
///
/// Create a new organization webhook.
pub async fn create_webhook(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
    Json(request): Json<CreateOrgWebhookRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    // Validate HTTPS
    request.validate_https().map_err(ApiError::Validation)?;

    // Validate event types
    request
        .validate_event_types()
        .map_err(ApiError::Validation)?;

    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let webhook_repo = OrgWebhookRepository::new(state.pool.clone());

    // Check webhook limit
    let count = webhook_repo.count_by_organization(org_id).await?;
    if count >= MAX_WEBHOOKS_PER_ORG {
        return Err(ApiError::Conflict(format!(
            "Maximum webhook limit ({}) reached for this organization",
            MAX_WEBHOOKS_PER_ORG
        )));
    }

    // Check if name already exists
    if webhook_repo.name_exists(org_id, &request.name).await? {
        return Err(ApiError::Conflict(
            "A webhook with this name already exists in this organization".to_string(),
        ));
    }

    // Create the webhook
    let entity = webhook_repo
        .create(
            org_id,
            &request.name,
            &request.target_url,
            &request.secret,
            &request.event_types,
        )
        .await?;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        webhook_id = %entity.webhook_id,
        name = %request.name,
        "Created organization webhook"
    );

    let response = entity_to_response(entity);

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/admin/v1/organizations/:org_id/webhooks
///
/// List all webhooks for an organization.
pub async fn list_webhooks(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let webhook_repo = OrgWebhookRepository::new(state.pool.clone());
    let entities = webhook_repo.list_by_organization(org_id).await?;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        webhook_count = entities.len(),
        "Listed organization webhooks"
    );

    let webhooks: Vec<OrgWebhookResponse> = entities.into_iter().map(entity_to_response).collect();

    Ok(Json(ListOrgWebhooksResponse { webhooks }))
}

/// GET /api/admin/v1/organizations/:org_id/webhooks/:webhook_id
///
/// Get details for a specific webhook.
pub async fn get_webhook(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, webhook_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let webhook_repo = OrgWebhookRepository::new(state.pool.clone());

    let entity = webhook_repo
        .find_by_id(webhook_id, org_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        webhook_id = %webhook_id,
        "Fetched organization webhook"
    );

    Ok(Json(entity_to_response(entity)))
}

/// PUT /api/admin/v1/organizations/:org_id/webhooks/:webhook_id
///
/// Update a webhook.
pub async fn update_webhook(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, webhook_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateOrgWebhookRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if there are any updates
    if !request.has_updates() {
        return Err(ApiError::Validation("No updates provided".to_string()));
    }

    // Validate request fields
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    // Validate HTTPS if provided
    request.validate_https().map_err(ApiError::Validation)?;

    // Validate event types if provided
    request
        .validate_event_types()
        .map_err(ApiError::Validation)?;

    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let webhook_repo = OrgWebhookRepository::new(state.pool.clone());

    // Check if webhook exists
    if webhook_repo.find_by_id(webhook_id, org_id).await?.is_none() {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    // Check if new name conflicts with existing webhook
    if let Some(ref new_name) = request.name {
        // Get existing webhooks to check for name conflict
        let existing = webhook_repo.list_by_organization(org_id).await?;
        for wh in existing {
            if wh.webhook_id != webhook_id && wh.name.to_lowercase() == new_name.to_lowercase() {
                return Err(ApiError::Conflict(
                    "A webhook with this name already exists in this organization".to_string(),
                ));
            }
        }
    }

    // Update the webhook
    let entity = webhook_repo
        .update(
            webhook_id,
            org_id,
            request.name.as_deref(),
            request.target_url.as_deref(),
            request.secret.as_deref(),
            request.enabled,
            request.event_types.as_deref(),
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        webhook_id = %webhook_id,
        "Updated organization webhook"
    );

    Ok(Json(entity_to_response(entity)))
}

/// DELETE /api/admin/v1/organizations/:org_id/webhooks/:webhook_id
///
/// Delete a webhook.
pub async fn delete_webhook(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, webhook_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let webhook_repo = OrgWebhookRepository::new(state.pool.clone());

    let deleted = webhook_repo.delete(webhook_id, org_id).await?;

    if !deleted {
        warn!(
            admin_key_id = auth.api_key_id,
            organization_id = %org_id,
            webhook_id = %webhook_id,
            "Attempted to delete non-existent webhook"
        );
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        webhook_id = %webhook_id,
        "Deleted organization webhook"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Webhook delivery timeout in seconds for test delivery.
const TEST_WEBHOOK_TIMEOUT_SECS: u64 = 10;

/// POST /api/admin/v1/organizations/:org_id/webhooks/:webhook_id/test
///
/// Test a webhook by sending a test event.
pub async fn test_webhook(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, webhook_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<TestOrgWebhookRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate event type if provided
    request
        .validate_event_type()
        .map_err(ApiError::Validation)?;

    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let webhook_repo = OrgWebhookRepository::new(state.pool.clone());
    let delivery_repo = WebhookDeliveryRepository::new(state.pool.clone());

    // Get the webhook
    let webhook = webhook_repo
        .find_by_id(webhook_id, org_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    // Create test payload
    let event_type = request.get_event_type();
    let test_payload = json!({
        "event_type": event_type,
        "test": true,
        "timestamp": Utc::now().timestamp_millis(),
        "organization_id": org_id.to_string(),
        "message": "This is a test webhook delivery"
    });

    // Create delivery record
    let delivery = delivery_repo
        .create(webhook_id, None, event_type, &test_payload)
        .await?;

    // Sign the payload
    let payload_json = serde_json::to_string(&test_payload)
        .map_err(|e| ApiError::Internal(format!("Failed to serialize payload: {}", e)))?;

    let signature = sign_payload(&payload_json, &webhook.secret)?;

    // Create HTTP client with timeout
    let client = Client::builder()
        .timeout(Duration::from_secs(TEST_WEBHOOK_TIMEOUT_SECS))
        .build()
        .map_err(|e| ApiError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    // Deliver the test webhook
    let start_time = Instant::now();
    let result = client
        .post(&webhook.target_url)
        .header("Content-Type", "application/json")
        .header("X-Webhook-Signature", &signature)
        .header("X-Webhook-Test", "true")
        .body(payload_json.clone())
        .send()
        .await;

    let duration_ms = start_time.elapsed().as_millis() as i64;

    // Update delivery record with result
    let (success, response_code, error) = match result {
        Ok(response) => {
            let status = response.status().as_u16() as i32;
            let is_success = (200..300).contains(&status);
            delivery_repo
                .update_attempt(delivery.delivery_id, is_success, Some(status), None)
                .await?;
            (is_success, Some(status), None)
        }
        Err(e) => {
            let error_msg = e.to_string();
            delivery_repo
                .update_attempt(delivery.delivery_id, false, None, Some(&error_msg))
                .await?;
            (false, None, Some(error_msg))
        }
    };

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        webhook_id = %webhook_id,
        delivery_id = %delivery.delivery_id,
        success = success,
        duration_ms = duration_ms,
        "Tested organization webhook"
    );

    let response = TestOrgWebhookResponse {
        success,
        delivery_id: delivery.delivery_id,
        response_code,
        error,
        duration_ms: Some(duration_ms),
    };

    Ok(Json(response))
}

/// GET /api/admin/v1/organizations/:org_id/webhooks/:webhook_id/deliveries
///
/// Get delivery logs for a webhook.
pub async fn list_deliveries(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, webhook_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<ListWebhookDeliveriesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate query
    query
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let webhook_repo = OrgWebhookRepository::new(state.pool.clone());
    let delivery_repo = WebhookDeliveryRepository::new(state.pool.clone());

    // Verify webhook exists and belongs to organization
    if webhook_repo.find_by_id(webhook_id, org_id).await?.is_none() {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    // Get pagination params
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let offset = ((page - 1) * per_page) as i64;

    // Get deliveries
    let deliveries = delivery_repo
        .list_by_webhook_id(webhook_id, query.status.as_deref(), per_page as i64, offset)
        .await?;

    // Get total count
    let total = delivery_repo
        .count_by_webhook_id(webhook_id, query.status.as_deref())
        .await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        webhook_id = %webhook_id,
        delivery_count = deliveries.len(),
        "Listed webhook deliveries"
    );

    let response = ListWebhookDeliveriesResponse {
        deliveries: deliveries.into_iter().map(delivery_to_response).collect(),
        pagination: WebhookPagination {
            page,
            per_page,
            total,
            total_pages,
        },
    };

    Ok(Json(response))
}

/// POST /api/admin/v1/organizations/:org_id/webhooks/:webhook_id/deliveries/:delivery_id/retry
///
/// Retry a failed webhook delivery.
pub async fn retry_delivery(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, webhook_id, delivery_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let webhook_repo = OrgWebhookRepository::new(state.pool.clone());
    let delivery_repo = WebhookDeliveryRepository::new(state.pool.clone());

    // Verify webhook exists and belongs to organization
    if webhook_repo.find_by_id(webhook_id, org_id).await?.is_none() {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    // Get the delivery
    let delivery = delivery_repo
        .find_by_delivery_id(delivery_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Delivery not found".to_string()))?;

    // Verify delivery belongs to the webhook
    if delivery.webhook_id != webhook_id {
        return Err(ApiError::NotFound("Delivery not found".to_string()));
    }

    // Check if delivery can be retried (only failed deliveries can be retried)
    if delivery.status != "failed" {
        return Err(ApiError::Conflict(format!(
            "Only failed deliveries can be retried. Current status: {}",
            delivery.status
        )));
    }

    // Reset the delivery for retry
    let updated = delivery_repo
        .reset_for_retry(delivery_id)
        .await?
        .ok_or_else(|| ApiError::Internal("Failed to reset delivery for retry".to_string()))?;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        webhook_id = %webhook_id,
        delivery_id = %delivery_id,
        "Queued webhook delivery for retry"
    );

    let response = RetryDeliveryResponse {
        success: true,
        delivery_id: updated.delivery_id,
        status: updated.status,
        message: "Delivery has been queued for retry".to_string(),
    };

    Ok(Json(response))
}

/// GET /api/admin/v1/organizations/:org_id/webhooks/:webhook_id/stats
///
/// Get delivery statistics for a webhook.
pub async fn get_webhook_stats(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, webhook_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let webhook_repo = OrgWebhookRepository::new(state.pool.clone());
    let delivery_repo = WebhookDeliveryRepository::new(state.pool.clone());

    // Get the webhook
    let webhook = webhook_repo
        .find_by_id(webhook_id, org_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".to_string()))?;

    // Get delivery stats for last 24 hours
    let stats = delivery_repo.get_webhook_stats(webhook_id, 24).await?;

    let total = stats.total_count.unwrap_or(0);
    let successful = stats.success_count.unwrap_or(0);
    let failed = stats.failed_count.unwrap_or(0);
    let pending = stats.pending_count.unwrap_or(0);

    let success_rate = if total > 0 {
        (successful as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    // Check if circuit breaker is open
    let circuit_breaker_open = webhook
        .circuit_open_until
        .map(|until| until > Utc::now())
        .unwrap_or(false);

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        webhook_id = %webhook_id,
        total_deliveries = total,
        success_rate = success_rate,
        "Fetched webhook statistics"
    );

    let response = WebhookStatsResponse {
        webhook_id,
        total_deliveries: total,
        successful_deliveries: successful,
        failed_deliveries: failed,
        pending_deliveries: pending,
        success_rate,
        avg_response_time_ms: None, // Not tracked currently
        circuit_breaker_open,
        consecutive_failures: webhook.consecutive_failures,
        time_period: "24h".to_string(),
    };

    Ok(Json(response))
}

/// Sign the payload with HMAC-SHA256.
fn sign_payload(payload: &str, secret: &str) -> Result<String, ApiError> {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| ApiError::Internal(format!("Failed to create HMAC: {}", e)))?;

    mac.update(payload.as_bytes());
    let result = mac.finalize();
    let signature = hex::encode(result.into_bytes());

    Ok(format!("sha256={}", signature))
}

/// Convert delivery entity to response.
fn delivery_to_response(entity: WebhookDeliveryEntity) -> WebhookDeliveryResponse {
    WebhookDeliveryResponse {
        id: entity.delivery_id,
        event_id: entity.event_id,
        event_type: entity.event_type,
        status: entity.status,
        attempts: entity.attempts,
        last_attempt_at: entity.last_attempt_at,
        next_retry_at: entity.next_retry_at,
        response_code: entity.response_code,
        error_message: entity.error_message,
        created_at: entity.created_at,
    }
}

/// Convert entity to response (excludes secret for security).
fn entity_to_response(entity: OrgWebhookEntity) -> OrgWebhookResponse {
    OrgWebhookResponse {
        id: entity.webhook_id,
        name: entity.name,
        target_url: entity.target_url,
        enabled: entity.enabled,
        event_types: entity.event_types,
        consecutive_failures: entity.consecutive_failures,
        circuit_open_until: entity.circuit_open_until,
        created_at: entity.created_at,
        updated_at: entity.updated_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::org_webhook::{CreateOrgWebhookRequest, UpdateOrgWebhookRequest};

    #[test]
    fn test_create_webhook_request_validation() {
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
    fn test_update_webhook_request_validation() {
        let valid = UpdateOrgWebhookRequest {
            name: Some("Updated Name".to_string()),
            target_url: Some("https://api.example.com/v2/webhooks".to_string()),
            secret: None,
            enabled: Some(false),
            event_types: Some(vec!["member.joined".to_string()]),
        };
        assert!(valid.validate().is_ok());
        assert!(valid.validate_https().is_ok());
        assert!(valid.validate_event_types().is_ok());
        assert!(valid.has_updates());
    }
}
