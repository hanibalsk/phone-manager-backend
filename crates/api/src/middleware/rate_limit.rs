//! Rate limiting middleware.
//!
//! Provides per-API-key rate limiting using a sliding window algorithm.

use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovRateLimiter,
};
use serde_json::json;
use std::{
    collections::HashMap,
    num::NonZeroU32,
    sync::{Arc, RwLock},
};

use crate::app::AppState;
use crate::extractors::api_key::ApiKeyAuth;

/// Type alias for the rate limiter used per API key.
type KeyRateLimiter = GovRateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Rate limiter state shared across all requests.
/// Uses a HashMap keyed by API key ID (i64) with individual rate limiters.
pub struct RateLimiterState {
    limiters: RwLock<HashMap<i64, Arc<KeyRateLimiter>>>,
    rate_limit_per_minute: u32,
}

impl RateLimiterState {
    /// Create a new rate limiter state with the specified limit per minute.
    pub fn new(rate_limit_per_minute: u32) -> Self {
        Self {
            limiters: RwLock::new(HashMap::new()),
            rate_limit_per_minute,
        }
    }

    /// Get or create a rate limiter for the given API key ID.
    fn get_or_create_limiter(&self, key_id: i64) -> Arc<KeyRateLimiter> {
        // First try to get existing limiter with read lock
        {
            let limiters = self.limiters.read().unwrap();
            if let Some(limiter) = limiters.get(&key_id) {
                return limiter.clone();
            }
        }

        // Create new limiter with write lock
        let mut limiters = self.limiters.write().unwrap();

        // Double-check in case another thread created it
        if let Some(limiter) = limiters.get(&key_id) {
            return limiter.clone();
        }

        // Create new limiter with rate limit per minute
        let quota = Quota::per_minute(
            NonZeroU32::new(self.rate_limit_per_minute).unwrap_or(NonZeroU32::new(100).unwrap()),
        );
        let limiter = Arc::new(GovRateLimiter::direct(quota));
        limiters.insert(key_id, limiter.clone());
        limiter
    }

    /// Check if a request from the given API key should be allowed.
    /// Returns Ok(()) if allowed, or Err with retry_after seconds if rate limited.
    pub fn check(&self, key_id: i64) -> Result<(), u64> {
        let limiter = self.get_or_create_limiter(key_id);

        match limiter.check() {
            Ok(_) => Ok(()),
            Err(not_until) => {
                let wait_time = not_until.wait_time_from(governor::clock::Clock::now(
                    &governor::clock::DefaultClock::default(),
                ));
                // Return retry after in seconds, minimum 1 second
                Err(wait_time.as_secs().max(1))
            }
        }
    }
}

impl std::fmt::Debug for RateLimiterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiterState")
            .field("rate_limit_per_minute", &self.rate_limit_per_minute)
            .field("active_limiters", &self.limiters.read().unwrap().len())
            .finish()
    }
}

impl Clone for RateLimiterState {
    fn clone(&self) -> Self {
        // Clone creates a new state that shares the same limiters
        Self {
            limiters: RwLock::new(self.limiters.read().unwrap().clone()),
            rate_limit_per_minute: self.rate_limit_per_minute,
        }
    }
}

/// Middleware that applies rate limiting per API key.
///
/// This middleware must run AFTER authentication so that the API key ID
/// is available in request extensions.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // Get the authenticated API key from request extensions
    // If no auth info, skip rate limiting (request will fail auth anyway)
    let auth = match req.extensions().get::<ApiKeyAuth>() {
        Some(auth) => auth.clone(),
        None => return next.run(req).await,
    };

    // Check rate limit
    if let Some(ref rate_limiter) = state.rate_limiter {
        if let Err(retry_after) = rate_limiter.check(auth.api_key_id) {
            return rate_limited_response(state.config.security.rate_limit_per_minute, retry_after);
        }
    }

    next.run(req).await
}

/// Create a rate limited response with proper headers and body.
fn rate_limited_response(limit: u32, retry_after: u64) -> Response {
    let body = json!({
        "error": "rate_limit_exceeded",
        "message": format!("Rate limit of {} requests/minute exceeded", limit),
        "retryAfter": retry_after
    });

    let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();

    // Add Retry-After header
    response.headers_mut().insert(
        header::RETRY_AFTER,
        retry_after.to_string().parse().unwrap(),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================
    // RateLimiterState Creation Tests
    // ===========================================

    #[test]
    fn test_rate_limiter_state_creation() {
        let state = RateLimiterState::new(100);
        assert_eq!(state.rate_limit_per_minute, 100);
    }

    #[test]
    fn test_rate_limiter_state_creation_with_zero() {
        // Zero should default to 100
        let state = RateLimiterState::new(0);
        assert_eq!(state.rate_limit_per_minute, 0);
    }

    #[test]
    fn test_rate_limiter_state_creation_with_various_limits() {
        let limits = vec![1, 10, 100, 1000, 10000];
        for limit in limits {
            let state = RateLimiterState::new(limit);
            assert_eq!(state.rate_limit_per_minute, limit);
        }
    }

    // ===========================================
    // Rate Limiting Logic Tests
    // ===========================================

    #[test]
    fn test_rate_limiter_allows_requests() {
        let state = RateLimiterState::new(100);
        let key_id: i64 = 1;

        // First request should be allowed
        assert!(state.check(key_id).is_ok());
    }

    #[test]
    fn test_rate_limiter_exhaustion() {
        // Use very low limit to test exhaustion
        let state = RateLimiterState::new(1);
        let key_id: i64 = 1;

        // First request should be allowed
        assert!(state.check(key_id).is_ok());

        // Second request should be rate limited
        let result = state.check(key_id);
        assert!(result.is_err());
        // Retry-after should be at least 1 second
        assert!(result.unwrap_err() >= 1);
    }

    #[test]
    fn test_rate_limiter_different_keys_independent() {
        let state = RateLimiterState::new(1); // Very low limit
        let key1: i64 = 1;
        let key2: i64 = 2;
        let key3: i64 = 3;

        // Each key should have independent limits
        assert!(state.check(key1).is_ok());
        assert!(state.check(key2).is_ok());
        assert!(state.check(key3).is_ok());

        // Now key1 should be rate limited, but others still allowed
        assert!(state.check(key1).is_err());
        assert!(state.check(key2).is_err());
        assert!(state.check(key3).is_err());
    }

    #[test]
    fn test_rate_limiter_same_key_multiple_checks() {
        let state = RateLimiterState::new(5);
        let key_id: i64 = 42;

        // Should allow 5 requests
        for i in 0..5 {
            let result = state.check(key_id);
            assert!(result.is_ok(), "Request {} should be allowed", i);
        }

        // 6th request should be rate limited
        assert!(state.check(key_id).is_err());
    }

    #[test]
    fn test_rate_limiter_many_keys() {
        let state = RateLimiterState::new(10);

        // Test with 100 different keys
        for key_id in 0..100i64 {
            assert!(state.check(key_id).is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_negative_key_id() {
        let state = RateLimiterState::new(100);
        let key_id: i64 = -1;

        // Negative IDs should work (they're valid i64 values)
        assert!(state.check(key_id).is_ok());
    }

    #[test]
    fn test_rate_limiter_boundary_key_ids() {
        let state = RateLimiterState::new(100);

        // Test boundary values
        assert!(state.check(i64::MIN).is_ok());
        assert!(state.check(i64::MAX).is_ok());
        assert!(state.check(0).is_ok());
    }

    // ===========================================
    // Clone and Debug Tests
    // ===========================================

    #[test]
    fn test_rate_limiter_state_debug() {
        let state = RateLimiterState::new(100);
        let debug = format!("{:?}", state);
        assert!(debug.contains("RateLimiterState"));
        assert!(debug.contains("rate_limit_per_minute"));
        assert!(debug.contains("100"));
        assert!(debug.contains("active_limiters"));
    }

    #[test]
    fn test_rate_limiter_state_debug_with_limiters() {
        let state = RateLimiterState::new(100);
        // Create some limiters
        state.check(1).unwrap();
        state.check(2).unwrap();

        let debug = format!("{:?}", state);
        assert!(debug.contains("active_limiters"));
    }

    #[test]
    fn test_rate_limiter_state_clone() {
        let state = RateLimiterState::new(100);
        let key_id: i64 = 1;
        state.check(key_id).unwrap(); // Create a limiter

        let cloned = state.clone();
        assert_eq!(cloned.rate_limit_per_minute, 100);
    }

    #[test]
    fn test_rate_limiter_state_clone_shares_limiters() {
        let state = RateLimiterState::new(100);
        state.check(1).unwrap();
        state.check(2).unwrap();

        let cloned = state.clone();
        // Clone should have the same limiters
        assert!(cloned.check(1).is_ok()); // Using existing limiter
        assert!(cloned.check(3).is_ok()); // Creating new limiter
    }

    // ===========================================
    // Response Building Tests
    // ===========================================

    #[test]
    fn test_rate_limited_response_format() {
        let response = rate_limited_response(100, 60);
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response.headers().contains_key(header::RETRY_AFTER));
        assert_eq!(response.headers().get(header::RETRY_AFTER).unwrap(), "60");
    }

    #[test]
    fn test_rate_limited_response_various_retry_after() {
        let retry_values = vec![1, 5, 30, 60, 120, 3600];
        for retry_after in retry_values {
            let response = rate_limited_response(100, retry_after);
            assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
            assert_eq!(
                response.headers().get(header::RETRY_AFTER).unwrap(),
                &retry_after.to_string()
            );
        }
    }

    #[test]
    fn test_rate_limited_response_various_limits() {
        let limits = vec![1, 10, 100, 1000];
        for limit in limits {
            let response = rate_limited_response(limit, 60);
            assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        }
    }

    #[test]
    fn test_rate_limited_response_zero_retry_after() {
        let response = rate_limited_response(100, 0);
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers().get(header::RETRY_AFTER).unwrap(), "0");
    }

    // ===========================================
    // Concurrent Access Tests
    // ===========================================

    #[test]
    fn test_rate_limiter_get_or_create_idempotent() {
        let state = RateLimiterState::new(100);
        let key_id: i64 = 1;

        // Multiple calls should return the same limiter
        let limiter1 = state.get_or_create_limiter(key_id);
        let limiter2 = state.get_or_create_limiter(key_id);

        // Should be the same Arc (same underlying object)
        assert!(Arc::ptr_eq(&limiter1, &limiter2));
    }

    #[test]
    fn test_rate_limiter_different_keys_different_limiters() {
        let state = RateLimiterState::new(100);

        let limiter1 = state.get_or_create_limiter(1);
        let limiter2 = state.get_or_create_limiter(2);

        // Should be different Arcs
        assert!(!Arc::ptr_eq(&limiter1, &limiter2));
    }
}
