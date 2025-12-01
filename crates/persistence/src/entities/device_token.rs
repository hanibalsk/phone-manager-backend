//! Device token entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the device_tokens table.
#[derive(Debug, Clone, FromRow)]
pub struct DeviceTokenEntity {
    pub id: Uuid,
    pub device_id: i64,
    pub organization_id: Uuid,
    pub token: String,
    pub token_prefix: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl From<DeviceTokenEntity> for domain::models::DeviceToken {
    fn from(entity: DeviceTokenEntity) -> Self {
        Self {
            id: entity.id,
            device_id: entity.device_id,
            organization_id: entity.organization_id,
            token: entity.token,
            token_prefix: entity.token_prefix,
            expires_at: entity.expires_at,
            created_at: entity.created_at,
            last_used_at: entity.last_used_at,
            revoked_at: entity.revoked_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_token_entity_to_domain() {
        let entity = DeviceTokenEntity {
            id: Uuid::new_v4(),
            device_id: 1,
            organization_id: Uuid::new_v4(),
            token: "dt_test123".to_string(),
            token_prefix: "dt_test1".to_string(),
            expires_at: Utc::now(),
            created_at: Utc::now(),
            last_used_at: None,
            revoked_at: None,
        };

        let domain: domain::models::DeviceToken = entity.clone().into();
        assert_eq!(domain.id, entity.id);
        assert_eq!(domain.device_id, entity.device_id);
        assert_eq!(domain.token, entity.token);
    }
}
