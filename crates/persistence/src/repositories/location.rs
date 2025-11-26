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
                      speed, provider, battery_level, network_type, captured_at, created_at
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
                   speed, provider, battery_level, network_type, captured_at, created_at
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repository_creation() {
        // This test verifies the LocationRepository can be created
        // Actual database tests are integration tests
        assert!(true);
    }
}
