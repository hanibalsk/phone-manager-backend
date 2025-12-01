//! Audit log entity.
//!
//! Story 13.9: Audit Logging System

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database entity for audit logs.
#[derive(Debug, Clone, FromRow)]
pub struct AuditLogEntity {
    /// Unique identifier.
    pub id: Uuid,

    /// Organization this audit log belongs to.
    pub organization_id: Uuid,

    /// Timestamp when the action occurred.
    pub timestamp: DateTime<Utc>,

    /// ID of the actor who performed the action.
    pub actor_id: Option<Uuid>,

    /// Type of actor (user, system, api_key).
    pub actor_type: String,

    /// Email of the actor (for user type).
    pub actor_email: Option<String>,

    /// Action performed (format: resource.operation).
    pub action: String,

    /// Type of resource affected.
    pub resource_type: String,

    /// ID of the resource affected.
    pub resource_id: Option<String>,

    /// Name of the resource affected (for display).
    pub resource_name: Option<String>,

    /// Changes made (before/after values).
    pub changes: Option<serde_json::Value>,

    /// Additional metadata (request context, etc.).
    pub metadata: Option<serde_json::Value>,

    /// IP address of the request.
    pub ip_address: Option<String>,

    /// User agent of the request.
    pub user_agent: Option<String>,

    /// Timestamp when the record was created.
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_entity_creation() {
        let now = Utc::now();
        let entity = AuditLogEntity {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            timestamp: now,
            actor_id: Some(Uuid::new_v4()),
            actor_type: "user".to_string(),
            actor_email: Some("admin@example.com".to_string()),
            action: "device.assign".to_string(),
            resource_type: "device".to_string(),
            resource_id: Some("123".to_string()),
            resource_name: Some("Field Tablet 1".to_string()),
            changes: Some(serde_json::json!({
                "assigned_user_id": {
                    "old": null,
                    "new": "user-uuid"
                }
            })),
            metadata: Some(serde_json::json!({
                "request_id": "req-123"
            })),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            created_at: now,
        };

        assert_eq!(entity.actor_type, "user");
        assert_eq!(entity.action, "device.assign");
        assert_eq!(entity.resource_type, "device");
    }
}
