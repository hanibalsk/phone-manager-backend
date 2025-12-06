//! Webhook entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::webhook::Webhook;

/// Database row mapping for the webhooks table.
#[derive(Debug, Clone, FromRow)]
pub struct WebhookEntity {
    pub id: i64,
    pub webhook_id: Uuid,
    pub owner_device_id: Uuid,
    pub name: String,
    pub target_url: String,
    pub secret: String,
    pub enabled: bool,
    pub consecutive_failures: i32,
    pub circuit_open_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WebhookEntity> for Webhook {
    fn from(entity: WebhookEntity) -> Self {
        Self {
            id: entity.id,
            webhook_id: entity.webhook_id,
            owner_device_id: entity.owner_device_id,
            name: entity.name,
            target_url: entity.target_url,
            secret: entity.secret,
            enabled: entity.enabled,
            consecutive_failures: entity.consecutive_failures,
            circuit_open_until: entity.circuit_open_until,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_webhook_entity() -> WebhookEntity {
        WebhookEntity {
            id: 1,
            webhook_id: Uuid::new_v4(),
            owner_device_id: Uuid::new_v4(),
            name: "Home Assistant".to_string(),
            target_url: "https://example.com/webhook".to_string(),
            secret: "test-secret-key-12345678".to_string(),
            enabled: true,
            consecutive_failures: 0,
            circuit_open_until: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_webhook_entity_creation() {
        let entity = create_test_webhook_entity();
        assert_eq!(entity.name, "Home Assistant");
        assert!(entity.enabled);
        assert!(entity.target_url.starts_with("https://"));
    }

    #[test]
    fn test_webhook_entity_clone() {
        let entity = create_test_webhook_entity();
        let cloned = entity.clone();

        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.webhook_id, entity.webhook_id);
        assert_eq!(cloned.name, entity.name);
        assert_eq!(cloned.target_url, entity.target_url);
        assert_eq!(cloned.secret, entity.secret);
        assert_eq!(cloned.enabled, entity.enabled);
        assert_eq!(cloned.consecutive_failures, entity.consecutive_failures);
        assert_eq!(cloned.circuit_open_until, entity.circuit_open_until);
    }
}
