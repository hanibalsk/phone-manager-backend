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
    map_matching_client: Option<std::sync::Arc<MapMatchingClient>>,
}

impl PathCorrectionService {
    /// Create a new PathCorrectionService with a shared map-matching client.
    ///
    /// If the client is None, corrections will be skipped.
    pub fn new(pool: PgPool, map_matching_client: Option<std::sync::Arc<MapMatchingClient>>) -> Self {
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

        // Need at least 2 points for meaningful path correction
        if locations.len() < 2 {
            info!(
                trip_id = %trip_id,
                location_count = locations.len(),
                "Not enough locations for path correction, creating SKIPPED record"
            );

            // Convert any existing locations to coordinates
            let coords: Vec<[f64; 2]> = locations
                .iter()
                .map(|loc| [loc.longitude, loc.latitude])
                .collect();

            // Create a SKIPPED record in the database so GET /trips/:id/path returns status
            // The repository handles 0 or 1 coordinate by creating a degenerate LINESTRING
            correction_repo.create_skipped(trip_id, &coords).await?;

            return Ok(PathCorrectionResult {
                trip_id,
                status: "SKIPPED".to_string(),
                quality: None,
                original_points: coords.len(),
                corrected_points: None,
            });
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
                // Determine if this is a "service unavailable" error (SKIPPED)
                // or an actual processing failure (FAILED)
                let (status, log_level) = match &e {
                    MapMatchingError::CircuitOpen
                    | MapMatchingError::RateLimited
                    | MapMatchingError::Disabled
                    | MapMatchingError::NotConfigured => {
                        // Service unavailable - mark as SKIPPED for retry later
                        warn!(
                            trip_id = %trip_id,
                            error = %e,
                            "Map-matching service unavailable, marking as SKIPPED"
                        );
                        ("SKIPPED", false)
                    }
                    _ => {
                        // Actual failure - request was processed but failed
                        error!(
                            trip_id = %trip_id,
                            error = %e,
                            "Map-matching failed, marking as FAILED"
                        );
                        ("FAILED", true)
                    }
                };

                // Suppress unused variable warning for log_level
                let _ = log_level;

                let update = TripPathCorrectionUpdateInput {
                    corrected_path_coords: None,
                    correction_quality: None,
                    correction_status: status.to_string(),
                };
                correction_repo.update(trip_id, update).await?;

                Ok(PathCorrectionResult {
                    trip_id,
                    status: status.to_string(),
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
        let err = PathCorrectionError::AlreadyExists(Uuid::new_v4());
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn test_path_correction_result_clone() {
        let result = PathCorrectionResult {
            trip_id: Uuid::new_v4(),
            status: "COMPLETED".to_string(),
            quality: Some(0.95),
            original_points: 100,
            corrected_points: Some(150),
        };

        let cloned = result.clone();
        assert_eq!(cloned.status, result.status);
        assert_eq!(cloned.quality, result.quality);
    }
}
