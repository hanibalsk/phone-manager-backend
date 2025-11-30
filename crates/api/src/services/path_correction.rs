//! Path correction service for automatic trip path correction.
//!
//! Orchestrates the path correction workflow:
//! 1. Extract trip locations
//! 2. Call map-matching service
//! 3. Store corrected path

use persistence::repositories::{
    LocationRepository, TripPathCorrectionInput, TripPathCorrectionRepository,
    TripPathCorrectionUpdateInput,
};
use sqlx::PgPool;
use thiserror::Error;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::MapMatchingConfig;
use crate::services::map_matching::{MapMatchingClient, MapMatchingError};

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during path correction operations.
#[derive(Debug, Error)]
pub enum PathCorrectionError {
    #[error("Map-matching error: {0}")]
    MapMatching(#[from] MapMatchingError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not enough locations for path correction (need at least 2, got {0})")]
    InsufficientLocations(usize),

    #[error("Path correction already exists for trip {0}")]
    AlreadyExists(Uuid),
}

// ============================================================================
// Path Correction Result
// ============================================================================

/// Result of a path correction operation.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used for logging and future API responses
pub struct PathCorrectionResult {
    /// Trip ID that was corrected.
    pub trip_id: Uuid,
    /// Correction status: COMPLETED, FAILED, or SKIPPED.
    pub status: String,
    /// Quality/confidence score (0.0-1.0) if correction succeeded.
    pub quality: Option<f32>,
    /// Number of original points.
    pub original_points: usize,
    /// Number of corrected points (may differ from original).
    pub corrected_points: Option<usize>,
}

// ============================================================================
// Path Correction Service
// ============================================================================

/// Service for correcting trip paths using map-matching.
pub struct PathCorrectionService {
    pool: PgPool,
    map_matching_client: Option<MapMatchingClient>,
}

impl PathCorrectionService {
    /// Create a new PathCorrectionService.
    ///
    /// If map-matching is disabled or not configured, the client will be None
    /// and corrections will be skipped.
    pub fn new(pool: PgPool, config: &MapMatchingConfig) -> Self {
        let map_matching_client = if config.enabled && !config.url.is_empty() {
            match MapMatchingClient::new(config.clone()) {
                Ok(client) => Some(client),
                Err(e) => {
                    error!(error = %e, "Failed to create map-matching client");
                    None
                }
            }
        } else {
            debug!("Map-matching is disabled or not configured");
            None
        };

        Self {
            pool,
            map_matching_client,
        }
    }

    /// Check if map-matching is available.
    #[allow(dead_code)] // Public API for monitoring
    pub fn is_map_matching_available(&self) -> bool {
        self.map_matching_client
            .as_ref()
            .is_some_and(|c| c.is_available())
    }

    /// Correct the path for a completed trip.
    ///
    /// This method:
    /// 1. Fetches all locations for the trip
    /// 2. Creates a PENDING correction record with original path
    /// 3. If map-matching is available, calls the service
    /// 4. Updates the record with corrected path or failure status
    ///
    /// If map-matching is disabled, the correction is marked as SKIPPED.
    pub async fn correct_trip_path(
        &self,
        trip_id: Uuid,
    ) -> Result<PathCorrectionResult, PathCorrectionError> {
        info!(trip_id = %trip_id, "Starting path correction for trip");

        let location_repo = LocationRepository::new(self.pool.clone());
        let correction_repo = TripPathCorrectionRepository::new(self.pool.clone());

        // Check if correction already exists
        if let Some(existing) = correction_repo.find_by_trip_id(trip_id).await? {
            warn!(
                trip_id = %trip_id,
                status = %existing.correction_status,
                "Path correction already exists for trip"
            );
            return Err(PathCorrectionError::AlreadyExists(trip_id));
        }

        // Fetch all locations for the trip
        let locations = location_repo.get_locations_for_trip(trip_id).await?;

        debug!(
            trip_id = %trip_id,
            location_count = locations.len(),
            "Fetched locations for trip"
        );

        // Need at least 2 points for a path
        if locations.len() < 2 {
            info!(
                trip_id = %trip_id,
                location_count = locations.len(),
                "Not enough locations for path correction, marking as SKIPPED"
            );

            // Create a skipped record if we have any locations
            if !locations.is_empty() {
                let coords: Vec<[f64; 2]> = locations
                    .iter()
                    .map(|loc| [loc.longitude, loc.latitude])
                    .collect();

                // Can't create a LINESTRING with 1 point, but we still want to record the attempt
                // We'll mark it as skipped without storing the path
                return Ok(PathCorrectionResult {
                    trip_id,
                    status: "SKIPPED".to_string(),
                    quality: None,
                    original_points: coords.len(),
                    corrected_points: None,
                });
            }

            return Err(PathCorrectionError::InsufficientLocations(locations.len()));
        }

        // Convert locations to coordinate array [lon, lat]
        let coords: Vec<[f64; 2]> = locations
            .iter()
            .map(|loc| [loc.longitude, loc.latitude])
            .collect();

        let original_points = coords.len();

        // Create initial correction record with PENDING status
        let input = TripPathCorrectionInput {
            trip_id,
            original_path_coords: coords.clone(),
        };
        correction_repo.create(input).await?;

        // If map-matching is not available, mark as SKIPPED
        let Some(ref client) = self.map_matching_client else {
            info!(
                trip_id = %trip_id,
                "Map-matching not available, marking correction as SKIPPED"
            );

            let update = TripPathCorrectionUpdateInput {
                corrected_path_coords: None,
                correction_quality: None,
                correction_status: "SKIPPED".to_string(),
            };
            correction_repo.update(trip_id, update).await?;

            return Ok(PathCorrectionResult {
                trip_id,
                status: "SKIPPED".to_string(),
                quality: None,
                original_points,
                corrected_points: None,
            });
        };

        // Call map-matching service
        match client.match_coordinates(&coords).await {
            Ok(result) => {
                info!(
                    trip_id = %trip_id,
                    confidence = result.confidence,
                    original_points = original_points,
                    matched_points = result.matched_coordinates.len(),
                    duration_ms = result.duration_ms,
                    "Map-matching successful"
                );

                let corrected_points = result.matched_coordinates.len();

                // Update with corrected path
                let update = TripPathCorrectionUpdateInput {
                    corrected_path_coords: Some(result.matched_coordinates),
                    correction_quality: Some(result.confidence),
                    correction_status: "COMPLETED".to_string(),
                };
                correction_repo.update(trip_id, update).await?;

                Ok(PathCorrectionResult {
                    trip_id,
                    status: "COMPLETED".to_string(),
                    quality: Some(result.confidence),
                    original_points,
                    corrected_points: Some(corrected_points),
                })
            }
            Err(e) => {
                error!(
                    trip_id = %trip_id,
                    error = %e,
                    "Map-matching failed"
                );

                // Update with FAILED status
                let update = TripPathCorrectionUpdateInput {
                    corrected_path_coords: None,
                    correction_quality: None,
                    correction_status: "FAILED".to_string(),
                };
                correction_repo.update(trip_id, update).await?;

                Ok(PathCorrectionResult {
                    trip_id,
                    status: "FAILED".to_string(),
                    quality: None,
                    original_points,
                    corrected_points: None,
                })
            }
        }
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
    fn test_path_correction_result_debug() {
        let result = PathCorrectionResult {
            trip_id: Uuid::new_v4(),
            status: "COMPLETED".to_string(),
            quality: Some(0.95),
            original_points: 100,
            corrected_points: Some(150),
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("PathCorrectionResult"));
        assert!(debug_str.contains("COMPLETED"));
    }

    #[test]
    fn test_path_correction_error_display() {
        let err = PathCorrectionError::InsufficientLocations(1);
        assert!(err.to_string().contains("at least 2"));

        let err = PathCorrectionError::AlreadyExists(Uuid::new_v4());
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn test_disabled_config() {
        let config = create_test_config(false);
        assert!(!config.enabled);
        assert!(config.url.is_empty());
    }
}
