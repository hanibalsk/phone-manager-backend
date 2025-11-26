//! Idempotency key header extractor.

use axum::{
    extract::FromRequestParts,
    http::{header::HeaderName, request::Parts, StatusCode},
};
use shared::crypto::sha256_hex;

/// The header name for idempotency keys.
pub const IDEMPOTENCY_KEY_HEADER: &str = "idempotency-key";

/// Optional idempotency key extracted from request headers.
///
/// The key is hashed using SHA-256 for storage.
#[derive(Debug, Clone)]
pub struct IdempotencyKey {
    /// The original key value from the header.
    pub original: String,
    /// SHA-256 hash of the key for database storage.
    pub hash: String,
}

impl IdempotencyKey {
    /// Create a new IdempotencyKey from the original value.
    pub fn new(original: String) -> Self {
        let hash = sha256_hex(&original);
        Self { original, hash }
    }
}

/// Optional idempotency key extractor.
/// Returns `None` if the header is not present.
#[derive(Debug, Clone)]
pub struct OptionalIdempotencyKey(pub Option<IdempotencyKey>);

impl<S> FromRequestParts<S> for OptionalIdempotencyKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        _state: &'life1 S,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<Self, Self::Rejection>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let header_name = HeaderName::from_static(IDEMPOTENCY_KEY_HEADER);

            let key = parts
                .headers
                .get(&header_name)
                .and_then(|value| value.to_str().ok())
                .filter(|s| !s.is_empty())
                .map(|s| IdempotencyKey::new(s.to_string()));

            Ok(OptionalIdempotencyKey(key))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idempotency_key_creation() {
        let key = IdempotencyKey::new("test-key-123".to_string());
        assert_eq!(key.original, "test-key-123");
        assert!(!key.hash.is_empty());
        assert_eq!(key.hash.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_idempotency_key_hash_deterministic() {
        let key1 = IdempotencyKey::new("test-key".to_string());
        let key2 = IdempotencyKey::new("test-key".to_string());
        assert_eq!(key1.hash, key2.hash);
    }

    #[test]
    fn test_idempotency_key_hash_different_for_different_keys() {
        let key1 = IdempotencyKey::new("key-1".to_string());
        let key2 = IdempotencyKey::new("key-2".to_string());
        assert_ne!(key1.hash, key2.hash);
    }

    #[test]
    fn test_idempotency_key_debug() {
        let key = IdempotencyKey::new("test".to_string());
        let debug_str = format!("{:?}", key);
        assert!(debug_str.contains("IdempotencyKey"));
    }

    #[test]
    fn test_idempotency_key_clone() {
        let key = IdempotencyKey::new("test".to_string());
        let cloned = key.clone();
        assert_eq!(cloned.original, key.original);
        assert_eq!(cloned.hash, key.hash);
    }

    #[test]
    fn test_optional_idempotency_key_debug() {
        let opt = OptionalIdempotencyKey(Some(IdempotencyKey::new("test".to_string())));
        let debug_str = format!("{:?}", opt);
        assert!(debug_str.contains("OptionalIdempotencyKey"));
    }

    #[test]
    fn test_optional_idempotency_key_none() {
        let opt = OptionalIdempotencyKey(None);
        assert!(opt.0.is_none());
    }

    #[test]
    fn test_header_constant() {
        assert_eq!(IDEMPOTENCY_KEY_HEADER, "idempotency-key");
    }
}
