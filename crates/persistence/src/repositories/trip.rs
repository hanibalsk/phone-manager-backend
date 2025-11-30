//! Trip repository for database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::TripEntity;
use crate::metrics::QueryTimer;

/// Input data for inserting a trip record.
#[derive(Debug, Clone)]
pub struct TripInput {
    pub device_id: Uuid,
    pub local_trip_id: String,
    pub start_timestamp: i64,
    pub start_latitude: f64,
    pub start_longitude: f64,
    pub transportation_mode: String,
    pub detection_source: String,
}

/// Input data for updating a trip state.
#[derive(Debug, Clone)]
pub struct TripUpdateInput {
    pub state: String,
    pub end_timestamp: Option<i64>,
    pub end_latitude: Option<f64>,
    pub end_longitude: Option<f64>,
}

/// Query parameters for trip pagination.
#[derive(Debug, Clone)]
pub struct TripQuery {
    pub device_id: Uuid,
    pub cursor_timestamp: Option<i64>,
    pub cursor_id: Option<Uuid>,
    pub from_timestamp: Option<i64>,
    pub to_timestamp: Option<i64>,
    pub state_filter: Option<String>,
    pub limit: i32,
}

/// Repository for trip database operations.
#[derive(Clone)]
pub struct TripRepository {
    pool: PgPool,
}

impl TripRepository {
    /// Creates a new TripRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Create a new trip with idempotency support.
    ///
    /// Uses INSERT ... ON CONFLICT to handle duplicate (device_id, local_trip_id) atomically.
    /// Returns (entity, was_created) tuple.
    pub async fn create_trip(
        &self,
        input: TripInput,
    ) -> Result<(TripEntity, bool), sqlx::Error> {
        let timer = QueryTimer::new("create_trip");

        // Use INSERT ... ON CONFLICT DO NOTHING for atomic idempotency
        // Then fetch the row (either newly created or existing)
        let insert_result = sqlx::query(
            r#"
            INSERT INTO trips (
                device_id, local_trip_id, state, start_timestamp,
                start_location, transportation_mode, detection_source
            )
            VALUES (
                $1, $2, 'ACTIVE', $3,
                ST_SetSRID(ST_MakePoint($4, $5), 4326)::geography,
                $6, $7
            )
            ON CONFLICT (device_id, local_trip_id) DO NOTHING
            "#,
        )
        .bind(input.device_id)
        .bind(&input.local_trip_id)
        .bind(input.start_timestamp)
        .bind(input.start_longitude) // Note: MakePoint takes (x=lon, y=lat)
        .bind(input.start_latitude)
        .bind(&input.transportation_mode)
        .bind(&input.detection_source)
        .execute(&self.pool)
        .await?;

        let was_created = insert_result.rows_affected() > 0;

        // Fetch the trip (whether newly created or existing)
        let entity = self
            .find_by_device_and_local_id(input.device_id, &input.local_trip_id)
            .await?
            .expect("Trip must exist after INSERT ON CONFLICT");

        timer.record();
        Ok((entity, was_created))
    }

    /// Find trip by device_id and local_trip_id.
    pub async fn find_by_device_and_local_id(
        &self,
        device_id: Uuid,
        local_trip_id: &str,
    ) -> Result<Option<TripEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_trip_by_device_local_id");

        let result = sqlx::query_as::<_, TripEntity>(
            r#"
            SELECT
                id, device_id, local_trip_id, state, start_timestamp, end_timestamp,
                ST_Y(start_location::geometry) as start_latitude,
                ST_X(start_location::geometry) as start_longitude,
                CASE WHEN end_location IS NULL THEN NULL ELSE ST_Y(end_location::geometry) END as end_latitude,
                CASE WHEN end_location IS NULL THEN NULL ELSE ST_X(end_location::geometry) END as end_longitude,
                transportation_mode, detection_source, distance_meters, duration_seconds,
                created_at, updated_at
            FROM trips
            WHERE device_id = $1 AND local_trip_id = $2
            "#,
        )
        .bind(device_id)
        .bind(local_trip_id)
        .fetch_optional(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Find trip by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<TripEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_trip_by_id");

        let result = sqlx::query_as::<_, TripEntity>(
            r#"
            SELECT
                id, device_id, local_trip_id, state, start_timestamp, end_timestamp,
                ST_Y(start_location::geometry) as start_latitude,
                ST_X(start_location::geometry) as start_longitude,
                CASE WHEN end_location IS NULL THEN NULL ELSE ST_Y(end_location::geometry) END as end_latitude,
                CASE WHEN end_location IS NULL THEN NULL ELSE ST_X(end_location::geometry) END as end_longitude,
                transportation_mode, detection_source, distance_meters, duration_seconds,
                created_at, updated_at
            FROM trips
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Find active trip for a device.
    pub async fn find_active_for_device(&self, device_id: Uuid) -> Result<Option<TripEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_active_trip_for_device");

        let result = sqlx::query_as::<_, TripEntity>(
            r#"
            SELECT
                id, device_id, local_trip_id, state, start_timestamp, end_timestamp,
                ST_Y(start_location::geometry) as start_latitude,
                ST_X(start_location::geometry) as start_longitude,
                CASE WHEN end_location IS NULL THEN NULL ELSE ST_Y(end_location::geometry) END as end_latitude,
                CASE WHEN end_location IS NULL THEN NULL ELSE ST_X(end_location::geometry) END as end_longitude,
                transportation_mode, detection_source, distance_meters, duration_seconds,
                created_at, updated_at
            FROM trips
            WHERE device_id = $1 AND state = 'ACTIVE'
            "#,
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Update trip state.
    pub async fn update_state(
        &self,
        trip_id: Uuid,
        input: TripUpdateInput,
    ) -> Result<Option<TripEntity>, sqlx::Error> {
        let timer = QueryTimer::new("update_trip_state");

        let result = if input.end_latitude.is_some() && input.end_longitude.is_some() {
            // Update with end location
            sqlx::query_as::<_, TripEntity>(
                r#"
                UPDATE trips
                SET state = $2,
                    end_timestamp = $3,
                    end_location = ST_SetSRID(ST_MakePoint($4, $5), 4326)::geography
                WHERE id = $1
                RETURNING
                    id, device_id, local_trip_id, state, start_timestamp, end_timestamp,
                    ST_Y(start_location::geometry) as start_latitude,
                    ST_X(start_location::geometry) as start_longitude,
                    CASE WHEN end_location IS NULL THEN NULL ELSE ST_Y(end_location::geometry) END as end_latitude,
                    CASE WHEN end_location IS NULL THEN NULL ELSE ST_X(end_location::geometry) END as end_longitude,
                    transportation_mode, detection_source, distance_meters, duration_seconds,
                    created_at, updated_at
                "#,
            )
            .bind(trip_id)
            .bind(&input.state)
            .bind(input.end_timestamp)
            .bind(input.end_longitude.unwrap()) // x = lon
            .bind(input.end_latitude.unwrap())   // y = lat
            .fetch_optional(&self.pool)
            .await
        } else {
            // Update without end location
            sqlx::query_as::<_, TripEntity>(
                r#"
                UPDATE trips
                SET state = $2,
                    end_timestamp = $3
                WHERE id = $1
                RETURNING
                    id, device_id, local_trip_id, state, start_timestamp, end_timestamp,
                    ST_Y(start_location::geometry) as start_latitude,
                    ST_X(start_location::geometry) as start_longitude,
                    CASE WHEN end_location IS NULL THEN NULL ELSE ST_Y(end_location::geometry) END as end_latitude,
                    CASE WHEN end_location IS NULL THEN NULL ELSE ST_X(end_location::geometry) END as end_longitude,
                    transportation_mode, detection_source, distance_meters, duration_seconds,
                    created_at, updated_at
                "#,
            )
            .bind(trip_id)
            .bind(&input.state)
            .bind(input.end_timestamp)
            .fetch_optional(&self.pool)
            .await
        };

        timer.record();
        result
    }

    /// Update trip statistics (distance and duration).
    pub async fn update_statistics(
        &self,
        trip_id: Uuid,
        distance_meters: f64,
        duration_seconds: i64,
    ) -> Result<(), sqlx::Error> {
        let timer = QueryTimer::new("update_trip_statistics");

        sqlx::query(
            r#"
            UPDATE trips
            SET distance_meters = $2, duration_seconds = $3
            WHERE id = $1
            "#,
        )
        .bind(trip_id)
        .bind(distance_meters)
        .bind(duration_seconds)
        .execute(&self.pool)
        .await?;

        timer.record();
        Ok(())
    }

    /// Get trips for a device with pagination.
    pub async fn get_trips_by_device(
        &self,
        query: TripQuery,
    ) -> Result<(Vec<TripEntity>, bool), sqlx::Error> {
        let timer = QueryTimer::new("get_trips_by_device");

        // Fetch limit + 1 to determine if more results exist
        let fetch_limit = (query.limit + 1) as i64;

        let trips = sqlx::query_as::<_, TripEntity>(
            r#"
            SELECT
                id, device_id, local_trip_id, state, start_timestamp, end_timestamp,
                ST_Y(start_location::geometry) as start_latitude,
                ST_X(start_location::geometry) as start_longitude,
                CASE WHEN end_location IS NULL THEN NULL ELSE ST_Y(end_location::geometry) END as end_latitude,
                CASE WHEN end_location IS NULL THEN NULL ELSE ST_X(end_location::geometry) END as end_longitude,
                transportation_mode, detection_source, distance_meters, duration_seconds,
                created_at, updated_at
            FROM trips
            WHERE device_id = $1
              AND ($2::text IS NULL OR state = $2)
              AND ($3::bigint IS NULL OR start_timestamp >= $3)
              AND ($4::bigint IS NULL OR start_timestamp <= $4)
              AND ($5::bigint IS NULL OR (start_timestamp, id) < ($5, $6))
            ORDER BY start_timestamp DESC, id DESC
            LIMIT $7
            "#,
        )
        .bind(query.device_id)
        .bind(&query.state_filter)
        .bind(query.from_timestamp)
        .bind(query.to_timestamp)
        .bind(query.cursor_timestamp)
        // Use max UUID as fallback when cursor_id is None but cursor_timestamp is Some
        // This ensures keyset pagination works correctly
        .bind(query.cursor_id.unwrap_or_else(|| Uuid::from_bytes([0xff; 16])))
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await?;

        timer.record();

        // Check if there are more results
        let has_more = trips.len() > query.limit as usize;
        let mut result = trips;
        if has_more {
            result.pop();
        }

        Ok((result, has_more))
    }

    /// Delete all trips for a device.
    pub async fn delete_all_for_device(&self, device_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM trips
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_trip_input_creation() {
        let input = TripInput {
            device_id: Uuid::new_v4(),
            local_trip_id: "trip-123".to_string(),
            start_timestamp: Utc::now().timestamp_millis(),
            start_latitude: 45.0,
            start_longitude: -120.0,
            transportation_mode: "WALKING".to_string(),
            detection_source: "ACTIVITY_RECOGNITION".to_string(),
        };

        assert_eq!(input.local_trip_id, "trip-123");
        assert!(input.start_latitude > 0.0);
    }

    #[test]
    fn test_trip_update_input_with_location() {
        let input = TripUpdateInput {
            state: "COMPLETED".to_string(),
            end_timestamp: Some(Utc::now().timestamp_millis()),
            end_latitude: Some(45.5),
            end_longitude: Some(-120.5),
        };

        assert_eq!(input.state, "COMPLETED");
        assert!(input.end_latitude.is_some());
    }

    #[test]
    fn test_trip_update_input_without_location() {
        let input = TripUpdateInput {
            state: "CANCELLED".to_string(),
            end_timestamp: None,
            end_latitude: None,
            end_longitude: None,
        };

        assert_eq!(input.state, "CANCELLED");
        assert!(input.end_latitude.is_none());
    }

    #[test]
    fn test_trip_query_creation() {
        let query = TripQuery {
            device_id: Uuid::new_v4(),
            cursor_timestamp: None,
            cursor_id: None,
            from_timestamp: Some(1000),
            to_timestamp: Some(2000),
            state_filter: Some("COMPLETED".to_string()),
            limit: 20,
        };

        assert_eq!(query.limit, 20);
        assert!(query.state_filter.is_some());
    }
}
