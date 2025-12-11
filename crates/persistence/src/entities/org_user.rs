//! Organization user entity (database row mapping).

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

/// Database enum for org_user_role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "org_user_role", rename_all = "lowercase")]
pub enum OrgUserRoleDb {
    Owner,
    Admin,
    Member,
}

impl From<OrgUserRoleDb> for domain::models::OrgUserRole {
    fn from(db: OrgUserRoleDb) -> Self {
        match db {
            OrgUserRoleDb::Owner => Self::Owner,
            OrgUserRoleDb::Admin => Self::Admin,
            OrgUserRoleDb::Member => Self::Member,
        }
    }
}

impl From<domain::models::OrgUserRole> for OrgUserRoleDb {
    fn from(domain: domain::models::OrgUserRole) -> Self {
        match domain {
            domain::models::OrgUserRole::Owner => Self::Owner,
            domain::models::OrgUserRole::Admin => Self::Admin,
            domain::models::OrgUserRole::Member => Self::Member,
        }
    }
}

/// Database row mapping for the org_users table.
#[derive(Debug, Clone, FromRow)]
pub struct OrgUserEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub role: OrgUserRoleDb,
    pub permissions: JsonValue,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
}

impl From<OrgUserEntity> for domain::models::OrgUser {
    fn from(entity: OrgUserEntity) -> Self {
        let permissions: Vec<String> =
            serde_json::from_value(entity.permissions.clone()).unwrap_or_default();
        Self {
            id: entity.id,
            organization_id: entity.organization_id,
            user_id: entity.user_id,
            role: entity.role.into(),
            permissions,
            granted_at: entity.granted_at,
            granted_by: entity.granted_by,
        }
    }
}

/// Organization user with user details for list responses.
#[derive(Debug, Clone, FromRow)]
pub struct OrgUserWithDetailsEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub role: OrgUserRoleDb,
    pub permissions: JsonValue,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
    // User details
    pub user_email: String,
    pub user_display_name: Option<String>,
}

impl From<OrgUserWithDetailsEntity> for domain::models::OrgUserWithDetails {
    fn from(entity: OrgUserWithDetailsEntity) -> Self {
        let permissions: Vec<String> =
            serde_json::from_value(entity.permissions.clone()).unwrap_or_default();
        Self {
            id: entity.id,
            organization_id: entity.organization_id,
            user: domain::models::OrgUserInfo {
                id: entity.user_id,
                email: entity.user_email,
                display_name: entity.user_display_name,
            },
            role: entity.role.into(),
            permissions,
            granted_at: entity.granted_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_org_user_role_conversion() {
        assert_eq!(
            domain::models::OrgUserRole::from(OrgUserRoleDb::Owner),
            domain::models::OrgUserRole::Owner
        );
        assert_eq!(
            OrgUserRoleDb::from(domain::models::OrgUserRole::Admin),
            OrgUserRoleDb::Admin
        );
    }
}
