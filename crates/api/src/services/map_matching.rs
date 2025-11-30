//! Map-matching service integration for GPS trace correction.
//!
//! Supports OSRM Match API for snapping GPS coordinates to road networks.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::MapMatchingConfig;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during map-matching operations.
#[derive(Debug, Error)]
pub enum MapMatchingError {
    #[error("Map-matching service is disabled")]
    Disabled,

    #[error("Map-matching service URL not configured")]
    NotConfigured,

    #[error("Circuit breaker is open, service temporarily unavailable")]
    CircuitOpen,

    #[error("Rate limit exceeded, try again later")]
    RateLimited,

    #[error("Request timeout after {0}ms")]
    Timeout(u64),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid response from map-matching service: {0}")]
    InvalidResponse(String),

    #[error("Map-matching service error: {0}")]
    ServiceError(String),

    #[error("Too few coordinates for map-matching (need at least 2)")]
    TooFewCoordinates,
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// A coordinate pair (longitude, latitude).
pub type Coordinate = [f64; 2];

/// Result of a map-matching operation.
#[derive(Debug, Clone)]
pub struct MapMatchingResult {
    /// Snapped coordinates on the road network.
    pub matched_coordinates: Vec<Coordinate>,

    /// Confidence score from 0.0 to 1.0.
    pub confidence: f32,

    /// Duration of the match operation in milliseconds.
    pub duration_ms: u64,
}

/// OSRM Match API response structure.
#[derive(Debug, Deserialize)]
struct OsrmMatchResponse {
    code: String,
    matchings: Option<Vec<OsrmMatching>>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OsrmMatching {
    confidence: f64,
    geometry: OsrmGeometry,
}

#[derive(Debug, Deserialize)]
struct OsrmGeometry {
    coordinates: Vec<Vec<f64>>,
}

// ============================================================================
// Rate Limiter
// ============================================================================

/// Simple token bucket rate limiter.
struct RateLimiter {
    /// Tokens available.
    tokens: AtomicU32,
    /// Max tokens (requests per minute).
    max_tokens: u32,
    /// Last refill timestamp (unix millis).
    last_refill: AtomicU64,
}

impl RateLimiter {
    fn new(requests_per_minute: u32) -> Self {
        Self {
            tokens: AtomicU32::new(requests_per_minute),
            max_tokens: requests_per_minute,
            last_refill: AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            ),
        }
    }

    /// Try to acquire a token. Returns true if allowed.
    fn try_acquire(&self) -> bool {
        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let last_refill = self.last_refill.load(Ordering::Relaxed);
        let elapsed = now_millis.saturating_sub(last_refill);

        // Refill tokens every minute
        if elapsed >= 60_000 {
            self.tokens.store(self.max_tokens, Ordering::Relaxed);
            self.last_refill.store(now_millis, Ordering::Relaxed);
        }

        // Try to take a token
        loop {
            let current = self.tokens.load(Ordering::Relaxed);
            if current == 0 {
                return false;
            }
            if self
                .tokens
                .compare_exchange_weak(current, current - 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return true;
            }
        }
    }
}

// ============================================================================
// Circuit Breaker
// ============================================================================

/// Circuit breaker states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Used for testing and monitoring
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for external service protection.
struct CircuitBreaker {
    /// Current state.
    is_open: AtomicBool,
    /// Consecutive failure count.
    failure_count: AtomicU32,
    /// Failure threshold to open.
    failure_threshold: u32,
    /// Time to stay open before half-open retry.
    reset_timeout: Duration,
    /// When circuit was opened.
    opened_at: RwLock<Option<Instant>>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, reset_timeout_secs: u64) -> Self {
        Self {
            is_open: AtomicBool::new(false),
            failure_count: AtomicU32::new(0),
            failure_threshold,
            reset_timeout: Duration::from_secs(reset_timeout_secs),
            opened_at: RwLock::new(None),
        }
    }

    /// Check if request is allowed.
    async fn is_allowed(&self) -> bool {
        if !self.is_open.load(Ordering::Relaxed) {
            return true;
        }

        // Check if we can transition to half-open
        let opened_at = *self.opened_at.read().await;
        if let Some(opened) = opened_at {
            if opened.elapsed() >= self.reset_timeout {
                debug!("Circuit breaker transitioning to half-open");
                return true; // Allow one request in half-open state
            }
        }

        false
    }

    /// Record a successful request.
    async fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        if self.is_open.load(Ordering::Relaxed) {
            info!("Circuit breaker closed after successful request");
            self.is_open.store(false, Ordering::Relaxed);
            *self.opened_at.write().await = None;
        }
    }

    /// Record a failed request.
    async fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

        if count >= self.failure_threshold && !self.is_open.load(Ordering::Relaxed) {
            warn!(
                failure_count = count,
                threshold = self.failure_threshold,
                "Circuit breaker opened due to consecutive failures"
            );
            self.is_open.store(true, Ordering::Relaxed);
            *self.opened_at.write().await = Some(Instant::now());
        }
    }

    /// Get current state.
    #[allow(dead_code)] // Used for testing and monitoring
    async fn state(&self) -> CircuitState {
        if !self.is_open.load(Ordering::Relaxed) {
            return CircuitState::Closed;
        }

        let opened_at = *self.opened_at.read().await;
        if let Some(opened) = opened_at {
            if opened.elapsed() >= self.reset_timeout {
                return CircuitState::HalfOpen;
            }
        }

        CircuitState::Open
    }
}

// ============================================================================
// Map Matching Client
// ============================================================================

/// Client for map-matching services.
pub struct MapMatchingClient {
    /// HTTP client.
    client: Client,
    /// Configuration.
    config: MapMatchingConfig,
    /// Rate limiter.
    rate_limiter: RateLimiter,
    /// Circuit breaker.
    circuit_breaker: CircuitBreaker,
}

impl MapMatchingClient {
    /// Create a new map-matching client.
    pub fn new(config: MapMatchingConfig) -> Result<Self, MapMatchingError> {
        let timeout = Duration::from_millis(config.timeout_ms);

        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(MapMatchingError::Http)?;

        let rate_limiter = RateLimiter::new(config.rate_limit_per_minute);
        let circuit_breaker =
            CircuitBreaker::new(config.circuit_breaker_failures, config.circuit_breaker_reset_secs);

        Ok(Self {
            client,
            config,
            rate_limiter,
            circuit_breaker,
        })
    }

    /// Check if map-matching is enabled and configured.
    #[allow(dead_code)] // Public API for monitoring
    pub fn is_available(&self) -> bool {
        self.config.enabled && !self.config.url.is_empty()
    }

    /// Get current circuit breaker state.
    #[allow(dead_code)] // Public API for monitoring
    pub async fn circuit_state(&self) -> CircuitState {
        self.circuit_breaker.state().await
    }

    /// Match coordinates to road network using OSRM.
    ///
    /// Returns snapped coordinates and confidence score.
    pub async fn match_coordinates(
        &self,
        coordinates: &[Coordinate],
    ) -> Result<MapMatchingResult, MapMatchingError> {
        // Check if enabled
        if !self.config.enabled {
            return Err(MapMatchingError::Disabled);
        }

        // Check if configured
        if self.config.url.is_empty() {
            return Err(MapMatchingError::NotConfigured);
        }

        // Need at least 2 coordinates
        if coordinates.len() < 2 {
            return Err(MapMatchingError::TooFewCoordinates);
        }

        // Check circuit breaker
        if !self.circuit_breaker.is_allowed().await {
            return Err(MapMatchingError::CircuitOpen);
        }

        // Check rate limit
        if !self.rate_limiter.try_acquire() {
            return Err(MapMatchingError::RateLimited);
        }

        let start = Instant::now();

        // Build OSRM URL with coordinates
        let result = self.call_osrm_match(coordinates).await;

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(mut res) => {
                res.duration_ms = duration_ms;
                self.circuit_breaker.record_success().await;
                debug!(
                    input_points = coordinates.len(),
                    output_points = res.matched_coordinates.len(),
                    confidence = res.confidence,
                    duration_ms = duration_ms,
                    "Map-matching successful"
                );
                Ok(res)
            }
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                error!(
                    error = %e,
                    duration_ms = duration_ms,
                    "Map-matching failed"
                );
                Err(e)
            }
        }
    }

    /// Call OSRM Match API.
    async fn call_osrm_match(
        &self,
        coordinates: &[Coordinate],
    ) -> Result<MapMatchingResult, MapMatchingError> {
        // Build coordinate string: lon,lat;lon,lat;...
        let coord_str: String = coordinates
            .iter()
            .map(|[lon, lat]| format!("{},{}", lon, lat))
            .collect::<Vec<_>>()
            .join(";");

        // OSRM Match URL: /match/v1/driving/{coordinates}?overview=full&geometries=geojson
        let url = format!(
            "{}/match/v1/driving/{}?overview=full&geometries=geojson",
            self.config.url.trim_end_matches('/'),
            coord_str
        );

        debug!(url = %url, "Calling OSRM Match API");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    MapMatchingError::Timeout(self.config.timeout_ms)
                } else {
                    MapMatchingError::Http(e)
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(MapMatchingError::ServiceError(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let osrm_response: OsrmMatchResponse = response
            .json()
            .await
            .map_err(|e| MapMatchingError::InvalidResponse(e.to_string()))?;

        if osrm_response.code != "Ok" {
            return Err(MapMatchingError::ServiceError(
                osrm_response
                    .message
                    .unwrap_or_else(|| osrm_response.code.clone()),
            ));
        }

        let matchings = osrm_response
            .matchings
            .ok_or_else(|| MapMatchingError::InvalidResponse("No matchings in response".into()))?;

        if matchings.is_empty() {
            return Err(MapMatchingError::InvalidResponse(
                "Empty matchings array".into(),
            ));
        }

        // Use first matching (typically there's only one for simple traces)
        let matching = &matchings[0];

        // Convert coordinates from [[lon, lat], ...] to [[lon, lat]; 2] array
        let matched_coordinates: Vec<Coordinate> = matching
            .geometry
            .coordinates
            .iter()
            .filter_map(|coord| {
                if coord.len() >= 2 {
                    Some([coord[0], coord[1]])
                } else {
                    None
                }
            })
            .collect();

        if matched_coordinates.is_empty() {
            return Err(MapMatchingError::InvalidResponse(
                "No coordinates in matched geometry".into(),
            ));
        }

        Ok(MapMatchingResult {
            matched_coordinates,
            confidence: matching.confidence as f32,
            duration_ms: 0, // Will be set by caller
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config(enabled: bool) -> MapMatchingConfig {
        MapMatchingConfig {
            provider: "osrm".to_string(),
            url: if enabled {
                "http://router.project-osrm.org".to_string()
            } else {
                "".to_string()
            },
            timeout_ms: 30000,
            rate_limit_per_minute: 30,
            circuit_breaker_failures: 5,
            circuit_breaker_reset_secs: 60,
            enabled,
        }
    }

    #[test]
    fn test_client_creation() {
        let config = create_test_config(false);
        let client = MapMatchingClient::new(config).unwrap();
        assert!(!client.is_available());
    }

    #[test]
    fn test_client_available_when_enabled() {
        let config = create_test_config(true);
        let client = MapMatchingClient::new(config).unwrap();
        assert!(client.is_available());
    }

    #[tokio::test]
    async fn test_disabled_service() {
        let config = create_test_config(false);
        let client = MapMatchingClient::new(config).unwrap();

        let coords = vec![[-120.0, 45.0], [-120.1, 45.1]];
        let result = client.match_coordinates(&coords).await;

        assert!(matches!(result, Err(MapMatchingError::Disabled)));
    }

    #[tokio::test]
    async fn test_too_few_coordinates() {
        let mut config = create_test_config(true);
        config.url = "http://example.com".to_string();
        let client = MapMatchingClient::new(config).unwrap();

        let coords = vec![[-120.0, 45.0]];
        let result = client.match_coordinates(&coords).await;

        assert!(matches!(result, Err(MapMatchingError::TooFewCoordinates)));
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(3);

        // Should allow 3 requests
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());

        // 4th should be denied
        assert!(!limiter.try_acquire());
    }

    #[tokio::test]
    async fn test_circuit_breaker_initial_state() {
        let breaker = CircuitBreaker::new(3, 60);
        assert_eq!(breaker.state().await, CircuitState::Closed);
        assert!(breaker.is_allowed().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let breaker = CircuitBreaker::new(3, 60);

        // Record 3 failures
        breaker.record_failure().await;
        breaker.record_failure().await;
        breaker.record_failure().await;

        assert_eq!(breaker.state().await, CircuitState::Open);
        assert!(!breaker.is_allowed().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_on_success() {
        let breaker = CircuitBreaker::new(3, 60);

        // Open circuit
        breaker.record_failure().await;
        breaker.record_failure().await;
        breaker.record_failure().await;

        // Manually reset for test
        breaker.is_open.store(false, Ordering::Relaxed);

        // Success should reset failure count
        breaker.record_success().await;

        assert_eq!(breaker.state().await, CircuitState::Closed);
    }

    #[test]
    fn test_map_matching_result_debug() {
        let result = MapMatchingResult {
            matched_coordinates: vec![[-120.0, 45.0], [-120.1, 45.1]],
            confidence: 0.95,
            duration_ms: 150,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("MapMatchingResult"));
        assert!(debug_str.contains("0.95"));
    }
}
