//! Proximity alert repository implementation.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::ProximityAlertEntity;

/// Repository for proximity alert database operations.
#[derive(Clone)]
pub struct ProximityAlertRepository {
    pool: PgPool,
}

impl ProximityAlertRepository {
    /// Creates a new proximity alert repository.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new proximity alert.
    pub async fn create(
        &self,
        source_device_id: Uuid,
        target_device_id: Uuid,
        name: Option<&str>,
        radius_meters: i32,
        is_active: bool,
        metadata: Option<serde_json::Value>,
    ) -> Result<ProximityAlertEntity, sqlx::Error> {
        sqlx::query_as::<_, ProximityAlertEntity>(
            r#"
            INSERT INTO proximity_alerts (
                source_device_id,
                target_device_id,
                name,
                radius_meters,
                is_active,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(source_device_id)
        .bind(target_device_id)
        .bind(name)
        .bind(radius_meters)
        .bind(is_active)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
    }

    /// Finds a proximity alert by its alert_id.
    pub async fn find_by_alert_id(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<ProximityAlertEntity>, sqlx::Error> {
        sqlx::query_as::<_, ProximityAlertEntity>(
            r#"
            SELECT * FROM proximity_alerts
            WHERE alert_id = $1
            "#,
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Finds all proximity alerts for a source device.
    pub async fn find_by_source_device_id(
        &self,
        source_device_id: Uuid,
        include_inactive: bool,
    ) -> Result<Vec<ProximityAlertEntity>, sqlx::Error> {
        if include_inactive {
            sqlx::query_as::<_, ProximityAlertEntity>(
                r#"
                SELECT * FROM proximity_alerts
                WHERE source_device_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(source_device_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, ProximityAlertEntity>(
                r#"
                SELECT * FROM proximity_alerts
                WHERE source_device_id = $1 AND is_active = TRUE
                ORDER BY created_at DESC
                "#,
            )
            .bind(source_device_id)
            .fetch_all(&self.pool)
            .await
        }
    }

    /// Counts proximity alerts for a source device.
    pub async fn count_by_source_device_id(
        &self,
        source_device_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM proximity_alerts
            WHERE source_device_id = $1
            "#,
        )
        .bind(source_device_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(result.0)
    }

    /// Checks if a proximity alert already exists between source and target devices.
    pub async fn exists_for_device_pair(
        &self,
        source_device_id: Uuid,
        target_device_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM proximity_alerts
                WHERE source_device_id = $1 AND target_device_id = $2
            )
            "#,
        )
        .bind(source_device_id)
        .bind(target_device_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(result.0)
    }

    /// Updates a proximity alert.
    pub async fn update(
        &self,
        alert_id: Uuid,
        name: Option<&str>,
        radius_meters: Option<i32>,
        is_active: Option<bool>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Option<ProximityAlertEntity>, sqlx::Error> {
        sqlx::query_as::<_, ProximityAlertEntity>(
            r#"
            UPDATE proximity_alerts
            SET
                name = COALESCE($2, name),
                radius_meters = COALESCE($3, radius_meters),
                is_active = COALESCE($4, is_active),
                metadata = COALESCE($5, metadata),
                updated_at = NOW()
            WHERE alert_id = $1
            RETURNING *
            "#,
        )
        .bind(alert_id)
        .bind(name)
        .bind(radius_meters)
        .bind(is_active)
        .bind(metadata)
        .fetch_optional(&self.pool)
        .await
    }

    /// Deletes a proximity alert.
    pub async fn delete(&self, alert_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM proximity_alerts
            WHERE alert_id = $1
            "#,
        )
        .bind(alert_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Updates the triggered state of a proximity alert.
    pub async fn set_triggered(
        &self,
        alert_id: Uuid,
        is_triggered: bool,
    ) -> Result<Option<ProximityAlertEntity>, sqlx::Error> {
        sqlx::query_as::<_, ProximityAlertEntity>(
            r#"
            UPDATE proximity_alerts
            SET
                is_triggered = $2,
                last_triggered_at = CASE WHEN $2 = TRUE THEN NOW() ELSE last_triggered_at END,
                updated_at = NOW()
            WHERE alert_id = $1
            RETURNING *
            "#,
        )
        .bind(alert_id)
        .bind(is_triggered)
        .fetch_optional(&self.pool)
        .await
    }
}
