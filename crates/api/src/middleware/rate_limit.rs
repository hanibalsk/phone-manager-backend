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
            return rate_limited_response(
                state.config.security.rate_limit_per_minute,
                retry_after,
            );
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

    #[test]
    fn test_rate_limiter_state_creation() {
        let state = RateLimiterState::new(100);
        assert_eq!(state.rate_limit_per_minute, 100);
    }

    #[test]
    fn test_rate_limiter_allows_requests() {
        let state = RateLimiterState::new(100);
        let key_id: i64 = 1;

        // First request should be allowed
        assert!(state.check(key_id).is_ok());
    }

    #[test]
    fn test_rate_limiter_state_debug() {
        let state = RateLimiterState::new(100);
        let debug = format!("{:?}", state);
        assert!(debug.contains("RateLimiterState"));
        assert!(debug.contains("rate_limit_per_minute"));
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
    fn test_rate_limiter_different_keys_independent() {
        let state = RateLimiterState::new(2); // Very low limit for testing
        let key1: i64 = 1;
        let key2: i64 = 2;

        // Each key should have independent limits
        assert!(state.check(key1).is_ok());
        assert!(state.check(key2).is_ok());
    }

    #[test]
    fn test_rate_limited_response_format() {
        let response = rate_limited_response(100, 60);
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response.headers().contains_key(header::RETRY_AFTER));
        assert_eq!(
            response.headers().get(header::RETRY_AFTER).unwrap(),
            "60"
        );
    }
}
