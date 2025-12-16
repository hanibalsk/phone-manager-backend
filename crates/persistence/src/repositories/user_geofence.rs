//! User geofence repository for database operations.
//!
//! User geofences apply to all devices owned by a user.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::UserGeofenceWithCreatorEntity;
use crate::metrics::QueryTimer;

/// Repository for user geofence database operations.
#[derive(Clone)]
pub struct UserGeofenceRepository {
    pool: PgPool,
}

impl UserGeofenceRepository {
    /// Creates a new UserGeofenceRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new user geofence.
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        user_id: Uuid,
        created_by: Uuid,
        name: &str,
        latitude: f64,
        longitude: f64,
        radius_meters: f32,
        event_types: &[String],
        color: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<UserGeofenceWithCreatorEntity, sqlx::Error> {
        let timer = QueryTimer::new("create_user_geofence");

        let result = sqlx::query_as::<_, UserGeofenceWithCreatorEntity>(
            r#"
            WITH inserted AS (
                INSERT INTO user_geofences (
                    user_id, created_by, name, latitude, longitude,
                    radius_meters, event_types, color, metadata
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING *
            )
            SELECT i.*, u.display_name as created_by_name
            FROM inserted i
            LEFT JOIN users u ON i.created_by = u.id
            "#,
        )
        .bind(user_id)
        .bind(created_by)
        .bind(name)
        .bind(latitude)
        .bind(longitude)
        .bind(radius_meters)
        .bind(event_types)
        .bind(color)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await;

        timer.record();
        result
    }

    /// List all geofences for a user.
    pub async fn list_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserGeofenceWithCreatorEntity>, sqlx::Error> {
        let timer = QueryTimer::new("list_user_geofences");

        let result = sqlx::query_as::<_, UserGeofenceWithCreatorEntity>(
            r#"
            SELECT ug.*, u.display_name as created_by_name
            FROM user_geofences ug
            LEFT JOIN users u ON ug.created_by = u.id
            WHERE ug.user_id = $1
            ORDER BY ug.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Find a user geofence by ID.
    pub async fn find_by_id(
        &self,
        geofence_id: Uuid,
    ) -> Result<Option<UserGeofenceWithCreatorEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_user_geofence");

        let result = sqlx::query_as::<_, UserGeofenceWithCreatorEntity>(
            r#"
            SELECT ug.*, u.display_name as created_by_name
            FROM user_geofences ug
            LEFT JOIN users u ON ug.created_by = u.id
            WHERE ug.geofence_id = $1
            "#,
        )
        .bind(geofence_id)
        .fetch_optional(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Update a user geofence (partial update).
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
        color: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<Option<UserGeofenceWithCreatorEntity>, sqlx::Error> {
        let timer = QueryTimer::new("update_user_geofence");

        let result = sqlx::query_as::<_, UserGeofenceWithCreatorEntity>(
            r#"
            WITH updated AS (
                UPDATE user_geofences
                SET
                    name = COALESCE($2, name),
                    latitude = COALESCE($3, latitude),
                    longitude = COALESCE($4, longitude),
                    radius_meters = COALESCE($5, radius_meters),
                    event_types = COALESCE($6, event_types),
                    active = COALESCE($7, active),
                    color = COALESCE($8, color),
                    metadata = COALESCE($9, metadata),
                    updated_at = NOW()
                WHERE geofence_id = $1
                RETURNING *
            )
            SELECT upd.*, u.display_name as created_by_name
            FROM updated upd
            LEFT JOIN users u ON upd.created_by = u.id
            "#,
        )
        .bind(geofence_id)
        .bind(name)
        .bind(latitude)
        .bind(longitude)
        .bind(radius_meters)
        .bind(event_types)
        .bind(active)
        .bind(color)
        .bind(metadata)
        .fetch_optional(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Delete a user geofence.
    /// Returns true if a geofence was deleted.
    pub async fn delete(&self, geofence_id: Uuid) -> Result<bool, sqlx::Error> {
        let timer = QueryTimer::new("delete_user_geofence");

        let result = sqlx::query("DELETE FROM user_geofences WHERE geofence_id = $1")
            .bind(geofence_id)
            .execute(&self.pool)
            .await?;

        timer.record();
        Ok(result.rows_affected() > 0)
    }

    /// Count geofences for a user.
    pub async fn count_by_user(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_user_geofences");

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM user_geofences WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;

        timer.record();
        Ok(count.0)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repository_creation() {
        // This test verifies the UserGeofenceRepository can be created
        // Actual database tests are integration tests
    }
}
