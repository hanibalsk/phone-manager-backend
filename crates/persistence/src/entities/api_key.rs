//! API key entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database row mapping for the api_keys table.
#[derive(Debug, Clone, FromRow)]
pub struct ApiKeyEntity {
    pub id: i64,
    pub key_hash: String,
    pub key_prefix: String,
    pub name: String,
    pub is_active: bool,
    pub is_admin: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_api_key_entity() -> ApiKeyEntity {
        ApiKeyEntity {
            id: 1,
            key_hash: "abc123def456".to_string(),
            key_prefix: "pm_live".to_string(),
            name: "Test API Key".to_string(),
            is_active: true,
            is_admin: false,
            last_used_at: Some(Utc::now()),
            created_at: Utc::now(),
            expires_at: None,
        }
    }

    #[test]
    fn test_api_key_entity_creation() {
        let entity = create_test_api_key_entity();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.key_hash, "abc123def456");
        assert_eq!(entity.key_prefix, "pm_live");
        assert_eq!(entity.name, "Test API Key");
        assert!(entity.is_active);
        assert!(!entity.is_admin);
    }

    #[test]
    fn test_api_key_entity_clone() {
        let entity = create_test_api_key_entity();
        let cloned = entity.clone();
        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.key_hash, entity.key_hash);
        assert_eq!(cloned.key_prefix, entity.key_prefix);
        assert_eq!(cloned.name, entity.name);
        assert_eq!(cloned.is_active, entity.is_active);
        assert_eq!(cloned.is_admin, entity.is_admin);
    }

    #[test]
    fn test_api_key_entity_debug() {
        let entity = create_test_api_key_entity();
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("ApiKeyEntity"));
        assert!(debug_str.contains("pm_live"));
        assert!(debug_str.contains("Test API Key"));
    }

    #[test]
    fn test_api_key_entity_admin_key() {
        let entity = ApiKeyEntity {
            id: 2,
            key_hash: "admin_hash_xyz".to_string(),
            key_prefix: "pm_admin".to_string(),
            name: "Admin Key".to_string(),
            is_active: true,
            is_admin: true,
            last_used_at: None,
            created_at: Utc::now(),
            expires_at: None,
        };
        assert!(entity.is_admin);
        assert!(entity.is_active);
        assert!(entity.last_used_at.is_none());
    }

    #[test]
    fn test_api_key_entity_inactive_key() {
        let entity = ApiKeyEntity {
            id: 3,
            key_hash: "inactive_hash".to_string(),
            key_prefix: "pm_test".to_string(),
            name: "Inactive Key".to_string(),
            is_active: false,
            is_admin: false,
            last_used_at: Some(Utc::now()),
            created_at: Utc::now(),
            expires_at: Some(Utc::now()),
        };
        assert!(!entity.is_active);
        assert!(entity.expires_at.is_some());
    }

    #[test]
    fn test_api_key_entity_with_expiration() {
        let now = Utc::now();
        let expires = now + chrono::Duration::days(30);
        let entity = ApiKeyEntity {
            id: 4,
            key_hash: "expiring_hash".to_string(),
            key_prefix: "pm_live".to_string(),
            name: "Expiring Key".to_string(),
            is_active: true,
            is_admin: false,
            last_used_at: None,
            created_at: now,
            expires_at: Some(expires),
        };
        assert!(entity.expires_at.is_some());
        assert!(entity.expires_at.unwrap() > entity.created_at);
    }

    #[test]
    fn test_api_key_entity_timestamps() {
        let entity = create_test_api_key_entity();
        assert!(entity.created_at <= Utc::now());
        if let Some(last_used) = entity.last_used_at {
            assert!(last_used <= Utc::now());
        }
    }
}
