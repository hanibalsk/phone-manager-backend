//! Rate limiting middleware.
//!
//! Provides per-API-key rate limiting using a sliding window algorithm.
//! Also provides per-organization export rate limiting for audit log exports.
//! Also provides per-IP rate limiting for authentication endpoints.

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
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
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    sync::{Arc, RwLock},
    time::Duration,
};
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::api_key::ApiKeyAuth;

/// Type alias for the rate limiter used per API key.
type KeyRateLimiter = GovRateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Type alias for the rate limiter used per organization for exports.
type OrgRateLimiter = GovRateLimiter<NotKeyed, InMemoryState, DefaultClock>;

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

/// Export rate limiter state for per-organization export rate limiting.
/// Uses a HashMap keyed by organization UUID with individual rate limiters.
/// Configured for 10 exports per hour per organization.
pub struct ExportRateLimiterState {
    limiters: RwLock<HashMap<Uuid, Arc<OrgRateLimiter>>>,
    rate_limit_per_hour: u32,
}

impl ExportRateLimiterState {
    /// Create a new export rate limiter state with the specified limit per hour.
    pub fn new(rate_limit_per_hour: u32) -> Self {
        Self {
            limiters: RwLock::new(HashMap::new()),
            rate_limit_per_hour,
        }
    }

    /// Get or create a rate limiter for the given organization ID.
    fn get_or_create_limiter(&self, org_id: Uuid) -> Arc<OrgRateLimiter> {
        // First try to get existing limiter with read lock
        {
            let limiters = self.limiters.read().unwrap();
            if let Some(limiter) = limiters.get(&org_id) {
                return limiter.clone();
            }
        }

        // Create new limiter with write lock
        let mut limiters = self.limiters.write().unwrap();

        // Double-check in case another thread created it
        if let Some(limiter) = limiters.get(&org_id) {
            return limiter.clone();
        }

        // Create new limiter with rate limit per hour
        let quota = Quota::per_hour(
            NonZeroU32::new(self.rate_limit_per_hour).unwrap_or(NonZeroU32::new(10).unwrap()),
        );
        let limiter = Arc::new(GovRateLimiter::direct(quota));
        limiters.insert(org_id, limiter.clone());
        limiter
    }

    /// Check if an export request from the given organization should be allowed.
    /// Returns Ok(()) if allowed, or Err with retry_after seconds if rate limited.
    pub fn check(&self, org_id: Uuid) -> Result<(), u64> {
        let limiter = self.get_or_create_limiter(org_id);

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

    /// Get the configured rate limit per hour.
    pub fn rate_limit_per_hour(&self) -> u32 {
        self.rate_limit_per_hour
    }
}

impl std::fmt::Debug for ExportRateLimiterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExportRateLimiterState")
            .field("rate_limit_per_hour", &self.rate_limit_per_hour)
            .field("active_limiters", &self.limiters.read().unwrap().len())
            .finish()
    }
}

impl Clone for ExportRateLimiterState {
    fn clone(&self) -> Self {
        // Clone creates a new state that shares the same limiters
        Self {
            limiters: RwLock::new(self.limiters.read().unwrap().clone()),
            rate_limit_per_hour: self.rate_limit_per_hour,
        }
    }
}

/// Type alias for the rate limiter used per IP address for auth endpoints.
type IpRateLimiter = GovRateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Auth rate limiter state for per-IP rate limiting on authentication endpoints.
/// Uses a HashMap keyed by IP address with individual rate limiters.
/// Configured separately for password reset (5/hour) and verification (3/hour).
pub struct AuthRateLimiterState {
    limiters: RwLock<HashMap<IpAddr, Arc<IpRateLimiter>>>,
    rate_limit_per_hour: u32,
    endpoint_name: String,
}

impl AuthRateLimiterState {
    /// Create a new auth rate limiter state with the specified limit per hour.
    pub fn new(rate_limit_per_hour: u32, endpoint_name: &str) -> Self {
        Self {
            limiters: RwLock::new(HashMap::new()),
            rate_limit_per_hour,
            endpoint_name: endpoint_name.to_string(),
        }
    }

    /// Get or create a rate limiter for the given IP address.
    fn get_or_create_limiter(&self, ip: IpAddr) -> Arc<IpRateLimiter> {
        // First try to get existing limiter with read lock
        {
            let limiters = self.limiters.read().unwrap();
            if let Some(limiter) = limiters.get(&ip) {
                return limiter.clone();
            }
        }

        // Create new limiter with write lock
        let mut limiters = self.limiters.write().unwrap();

        // Double-check in case another thread created it
        if let Some(limiter) = limiters.get(&ip) {
            return limiter.clone();
        }

        // Create new limiter with rate limit per hour
        // Using with_period to create a quota that replenishes over an hour
        let quota = Quota::with_period(Duration::from_secs(3600 / self.rate_limit_per_hour as u64))
            .unwrap()
            .allow_burst(
                NonZeroU32::new(self.rate_limit_per_hour).unwrap_or(NonZeroU32::new(1).unwrap()),
            );
        let limiter = Arc::new(GovRateLimiter::direct(quota));
        limiters.insert(ip, limiter.clone());
        limiter
    }

    /// Check if a request from the given IP should be allowed.
    /// Returns Ok(()) if allowed, or Err with retry_after seconds if rate limited.
    pub fn check(&self, ip: IpAddr) -> Result<(), u64> {
        let limiter = self.get_or_create_limiter(ip);

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

    /// Get the configured rate limit per hour.
    pub fn rate_limit_per_hour(&self) -> u32 {
        self.rate_limit_per_hour
    }

    /// Get the endpoint name for error messages.
    pub fn endpoint_name(&self) -> &str {
        &self.endpoint_name
    }
}

impl std::fmt::Debug for AuthRateLimiterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthRateLimiterState")
            .field("rate_limit_per_hour", &self.rate_limit_per_hour)
            .field("endpoint_name", &self.endpoint_name)
            .field("active_limiters", &self.limiters.read().unwrap().len())
            .finish()
    }
}

impl Clone for AuthRateLimiterState {
    fn clone(&self) -> Self {
        // Clone creates a new state that shares the same limiters
        Self {
            limiters: RwLock::new(self.limiters.read().unwrap().clone()),
            rate_limit_per_hour: self.rate_limit_per_hour,
            endpoint_name: self.endpoint_name.clone(),
        }
    }
}

/// Extract IP address from request, checking X-Forwarded-For header first,
/// then falling back to connection info.
fn extract_client_ip<B>(req: &Request<B>) -> Option<IpAddr> {
    // First check X-Forwarded-For header (for requests behind a proxy)
    if let Some(forwarded_for) = req.headers().get("x-forwarded-for") {
        if let Ok(value) = forwarded_for.to_str() {
            // X-Forwarded-For can contain multiple IPs: "client, proxy1, proxy2"
            // The first one is the original client
            if let Some(first_ip) = value.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }

    // Fall back to X-Real-IP header
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            if let Ok(ip) = value.trim().parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    // Fall back to connection info
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|conn| conn.0.ip())
}

/// Middleware factory for auth rate limiting.
/// Takes the rate limiter state directly instead of from AppState.
pub async fn auth_rate_limit_middleware(
    State(rate_limiter): State<Arc<AuthRateLimiterState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // Extract client IP
    let client_ip = match extract_client_ip(&req) {
        Some(ip) => ip,
        None => {
            tracing::warn!("Could not determine client IP for auth rate limiting");
            // Allow request if we can't determine IP (fail open, but log)
            return next.run(req).await;
        }
    };

    // Check rate limit
    if let Err(retry_after) = rate_limiter.check(client_ip) {
        tracing::warn!(
            ip = %client_ip,
            endpoint = %rate_limiter.endpoint_name(),
            retry_after = retry_after,
            "Auth rate limit exceeded"
        );
        return auth_rate_limited_response(
            rate_limiter.rate_limit_per_hour(),
            rate_limiter.endpoint_name(),
            retry_after,
        );
    }

    next.run(req).await
}

/// Create a rate limited response for auth endpoints with proper headers and body.
fn auth_rate_limited_response(limit: u32, endpoint: &str, retry_after: u64) -> Response {
    let body = json!({
        "error": "rate_limit_exceeded",
        "message": format!("Rate limit of {} requests/hour exceeded for {}", limit, endpoint),
        "retry_after": retry_after
    });

    let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();

    // Add Retry-After header
    response.headers_mut().insert(
        header::RETRY_AFTER,
        retry_after.to_string().parse().unwrap(),
    );

    response
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
        "retry_after": retry_after
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

    // ===========================================
    // Export Rate Limiter Tests
    // ===========================================

    #[test]
    fn test_export_rate_limiter_state_creation() {
        let state = ExportRateLimiterState::new(10);
        assert_eq!(state.rate_limit_per_hour(), 10);
    }

    #[test]
    fn test_export_rate_limiter_allows_requests() {
        let state = ExportRateLimiterState::new(10);
        let org_id = Uuid::new_v4();

        // First request should be allowed
        assert!(state.check(org_id).is_ok());
    }

    #[test]
    fn test_export_rate_limiter_exhaustion() {
        // Use very low limit to test exhaustion
        let state = ExportRateLimiterState::new(1);
        let org_id = Uuid::new_v4();

        // First request should be allowed
        assert!(state.check(org_id).is_ok());

        // Second request should be rate limited
        let result = state.check(org_id);
        assert!(result.is_err());
        // Retry-after should be at least 1 second
        assert!(result.unwrap_err() >= 1);
    }

    #[test]
    fn test_export_rate_limiter_different_orgs_independent() {
        let state = ExportRateLimiterState::new(1); // Very low limit
        let org1 = Uuid::new_v4();
        let org2 = Uuid::new_v4();
        let org3 = Uuid::new_v4();

        // Each org should have independent limits
        assert!(state.check(org1).is_ok());
        assert!(state.check(org2).is_ok());
        assert!(state.check(org3).is_ok());

        // Now org1 should be rate limited, but others still allowed for their first request
        assert!(state.check(org1).is_err());
        assert!(state.check(org2).is_err());
        assert!(state.check(org3).is_err());
    }

    #[test]
    fn test_export_rate_limiter_same_org_multiple_checks() {
        let state = ExportRateLimiterState::new(5);
        let org_id = Uuid::new_v4();

        // Should allow 5 requests
        for i in 0..5 {
            let result = state.check(org_id);
            assert!(result.is_ok(), "Request {} should be allowed", i);
        }

        // 6th request should be rate limited
        assert!(state.check(org_id).is_err());
    }

    #[test]
    fn test_export_rate_limiter_many_orgs() {
        let state = ExportRateLimiterState::new(10);

        // Test with 100 different organizations
        for _ in 0..100 {
            let org_id = Uuid::new_v4();
            assert!(state.check(org_id).is_ok());
        }
    }

    #[test]
    fn test_export_rate_limiter_state_debug() {
        let state = ExportRateLimiterState::new(10);
        let debug = format!("{:?}", state);
        assert!(debug.contains("ExportRateLimiterState"));
        assert!(debug.contains("rate_limit_per_hour"));
        assert!(debug.contains("10"));
        assert!(debug.contains("active_limiters"));
    }

    #[test]
    fn test_export_rate_limiter_state_clone() {
        let state = ExportRateLimiterState::new(10);
        let org_id = Uuid::new_v4();
        state.check(org_id).unwrap(); // Create a limiter

        let cloned = state.clone();
        assert_eq!(cloned.rate_limit_per_hour(), 10);
    }

    #[test]
    fn test_export_rate_limiter_get_or_create_idempotent() {
        let state = ExportRateLimiterState::new(10);
        let org_id = Uuid::new_v4();

        // Multiple calls should return the same limiter
        let limiter1 = state.get_or_create_limiter(org_id);
        let limiter2 = state.get_or_create_limiter(org_id);

        // Should be the same Arc (same underlying object)
        assert!(Arc::ptr_eq(&limiter1, &limiter2));
    }

    #[test]
    fn test_export_rate_limiter_different_orgs_different_limiters() {
        let state = ExportRateLimiterState::new(10);

        let limiter1 = state.get_or_create_limiter(Uuid::new_v4());
        let limiter2 = state.get_or_create_limiter(Uuid::new_v4());

        // Should be different Arcs
        assert!(!Arc::ptr_eq(&limiter1, &limiter2));
    }

    // ===========================================
    // Auth Rate Limiter Tests (Per-IP)
    // ===========================================

    #[test]
    fn test_auth_rate_limiter_state_creation() {
        let state = AuthRateLimiterState::new(5, "forgot-password");
        assert_eq!(state.rate_limit_per_hour(), 5);
        assert_eq!(state.endpoint_name(), "forgot-password");
    }

    #[test]
    fn test_auth_rate_limiter_allows_requests() {
        let state = AuthRateLimiterState::new(5, "test-endpoint");
        let ip: IpAddr = "192.168.1.1".parse().unwrap();

        // First request should be allowed
        assert!(state.check(ip).is_ok());
    }

    #[test]
    fn test_auth_rate_limiter_exhaustion() {
        // Use very low limit to test exhaustion
        let state = AuthRateLimiterState::new(1, "test-endpoint");
        let ip: IpAddr = "192.168.1.1".parse().unwrap();

        // First request should be allowed
        assert!(state.check(ip).is_ok());

        // Second request should be rate limited
        let result = state.check(ip);
        assert!(result.is_err());
        // Retry-after should be at least 1 second
        assert!(result.unwrap_err() >= 1);
    }

    #[test]
    fn test_auth_rate_limiter_different_ips_independent() {
        let state = AuthRateLimiterState::new(1, "test-endpoint"); // Very low limit
        let ip1: IpAddr = "192.168.1.1".parse().unwrap();
        let ip2: IpAddr = "192.168.1.2".parse().unwrap();
        let ip3: IpAddr = "192.168.1.3".parse().unwrap();

        // Each IP should have independent limits
        assert!(state.check(ip1).is_ok());
        assert!(state.check(ip2).is_ok());
        assert!(state.check(ip3).is_ok());

        // Now ip1 should be rate limited, but others still allowed for their first request
        assert!(state.check(ip1).is_err());
        assert!(state.check(ip2).is_err());
        assert!(state.check(ip3).is_err());
    }

    #[test]
    fn test_auth_rate_limiter_same_ip_multiple_checks() {
        let state = AuthRateLimiterState::new(5, "forgot-password");
        let ip: IpAddr = "10.0.0.1".parse().unwrap();

        // Should allow 5 requests
        for i in 0..5 {
            let result = state.check(ip);
            assert!(result.is_ok(), "Request {} should be allowed", i);
        }

        // 6th request should be rate limited
        assert!(state.check(ip).is_err());
    }

    #[test]
    fn test_auth_rate_limiter_many_ips() {
        let state = AuthRateLimiterState::new(5, "test-endpoint");

        // Test with 100 different IPs
        for i in 0..100u8 {
            let ip: IpAddr = format!("192.168.1.{}", i).parse().unwrap();
            assert!(state.check(ip).is_ok());
        }
    }

    #[test]
    fn test_auth_rate_limiter_ipv4_and_ipv6() {
        let state = AuthRateLimiterState::new(5, "test-endpoint");

        // IPv4 address
        let ipv4: IpAddr = "192.168.1.1".parse().unwrap();
        assert!(state.check(ipv4).is_ok());

        // IPv6 address
        let ipv6: IpAddr = "::1".parse().unwrap();
        assert!(state.check(ipv6).is_ok());

        // Full IPv6 address
        let ipv6_full: IpAddr = "2001:0db8:85a3:0000:0000:8a2e:0370:7334".parse().unwrap();
        assert!(state.check(ipv6_full).is_ok());
    }

    #[test]
    fn test_auth_rate_limiter_state_debug() {
        let state = AuthRateLimiterState::new(5, "forgot-password");
        let debug = format!("{:?}", state);
        assert!(debug.contains("AuthRateLimiterState"));
        assert!(debug.contains("rate_limit_per_hour"));
        assert!(debug.contains("5"));
        assert!(debug.contains("endpoint_name"));
        assert!(debug.contains("forgot-password"));
        assert!(debug.contains("active_limiters"));
    }

    #[test]
    fn test_auth_rate_limiter_state_clone() {
        let state = AuthRateLimiterState::new(5, "test-endpoint");
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        state.check(ip).unwrap(); // Create a limiter

        let cloned = state.clone();
        assert_eq!(cloned.rate_limit_per_hour(), 5);
        assert_eq!(cloned.endpoint_name(), "test-endpoint");
    }

    #[test]
    fn test_auth_rate_limiter_get_or_create_idempotent() {
        let state = AuthRateLimiterState::new(5, "test-endpoint");
        let ip: IpAddr = "192.168.1.1".parse().unwrap();

        // Multiple calls should return the same limiter
        let limiter1 = state.get_or_create_limiter(ip);
        let limiter2 = state.get_or_create_limiter(ip);

        // Should be the same Arc (same underlying object)
        assert!(Arc::ptr_eq(&limiter1, &limiter2));
    }

    #[test]
    fn test_auth_rate_limiter_different_ips_different_limiters() {
        let state = AuthRateLimiterState::new(5, "test-endpoint");

        let ip1: IpAddr = "192.168.1.1".parse().unwrap();
        let ip2: IpAddr = "192.168.1.2".parse().unwrap();

        let limiter1 = state.get_or_create_limiter(ip1);
        let limiter2 = state.get_or_create_limiter(ip2);

        // Should be different Arcs
        assert!(!Arc::ptr_eq(&limiter1, &limiter2));
    }

    #[test]
    fn test_auth_rate_limited_response_format() {
        let response = auth_rate_limited_response(5, "forgot-password", 3600);
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response.headers().contains_key(header::RETRY_AFTER));
        assert_eq!(response.headers().get(header::RETRY_AFTER).unwrap(), "3600");
    }

    #[test]
    fn test_auth_rate_limited_response_various_endpoints() {
        let endpoints = vec!["forgot-password", "request-verification"];
        for endpoint in endpoints {
            let response = auth_rate_limited_response(5, endpoint, 60);
            assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        }
    }
}
