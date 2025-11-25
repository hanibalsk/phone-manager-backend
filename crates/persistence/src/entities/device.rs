//! Device entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the devices table.
#[derive(Debug, Clone, FromRow)]
pub struct DeviceEntity {
    pub id: i64,
    pub device_id: Uuid,
    pub display_name: String,
    pub group_id: String,
    pub platform: String,
    pub fcm_token: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

impl From<DeviceEntity> for domain::models::Device {
    fn from(entity: DeviceEntity) -> Self {
        Self {
            id: entity.id,
            device_id: entity.device_id,
            display_name: entity.display_name,
            group_id: entity.group_id,
            platform: entity.platform,
            fcm_token: entity.fcm_token,
            active: entity.active,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            last_seen_at: entity.last_seen_at,
        }
    }
}

/// Database row mapping for the devices_with_last_location view.
#[derive(Debug, Clone, FromRow)]
pub struct DeviceWithLastLocationEntity {
    pub id: i64,
    pub device_id: Uuid,
    pub display_name: String,
    pub group_id: String,
    pub platform: String,
    pub fcm_token: Option<String>,
    pub active: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_latitude: Option<f64>,
    pub last_longitude: Option<f64>,
    pub last_location_time: Option<DateTime<Utc>>,
    pub last_accuracy: Option<f32>,
}
