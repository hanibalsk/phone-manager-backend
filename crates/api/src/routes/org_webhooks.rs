//! Organization webhook routes.
//!
//! Provides endpoints for managing organization-level webhooks.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use domain::models::{
    CreateOrgWebhookRequest, ListOrgWebhooksResponse, OrgWebhookResponse, UpdateOrgWebhookRequest,
    MAX_WEBHOOKS_PER_ORG,
};
use persistence::entities::OrgWebhookEntity;
use persistence::repositories::{OrgWebhookRepository, OrganizationRepository};
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
    request.validate().map_err(|e| {
        ApiError::Validation(format!("Validation error: {}", e))
    })?;

    // Validate HTTPS
    request.validate_https().map_err(ApiError::Validation)?;

    // Validate event types
    request.validate_event_types().map_err(ApiError::Validation)?;

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
    request.validate().map_err(|e| {
        ApiError::Validation(format!("Validation error: {}", e))
    })?;

    // Validate HTTPS if provided
    request.validate_https().map_err(ApiError::Validation)?;

    // Validate event types if provided
    request.validate_event_types().map_err(ApiError::Validation)?;

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
