//! Trip path correction entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the trip_path_corrections table.
///
/// Note: The original_path and corrected_path columns use PostGIS GEOGRAPHY(LINESTRING)
/// which we read as GeoJSON strings via ST_AsGeoJSON in queries.
#[derive(Debug, Clone, FromRow)]
pub struct TripPathCorrectionEntity {
    pub id: Uuid,
    pub trip_id: Uuid,
    /// Original path as GeoJSON string (from ST_AsGeoJSON)
    pub original_path: String,
    /// Corrected path as GeoJSON string (from ST_AsGeoJSON), null if not yet corrected
    pub corrected_path: Option<String>,
    /// Confidence metric from map-matching service (0.0-1.0)
    pub correction_quality: Option<f32>,
    /// Status: PENDING, COMPLETED, FAILED, SKIPPED
    pub correction_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TripPathCorrectionEntity {
    /// Convert to domain model.
    pub fn into_domain(self) -> domain::models::TripPathCorrection {
        use domain::models::trip_path_correction::CorrectionStatus;

        let status = self
            .correction_status
            .parse::<CorrectionStatus>()
            .unwrap_or(CorrectionStatus::Pending);

        domain::models::TripPathCorrection {
            id: self.id,
            trip_id: self.trip_id,
            original_path: self.original_path,
            corrected_path: self.corrected_path,
            correction_quality: self.correction_quality,
            correction_status: status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl From<TripPathCorrectionEntity> for domain::models::TripPathCorrection {
    fn from(entity: TripPathCorrectionEntity) -> Self {
        entity.into_domain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::trip_path_correction::CorrectionStatus;

    fn create_test_entity() -> TripPathCorrectionEntity {
        TripPathCorrectionEntity {
            id: Uuid::new_v4(),
            trip_id: Uuid::new_v4(),
            original_path: r#"{"type":"LineString","coordinates":[[-120.0,45.0],[-120.1,45.1]]}"#
                .to_string(),
            corrected_path: None,
            correction_quality: None,
            correction_status: "PENDING".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_entity_to_domain() {
        let entity = create_test_entity();
        let correction: domain::models::TripPathCorrection = entity.clone().into();

        assert_eq!(correction.id, entity.id);
        assert_eq!(correction.trip_id, entity.trip_id);
        assert_eq!(correction.original_path, entity.original_path);
        assert_eq!(correction.corrected_path, None);
        assert_eq!(correction.correction_quality, None);
        assert_eq!(correction.correction_status, CorrectionStatus::Pending);
    }

    #[test]
    fn test_entity_with_completed_status() {
        let mut entity = create_test_entity();
        entity.correction_status = "COMPLETED".to_string();
        entity.corrected_path = Some(
            r#"{"type":"LineString","coordinates":[[-120.0,45.0],[-120.05,45.05],[-120.1,45.1]]}"#
                .to_string(),
        );
        entity.correction_quality = Some(0.95);

        let correction: domain::models::TripPathCorrection = entity.into();
        assert_eq!(correction.correction_status, CorrectionStatus::Completed);
        assert!(correction.corrected_path.is_some());
        assert_eq!(correction.correction_quality, Some(0.95));
    }

    #[test]
    fn test_entity_with_failed_status() {
        let mut entity = create_test_entity();
        entity.correction_status = "FAILED".to_string();

        let correction: domain::models::TripPathCorrection = entity.into();
        assert_eq!(correction.correction_status, CorrectionStatus::Failed);
    }

    #[test]
    fn test_entity_with_skipped_status() {
        let mut entity = create_test_entity();
        entity.correction_status = "SKIPPED".to_string();

        let correction: domain::models::TripPathCorrection = entity.into();
        assert_eq!(correction.correction_status, CorrectionStatus::Skipped);
    }

    #[test]
    fn test_entity_with_unknown_status_defaults_to_pending() {
        let mut entity = create_test_entity();
        entity.correction_status = "INVALID".to_string();

        let correction: domain::models::TripPathCorrection = entity.into();
        assert_eq!(correction.correction_status, CorrectionStatus::Pending);
    }

    #[test]
    fn test_entity_clone() {
        let entity = create_test_entity();
        let cloned = entity.clone();
        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.trip_id, entity.trip_id);
    }

    #[test]
    fn test_entity_debug() {
        let entity = create_test_entity();
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("TripPathCorrectionEntity"));
        assert!(debug_str.contains("PENDING"));
    }
}
