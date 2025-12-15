//! Setting change repository for database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{SettingChangeEntity, SettingChangeTypeDb, SettingChangeWithUserEntity};
use crate::metrics::QueryTimer;

/// Input for creating a setting change record.
#[derive(Debug, Clone)]
pub struct CreateSettingChangeInput {
    pub device_id: Uuid,
    pub setting_key: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub changed_by: Uuid,
    pub change_type: SettingChangeTypeDb,
}

/// Repository for setting change database operations.
#[derive(Clone)]
pub struct SettingChangeRepository {
    pool: PgPool,
}

impl SettingChangeRepository {
    /// Creates a new SettingChangeRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Create a new setting change record.
    pub async fn create(
        &self,
        input: CreateSettingChangeInput,
    ) -> Result<SettingChangeEntity, sqlx::Error> {
        let timer = QueryTimer::new("setting_change_create");
        let result = sqlx::query_as::<_, SettingChangeEntity>(
            r#"
            INSERT INTO setting_changes (device_id, setting_key, old_value, new_value, changed_by, change_type)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, device_id, setting_key, old_value, new_value, changed_by, changed_at, change_type
            "#,
        )
        .bind(input.device_id)
        .bind(&input.setting_key)
        .bind(&input.old_value)
        .bind(&input.new_value)
        .bind(input.changed_by)
        .bind(input.change_type)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// List setting changes for a device with pagination.
    pub async fn list_for_device(
        &self,
        device_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SettingChangeWithUserEntity>, sqlx::Error> {
        let timer = QueryTimer::new("setting_change_list_for_device");
        let result = sqlx::query_as::<_, SettingChangeWithUserEntity>(
            r#"
            SELECT
                sc.id, sc.device_id, sc.setting_key, sc.old_value, sc.new_value,
                sc.changed_by, sc.changed_at, sc.change_type,
                u.display_name as changed_by_name
            FROM setting_changes sc
            LEFT JOIN users u ON sc.changed_by = u.id
            WHERE sc.device_id = $1
            ORDER BY sc.changed_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(device_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Count total setting changes for a device.
    pub async fn count_for_device(&self, device_id: Uuid) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("setting_change_count_for_device");
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM setting_changes WHERE device_id = $1",
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_setting_change_input() {
        let input = CreateSettingChangeInput {
            device_id: Uuid::new_v4(),
            setting_key: "tracking_enabled".to_string(),
            old_value: Some(serde_json::json!(true)),
            new_value: Some(serde_json::json!(false)),
            changed_by: Uuid::new_v4(),
            change_type: SettingChangeTypeDb::ValueChanged,
        };

        assert_eq!(input.setting_key, "tracking_enabled");
        assert_eq!(input.change_type, SettingChangeTypeDb::ValueChanged);
    }
}
