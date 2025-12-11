//! Admin geofence repository for organization-wide geofence management.

use sqlx::PgPool;
use uuid::Uuid;

use chrono::{DateTime, Utc};

use crate::entities::{
    AdminGeofenceEntity, AdminGeofenceEventEntity, AdminGeofenceWithCreatorEntity,
    GeofenceVisitCountEntity, LocationAnalyticsEntity,
};
use crate::metrics::QueryTimer;

/// Repository for admin geofence database operations.
#[derive(Clone)]
pub struct AdminGeofenceRepository {
    pool: PgPool,
}

impl AdminGeofenceRepository {
    /// Creates a new AdminGeofenceRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Count admin geofences in organization matching filters.
    pub async fn count_geofences(
        &self,
        org_id: Uuid,
        active: Option<bool>,
        search: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_admin_geofences");

        let mut query =
            String::from("SELECT COUNT(*) FROM admin_geofences WHERE organization_id = $1");

        if active.is_some() {
            query.push_str(" AND active = $2");
        }

        if search.is_some() {
            if active.is_some() {
                query.push_str(" AND (name ILIKE $3 OR description ILIKE $3)");
            } else {
                query.push_str(" AND (name ILIKE $2 OR description ILIKE $2)");
            }
        }

        let result = match (active, search) {
            (Some(active), Some(search)) => {
                let search_pattern = format!("%{}%", search);
                sqlx::query_scalar::<_, i64>(&query)
                    .bind(org_id)
                    .bind(active)
                    .bind(search_pattern)
                    .fetch_one(&self.pool)
                    .await
            }
            (Some(active), None) => {
                sqlx::query_scalar::<_, i64>(&query)
                    .bind(org_id)
                    .bind(active)
                    .fetch_one(&self.pool)
                    .await
            }
            (None, Some(search)) => {
                let search_pattern = format!("%{}%", search);
                sqlx::query_scalar::<_, i64>(&query)
                    .bind(org_id)
                    .bind(search_pattern)
                    .fetch_one(&self.pool)
                    .await
            }
            (None, None) => {
                sqlx::query_scalar::<_, i64>(&query)
                    .bind(org_id)
                    .fetch_one(&self.pool)
                    .await
            }
        };

        timer.record();
        result
    }

    /// List admin geofences with pagination.
    pub async fn list_geofences(
        &self,
        org_id: Uuid,
        active: Option<bool>,
        search: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<AdminGeofenceWithCreatorEntity>, sqlx::Error> {
        let timer = QueryTimer::new("list_admin_geofences");

        let result = sqlx::query_as::<_, AdminGeofenceWithCreatorEntity>(
            r#"
            SELECT
                ag.id, ag.geofence_id, ag.organization_id, ag.name, ag.description,
                ag.latitude, ag.longitude, ag.radius_meters, ag.event_types,
                ag.active, ag.color, ag.metadata, ag.created_by,
                u.display_name as creator_name,
                ag.created_at, ag.updated_at
            FROM admin_geofences ag
            LEFT JOIN users u ON ag.created_by = u.id
            WHERE ag.organization_id = $1
              AND ($2::BOOLEAN IS NULL OR ag.active = $2)
              AND ($3::TEXT IS NULL OR ag.name ILIKE '%' || $3 || '%' OR ag.description ILIKE '%' || $3 || '%')
            ORDER BY ag.created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(org_id)
        .bind(active)
        .bind(search)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Get a single admin geofence by ID.
    pub async fn get_geofence(
        &self,
        org_id: Uuid,
        geofence_id: Uuid,
    ) -> Result<Option<AdminGeofenceWithCreatorEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_admin_geofence");

        let result = sqlx::query_as::<_, AdminGeofenceWithCreatorEntity>(
            r#"
            SELECT
                ag.id, ag.geofence_id, ag.organization_id, ag.name, ag.description,
                ag.latitude, ag.longitude, ag.radius_meters, ag.event_types,
                ag.active, ag.color, ag.metadata, ag.created_by,
                u.display_name as creator_name,
                ag.created_at, ag.updated_at
            FROM admin_geofences ag
            LEFT JOIN users u ON ag.created_by = u.id
            WHERE ag.organization_id = $1 AND ag.geofence_id = $2
            "#,
        )
        .bind(org_id)
        .bind(geofence_id)
        .fetch_optional(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Create a new admin geofence.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_geofence(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        latitude: f64,
        longitude: f64,
        radius_meters: f32,
        event_types: &[String],
        color: Option<&str>,
        metadata: Option<&serde_json::Value>,
        created_by: Uuid,
    ) -> Result<AdminGeofenceEntity, sqlx::Error> {
        let timer = QueryTimer::new("create_admin_geofence");

        let result = sqlx::query_as::<_, AdminGeofenceEntity>(
            r#"
            INSERT INTO admin_geofences (organization_id, name, description, latitude, longitude, radius_meters, event_types, color, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, geofence_id, organization_id, name, description, latitude, longitude, radius_meters, event_types, active, color, metadata, created_by, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(latitude)
        .bind(longitude)
        .bind(radius_meters)
        .bind(event_types)
        .bind(color)
        .bind(metadata)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Update an admin geofence.
    #[allow(clippy::too_many_arguments)]
    pub async fn update_geofence(
        &self,
        org_id: Uuid,
        geofence_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        radius_meters: Option<f32>,
        event_types: Option<&[String]>,
        active: Option<bool>,
        color: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<Option<AdminGeofenceEntity>, sqlx::Error> {
        let timer = QueryTimer::new("update_admin_geofence");

        let result = sqlx::query_as::<_, AdminGeofenceEntity>(
            r#"
            UPDATE admin_geofences
            SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                latitude = COALESCE($5, latitude),
                longitude = COALESCE($6, longitude),
                radius_meters = COALESCE($7, radius_meters),
                event_types = COALESCE($8, event_types),
                active = COALESCE($9, active),
                color = COALESCE($10, color),
                metadata = COALESCE($11, metadata)
            WHERE organization_id = $1 AND geofence_id = $2
            RETURNING id, geofence_id, organization_id, name, description, latitude, longitude, radius_meters, event_types, active, color, metadata, created_by, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(geofence_id)
        .bind(name)
        .bind(description)
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

    /// Delete an admin geofence.
    pub async fn delete_geofence(
        &self,
        org_id: Uuid,
        geofence_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let timer = QueryTimer::new("delete_admin_geofence");

        let result = sqlx::query(
            r#"
            DELETE FROM admin_geofences
            WHERE organization_id = $1 AND geofence_id = $2
            "#,
        )
        .bind(org_id)
        .bind(geofence_id)
        .execute(&self.pool)
        .await?;

        timer.record();
        Ok(result.rows_affected() > 0)
    }

    /// Count geofence events in organization matching filters.
    #[allow(clippy::too_many_arguments)]
    pub async fn count_geofence_events(
        &self,
        org_id: Uuid,
        device_id: Option<Uuid>,
        geofence_id: Option<Uuid>,
        event_type: Option<&str>,
        from_timestamp: Option<DateTime<Utc>>,
        to_timestamp: Option<DateTime<Utc>>,
    ) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_admin_geofence_events");

        let result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM geofence_events ge
            JOIN devices d ON ge.device_id = d.device_id
            JOIN admin_geofences ag ON ge.geofence_id = ag.geofence_id
            WHERE d.organization_id = $1
              AND ag.organization_id = $1
              AND ($2::UUID IS NULL OR ge.device_id = $2)
              AND ($3::UUID IS NULL OR ge.geofence_id = $3)
              AND ($4::TEXT IS NULL OR ge.event_type = $4)
              AND ($5::TIMESTAMPTZ IS NULL OR ge.created_at >= $5)
              AND ($6::TIMESTAMPTZ IS NULL OR ge.created_at <= $6)
            "#,
        )
        .bind(org_id)
        .bind(device_id)
        .bind(geofence_id)
        .bind(event_type)
        .bind(from_timestamp)
        .bind(to_timestamp)
        .fetch_one(&self.pool)
        .await;

        timer.record();
        result
    }

    /// List geofence events for organization with pagination.
    #[allow(clippy::too_many_arguments)]
    pub async fn list_geofence_events(
        &self,
        org_id: Uuid,
        device_id: Option<Uuid>,
        geofence_id: Option<Uuid>,
        event_type: Option<&str>,
        from_timestamp: Option<DateTime<Utc>>,
        to_timestamp: Option<DateTime<Utc>>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<AdminGeofenceEventEntity>, sqlx::Error> {
        let timer = QueryTimer::new("list_admin_geofence_events");

        let result = sqlx::query_as::<_, AdminGeofenceEventEntity>(
            r#"
            SELECT
                ge.id,
                ge.event_id,
                ge.device_id,
                d.display_name as device_name,
                ge.geofence_id,
                ag.name as geofence_name,
                ge.event_type,
                ge.timestamp,
                ge.latitude,
                ge.longitude,
                ge.created_at
            FROM geofence_events ge
            JOIN devices d ON ge.device_id = d.device_id
            JOIN admin_geofences ag ON ge.geofence_id = ag.geofence_id
            WHERE d.organization_id = $1
              AND ag.organization_id = $1
              AND ($2::UUID IS NULL OR ge.device_id = $2)
              AND ($3::UUID IS NULL OR ge.geofence_id = $3)
              AND ($4::TEXT IS NULL OR ge.event_type = $4)
              AND ($5::TIMESTAMPTZ IS NULL OR ge.created_at >= $5)
              AND ($6::TIMESTAMPTZ IS NULL OR ge.created_at <= $6)
            ORDER BY ge.created_at DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(org_id)
        .bind(device_id)
        .bind(geofence_id)
        .bind(event_type)
        .bind(from_timestamp)
        .bind(to_timestamp)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Get location analytics summary for organization.
    pub async fn get_location_analytics(
        &self,
        org_id: Uuid,
    ) -> Result<LocationAnalyticsEntity, sqlx::Error> {
        let timer = QueryTimer::new("get_location_analytics");

        let result = sqlx::query_as::<_, LocationAnalyticsEntity>(
            r#"
            SELECT
                (SELECT COUNT(*) FROM devices WHERE organization_id = $1 AND is_managed = true) as total_devices,
                (SELECT COUNT(DISTINCT d.device_id)
                 FROM devices d
                 JOIN locations l ON d.device_id = l.device_id
                 WHERE d.organization_id = $1 AND d.is_managed = true
                 AND l.created_at >= CURRENT_DATE) as devices_with_location,
                (SELECT COUNT(*)
                 FROM locations l
                 JOIN devices d ON l.device_id = d.device_id
                 WHERE d.organization_id = $1 AND d.is_managed = true
                 AND l.created_at >= CURRENT_DATE) as total_locations_today,
                (SELECT COUNT(*) FROM admin_geofences WHERE organization_id = $1 AND active = true) as total_geofences,
                (SELECT COUNT(*)
                 FROM geofence_events ge
                 JOIN devices d ON ge.device_id = d.device_id
                 JOIN admin_geofences ag ON ge.geofence_id = ag.geofence_id
                 WHERE d.organization_id = $1 AND ag.organization_id = $1
                 AND ge.created_at >= CURRENT_DATE) as total_geofence_events_today
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await;

        timer.record();
        result
    }

    /// Get most visited geofences for organization.
    pub async fn get_most_visited_geofences(
        &self,
        org_id: Uuid,
        limit: u32,
    ) -> Result<Vec<GeofenceVisitCountEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_most_visited_geofences");

        let result = sqlx::query_as::<_, GeofenceVisitCountEntity>(
            r#"
            SELECT
                ag.geofence_id,
                ag.name as geofence_name,
                COUNT(ge.id) as visit_count
            FROM admin_geofences ag
            LEFT JOIN geofence_events ge ON ag.geofence_id = ge.geofence_id
            LEFT JOIN devices d ON ge.device_id = d.device_id
            WHERE ag.organization_id = $1
              AND ag.active = true
              AND (ge.id IS NULL OR (d.organization_id = $1 AND ge.event_type = 'enter'))
            GROUP BY ag.geofence_id, ag.name
            ORDER BY visit_count DESC
            LIMIT $2
            "#,
        )
        .bind(org_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await;

        timer.record();
        result
    }
}

#[cfg(test)]
mod tests {
    // Note: AdminGeofenceRepository tests require database connection and are covered by integration tests
}
