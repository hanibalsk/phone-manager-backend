//! Setting repository for database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{DeviceSettingEntity, SettingDefinitionEntity, SettingLockEntity};
use crate::metrics::QueryTimer;

/// Repository for setting-related database operations.
#[derive(Clone)]
pub struct SettingRepository {
    pool: PgPool,
}

impl SettingRepository {
    /// Creates a new SettingRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // =========================================================================
    // Setting Definitions
    // =========================================================================

    /// Get all setting definitions.
    pub async fn get_all_definitions(&self) -> Result<Vec<SettingDefinitionEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_all_definitions");
        let result = sqlx::query_as::<_, SettingDefinitionEntity>(
            r#"
            SELECT key, display_name, description, data_type, default_value,
                   is_lockable, category, validation_rules, sort_order, created_at, updated_at
            FROM setting_definitions
            ORDER BY sort_order, key
            "#,
        )
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Get a setting definition by key.
    pub async fn get_definition(
        &self,
        key: &str,
    ) -> Result<Option<SettingDefinitionEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_definition");
        let result = sqlx::query_as::<_, SettingDefinitionEntity>(
            r#"
            SELECT key, display_name, description, data_type, default_value,
                   is_lockable, category, validation_rules, sort_order, created_at, updated_at
            FROM setting_definitions
            WHERE key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    // =========================================================================
    // Device Settings
    // =========================================================================

    /// Get all settings for a device.
    pub async fn get_device_settings(
        &self,
        device_id: Uuid,
    ) -> Result<Vec<DeviceSettingEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_device_settings");
        let result = sqlx::query_as::<_, DeviceSettingEntity>(
            r#"
            SELECT id, device_id, setting_key, value, is_locked, locked_by,
                   locked_at, lock_reason, updated_by, updated_at, created_at
            FROM device_settings
            WHERE device_id = $1
            ORDER BY setting_key
            "#,
        )
        .bind(device_id)
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Get a single setting for a device.
    pub async fn get_device_setting(
        &self,
        device_id: Uuid,
        setting_key: &str,
    ) -> Result<Option<DeviceSettingEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_device_setting");
        let result = sqlx::query_as::<_, DeviceSettingEntity>(
            r#"
            SELECT id, device_id, setting_key, value, is_locked, locked_by,
                   locked_at, lock_reason, updated_by, updated_at, created_at
            FROM device_settings
            WHERE device_id = $1 AND setting_key = $2
            "#,
        )
        .bind(device_id)
        .bind(setting_key)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Upsert a device setting value.
    pub async fn upsert_setting(
        &self,
        device_id: Uuid,
        setting_key: &str,
        value: serde_json::Value,
        updated_by: Option<Uuid>,
    ) -> Result<DeviceSettingEntity, sqlx::Error> {
        let timer = QueryTimer::new("upsert_setting");
        let result = sqlx::query_as::<_, DeviceSettingEntity>(
            r#"
            INSERT INTO device_settings (device_id, setting_key, value, updated_by)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (device_id, setting_key)
            DO UPDATE SET value = $3, updated_by = $4, updated_at = NOW()
            WHERE device_settings.is_locked = false
            RETURNING id, device_id, setting_key, value, is_locked, locked_by,
                      locked_at, lock_reason, updated_by, updated_at, created_at
            "#,
        )
        .bind(device_id)
        .bind(setting_key)
        .bind(value)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Upsert a device setting with admin override (ignores lock).
    pub async fn upsert_setting_force(
        &self,
        device_id: Uuid,
        setting_key: &str,
        value: serde_json::Value,
        updated_by: Uuid,
    ) -> Result<DeviceSettingEntity, sqlx::Error> {
        let timer = QueryTimer::new("upsert_setting_force");
        let result = sqlx::query_as::<_, DeviceSettingEntity>(
            r#"
            INSERT INTO device_settings (device_id, setting_key, value, updated_by)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (device_id, setting_key)
            DO UPDATE SET value = $3, updated_by = $4, updated_at = NOW()
            RETURNING id, device_id, setting_key, value, is_locked, locked_by,
                      locked_at, lock_reason, updated_by, updated_at, created_at
            "#,
        )
        .bind(device_id)
        .bind(setting_key)
        .bind(value)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    // =========================================================================
    // Lock Management
    // =========================================================================

    /// Lock a setting.
    pub async fn lock_setting(
        &self,
        device_id: Uuid,
        setting_key: &str,
        locked_by: Uuid,
        reason: Option<&str>,
        value: Option<serde_json::Value>,
    ) -> Result<DeviceSettingEntity, sqlx::Error> {
        let timer = QueryTimer::new("lock_setting");

        // Get the definition for default value if needed
        let default_value = if value.is_none() {
            sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT default_value FROM setting_definitions WHERE key = $1",
            )
            .bind(setting_key)
            .fetch_optional(&self.pool)
            .await?
            .unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Null
        };

        let final_value = value.unwrap_or(default_value);

        let result = sqlx::query_as::<_, DeviceSettingEntity>(
            r#"
            INSERT INTO device_settings (device_id, setting_key, value, is_locked, locked_by, locked_at, lock_reason, updated_by)
            VALUES ($1, $2, $3, true, $4, NOW(), $5, $4)
            ON CONFLICT (device_id, setting_key)
            DO UPDATE SET
                value = COALESCE($3, device_settings.value),
                is_locked = true,
                locked_by = $4,
                locked_at = NOW(),
                lock_reason = $5,
                updated_by = $4,
                updated_at = NOW()
            RETURNING id, device_id, setting_key, value, is_locked, locked_by,
                      locked_at, lock_reason, updated_by, updated_at, created_at
            "#,
        )
        .bind(device_id)
        .bind(setting_key)
        .bind(&final_value)
        .bind(locked_by)
        .bind(reason)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Unlock a setting.
    pub async fn unlock_setting(
        &self,
        device_id: Uuid,
        setting_key: &str,
        unlocked_by: Uuid,
    ) -> Result<Option<DeviceSettingEntity>, sqlx::Error> {
        let timer = QueryTimer::new("unlock_setting");
        let result = sqlx::query_as::<_, DeviceSettingEntity>(
            r#"
            UPDATE device_settings
            SET is_locked = false, locked_by = NULL, locked_at = NULL, lock_reason = NULL,
                updated_by = $3, updated_at = NOW()
            WHERE device_id = $1 AND setting_key = $2
            RETURNING id, device_id, setting_key, value, is_locked, locked_by,
                      locked_at, lock_reason, updated_by, updated_at, created_at
            "#,
        )
        .bind(device_id)
        .bind(setting_key)
        .bind(unlocked_by)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Get all locks for a device.
    pub async fn get_device_locks(
        &self,
        device_id: Uuid,
    ) -> Result<Vec<SettingLockEntity>, sqlx::Error> {
        let timer = QueryTimer::new("get_device_locks");
        let result = sqlx::query_as::<_, SettingLockEntity>(
            r#"
            SELECT ds.setting_key, ds.is_locked, ds.locked_by, ds.locked_at, ds.lock_reason,
                   u.display_name as locker_display_name
            FROM device_settings ds
            LEFT JOIN users u ON ds.locked_by = u.id
            WHERE ds.device_id = $1 AND ds.is_locked = true
            ORDER BY ds.locked_at DESC
            "#,
        )
        .bind(device_id)
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Count lockable settings.
    pub async fn count_lockable_settings(&self) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_lockable_settings");
        let result =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM setting_definitions WHERE is_lockable = true")
                .fetch_one(&self.pool)
                .await;
        timer.record();
        result
    }

    /// Check if a setting is lockable.
    pub async fn is_setting_lockable(&self, setting_key: &str) -> Result<bool, sqlx::Error> {
        let timer = QueryTimer::new("is_setting_lockable");
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT is_lockable FROM setting_definitions WHERE key = $1",
        )
        .bind(setting_key)
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or(false);
        timer.record();
        Ok(result)
    }

    /// Check if a setting is locked.
    pub async fn is_setting_locked(
        &self,
        device_id: Uuid,
        setting_key: &str,
    ) -> Result<bool, sqlx::Error> {
        let timer = QueryTimer::new("is_setting_locked");
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT is_locked FROM device_settings WHERE device_id = $1 AND setting_key = $2",
        )
        .bind(device_id)
        .bind(setting_key)
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or(false);
        timer.record();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_repository_new() {
        // This is a structural test - we can't test actual database operations without a test DB
        // The repository structure follows the established pattern
    }
}
