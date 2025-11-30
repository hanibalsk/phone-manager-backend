//! Location repository for database operations.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::LocationEntity;
use crate::metrics::QueryTimer;

/// Input data for inserting a location record.
#[derive(Debug, Clone)]
pub struct LocationInput {
    pub device_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub altitude: Option<f64>,
    pub bearing: Option<f64>,
    pub speed: Option<f64>,
    pub provider: Option<String>,
    pub battery_level: Option<i32>,
    pub network_type: Option<String>,
    pub captured_at: DateTime<Utc>,
}

/// Repository for location-related database operations.
#[derive(Clone)]
pub struct LocationRepository {
    pool: PgPool,
}

impl LocationRepository {
    /// Creates a new LocationRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Insert a single location record.
    pub async fn insert_location(&self, input: LocationInput) -> Result<LocationEntity, sqlx::Error> {
        let timer = QueryTimer::new("insert_location");
        let result = sqlx::query_as::<_, LocationEntity>(
            r#"
            INSERT INTO locations (
                device_id, latitude, longitude, accuracy, altitude, bearing,
                speed, provider, battery_level, network_type, captured_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, device_id, latitude, longitude, accuracy, altitude, bearing,
                      speed, provider, battery_level, network_type, captured_at, created_at,
                      transportation_mode, detection_source, trip_id
            "#,
        )
        .bind(input.device_id)
        .bind(input.latitude)
        .bind(input.longitude)
        .bind(input.accuracy as f32) // accuracy is REAL (f32) in schema
        .bind(input.altitude)
        .bind(input.bearing.map(|b| b as f32)) // bearing is REAL (f32) in schema
        .bind(input.speed.map(|s| s as f32)) // speed is REAL (f32) in schema
        .bind(&input.provider)
        .bind(input.battery_level.map(|b| b as i16)) // battery_level is SMALLINT in schema
        .bind(&input.network_type)
        .bind(input.captured_at)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Insert multiple locations in a batch (within a transaction).
    pub async fn insert_locations_batch(
        &self,
        device_id: Uuid,
        locations: Vec<LocationInput>,
    ) -> Result<usize, sqlx::Error> {
        let timer = QueryTimer::new("insert_locations_batch");
        let mut tx = self.pool.begin().await?;
        let count = locations.len();

        for loc in &locations {
            sqlx::query(
                r#"
                INSERT INTO locations (
                    device_id, latitude, longitude, accuracy, altitude, bearing,
                    speed, provider, battery_level, network_type, captured_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
            )
            .bind(device_id)
            .bind(loc.latitude)
            .bind(loc.longitude)
            .bind(loc.accuracy as f32) // accuracy
            .bind(loc.altitude) // altitude
            .bind(loc.bearing.map(|b| b as f32)) // bearing
            .bind(loc.speed.map(|s| s as f32)) // speed
            .bind(&loc.provider) // provider
            .bind(loc.battery_level.map(|b| b as i16)) // battery_level
            .bind(&loc.network_type) // network_type
            .bind(loc.captured_at) // captured_at
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        timer.record();
        Ok(count)
    }

    /// Delete locations older than specified retention days.
    /// Returns the number of deleted records.
    pub async fn delete_old_locations(&self, retention_days: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM locations
            WHERE created_at < NOW() - make_interval(days => $1)
            "#,
        )
        .bind(retention_days as i32)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get all locations for a device (for data export).
    /// Returns locations sorted by captured_at in descending order.
    pub async fn get_all_locations_for_device(
        &self,
        device_id: Uuid,
    ) -> Result<Vec<LocationEntity>, sqlx::Error> {
        sqlx::query_as::<_, LocationEntity>(
            r#"
            SELECT id, device_id, latitude, longitude, accuracy, altitude, bearing,
                   speed, provider, battery_level, network_type, captured_at, created_at,
                   transportation_mode, detection_source, trip_id
            FROM locations
            WHERE device_id = $1
            ORDER BY captured_at DESC
            "#,
        )
        .bind(device_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Delete all locations for a device (hard delete for GDPR).
    /// Returns the number of deleted records.
    pub async fn delete_all_locations_for_device(
        &self,
        device_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM locations
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get location history for a device with cursor-based pagination.
    ///
    /// Returns `(locations, has_more)` tuple. The `has_more` flag indicates
    /// if there are more results available after the current page.
    ///
    /// # Arguments
    /// * `query` - Query parameters including cursor, limit, filters, and sort order
    ///
    /// # Returns
    /// * Vector of location entities and a boolean indicating if more results exist
    pub async fn get_location_history(
        &self,
        query: LocationHistoryQuery,
    ) -> Result<(Vec<LocationEntity>, bool), sqlx::Error> {
        let timer = QueryTimer::new("get_location_history");

        // Fetch limit + 1 to determine if more results exist
        let fetch_limit = (query.limit + 1) as i64;

        let locations = if query.ascending {
            self.get_location_history_asc(&query, fetch_limit).await?
        } else {
            self.get_location_history_desc(&query, fetch_limit).await?
        };

        timer.record();

        // Check if there are more results
        let has_more = locations.len() > query.limit as usize;
        let mut result = locations;
        if has_more {
            result.pop(); // Remove the extra record
        }

        Ok((result, has_more))
    }

    /// Get location history in descending order (newest first).
    async fn get_location_history_desc(
        &self,
        query: &LocationHistoryQuery,
        fetch_limit: i64,
    ) -> Result<Vec<LocationEntity>, sqlx::Error> {
        sqlx::query_as::<_, LocationEntity>(
            r#"
            SELECT id, device_id, latitude, longitude, accuracy, altitude, bearing,
                   speed, provider, battery_level, network_type, captured_at, created_at,
                   transportation_mode, detection_source, trip_id
            FROM locations
            WHERE device_id = $1
              AND ($2::timestamptz IS NULL OR captured_at >= $2)
              AND ($3::timestamptz IS NULL OR captured_at <= $3)
              AND ($4::timestamptz IS NULL OR (captured_at, id) < ($4, $5))
            ORDER BY captured_at DESC, id DESC
            LIMIT $6
            "#,
        )
        .bind(query.device_id)
        .bind(query.from_timestamp) // lower bound (filter: show records >= from)
        .bind(query.to_timestamp) // upper bound (filter: show records <= to)
        .bind(query.cursor_timestamp)
        .bind(query.cursor_id.unwrap_or(i64::MAX))
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Get location history in ascending order (oldest first).
    async fn get_location_history_asc(
        &self,
        query: &LocationHistoryQuery,
        fetch_limit: i64,
    ) -> Result<Vec<LocationEntity>, sqlx::Error> {
        sqlx::query_as::<_, LocationEntity>(
            r#"
            SELECT id, device_id, latitude, longitude, accuracy, altitude, bearing,
                   speed, provider, battery_level, network_type, captured_at, created_at,
                   transportation_mode, detection_source, trip_id
            FROM locations
            WHERE device_id = $1
              AND ($2::timestamptz IS NULL OR captured_at >= $2)
              AND ($3::timestamptz IS NULL OR captured_at <= $3)
              AND ($4::timestamptz IS NULL OR (captured_at, id) > ($4, $5))
            ORDER BY captured_at ASC, id ASC
            LIMIT $6
            "#,
        )
        .bind(query.device_id)
        .bind(query.from_timestamp) // lower bound for ASC (filter: show records >= from)
        .bind(query.to_timestamp) // upper bound for ASC (filter: show records <= to)
        .bind(query.cursor_timestamp)
        .bind(query.cursor_id.unwrap_or(i64::MIN))
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Get all locations for a device within a time range (no pagination).
    ///
    /// Used for simplification operations that need the complete path.
    /// Always returns in ascending order (oldest to newest) for proper
    /// trajectory simplification.
    ///
    /// # Arguments
    /// * `device_id` - Device to fetch locations for
    /// * `from_timestamp` - Optional start timestamp filter
    /// * `to_timestamp` - Optional end timestamp filter
    ///
    /// # Returns
    /// * Vector of location entities ordered by captured_at ASC
    pub async fn get_all_locations_in_range(
        &self,
        device_id: Uuid,
        from_timestamp: Option<DateTime<Utc>>,
        to_timestamp: Option<DateTime<Utc>>,
    ) -> Result<Vec<LocationEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_all_locations_in_range");

        let result = sqlx::query_as::<_, LocationEntity>(
            r#"
            SELECT id, device_id, latitude, longitude, accuracy, altitude, bearing,
                   speed, provider, battery_level, network_type, captured_at, created_at,
                   transportation_mode, detection_source, trip_id
            FROM locations
            WHERE device_id = $1
              AND ($2::timestamptz IS NULL OR captured_at >= $2)
              AND ($3::timestamptz IS NULL OR captured_at <= $3)
            ORDER BY captured_at ASC, id ASC
            "#,
        )
        .bind(device_id)
        .bind(from_timestamp)
        .bind(to_timestamp)
        .fetch_all(&self.pool)
        .await;

        timer.record();
        result
    }
}

/// Query parameters for location history with cursor-based pagination.
#[derive(Debug, Clone)]
pub struct LocationHistoryQuery {
    /// Device ID to fetch locations for.
    pub device_id: Uuid,
    /// Cursor timestamp (from decoded cursor).
    pub cursor_timestamp: Option<DateTime<Utc>>,
    /// Cursor ID (from decoded cursor).
    pub cursor_id: Option<i64>,
    /// Start timestamp filter.
    pub from_timestamp: Option<DateTime<Utc>>,
    /// End timestamp filter.
    pub to_timestamp: Option<DateTime<Utc>>,
    /// Number of results to return.
    pub limit: i32,
    /// Whether to sort in ascending order.
    pub ascending: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // ===========================================
    // LocationInput Tests
    // ===========================================

    #[test]
    fn test_location_input_creation() {
        let input = LocationInput {
            device_id: Uuid::new_v4(),
            latitude: 45.0,
            longitude: -122.0,
            accuracy: 10.0,
            altitude: Some(100.0),
            bearing: Some(90.0),
            speed: Some(5.5),
            provider: Some("gps".to_string()),
            battery_level: Some(85),
            network_type: Some("wifi".to_string()),
            captured_at: Utc::now(),
        };

        assert!(input.latitude > 0.0);
        assert!(input.longitude < 0.0);
        assert_eq!(input.accuracy, 10.0);
    }

    #[test]
    fn test_location_input_minimal() {
        // Minimum required fields
        let input = LocationInput {
            device_id: Uuid::new_v4(),
            latitude: 0.0,
            longitude: 0.0,
            accuracy: 1.0,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
            captured_at: Utc::now(),
        };

        assert!(input.altitude.is_none());
        assert!(input.bearing.is_none());
        assert!(input.speed.is_none());
    }

    #[test]
    fn test_location_input_boundary_coordinates() {
        // Test with boundary coordinate values
        let input = LocationInput {
            device_id: Uuid::new_v4(),
            latitude: 90.0,  // Max latitude
            longitude: 180.0,  // Max longitude
            accuracy: 0.0,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
            captured_at: Utc::now(),
        };

        assert_eq!(input.latitude, 90.0);
        assert_eq!(input.longitude, 180.0);
    }

    #[test]
    fn test_location_input_negative_coordinates() {
        let input = LocationInput {
            device_id: Uuid::new_v4(),
            latitude: -90.0,  // Min latitude
            longitude: -180.0,  // Min longitude
            accuracy: 5.0,
            altitude: Some(-100.0),  // Below sea level
            bearing: None,
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
            captured_at: Utc::now(),
        };

        assert_eq!(input.latitude, -90.0);
        assert_eq!(input.longitude, -180.0);
        assert_eq!(input.altitude, Some(-100.0));
    }

    #[test]
    fn test_location_input_clone() {
        let input = LocationInput {
            device_id: Uuid::new_v4(),
            latitude: 45.0,
            longitude: -122.0,
            accuracy: 10.0,
            altitude: Some(100.0),
            bearing: Some(90.0),
            speed: Some(5.5),
            provider: Some("gps".to_string()),
            battery_level: Some(85),
            network_type: Some("wifi".to_string()),
            captured_at: Utc::now(),
        };

        let cloned = input.clone();
        assert_eq!(cloned.latitude, input.latitude);
        assert_eq!(cloned.longitude, input.longitude);
        assert_eq!(cloned.device_id, input.device_id);
    }

    #[test]
    fn test_location_input_debug() {
        let input = LocationInput {
            device_id: Uuid::new_v4(),
            latitude: 45.0,
            longitude: -122.0,
            accuracy: 10.0,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
            captured_at: Utc::now(),
        };

        let debug = format!("{:?}", input);
        assert!(debug.contains("LocationInput"));
        assert!(debug.contains("latitude"));
        assert!(debug.contains("longitude"));
    }

    #[test]
    fn test_location_input_battery_boundaries() {
        // Battery can be 0-100
        let input_low = LocationInput {
            device_id: Uuid::new_v4(),
            latitude: 0.0,
            longitude: 0.0,
            accuracy: 1.0,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: Some(0),
            network_type: None,
            captured_at: Utc::now(),
        };
        assert_eq!(input_low.battery_level, Some(0));

        let input_high = LocationInput {
            device_id: Uuid::new_v4(),
            latitude: 0.0,
            longitude: 0.0,
            accuracy: 1.0,
            altitude: None,
            bearing: None,
            speed: None,
            provider: None,
            battery_level: Some(100),
            network_type: None,
            captured_at: Utc::now(),
        };
        assert_eq!(input_high.battery_level, Some(100));
    }

    #[test]
    fn test_location_input_bearing_boundaries() {
        // Bearing is 0-360 degrees
        let input_zero = LocationInput {
            device_id: Uuid::new_v4(),
            latitude: 0.0,
            longitude: 0.0,
            accuracy: 1.0,
            altitude: None,
            bearing: Some(0.0),
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
            captured_at: Utc::now(),
        };
        assert_eq!(input_zero.bearing, Some(0.0));

        let input_max = LocationInput {
            device_id: Uuid::new_v4(),
            latitude: 0.0,
            longitude: 0.0,
            accuracy: 1.0,
            altitude: None,
            bearing: Some(359.99),
            speed: None,
            provider: None,
            battery_level: None,
            network_type: None,
            captured_at: Utc::now(),
        };
        assert!(input_max.bearing.unwrap() < 360.0);
    }

    // ===========================================
    // LocationHistoryQuery Tests
    // ===========================================

    #[test]
    fn test_location_history_query_minimal() {
        let query = LocationHistoryQuery {
            device_id: Uuid::new_v4(),
            cursor_timestamp: None,
            cursor_id: None,
            from_timestamp: None,
            to_timestamp: None,
            limit: 50,
            ascending: false,
        };

        assert_eq!(query.limit, 50);
        assert!(!query.ascending);
        assert!(query.cursor_timestamp.is_none());
    }

    #[test]
    fn test_location_history_query_with_filters() {
        let from = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let to = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let query = LocationHistoryQuery {
            device_id: Uuid::new_v4(),
            cursor_timestamp: None,
            cursor_id: None,
            from_timestamp: Some(from),
            to_timestamp: Some(to),
            limit: 100,
            ascending: true,
        };

        assert!(query.from_timestamp.is_some());
        assert!(query.to_timestamp.is_some());
        assert!(query.ascending);
    }

    #[test]
    fn test_location_history_query_with_cursor() {
        let cursor_time = Utc::now();

        let query = LocationHistoryQuery {
            device_id: Uuid::new_v4(),
            cursor_timestamp: Some(cursor_time),
            cursor_id: Some(12345),
            from_timestamp: None,
            to_timestamp: None,
            limit: 25,
            ascending: false,
        };

        assert_eq!(query.cursor_id, Some(12345));
        assert!(query.cursor_timestamp.is_some());
    }

    #[test]
    fn test_location_history_query_clone() {
        let query = LocationHistoryQuery {
            device_id: Uuid::new_v4(),
            cursor_timestamp: Some(Utc::now()),
            cursor_id: Some(100),
            from_timestamp: None,
            to_timestamp: None,
            limit: 50,
            ascending: true,
        };

        let cloned = query.clone();
        assert_eq!(cloned.limit, query.limit);
        assert_eq!(cloned.ascending, query.ascending);
        assert_eq!(cloned.cursor_id, query.cursor_id);
    }

    #[test]
    fn test_location_history_query_debug() {
        let query = LocationHistoryQuery {
            device_id: Uuid::new_v4(),
            cursor_timestamp: None,
            cursor_id: None,
            from_timestamp: None,
            to_timestamp: None,
            limit: 50,
            ascending: false,
        };

        let debug = format!("{:?}", query);
        assert!(debug.contains("LocationHistoryQuery"));
        assert!(debug.contains("limit"));
        assert!(debug.contains("ascending"));
    }

    #[test]
    fn test_location_history_query_limit_boundaries() {
        // Test different limit values
        let limits = vec![1, 10, 50, 100];
        for limit in limits {
            let query = LocationHistoryQuery {
                device_id: Uuid::new_v4(),
                cursor_timestamp: None,
                cursor_id: None,
                from_timestamp: None,
                to_timestamp: None,
                limit,
                ascending: false,
            };
            assert_eq!(query.limit, limit);
        }
    }

    #[test]
    fn test_location_history_query_ascending_vs_descending() {
        let device_id = Uuid::new_v4();

        let asc_query = LocationHistoryQuery {
            device_id,
            cursor_timestamp: None,
            cursor_id: None,
            from_timestamp: None,
            to_timestamp: None,
            limit: 50,
            ascending: true,
        };

        let desc_query = LocationHistoryQuery {
            device_id,
            cursor_timestamp: None,
            cursor_id: None,
            from_timestamp: None,
            to_timestamp: None,
            limit: 50,
            ascending: false,
        };

        assert!(asc_query.ascending);
        assert!(!desc_query.ascending);
    }

    // ===========================================
    // LocationRepository Struct Tests
    // ===========================================

    // Note: Actual database operations are tested in integration tests.
    // These tests verify struct creation and basic properties.

    #[test]
    fn test_location_input_various_providers() {
        let providers = vec!["gps", "network", "fused", "passive", "unknown"];

        for provider in providers {
            let input = LocationInput {
                device_id: Uuid::new_v4(),
                latitude: 0.0,
                longitude: 0.0,
                accuracy: 1.0,
                altitude: None,
                bearing: None,
                speed: None,
                provider: Some(provider.to_string()),
                battery_level: None,
                network_type: None,
                captured_at: Utc::now(),
            };
            assert_eq!(input.provider, Some(provider.to_string()));
        }
    }

    #[test]
    fn test_location_input_various_network_types() {
        let network_types = vec!["wifi", "4g", "5g", "3g", "2g", "ethernet", "unknown"];

        for network_type in network_types {
            let input = LocationInput {
                device_id: Uuid::new_v4(),
                latitude: 0.0,
                longitude: 0.0,
                accuracy: 1.0,
                altitude: None,
                bearing: None,
                speed: None,
                provider: None,
                battery_level: None,
                network_type: Some(network_type.to_string()),
                captured_at: Utc::now(),
            };
            assert_eq!(input.network_type, Some(network_type.to_string()));
        }
    }

    #[test]
    fn test_location_input_speed_values() {
        // Speed in m/s
        let speeds = vec![0.0, 1.5, 10.0, 30.0, 100.0];  // Walking to driving speeds

        for speed in speeds {
            let input = LocationInput {
                device_id: Uuid::new_v4(),
                latitude: 0.0,
                longitude: 0.0,
                accuracy: 1.0,
                altitude: None,
                bearing: None,
                speed: Some(speed),
                provider: None,
                battery_level: None,
                network_type: None,
                captured_at: Utc::now(),
            };
            assert_eq!(input.speed, Some(speed));
        }
    }

    #[test]
    fn test_location_input_accuracy_values() {
        // Accuracy in meters
        let accuracies = vec![1.0, 5.0, 10.0, 50.0, 100.0, 1000.0];

        for accuracy in accuracies {
            let input = LocationInput {
                device_id: Uuid::new_v4(),
                latitude: 0.0,
                longitude: 0.0,
                accuracy,
                altitude: None,
                bearing: None,
                speed: None,
                provider: None,
                battery_level: None,
                network_type: None,
                captured_at: Utc::now(),
            };
            assert_eq!(input.accuracy, accuracy);
        }
    }

    #[test]
    fn test_location_input_altitude_values() {
        // Altitude in meters - can be negative (below sea level)
        let altitudes = vec![-400.0, 0.0, 100.0, 1000.0, 8848.0];  // Dead Sea to Everest

        for altitude in altitudes {
            let input = LocationInput {
                device_id: Uuid::new_v4(),
                latitude: 0.0,
                longitude: 0.0,
                accuracy: 1.0,
                altitude: Some(altitude),
                bearing: None,
                speed: None,
                provider: None,
                battery_level: None,
                network_type: None,
                captured_at: Utc::now(),
            };
            assert_eq!(input.altitude, Some(altitude));
        }
    }
}
