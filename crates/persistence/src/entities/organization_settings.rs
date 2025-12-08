//! Organization settings entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the organization_settings table.
#[derive(Debug, Clone, FromRow)]
pub struct OrganizationSettingsEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub unlock_pin_hash: Option<String>,
    pub default_daily_limit_minutes: i32,
    pub notifications_enabled: bool,
    pub auto_approve_unlock_requests: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<OrganizationSettingsEntity> for domain::models::OrganizationSettings {
    fn from(entity: OrganizationSettingsEntity) -> Self {
        Self {
            id: entity.id,
            organization_id: entity.organization_id,
            has_unlock_pin: entity.unlock_pin_hash.is_some(),
            unlock_pin_hash: entity.unlock_pin_hash,
            default_daily_limit_minutes: entity.default_daily_limit_minutes,
            notifications_enabled: entity.notifications_enabled,
            auto_approve_unlock_requests: entity.auto_approve_unlock_requests,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_to_domain_with_pin() {
        let entity = OrganizationSettingsEntity {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            unlock_pin_hash: Some("$argon2id$v=19$...".to_string()),
            default_daily_limit_minutes: 120,
            notifications_enabled: true,
            auto_approve_unlock_requests: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let domain: domain::models::OrganizationSettings = entity.into();
        assert!(domain.has_unlock_pin);
        assert!(domain.unlock_pin_hash.is_some());
    }

    #[test]
    fn test_entity_to_domain_without_pin() {
        let entity = OrganizationSettingsEntity {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            unlock_pin_hash: None,
            default_daily_limit_minutes: 60,
            notifications_enabled: false,
            auto_approve_unlock_requests: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let domain: domain::models::OrganizationSettings = entity.into();
        assert!(!domain.has_unlock_pin);
        assert!(domain.unlock_pin_hash.is_none());
    }
}
