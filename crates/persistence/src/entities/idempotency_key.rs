//! Idempotency key entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the idempotency_keys table.
#[derive(Debug, Clone, FromRow)]
pub struct IdempotencyKeyEntity {
    pub id: i64,
    pub key_hash: String,
    pub device_id: Uuid,
    pub response_body: serde_json::Value,
    pub response_status: i16,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entity() -> IdempotencyKeyEntity {
        IdempotencyKeyEntity {
            id: 1,
            key_hash: "abc123def456".to_string(),
            device_id: Uuid::new_v4(),
            response_body: serde_json::json!({"success": true, "processed_count": 1}),
            response_status: 200,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(24),
        }
    }

    #[test]
    fn test_idempotency_key_entity_creation() {
        let entity = create_test_entity();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.response_status, 200);
    }

    #[test]
    fn test_idempotency_key_entity_clone() {
        let entity = create_test_entity();
        let cloned = entity.clone();
        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.key_hash, entity.key_hash);
    }

    #[test]
    fn test_idempotency_key_entity_debug() {
        let entity = create_test_entity();
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("IdempotencyKeyEntity"));
    }

    #[test]
    fn test_idempotency_key_response_body() {
        let entity = create_test_entity();
        assert_eq!(entity.response_body["success"], true);
        assert_eq!(entity.response_body["processed_count"], 1);
    }

    #[test]
    fn test_idempotency_key_expiry() {
        let entity = create_test_entity();
        assert!(entity.expires_at > entity.created_at);
    }
}
