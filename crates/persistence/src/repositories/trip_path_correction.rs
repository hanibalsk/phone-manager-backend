//! Trip path correction repository for database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::TripPathCorrectionEntity;
use crate::metrics::QueryTimer;

/// Input data for creating a trip path correction record.
#[derive(Debug, Clone)]
pub struct TripPathCorrectionInput {
    pub trip_id: Uuid,
    /// Original path as array of [lon, lat] coordinate pairs
    pub original_path_coords: Vec<[f64; 2]>,
}

/// Input data for updating a trip path correction.
#[derive(Debug, Clone)]
pub struct TripPathCorrectionUpdateInput {
    pub corrected_path_coords: Option<Vec<[f64; 2]>>,
    pub correction_quality: Option<f32>,
    pub correction_status: String,
}

/// Repository for trip path correction database operations.
#[derive(Clone)]
pub struct TripPathCorrectionRepository {
    pool: PgPool,
}

impl TripPathCorrectionRepository {
    /// Creates a new TripPathCorrectionRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Create a new trip path correction record.
    ///
    /// Converts coordinate array to PostGIS LINESTRING.
    pub async fn create(
        &self,
        input: TripPathCorrectionInput,
    ) -> Result<TripPathCorrectionEntity, sqlx::Error> {
        let timer = QueryTimer::new("create_trip_path_correction");

        // Build WKT LINESTRING from coordinates
        let linestring_wkt = coords_to_linestring_wkt(&input.original_path_coords);

        let result = sqlx::query_as::<_, TripPathCorrectionEntity>(
            r#"
            INSERT INTO trip_path_corrections (
                trip_id, original_path, correction_status
            )
            VALUES (
                $1, ST_GeogFromText($2), 'PENDING'
            )
            RETURNING
                id, trip_id,
                ST_AsGeoJSON(original_path) as original_path,
                CASE WHEN corrected_path IS NULL THEN NULL ELSE ST_AsGeoJSON(corrected_path) END as corrected_path,
                correction_quality, correction_status,
                created_at, updated_at
            "#,
        )
        .bind(input.trip_id)
        .bind(&linestring_wkt)
        .fetch_one(&self.pool)
        .await?;

        timer.record();
        Ok(result)
    }

    /// Find trip path correction by trip ID.
    pub async fn find_by_trip_id(
        &self,
        trip_id: Uuid,
    ) -> Result<Option<TripPathCorrectionEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_trip_path_correction_by_trip_id");

        let result = sqlx::query_as::<_, TripPathCorrectionEntity>(
            r#"
            SELECT
                id, trip_id,
                ST_AsGeoJSON(original_path) as original_path,
                CASE WHEN corrected_path IS NULL THEN NULL ELSE ST_AsGeoJSON(corrected_path) END as corrected_path,
                correction_quality, correction_status,
                created_at, updated_at
            FROM trip_path_corrections
            WHERE trip_id = $1
            "#,
        )
        .bind(trip_id)
        .fetch_optional(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Find trip path correction by ID.
    pub async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<TripPathCorrectionEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_trip_path_correction_by_id");

        let result = sqlx::query_as::<_, TripPathCorrectionEntity>(
            r#"
            SELECT
                id, trip_id,
                ST_AsGeoJSON(original_path) as original_path,
                CASE WHEN corrected_path IS NULL THEN NULL ELSE ST_AsGeoJSON(corrected_path) END as corrected_path,
                correction_quality, correction_status,
                created_at, updated_at
            FROM trip_path_corrections
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Find all trip path corrections with a specific status.
    pub async fn find_by_status(
        &self,
        status: &str,
        limit: i64,
    ) -> Result<Vec<TripPathCorrectionEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_trip_path_corrections_by_status");

        let result = sqlx::query_as::<_, TripPathCorrectionEntity>(
            r#"
            SELECT
                id, trip_id,
                ST_AsGeoJSON(original_path) as original_path,
                CASE WHEN corrected_path IS NULL THEN NULL ELSE ST_AsGeoJSON(corrected_path) END as corrected_path,
                correction_quality, correction_status,
                created_at, updated_at
            FROM trip_path_corrections
            WHERE correction_status = $1
            ORDER BY created_at ASC
            LIMIT $2
            "#,
        )
        .bind(status)
        .bind(limit)
        .fetch_all(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Update trip path correction status and optionally the corrected path.
    pub async fn update(
        &self,
        trip_id: Uuid,
        input: TripPathCorrectionUpdateInput,
    ) -> Result<Option<TripPathCorrectionEntity>, sqlx::Error> {
        let timer = QueryTimer::new("update_trip_path_correction");

        let result = if let Some(ref coords) = input.corrected_path_coords {
            // Update with corrected path
            let linestring_wkt = coords_to_linestring_wkt(coords);

            sqlx::query_as::<_, TripPathCorrectionEntity>(
                r#"
                UPDATE trip_path_corrections
                SET corrected_path = ST_GeogFromText($2),
                    correction_quality = $3,
                    correction_status = $4
                WHERE trip_id = $1
                RETURNING
                    id, trip_id,
                    ST_AsGeoJSON(original_path) as original_path,
                    CASE WHEN corrected_path IS NULL THEN NULL ELSE ST_AsGeoJSON(corrected_path) END as corrected_path,
                    correction_quality, correction_status,
                    created_at, updated_at
                "#,
            )
            .bind(trip_id)
            .bind(&linestring_wkt)
            .bind(input.correction_quality)
            .bind(&input.correction_status)
            .fetch_optional(&self.pool)
            .await
        } else {
            // Update status only (no corrected path)
            sqlx::query_as::<_, TripPathCorrectionEntity>(
                r#"
                UPDATE trip_path_corrections
                SET correction_status = $2
                WHERE trip_id = $1
                RETURNING
                    id, trip_id,
                    ST_AsGeoJSON(original_path) as original_path,
                    CASE WHEN corrected_path IS NULL THEN NULL ELSE ST_AsGeoJSON(corrected_path) END as corrected_path,
                    correction_quality, correction_status,
                    created_at, updated_at
                "#,
            )
            .bind(trip_id)
            .bind(&input.correction_status)
            .fetch_optional(&self.pool)
            .await
        };

        timer.record();
        result
    }

    /// Delete trip path correction by trip ID.
    pub async fn delete_by_trip_id(&self, trip_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM trip_path_corrections
            WHERE trip_id = $1
            "#,
        )
        .bind(trip_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Create a SKIPPED trip path correction record.
    ///
    /// Used when path correction cannot be performed (e.g., insufficient locations).
    /// Handles 0 or 1 coordinate gracefully by creating a valid degenerate LINESTRING.
    pub async fn create_skipped(
        &self,
        trip_id: Uuid,
        coords: &[[f64; 2]],
    ) -> Result<TripPathCorrectionEntity, sqlx::Error> {
        let timer = QueryTimer::new("create_trip_path_correction_skipped");

        // Build WKT LINESTRING from coordinates (handles 0 or 1 point)
        let linestring_wkt = coords_to_linestring_wkt(coords);

        let result = sqlx::query_as::<_, TripPathCorrectionEntity>(
            r#"
            INSERT INTO trip_path_corrections (
                trip_id, original_path, correction_status
            )
            VALUES (
                $1, ST_GeogFromText($2), 'SKIPPED'
            )
            RETURNING
                id, trip_id,
                ST_AsGeoJSON(original_path) as original_path,
                CASE WHEN corrected_path IS NULL THEN NULL ELSE ST_AsGeoJSON(corrected_path) END as corrected_path,
                correction_quality, correction_status,
                created_at, updated_at
            "#,
        )
        .bind(trip_id)
        .bind(&linestring_wkt)
        .fetch_one(&self.pool)
        .await?;

        timer.record();
        Ok(result)
    }
}

/// Convert coordinate array to WKT LINESTRING format.
///
/// Expects coordinates as [longitude, latitude] pairs.
/// Handles edge cases:
/// - 0 coords: Creates a degenerate line at origin (0 0, 0 0)
/// - 1 coord: Duplicates the point to create a valid line
/// - 2+ coords: Normal LINESTRING
fn coords_to_linestring_wkt(coords: &[[f64; 2]]) -> String {
    let points: Vec<String> = if coords.is_empty() {
        // Create a degenerate line at origin for 0 locations
        vec!["0 0".to_string(), "0 0".to_string()]
    } else if coords.len() == 1 {
        // Duplicate single point to create valid LINESTRING
        let [lon, lat] = coords[0];
        let point = format!("{} {}", lon, lat);
        vec![point.clone(), point]
    } else {
        coords
            .iter()
            .map(|[lon, lat]| format!("{} {}", lon, lat))
            .collect()
    };
    format!("SRID=4326;LINESTRING({})", points.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coords_to_linestring_wkt() {
        let coords = vec![[-120.0, 45.0], [-120.1, 45.1], [-120.2, 45.2]];
        let wkt = coords_to_linestring_wkt(&coords);
        assert_eq!(
            wkt,
            "SRID=4326;LINESTRING(-120 45, -120.1 45.1, -120.2 45.2)"
        );
    }

    #[test]
    fn test_coords_to_linestring_wkt_two_points() {
        let coords = vec![[-120.0, 45.0], [-120.1, 45.1]];
        let wkt = coords_to_linestring_wkt(&coords);
        assert_eq!(wkt, "SRID=4326;LINESTRING(-120 45, -120.1 45.1)");
    }

    #[test]
    fn test_coords_to_linestring_wkt_one_point() {
        // Single point should be duplicated to form valid LINESTRING
        let coords = vec![[-120.0, 45.0]];
        let wkt = coords_to_linestring_wkt(&coords);
        assert_eq!(wkt, "SRID=4326;LINESTRING(-120 45, -120 45)");
    }

    #[test]
    fn test_coords_to_linestring_wkt_empty() {
        // Empty coords should produce degenerate line at origin
        let coords: Vec<[f64; 2]> = vec![];
        let wkt = coords_to_linestring_wkt(&coords);
        assert_eq!(wkt, "SRID=4326;LINESTRING(0 0, 0 0)");
    }

    #[test]
    fn test_trip_path_correction_input_creation() {
        let input = TripPathCorrectionInput {
            trip_id: Uuid::new_v4(),
            original_path_coords: vec![[-120.0, 45.0], [-120.1, 45.1]],
        };

        assert_eq!(input.original_path_coords.len(), 2);
    }

    #[test]
    fn test_trip_path_correction_update_input_with_correction() {
        let input = TripPathCorrectionUpdateInput {
            corrected_path_coords: Some(vec![[-120.0, 45.0], [-120.05, 45.05], [-120.1, 45.1]]),
            correction_quality: Some(0.95),
            correction_status: "COMPLETED".to_string(),
        };

        assert!(input.corrected_path_coords.is_some());
        assert_eq!(input.correction_quality, Some(0.95));
    }

    #[test]
    fn test_trip_path_correction_update_input_failed() {
        let input = TripPathCorrectionUpdateInput {
            corrected_path_coords: None,
            correction_quality: None,
            correction_status: "FAILED".to_string(),
        };

        assert!(input.corrected_path_coords.is_none());
        assert_eq!(input.correction_status, "FAILED");
    }
}
