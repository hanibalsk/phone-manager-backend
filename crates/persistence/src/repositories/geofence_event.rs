//! Geofence event repository.
//!
//! Story 15.2: Webhook Event Delivery
//! Provides data access for geofence events.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::geofence_event::{GeofenceEventEntity, GeofenceEventWithName};

/// Repository for geofence event operations.
pub struct GeofenceEventRepository {
    pool: PgPool,
}

impl GeofenceEventRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new geofence event.
    pub async fn create(
        &self,
        device_id: Uuid,
        geofence_id: Uuid,
        event_type: &str,
        timestamp: i64,
        latitude: f64,
        longitude: f64,
    ) -> Result<GeofenceEventEntity, sqlx::Error> {
        let entity = sqlx::query_as::<_, GeofenceEventEntity>(
            r#"
            INSERT INTO geofence_events (device_id, geofence_id, event_type, timestamp, latitude, longitude)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, event_id, device_id, geofence_id, event_type, timestamp, latitude, longitude,
                      webhook_delivered, webhook_response_code, created_at
            "#,
        )
        .bind(device_id)
        .bind(geofence_id)
        .bind(event_type)
        .bind(timestamp)
        .bind(latitude)
        .bind(longitude)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity)
    }

    /// Find a geofence event by event_id.
    pub async fn find_by_event_id(
        &self,
        event_id: Uuid,
    ) -> Result<Option<GeofenceEventWithName>, sqlx::Error> {
        let entity = sqlx::query_as::<_, GeofenceEventWithName>(
            r#"
            SELECT
                e.id, e.event_id, e.device_id, e.geofence_id,
                g.name as geofence_name,
                e.event_type, e.timestamp, e.latitude, e.longitude,
                e.webhook_delivered, e.webhook_response_code, e.created_at
            FROM geofence_events e
            LEFT JOIN geofences g ON e.geofence_id = g.geofence_id
            WHERE e.event_id = $1
            "#,
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity)
    }

    /// Find geofence events by device_id.
    pub async fn find_by_device_id(
        &self,
        device_id: Uuid,
        geofence_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<GeofenceEventWithName>, sqlx::Error> {
        let entities = if let Some(gf_id) = geofence_id {
            sqlx::query_as::<_, GeofenceEventWithName>(
                r#"
                SELECT
                    e.id, e.event_id, e.device_id, e.geofence_id,
                    g.name as geofence_name,
                    e.event_type, e.timestamp, e.latitude, e.longitude,
                    e.webhook_delivered, e.webhook_response_code, e.created_at
                FROM geofence_events e
                LEFT JOIN geofences g ON e.geofence_id = g.geofence_id
                WHERE e.device_id = $1 AND e.geofence_id = $2
                ORDER BY e.timestamp DESC
                LIMIT $3
                "#,
            )
            .bind(device_id)
            .bind(gf_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, GeofenceEventWithName>(
                r#"
                SELECT
                    e.id, e.event_id, e.device_id, e.geofence_id,
                    g.name as geofence_name,
                    e.event_type, e.timestamp, e.latitude, e.longitude,
                    e.webhook_delivered, e.webhook_response_code, e.created_at
                FROM geofence_events e
                LEFT JOIN geofences g ON e.geofence_id = g.geofence_id
                WHERE e.device_id = $1
                ORDER BY e.timestamp DESC
                LIMIT $2
                "#,
            )
            .bind(device_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(entities)
    }

    /// Count geofence events by device_id.
    pub async fn count_by_device_id(
        &self,
        device_id: Uuid,
        geofence_id: Option<Uuid>,
    ) -> Result<i64, sqlx::Error> {
        let count: (i64,) = if let Some(gf_id) = geofence_id {
            sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM geofence_events
                WHERE device_id = $1 AND geofence_id = $2
                "#,
            )
            .bind(device_id)
            .bind(gf_id)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM geofence_events
                WHERE device_id = $1
                "#,
            )
            .bind(device_id)
            .fetch_one(&self.pool)
            .await?
        };

        Ok(count.0)
    }

    /// Update webhook delivery status for an event.
    pub async fn update_webhook_status(
        &self,
        event_id: Uuid,
        delivered: bool,
        response_code: Option<i32>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE geofence_events
            SET webhook_delivered = $2, webhook_response_code = $3
            WHERE event_id = $1
            "#,
        )
        .bind(event_id)
        .bind(delivered)
        .bind(response_code)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete all geofence events for a device (used for GDPR compliance).
    pub async fn delete_all_by_device_id(&self, device_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM geofence_events
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
    fn test_repository_new() {
        // Just verify the repository can be created with a pool
        // Actual database tests would require integration tests
    }
}
