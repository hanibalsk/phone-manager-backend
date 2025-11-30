//! Movement event repository for database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::MovementEventEntity;
use crate::metrics::QueryTimer;

/// Input data for inserting a movement event record.
#[derive(Debug, Clone)]
pub struct MovementEventInput {
    pub device_id: Uuid,
    pub trip_id: Option<Uuid>,
    pub timestamp: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub speed: Option<f64>,
    pub bearing: Option<f64>,
    pub altitude: Option<f64>,
    pub transportation_mode: String,
    pub confidence: f64,
    pub detection_source: String,
}

/// Repository for movement event database operations.
#[derive(Clone)]
pub struct MovementEventRepository {
    pool: PgPool,
}

impl MovementEventRepository {
    /// Creates a new MovementEventRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Insert a single movement event.
    ///
    /// Uses PostGIS ST_SetSRID and ST_MakePoint to create the geography point.
    pub async fn insert_event(
        &self,
        input: MovementEventInput,
    ) -> Result<MovementEventEntity, sqlx::Error> {
        let timer = QueryTimer::new("insert_movement_event");

        let result = sqlx::query_as::<_, MovementEventEntity>(
            r#"
            INSERT INTO movement_events (
                device_id, trip_id, timestamp, location, accuracy, speed, bearing,
                altitude, transportation_mode, confidence, detection_source
            )
            VALUES (
                $1, $2, $3,
                ST_SetSRID(ST_MakePoint($4, $5), 4326)::geography,
                $6, $7, $8, $9, $10, $11, $12
            )
            RETURNING
                id, device_id, trip_id, timestamp,
                ST_Y(location::geometry) as latitude,
                ST_X(location::geometry) as longitude,
                accuracy, speed, bearing, altitude,
                transportation_mode, confidence, detection_source, created_at
            "#,
        )
        .bind(input.device_id)
        .bind(input.trip_id)
        .bind(input.timestamp)
        .bind(input.longitude) // Note: MakePoint takes (x=lon, y=lat)
        .bind(input.latitude)
        .bind(input.accuracy as f32)
        .bind(input.speed.map(|s| s as f32))
        .bind(input.bearing.map(|b| b as f32))
        .bind(input.altitude)
        .bind(&input.transportation_mode)
        .bind(input.confidence as f32)
        .bind(&input.detection_source)
        .fetch_one(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Insert multiple movement events in a batch (within a transaction).
    pub async fn insert_events_batch(
        &self,
        events: Vec<MovementEventInput>,
    ) -> Result<usize, sqlx::Error> {
        let timer = QueryTimer::new("insert_movement_events_batch");
        let mut tx = self.pool.begin().await?;
        let count = events.len();

        for event in &events {
            sqlx::query(
                r#"
                INSERT INTO movement_events (
                    device_id, trip_id, timestamp, location, accuracy, speed, bearing,
                    altitude, transportation_mode, confidence, detection_source
                )
                VALUES (
                    $1, $2, $3,
                    ST_SetSRID(ST_MakePoint($4, $5), 4326)::geography,
                    $6, $7, $8, $9, $10, $11, $12
                )
                "#,
            )
            .bind(event.device_id)
            .bind(event.trip_id)
            .bind(event.timestamp)
            .bind(event.longitude) // Note: MakePoint takes (x=lon, y=lat)
            .bind(event.latitude)
            .bind(event.accuracy as f32)
            .bind(event.speed.map(|s| s as f32))
            .bind(event.bearing.map(|b| b as f32))
            .bind(event.altitude)
            .bind(&event.transportation_mode)
            .bind(event.confidence as f32)
            .bind(&event.detection_source)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        timer.record();
        Ok(count)
    }

    /// Get a movement event by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<MovementEventEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_movement_event_by_id");

        let result = sqlx::query_as::<_, MovementEventEntity>(
            r#"
            SELECT
                id, device_id, trip_id, timestamp,
                ST_Y(location::geometry) as latitude,
                ST_X(location::geometry) as longitude,
                accuracy, speed, bearing, altitude,
                transportation_mode, confidence, detection_source, created_at
            FROM movement_events
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Get movement events for a device with pagination.
    ///
    /// Returns `(events, has_more)` tuple.
    pub async fn get_events_by_device(
        &self,
        query: MovementEventQuery,
    ) -> Result<(Vec<MovementEventEntity>, bool), sqlx::Error> {
        let timer = QueryTimer::new("get_movement_events_by_device");

        // Fetch limit + 1 to determine if more results exist
        let fetch_limit = (query.limit + 1) as i64;

        let events = if query.ascending {
            self.get_events_asc(&query, fetch_limit).await?
        } else {
            self.get_events_desc(&query, fetch_limit).await?
        };

        timer.record();

        // Check if there are more results
        let has_more = events.len() > query.limit as usize;
        let mut result = events;
        if has_more {
            result.pop(); // Remove the extra record
        }

        Ok((result, has_more))
    }

    /// Get events in descending order (newest first).
    async fn get_events_desc(
        &self,
        query: &MovementEventQuery,
        fetch_limit: i64,
    ) -> Result<Vec<MovementEventEntity>, sqlx::Error> {
        sqlx::query_as::<_, MovementEventEntity>(
            r#"
            SELECT
                id, device_id, trip_id, timestamp,
                ST_Y(location::geometry) as latitude,
                ST_X(location::geometry) as longitude,
                accuracy, speed, bearing, altitude,
                transportation_mode, confidence, detection_source, created_at
            FROM movement_events
            WHERE device_id = $1
              AND ($2::bigint IS NULL OR timestamp >= $2)
              AND ($3::bigint IS NULL OR timestamp <= $3)
              AND ($4::bigint IS NULL OR (timestamp, id) < ($4, $5))
            ORDER BY timestamp DESC, id DESC
            LIMIT $6
            "#,
        )
        .bind(query.device_id)
        .bind(query.from_timestamp)
        .bind(query.to_timestamp)
        .bind(query.cursor_timestamp)
        .bind(query.cursor_id.unwrap_or(Uuid::max()))
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Get events in ascending order (oldest first).
    async fn get_events_asc(
        &self,
        query: &MovementEventQuery,
        fetch_limit: i64,
    ) -> Result<Vec<MovementEventEntity>, sqlx::Error> {
        sqlx::query_as::<_, MovementEventEntity>(
            r#"
            SELECT
                id, device_id, trip_id, timestamp,
                ST_Y(location::geometry) as latitude,
                ST_X(location::geometry) as longitude,
                accuracy, speed, bearing, altitude,
                transportation_mode, confidence, detection_source, created_at
            FROM movement_events
            WHERE device_id = $1
              AND ($2::bigint IS NULL OR timestamp >= $2)
              AND ($3::bigint IS NULL OR timestamp <= $3)
              AND ($4::bigint IS NULL OR (timestamp, id) > ($4, $5))
            ORDER BY timestamp ASC, id ASC
            LIMIT $6
            "#,
        )
        .bind(query.device_id)
        .bind(query.from_timestamp)
        .bind(query.to_timestamp)
        .bind(query.cursor_timestamp)
        .bind(query.cursor_id.unwrap_or(Uuid::nil()))
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Delete all movement events for a device.
    /// Returns the number of deleted records.
    pub async fn delete_all_for_device(&self, device_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM movement_events
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get all movement events for a trip, ordered by timestamp ascending.
    /// Used for trip statistics calculation (distance, duration).
    pub async fn get_events_for_trip(
        &self,
        trip_id: Uuid,
    ) -> Result<Vec<MovementEventEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_events_for_trip");

        let result = sqlx::query_as::<_, MovementEventEntity>(
            r#"
            SELECT
                id, device_id, trip_id, timestamp,
                ST_Y(location::geometry) as latitude,
                ST_X(location::geometry) as longitude,
                accuracy, speed, bearing, altitude,
                transportation_mode, confidence, detection_source, created_at
            FROM movement_events
            WHERE trip_id = $1
            ORDER BY timestamp ASC, id ASC
            "#,
        )
        .bind(trip_id)
        .fetch_all(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Calculate total distance for a trip using PostGIS ST_Distance.
    /// Returns distance in meters, summing point-to-point distances.
    pub async fn calculate_trip_distance(&self, trip_id: Uuid) -> Result<f64, sqlx::Error> {
        let timer = QueryTimer::new("calculate_trip_distance");

        let result: (Option<f64>,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(distance), 0) as total_distance
            FROM (
                SELECT ST_Distance(
                    location,
                    LAG(location) OVER (ORDER BY timestamp, id)
                ) as distance
                FROM movement_events
                WHERE trip_id = $1
            ) distances
            "#,
        )
        .bind(trip_id)
        .fetch_one(&self.pool)
        .await?;

        timer.record();
        Ok(result.0.unwrap_or(0.0))
    }
}

/// Query parameters for movement event pagination.
#[derive(Debug, Clone)]
pub struct MovementEventQuery {
    /// Device ID to fetch events for.
    pub device_id: Uuid,
    /// Cursor timestamp (for pagination).
    pub cursor_timestamp: Option<i64>,
    /// Cursor ID (for pagination).
    pub cursor_id: Option<Uuid>,
    /// Start timestamp filter.
    pub from_timestamp: Option<i64>,
    /// End timestamp filter.
    pub to_timestamp: Option<i64>,
    /// Number of results to return.
    pub limit: i32,
    /// Whether to sort in ascending order.
    pub ascending: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_movement_event_input_creation() {
        let input = MovementEventInput {
            device_id: Uuid::new_v4(),
            trip_id: Some(Uuid::new_v4()),
            timestamp: Utc::now().timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            speed: Some(5.5),
            bearing: Some(180.0),
            altitude: Some(100.0),
            transportation_mode: "WALKING".to_string(),
            confidence: 0.95,
            detection_source: "ACTIVITY_RECOGNITION".to_string(),
        };

        assert!(input.latitude > 0.0);
        assert!(input.longitude < 0.0);
        assert_eq!(input.accuracy, 10.0);
        assert_eq!(input.transportation_mode, "WALKING");
    }

    #[test]
    fn test_movement_event_input_minimal() {
        let input = MovementEventInput {
            device_id: Uuid::new_v4(),
            trip_id: None,
            timestamp: Utc::now().timestamp_millis(),
            latitude: 0.0,
            longitude: 0.0,
            accuracy: 0.0,
            speed: None,
            bearing: None,
            altitude: None,
            transportation_mode: "STATIONARY".to_string(),
            confidence: 0.0,
            detection_source: "NONE".to_string(),
        };

        assert!(input.trip_id.is_none());
        assert!(input.speed.is_none());
        assert!(input.bearing.is_none());
        assert!(input.altitude.is_none());
    }

    #[test]
    fn test_movement_event_query_creation() {
        let query = MovementEventQuery {
            device_id: Uuid::new_v4(),
            cursor_timestamp: None,
            cursor_id: None,
            from_timestamp: Some(1000),
            to_timestamp: Some(2000),
            limit: 50,
            ascending: false,
        };

        assert_eq!(query.limit, 50);
        assert!(!query.ascending);
        assert!(query.from_timestamp.is_some());
        assert!(query.to_timestamp.is_some());
    }

    #[test]
    fn test_movement_event_query_with_cursor() {
        let query = MovementEventQuery {
            device_id: Uuid::new_v4(),
            cursor_timestamp: Some(1234567890),
            cursor_id: Some(Uuid::new_v4()),
            from_timestamp: None,
            to_timestamp: None,
            limit: 25,
            ascending: true,
        };

        assert!(query.cursor_timestamp.is_some());
        assert!(query.cursor_id.is_some());
        assert!(query.ascending);
    }

    #[test]
    fn test_movement_event_input_clone() {
        let input = MovementEventInput {
            device_id: Uuid::new_v4(),
            trip_id: Some(Uuid::new_v4()),
            timestamp: Utc::now().timestamp_millis(),
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            speed: Some(5.5),
            bearing: Some(180.0),
            altitude: Some(100.0),
            transportation_mode: "WALKING".to_string(),
            confidence: 0.95,
            detection_source: "ACTIVITY_RECOGNITION".to_string(),
        };

        let cloned = input.clone();
        assert_eq!(cloned.device_id, input.device_id);
        assert_eq!(cloned.latitude, input.latitude);
        assert_eq!(cloned.longitude, input.longitude);
    }

    #[test]
    fn test_movement_event_input_debug() {
        let input = MovementEventInput {
            device_id: Uuid::new_v4(),
            trip_id: None,
            timestamp: 1234567890,
            latitude: 45.0,
            longitude: -120.0,
            accuracy: 10.0,
            speed: None,
            bearing: None,
            altitude: None,
            transportation_mode: "WALKING".to_string(),
            confidence: 0.95,
            detection_source: "ACTIVITY_RECOGNITION".to_string(),
        };

        let debug_str = format!("{:?}", input);
        assert!(debug_str.contains("MovementEventInput"));
        assert!(debug_str.contains("WALKING"));
    }
}
