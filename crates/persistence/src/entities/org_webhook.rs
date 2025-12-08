//! Organization webhook entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the org_webhooks table.
#[derive(Debug, Clone, FromRow)]
pub struct OrgWebhookEntity {
    pub id: i64,
    pub webhook_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub target_url: String,
    pub secret: String,
    pub enabled: bool,
    pub event_types: Vec<String>,
    pub consecutive_failures: i32,
    pub circuit_open_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_org_webhook_entity() -> OrgWebhookEntity {
        OrgWebhookEntity {
            id: 1,
            webhook_id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "Production Events".to_string(),
            target_url: "https://api.example.com/webhooks".to_string(),
            secret: "whsec_test-secret-key-12345678".to_string(),
            enabled: true,
            event_types: vec![
                "device.enrolled".to_string(),
                "member.joined".to_string(),
            ],
            consecutive_failures: 0,
            circuit_open_until: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_org_webhook_entity_creation() {
        let entity = create_test_org_webhook_entity();
        assert_eq!(entity.name, "Production Events");
        assert!(entity.enabled);
        assert!(entity.target_url.starts_with("https://"));
        assert_eq!(entity.event_types.len(), 2);
    }

    #[test]
    fn test_org_webhook_entity_clone() {
        let entity = create_test_org_webhook_entity();
        let cloned = entity.clone();

        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.webhook_id, entity.webhook_id);
        assert_eq!(cloned.organization_id, entity.organization_id);
        assert_eq!(cloned.name, entity.name);
        assert_eq!(cloned.target_url, entity.target_url);
        assert_eq!(cloned.secret, entity.secret);
        assert_eq!(cloned.enabled, entity.enabled);
        assert_eq!(cloned.event_types, entity.event_types);
        assert_eq!(cloned.consecutive_failures, entity.consecutive_failures);
        assert_eq!(cloned.circuit_open_until, entity.circuit_open_until);
    }
}
