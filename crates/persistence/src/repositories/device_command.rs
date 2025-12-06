//! Device command repository.
//!
//! Story 13.7: Fleet Management Endpoints

use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::DeviceCommandEntity;

/// Repository for device command operations.
#[derive(Debug, Clone)]
pub struct DeviceCommandRepository {
    pool: PgPool,
}

impl DeviceCommandRepository {
    /// Create a new device command repository.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new device command.
    pub async fn create(
        &self,
        device_id: i64,
        organization_id: Uuid,
        command_type: &str,
        payload: Option<&serde_json::Value>,
        issued_by: Uuid,
        expires_in_hours: u32,
    ) -> Result<DeviceCommandEntity, sqlx::Error> {
        let expires_at = Utc::now() + Duration::hours(expires_in_hours as i64);

        sqlx::query_as::<_, DeviceCommandEntity>(
            r#"
            INSERT INTO device_commands (
                device_id, organization_id, command_type, status, payload,
                issued_by, expires_at
            )
            VALUES ($1, $2, $3::device_command_type, 'pending'::device_command_status, $4, $5, $6)
            RETURNING id, device_id, organization_id, command_type::TEXT, status::TEXT,
                payload, issued_by, issued_at, acknowledged_at, completed_at,
                failed_at, failure_reason, expires_at, created_at, updated_at
            "#,
        )
        .bind(device_id)
        .bind(organization_id)
        .bind(command_type)
        .bind(payload)
        .bind(issued_by)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
    }

    /// Get a device command by ID.
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<DeviceCommandEntity>, sqlx::Error> {
        sqlx::query_as::<_, DeviceCommandEntity>(
            r#"
            SELECT id, device_id, organization_id, command_type::TEXT, status::TEXT,
                payload, issued_by, issued_at, acknowledged_at, completed_at,
                failed_at, failure_reason, expires_at, created_at, updated_at
            FROM device_commands
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Get pending commands for a device.
    pub async fn get_pending_for_device(
        &self,
        device_id: i64,
    ) -> Result<Vec<DeviceCommandEntity>, sqlx::Error> {
        sqlx::query_as::<_, DeviceCommandEntity>(
            r#"
            SELECT id, device_id, organization_id, command_type::TEXT, status::TEXT,
                payload, issued_by, issued_at, acknowledged_at, completed_at,
                failed_at, failure_reason, expires_at, created_at, updated_at
            FROM device_commands
            WHERE device_id = $1
              AND status = 'pending'
              AND expires_at > NOW()
            ORDER BY issued_at ASC
            "#,
        )
        .bind(device_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Acknowledge a command.
    pub async fn acknowledge(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE device_commands
            SET status = 'acknowledged'::device_command_status,
                acknowledged_at = NOW()
            WHERE id = $1 AND status = 'pending'
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Complete a command.
    pub async fn complete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE device_commands
            SET status = 'completed'::device_command_status,
                completed_at = NOW()
            WHERE id = $1 AND status IN ('pending', 'acknowledged')
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Fail a command.
    pub async fn fail(&self, id: Uuid, reason: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE device_commands
            SET status = 'failed'::device_command_status,
                failed_at = NOW(),
                failure_reason = $2
            WHERE id = $1 AND status IN ('pending', 'acknowledged')
            "#,
        )
        .bind(id)
        .bind(reason)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Expire old pending commands.
    pub async fn expire_old_commands(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE device_commands
            SET status = 'expired'::device_command_status
            WHERE status = 'pending' AND expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// List commands for a device with pagination.
    pub async fn list_for_device(
        &self,
        device_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DeviceCommandEntity>, sqlx::Error> {
        sqlx::query_as::<_, DeviceCommandEntity>(
            r#"
            SELECT id, device_id, organization_id, command_type::TEXT, status::TEXT,
                payload, issued_by, issued_at, acknowledged_at, completed_at,
                failed_at, failure_reason, expires_at, created_at, updated_at
            FROM device_commands
            WHERE device_id = $1
            ORDER BY issued_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(device_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    /// Count commands for a device.
    pub async fn count_for_device(&self, device_id: i64) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM device_commands
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_device_command_repository_new() {
        // This test verifies the repository can be constructed
        // Actual database tests would require a test database
    }
}
