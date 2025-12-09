//! System role entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database enum for system_role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "system_role", rename_all = "snake_case")]
pub enum SystemRoleDb {
    SuperAdmin,
    OrgAdmin,
    OrgManager,
    Support,
    Viewer,
}

impl From<SystemRoleDb> for domain::models::SystemRole {
    fn from(db: SystemRoleDb) -> Self {
        match db {
            SystemRoleDb::SuperAdmin => Self::SuperAdmin,
            SystemRoleDb::OrgAdmin => Self::OrgAdmin,
            SystemRoleDb::OrgManager => Self::OrgManager,
            SystemRoleDb::Support => Self::Support,
            SystemRoleDb::Viewer => Self::Viewer,
        }
    }
}

impl From<domain::models::SystemRole> for SystemRoleDb {
    fn from(domain: domain::models::SystemRole) -> Self {
        match domain {
            domain::models::SystemRole::SuperAdmin => Self::SuperAdmin,
            domain::models::SystemRole::OrgAdmin => Self::OrgAdmin,
            domain::models::SystemRole::OrgManager => Self::OrgManager,
            domain::models::SystemRole::Support => Self::Support,
            domain::models::SystemRole::Viewer => Self::Viewer,
        }
    }
}

/// Database row mapping for the user_system_roles table.
#[derive(Debug, Clone, FromRow)]
pub struct UserSystemRoleEntity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: SystemRoleDb,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
}

impl From<UserSystemRoleEntity> for domain::models::UserSystemRole {
    fn from(entity: UserSystemRoleEntity) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            role: entity.role.into(),
            granted_at: entity.granted_at,
            granted_by: entity.granted_by,
        }
    }
}

/// Database row mapping for the admin_org_assignments table.
#[derive(Debug, Clone, FromRow)]
pub struct AdminOrgAssignmentEntity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
}

impl From<AdminOrgAssignmentEntity> for domain::models::AdminOrgAssignment {
    fn from(entity: AdminOrgAssignmentEntity) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            organization_id: entity.organization_id,
            assigned_at: entity.assigned_at,
            assigned_by: entity.assigned_by,
        }
    }
}

/// Admin org assignment with organization name for list responses.
#[derive(Debug, Clone, FromRow)]
pub struct AdminOrgAssignmentWithNameEntity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
    pub organization_name: Option<String>,
}

impl From<AdminOrgAssignmentWithNameEntity> for domain::models::OrgAssignmentDetail {
    fn from(entity: AdminOrgAssignmentWithNameEntity) -> Self {
        Self {
            id: entity.id,
            organization_id: entity.organization_id,
            organization_name: entity.organization_name,
            assigned_at: entity.assigned_at,
            assigned_by: entity.assigned_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_role_conversion() {
        assert_eq!(
            domain::models::SystemRole::from(SystemRoleDb::SuperAdmin),
            domain::models::SystemRole::SuperAdmin
        );
        assert_eq!(
            SystemRoleDb::from(domain::models::SystemRole::OrgAdmin),
            SystemRoleDb::OrgAdmin
        );
        assert_eq!(
            domain::models::SystemRole::from(SystemRoleDb::Viewer),
            domain::models::SystemRole::Viewer
        );
    }
}
