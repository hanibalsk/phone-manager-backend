//! Apple Sign-In JWT verification with JWKS.
//!
//! This module handles verification of Apple ID tokens by fetching and caching
//! Apple's public keys from their JWKS endpoint.

use chrono::Utc;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Apple's JWKS endpoint
const APPLE_JWKS_URL: &str = "https://appleid.apple.com/auth/keys";

/// Cache TTL in seconds (1 hour)
const CACHE_TTL_SECS: i64 = 3600;

/// Error type for Apple authentication operations.
#[derive(Debug, thiserror::Error)]
pub enum AppleAuthError {
    #[error("Failed to fetch Apple keys: {0}")]
    KeyFetchError(String),

    #[error("Invalid token format")]
    InvalidTokenFormat,

    #[error("Key not found for kid: {0}")]
    KeyNotFound(String),

    #[error("Invalid token signature")]
    InvalidSignature,

    #[error("Token validation failed: {0}")]
    ValidationError(String),

    #[error("Invalid issuer")]
    InvalidIssuer,

    #[error("Invalid audience")]
    InvalidAudience,

    #[error("Token expired")]
    TokenExpired,

    #[error("Missing email claim")]
    MissingEmail,
}

/// Apple JWKS response structure.
#[derive(Debug, Clone, Deserialize)]
struct AppleJwks {
    keys: Vec<AppleJwk>,
}

/// Individual JWK from Apple's key set.
#[derive(Debug, Clone, Deserialize)]
struct AppleJwk {
    /// Key type (should be "RSA")
    #[allow(dead_code)]
    kty: String,
    /// Key ID
    kid: String,
    /// Algorithm (should be "RS256")
    #[allow(dead_code)]
    alg: String,
    /// RSA modulus (base64url encoded)
    n: String,
    /// RSA exponent (base64url encoded)
    e: String,
}

/// Apple ID token claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleIdTokenClaims {
    /// Issuer (should be "https://appleid.apple.com")
    pub iss: String,
    /// Subject (Apple user ID)
    pub sub: String,
    /// Audience (client ID)
    pub aud: String,
    /// Expiration time
    pub exp: i64,
    /// Issued at
    pub iat: i64,
    /// User's email (optional on subsequent sign-ins)
    pub email: Option<String>,
    /// Whether email is verified
    pub email_verified: Option<bool>,
}

/// Cached JWKS with timestamp.
struct CachedJwks {
    keys: AppleJwks,
    fetched_at: i64,
}

/// Apple authentication client with JWKS caching.
pub struct AppleAuthClient {
    http_client: reqwest::Client,
    cache: Arc<RwLock<Option<CachedJwks>>>,
    expected_client_id: Option<String>,
}

impl AppleAuthClient {
    /// Creates a new Apple auth client.
    pub fn new(expected_client_id: Option<String>) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(None)),
            expected_client_id,
        }
    }

    /// Verifies an Apple ID token and returns the claims.
    pub async fn verify_token(&self, id_token: &str) -> Result<AppleIdTokenClaims, AppleAuthError> {
        // Decode the header to get the kid
        let header = decode_header(id_token).map_err(|_| AppleAuthError::InvalidTokenFormat)?;

        let kid = header.kid.ok_or(AppleAuthError::InvalidTokenFormat)?;

        // Get the JWK for this kid
        let jwk = self.get_jwk(&kid).await?;

        // Create decoding key from JWK
        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|e| AppleAuthError::ValidationError(format!("Invalid key: {}", e)))?;

        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.set_issuer(&["https://appleid.apple.com"]);

        // Set audience if configured
        if let Some(ref client_id) = self.expected_client_id {
            validation.set_audience(&[client_id]);
        } else {
            // Skip audience validation if not configured (but log warning)
            validation.validate_aud = false;
            tracing::warn!("Apple OAuth client ID not configured - audience validation skipped");
        }

        // Decode and validate the token
        let token_data = decode::<AppleIdTokenClaims>(id_token, &decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppleAuthError::TokenExpired,
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    AppleAuthError::InvalidSignature
                }
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => AppleAuthError::InvalidIssuer,
                jsonwebtoken::errors::ErrorKind::InvalidAudience => AppleAuthError::InvalidAudience,
                _ => AppleAuthError::ValidationError(e.to_string()),
            })?;

        Ok(token_data.claims)
    }

    /// Gets a JWK by kid, fetching fresh keys if needed.
    async fn get_jwk(&self, kid: &str) -> Result<AppleJwk, AppleAuthError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(ref cached) = *cache {
                let now = Utc::now().timestamp();
                if now - cached.fetched_at < CACHE_TTL_SECS {
                    if let Some(jwk) = cached.keys.keys.iter().find(|k| k.kid == kid) {
                        return Ok(jwk.clone());
                    }
                }
            }
        }

        // Fetch fresh keys
        let jwks = self.fetch_jwks().await?;

        // Find the key
        let jwk = jwks
            .keys
            .iter()
            .find(|k| k.kid == kid)
            .cloned()
            .ok_or_else(|| AppleAuthError::KeyNotFound(kid.to_string()))?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedJwks {
                keys: jwks,
                fetched_at: Utc::now().timestamp(),
            });
        }

        Ok(jwk)
    }

    /// Fetches Apple's JWKS from their endpoint.
    async fn fetch_jwks(&self) -> Result<AppleJwks, AppleAuthError> {
        let response = self
            .http_client
            .get(APPLE_JWKS_URL)
            .send()
            .await
            .map_err(|e| AppleAuthError::KeyFetchError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AppleAuthError::KeyFetchError(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let jwks: AppleJwks = response
            .json()
            .await
            .map_err(|e| AppleAuthError::KeyFetchError(e.to_string()))?;

        Ok(jwks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_auth_error_display() {
        assert!(format!("{}", AppleAuthError::InvalidSignature).contains("signature"));
        assert!(format!("{}", AppleAuthError::TokenExpired).contains("expired"));
        assert!(format!("{}", AppleAuthError::KeyNotFound("abc".to_string())).contains("abc"));
    }

    #[test]
    fn test_apple_auth_client_creation() {
        let client = AppleAuthClient::new(Some("com.example.app".to_string()));
        assert!(client.expected_client_id.is_some());

        let client = AppleAuthClient::new(None);
        assert!(client.expected_client_id.is_none());
    }
}
