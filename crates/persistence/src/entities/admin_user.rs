//! Admin user entity definitions for database queries.
//!
//! Story 14.3: User Management Endpoints

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for admin user list with joined data.
#[derive(Debug, Clone, FromRow)]
pub struct AdminUserEntity {
    // User fields
    pub user_id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    // Org user fields
    pub role: String,
    pub permissions: serde_json::Value,
    pub granted_at: DateTime<Utc>,
    // Aggregate counts
    pub device_count: i64,
    pub group_count: i64,
}

/// Database row mapping for user device info.
#[derive(Debug, Clone, FromRow)]
pub struct UserDeviceEntity {
    pub id: i64,
    pub device_id: Uuid,
    pub display_name: String,
    pub platform: String,
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// Database row mapping for user group info.
#[derive(Debug, Clone, FromRow)]
pub struct UserGroupEntity {
    pub group_id: String,
    pub name: String,
    pub role: String,
}

/// Database row mapping for user profile detail.
#[derive(Debug, Clone, FromRow)]
pub struct AdminUserProfileEntity {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    // Org user fields
    pub role: String,
    pub permissions: serde_json::Value,
    pub granted_at: DateTime<Utc>,
    pub granted_by_email: Option<String>,
}

/// Database row mapping for recent action.
#[derive(Debug, Clone, FromRow)]
pub struct RecentActionEntity {
    pub action: String,
    pub resource_type: String,
    pub timestamp: DateTime<Utc>,
}

/// Database row mapping for user summary counts.
#[derive(Debug, Clone, FromRow)]
pub struct AdminUserSummaryEntity {
    pub owners: i64,
    pub admins: i64,
    pub members: i64,
    pub with_devices: i64,
    pub without_devices: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_user_entity_fields() {
        let entity = AdminUserEntity {
            user_id: Uuid::nil(),
            email: "test@example.com".to_string(),
            display_name: Some("Test".to_string()),
            avatar_url: None,
            last_login_at: None,
            role: "admin".to_string(),
            permissions: serde_json::json!(["device:read"]),
            granted_at: Utc::now(),
            device_count: 0,
            group_count: 0,
        };
        assert_eq!(entity.email, "test@example.com");
        assert_eq!(entity.role, "admin");
    }
}
