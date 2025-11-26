//! API key authentication extractor.
//!
//! Provides an Axum extractor for validating API keys from requests.

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use sqlx::PgPool;

use crate::app::AppState;
use crate::error::ApiError;
use persistence::repositories::ApiKeyRepository;
use shared::crypto::sha256_hex;

/// Authenticated API key information.
///
/// This extractor validates the `X-API-Key` header against the database
/// and provides access to the authenticated key's details.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used in future stories (rate limiting, admin routes)
pub struct ApiKeyAuth {
    /// Database ID of the authenticated API key.
    pub api_key_id: i64,
    /// Key prefix for identification (e.g., "pm_aBcDe").
    pub key_prefix: String,
    /// Whether this is an admin API key.
    pub is_admin: bool,
}

impl ApiKeyAuth {
    /// Validates an API key and returns authentication info.
    ///
    /// This is the core authentication logic, extracted for testability.
    pub async fn validate(pool: &PgPool, api_key: &str) -> Result<Self, ApiError> {
        // Validate minimum key length (pm_ prefix + 8 chars minimum)
        if api_key.len() < 11 || !api_key.starts_with("pm_") {
            return Err(ApiError::Unauthorized(
                "Invalid or missing API key".to_string(),
            ));
        }

        // Hash the key
        let key_hash = sha256_hex(api_key);

        // Look up the key in the database
        let repo = ApiKeyRepository::new(pool.clone());
        let key = repo
            .find_by_key_hash(&key_hash)
            .await
            .map_err(|e| {
                tracing::error!("Database error during API key lookup: {}", e);
                ApiError::Internal("Authentication service unavailable".to_string())
            })?
            .ok_or_else(|| ApiError::Unauthorized("Invalid or missing API key".to_string()))?;

        // Check if key is valid (active and not expired)
        if !ApiKeyRepository::is_key_valid(&key) {
            if !key.is_active {
                return Err(ApiError::Unauthorized(
                    "Invalid or missing API key".to_string(),
                ));
            } else {
                // Key is expired
                return Err(ApiError::Unauthorized("API key has expired".to_string()));
            }
        }

        // Update last_used_at asynchronously (fire and forget)
        let pool_clone = pool.clone();
        let key_id = key.id;
        tokio::spawn(async move {
            let repo = ApiKeyRepository::new(pool_clone);
            if let Err(e) = repo.update_last_used(key_id).await {
                tracing::warn!("Failed to update API key last_used_at: {}", e);
            }
        });

        Ok(ApiKeyAuth {
            api_key_id: key.id,
            key_prefix: key.key_prefix,
            is_admin: key.is_admin,
        })
    }
}

#[async_trait]
impl FromRequestParts<AppState> for ApiKeyAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract API key from X-API-Key header
        let api_key = parts
            .headers
            .get("X-API-Key")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("Invalid or missing API key".to_string()))?;

        Self::validate(&state.pool, api_key).await
    }
}

/// Optional API key authentication.
///
/// This extractor allows routes to optionally check for authentication
/// without rejecting unauthenticated requests.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used in future stories (optional auth routes)
pub struct OptionalApiKeyAuth(pub Option<ApiKeyAuth>);

#[async_trait]
impl FromRequestParts<AppState> for OptionalApiKeyAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Try to extract API key
        let api_key = parts.headers.get("X-API-Key").and_then(|v| v.to_str().ok());

        match api_key {
            Some(key) => {
                // Validate the key if present
                match ApiKeyAuth::validate(&state.pool, key).await {
                    Ok(auth) => Ok(OptionalApiKeyAuth(Some(auth))),
                    Err(_) => Ok(OptionalApiKeyAuth(None)),
                }
            }
            None => Ok(OptionalApiKeyAuth(None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_auth_struct() {
        let auth = ApiKeyAuth {
            api_key_id: 1,
            key_prefix: "pm_aBcDe".to_string(),
            is_admin: false,
        };
        assert_eq!(auth.api_key_id, 1);
        assert_eq!(auth.key_prefix, "pm_aBcDe");
        assert!(!auth.is_admin);
    }

    #[test]
    fn test_api_key_auth_admin() {
        let auth = ApiKeyAuth {
            api_key_id: 42,
            key_prefix: "pm_Admin".to_string(),
            is_admin: true,
        };
        assert_eq!(auth.api_key_id, 42);
        assert!(auth.is_admin);
    }

    #[test]
    fn test_api_key_auth_clone() {
        let auth = ApiKeyAuth {
            api_key_id: 1,
            key_prefix: "pm_test1".to_string(),
            is_admin: false,
        };
        let cloned = auth.clone();
        assert_eq!(cloned.api_key_id, auth.api_key_id);
        assert_eq!(cloned.key_prefix, auth.key_prefix);
        assert_eq!(cloned.is_admin, auth.is_admin);
    }

    #[test]
    fn test_api_key_auth_debug() {
        let auth = ApiKeyAuth {
            api_key_id: 1,
            key_prefix: "pm_debug".to_string(),
            is_admin: false,
        };
        let debug_str = format!("{:?}", auth);
        assert!(debug_str.contains("api_key_id"));
        assert!(debug_str.contains("pm_debug"));
    }

    #[test]
    fn test_optional_api_key_auth_some() {
        let auth = ApiKeyAuth {
            api_key_id: 1,
            key_prefix: "pm_aBcDe".to_string(),
            is_admin: true,
        };
        let optional = OptionalApiKeyAuth(Some(auth));
        assert!(optional.0.is_some());
        assert!(optional.0.unwrap().is_admin);
    }

    #[test]
    fn test_optional_api_key_auth_none() {
        let optional = OptionalApiKeyAuth(None);
        assert!(optional.0.is_none());
    }

    #[test]
    fn test_optional_api_key_auth_clone() {
        let auth = ApiKeyAuth {
            api_key_id: 5,
            key_prefix: "pm_clone".to_string(),
            is_admin: false,
        };
        let optional = OptionalApiKeyAuth(Some(auth));
        let cloned = optional.clone();
        assert!(cloned.0.is_some());
        assert_eq!(cloned.0.unwrap().api_key_id, 5);
    }

    #[test]
    fn test_optional_api_key_auth_debug() {
        let optional = OptionalApiKeyAuth(None);
        let debug_str = format!("{:?}", optional);
        assert!(debug_str.contains("OptionalApiKeyAuth"));
    }
}
