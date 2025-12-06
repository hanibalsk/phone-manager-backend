//! Geofence repository for database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::GeofenceEntity;
use crate::metrics::QueryTimer;

/// Repository for geofence-related database operations.
#[derive(Clone)]
pub struct GeofenceRepository {
    pool: PgPool,
}

impl GeofenceRepository {
    /// Creates a new GeofenceRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new geofence.
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        device_id: Uuid,
        name: &str,
        latitude: f64,
        longitude: f64,
        radius_meters: f32,
        event_types: &[String],
        active: bool,
        metadata: Option<serde_json::Value>,
    ) -> Result<GeofenceEntity, sqlx::Error> {
        let timer = QueryTimer::new("create_geofence");
        let result = sqlx::query_as::<_, GeofenceEntity>(
            r#"
            INSERT INTO geofences (device_id, name, latitude, longitude, radius_meters,
                                   event_types, active, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(device_id)
        .bind(name)
        .bind(latitude)
        .bind(longitude)
        .bind(radius_meters)
        .bind(event_types)
        .bind(active)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find geofence by UUID.
    pub async fn find_by_geofence_id(
        &self,
        geofence_id: Uuid,
    ) -> Result<Option<GeofenceEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_geofence_by_id");
        let result = sqlx::query_as::<_, GeofenceEntity>(
            r#"
            SELECT * FROM geofences WHERE geofence_id = $1
            "#,
        )
        .bind(geofence_id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find all geofences for a device.
    pub async fn find_by_device_id(
        &self,
        device_id: Uuid,
        include_inactive: bool,
    ) -> Result<Vec<GeofenceEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_geofences_by_device");
        let result = if include_inactive {
            sqlx::query_as::<_, GeofenceEntity>(
                r#"
                SELECT * FROM geofences
                WHERE device_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(device_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, GeofenceEntity>(
                r#"
                SELECT * FROM geofences
                WHERE device_id = $1 AND active = true
                ORDER BY created_at DESC
                "#,
            )
            .bind(device_id)
            .fetch_all(&self.pool)
            .await
        };
        timer.record();
        result
    }

    /// Count geofences for a device.
    pub async fn count_by_device_id(&self, device_id: Uuid) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_geofences_by_device");
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM geofences WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await?;
        timer.record();
        Ok(count.0)
    }

    /// Update a geofence (partial update).
    /// Only provided fields are updated; None values are preserved.
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        geofence_id: Uuid,
        name: Option<&str>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        radius_meters: Option<f32>,
        event_types: Option<&[String]>,
        active: Option<bool>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Option<GeofenceEntity>, sqlx::Error> {
        let timer = QueryTimer::new("update_geofence");

        let result = sqlx::query_as::<_, GeofenceEntity>(
            r#"
            UPDATE geofences SET
                name = COALESCE($2, name),
                latitude = COALESCE($3, latitude),
                longitude = COALESCE($4, longitude),
                radius_meters = COALESCE($5, radius_meters),
                event_types = COALESCE($6, event_types),
                active = COALESCE($7, active),
                metadata = COALESCE($8, metadata),
                updated_at = NOW()
            WHERE geofence_id = $1
            RETURNING *
            "#,
        )
        .bind(geofence_id)
        .bind(name)
        .bind(latitude)
        .bind(longitude)
        .bind(radius_meters)
        .bind(event_types)
        .bind(active)
        .bind(metadata)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Delete a geofence.
    /// Returns the number of rows deleted (0 or 1).
    pub async fn delete(&self, geofence_id: Uuid) -> Result<u64, sqlx::Error> {
        let timer = QueryTimer::new("delete_geofence");
        let result = sqlx::query(
            r#"
            DELETE FROM geofences WHERE geofence_id = $1
            "#,
        )
        .bind(geofence_id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(result.rows_affected())
    }

    /// Delete all geofences for a device.
    /// Returns the number of rows deleted.
    pub async fn delete_all_by_device_id(&self, device_id: Uuid) -> Result<u64, sqlx::Error> {
        let timer = QueryTimer::new("delete_all_geofences_by_device");
        let result = sqlx::query(
            r#"
            DELETE FROM geofences WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repository_creation() {
        // This test verifies the GeofenceRepository can be created
        // Actual database tests are integration tests
    }
}
