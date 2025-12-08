//! Organization API key management routes.
//!
//! Provides endpoints for creating, listing, updating, and revoking
//! organization-scoped API keys. These routes require B2B admin authentication.

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use domain::models::{
    ApiKeyPagination, ApiKeyResponse, CreateApiKeyRequest, CreateApiKeyResponse,
    ListApiKeysQuery, ListApiKeysResponse, UpdateApiKeyRequest, MAX_API_KEYS_PER_ORG,
};
use rand::Rng;
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::api_key::ApiKeyAuth;
use persistence::repositories::{ApiKeyRepository, OrganizationRepository};

/// API key prefix for organization keys.
const API_KEY_PREFIX: &str = "pm_live_";

/// Number of random bytes for API key generation.
const API_KEY_RANDOM_BYTES: usize = 32;

/// Generate a new organization API key.
fn generate_api_key() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..API_KEY_RANDOM_BYTES).map(|_| rng.gen()).collect();
    let encoded = URL_SAFE_NO_PAD.encode(&random_bytes);
    format!("{}{}", API_KEY_PREFIX, encoded)
}

/// Extract the key prefix from a full API key (first 8 chars after prefix).
fn extract_key_prefix(key: &str) -> String {
    // pm_live_ is 8 chars, take next 8 chars
    key.chars().skip(8).take(8).collect()
}

/// POST /api/admin/v1/organizations/:org_id/api-keys
///
/// Create a new organization API key.
/// The full key is returned only once - store it securely.
pub async fn create_api_key(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request.validate().map_err(|e| {
        ApiError::Validation(format!("Validation error: {}", e))
    })?;

    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let api_key_repo = ApiKeyRepository::new(state.pool.clone());

    // Check key limit
    let current_count = api_key_repo.count_by_organization(org_id).await?;
    if current_count >= MAX_API_KEYS_PER_ORG {
        return Err(ApiError::Conflict(format!(
            "Maximum API key limit ({}) reached for this organization",
            MAX_API_KEYS_PER_ORG
        )));
    }

    // Generate new API key
    let raw_key = generate_api_key();
    let key_prefix = extract_key_prefix(&raw_key);
    let key_hash = shared::crypto::sha256_hex(&raw_key);

    // Calculate expiration if provided
    let expires_at = request
        .expires_in_days
        .map(|days| Utc::now() + Duration::days(days as i64));

    // Create the key in database
    let entity = api_key_repo
        .create_for_organization(
            org_id,
            &key_hash,
            &key_prefix,
            &request.name,
            request.description.as_deref(),
            expires_at,
        )
        .await?;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        api_key_id = entity.id,
        key_prefix = %key_prefix,
        "Created organization API key"
    );

    let response = CreateApiKeyResponse {
        id: entity.id,
        key: raw_key,
        key_prefix: format!("pm_live_{}", key_prefix),
        name: entity.name.unwrap_or_default(),
        description: entity.description,
        expires_at: entity.expires_at,
        created_at: entity.created_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/admin/v1/organizations/:org_id/api-keys
///
/// List all API keys for an organization.
/// Keys are returned without the actual key value (only prefix).
pub async fn list_api_keys(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListApiKeysQuery>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let api_key_repo = ApiKeyRepository::new(state.pool.clone());

    // Get all keys for the organization
    let entities = api_key_repo
        .list_by_organization(org_id, query.include_inactive)
        .await?;

    let total = entities.len() as i64;
    let per_page = query.per_page_clamped();
    let page = query.page_clamped();
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

    // Apply pagination
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(entities.len());
    let paginated_entities = if start < entities.len() {
        &entities[start..end]
    } else {
        &[]
    };

    let api_keys: Vec<ApiKeyResponse> = paginated_entities
        .iter()
        .map(|entity| ApiKeyResponse {
            id: entity.id,
            key_prefix: format!("pm_live_{}", entity.key_prefix),
            name: entity.name.clone().unwrap_or_default(),
            description: entity.description.clone(),
            is_active: entity.is_active,
            last_used_at: entity.last_used_at,
            created_at: entity.created_at,
            expires_at: entity.expires_at,
        })
        .collect();

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        total_keys = total,
        "Listed organization API keys"
    );

    Ok(Json(ListApiKeysResponse {
        api_keys,
        pagination: ApiKeyPagination {
            page,
            per_page,
            total,
            total_pages,
        },
    }))
}

/// GET /api/admin/v1/organizations/:org_id/api-keys/:key_id
///
/// Get details for a specific API key.
pub async fn get_api_key(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, key_id)): Path<(Uuid, i64)>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let api_key_repo = ApiKeyRepository::new(state.pool.clone());

    // Find the key
    let entity = api_key_repo
        .find_by_id_and_org(key_id, org_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("API key not found".to_string()))?;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        api_key_id = key_id,
        "Fetched organization API key"
    );

    Ok(Json(ApiKeyResponse {
        id: entity.id,
        key_prefix: format!("pm_live_{}", entity.key_prefix),
        name: entity.name.unwrap_or_default(),
        description: entity.description,
        is_active: entity.is_active,
        last_used_at: entity.last_used_at,
        created_at: entity.created_at,
        expires_at: entity.expires_at,
    }))
}

/// PATCH /api/admin/v1/organizations/:org_id/api-keys/:key_id
///
/// Update an API key's metadata (name and/or description).
pub async fn update_api_key(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, key_id)): Path<(Uuid, i64)>,
    Json(request): Json<UpdateApiKeyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request.validate().map_err(|e| {
        ApiError::Validation(format!("Validation error: {}", e))
    })?;

    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let api_key_repo = ApiKeyRepository::new(state.pool.clone());

    // Update the key
    let entity = api_key_repo
        .update(
            key_id,
            org_id,
            request.name.as_deref(),
            request.description.as_deref(),
        )
        .await?
        .ok_or_else(|| ApiError::NotFound("API key not found".to_string()))?;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        api_key_id = key_id,
        "Updated organization API key"
    );

    Ok(Json(ApiKeyResponse {
        id: entity.id,
        key_prefix: format!("pm_live_{}", entity.key_prefix),
        name: entity.name.unwrap_or_default(),
        description: entity.description,
        is_active: entity.is_active,
        last_used_at: entity.last_used_at,
        created_at: entity.created_at,
        expires_at: entity.expires_at,
    }))
}

/// DELETE /api/admin/v1/organizations/:org_id/api-keys/:key_id
///
/// Revoke an API key (soft delete).
/// The key remains in the database with is_active=false for audit purposes.
pub async fn revoke_api_key(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path((org_id, key_id)): Path<(Uuid, i64)>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify organization exists
    let org_repo = OrganizationRepository::new(state.pool.clone());
    if org_repo.find_by_id(org_id).await?.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    let api_key_repo = ApiKeyRepository::new(state.pool.clone());

    // Revoke the key
    let revoked = api_key_repo.revoke(key_id, org_id).await?;

    if !revoked {
        warn!(
            admin_key_id = auth.api_key_id,
            organization_id = %org_id,
            api_key_id = key_id,
            "Attempted to revoke non-existent or already revoked API key"
        );
        return Err(ApiError::NotFound(
            "API key not found or already revoked".to_string(),
        ));
    }

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        api_key_id = key_id,
        "Revoked organization API key"
    );

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let key = generate_api_key();
        assert!(key.starts_with(API_KEY_PREFIX));
        assert!(key.len() > 20); // Should be reasonably long
    }

    #[test]
    fn test_generate_api_key_uniqueness() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_extract_key_prefix() {
        let key = "pm_live_abcdefgh12345678901234567890";
        let prefix = extract_key_prefix(key);
        assert_eq!(prefix, "abcdefgh");
    }

    #[test]
    fn test_extract_key_prefix_from_generated() {
        let key = generate_api_key();
        let prefix = extract_key_prefix(&key);
        assert_eq!(prefix.len(), 8);
    }

    #[test]
    fn test_create_request_validation() {
        let request = CreateApiKeyRequest {
            name: "Valid Key".to_string(),
            description: Some("A valid description".to_string()),
            expires_in_days: Some(30),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_request_validation_empty_name() {
        let request = CreateApiKeyRequest {
            name: "".to_string(),
            description: None,
            expires_in_days: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_request_validation_expires_too_long() {
        let request = CreateApiKeyRequest {
            name: "Test Key".to_string(),
            description: None,
            expires_in_days: Some(400),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_update_request_validation() {
        let request = UpdateApiKeyRequest {
            name: Some("Updated Name".to_string()),
            description: Some("Updated description".to_string()),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_update_request_validation_name_too_long() {
        let request = UpdateApiKeyRequest {
            name: Some("a".repeat(101)),
            description: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_list_query_defaults() {
        let json = r#"{}"#;
        let query: ListApiKeysQuery = serde_json::from_str(json).unwrap();
        assert!(!query.include_inactive);
        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, 50);
    }

    #[test]
    fn test_list_query_per_page_clamped() {
        let query = ListApiKeysQuery {
            include_inactive: false,
            page: 1,
            per_page: 200,
        };
        assert_eq!(query.per_page_clamped(), 100);
    }
}
