//! Proximity alert database entity.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database entity for proximity_alerts table.
#[derive(Debug, Clone, FromRow)]
pub struct ProximityAlertEntity {
    pub id: i64,
    pub alert_id: Uuid,
    pub source_device_id: Uuid,
    pub target_device_id: Uuid,
    pub name: Option<String>,
    pub radius_meters: i32,
    pub is_active: bool,
    pub is_triggered: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProximityAlertEntity> for domain::models::ProximityAlert {
    fn from(entity: ProximityAlertEntity) -> Self {
        Self {
            id: entity.id,
            alert_id: entity.alert_id,
            source_device_id: entity.source_device_id,
            target_device_id: entity.target_device_id,
            name: entity.name,
            radius_meters: entity.radius_meters,
            is_active: entity.is_active,
            is_triggered: entity.is_triggered,
            last_triggered_at: entity.last_triggered_at,
            metadata: entity.metadata,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
