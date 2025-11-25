//! Location entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the locations table.
#[derive(Debug, Clone, FromRow)]
pub struct LocationEntity {
    pub id: i64,
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
    pub created_at: DateTime<Utc>,
}

impl From<LocationEntity> for domain::models::Location {
    fn from(entity: LocationEntity) -> Self {
        Self {
            id: entity.id,
            device_id: entity.device_id,
            latitude: entity.latitude,
            longitude: entity.longitude,
            accuracy: entity.accuracy,
            altitude: entity.altitude,
            bearing: entity.bearing,
            speed: entity.speed,
            provider: entity.provider,
            battery_level: entity.battery_level,
            network_type: entity.network_type,
            captured_at: entity.captured_at,
            created_at: entity.created_at,
        }
    }
}
