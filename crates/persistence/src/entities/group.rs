//! Group entity (database row mapping).

use chrono::{DateTime, Utc};
use domain::models::group::GroupRole;
use sqlx::FromRow;
use uuid::Uuid;

/// Database enum for group_role that maps to PostgreSQL enum type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "group_role", rename_all = "lowercase")]
pub enum GroupRoleDb {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl From<GroupRoleDb> for GroupRole {
    fn from(db_role: GroupRoleDb) -> Self {
        match db_role {
            GroupRoleDb::Owner => GroupRole::Owner,
            GroupRoleDb::Admin => GroupRole::Admin,
            GroupRoleDb::Member => GroupRole::Member,
            GroupRoleDb::Viewer => GroupRole::Viewer,
        }
    }
}

impl From<GroupRole> for GroupRoleDb {
    fn from(role: GroupRole) -> Self {
        match role {
            GroupRole::Owner => GroupRoleDb::Owner,
            GroupRole::Admin => GroupRoleDb::Admin,
            GroupRole::Member => GroupRoleDb::Member,
            GroupRole::Viewer => GroupRoleDb::Viewer,
        }
    }
}

/// Database row mapping for the groups table.
#[derive(Debug, Clone, FromRow)]
pub struct GroupEntity {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon_emoji: Option<String>,
    pub max_devices: i32,
    pub is_active: bool,
    pub settings: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<GroupEntity> for domain::models::Group {
    fn from(entity: GroupEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            slug: entity.slug,
            description: entity.description,
            icon_emoji: entity.icon_emoji,
            max_devices: entity.max_devices,
            is_active: entity.is_active,
            settings: entity.settings,
            created_by: entity.created_by,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

/// Database row mapping for the group_memberships table.
#[derive(Debug, Clone, FromRow)]
pub struct GroupMembershipEntity {
    pub id: Uuid,
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub role: GroupRoleDb,
    pub invited_by: Option<Uuid>,
    pub joined_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<GroupMembershipEntity> for domain::models::GroupMembership {
    fn from(entity: GroupMembershipEntity) -> Self {
        Self {
            id: entity.id,
            group_id: entity.group_id,
            user_id: entity.user_id,
            role: entity.role.into(),
            invited_by: entity.invited_by,
            joined_at: entity.joined_at,
            updated_at: entity.updated_at,
        }
    }
}

/// Extended group entity with member count and user's membership info.
#[derive(Debug, Clone, FromRow)]
pub struct GroupWithMembershipEntity {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon_emoji: Option<String>,
    pub max_devices: i32,
    pub is_active: bool,
    pub settings: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Membership fields
    pub membership_id: Uuid,
    pub role: GroupRoleDb,
    pub joined_at: DateTime<Utc>,
    // Aggregates
    pub member_count: i64,
    pub device_count: i64,
    /// Whether the requesting user's device is assigned to this group.
    pub has_current_device: bool,
}

/// Member entity with user info for listing members.
#[derive(Debug, Clone, FromRow)]
pub struct MemberWithUserEntity {
    // Membership fields
    pub id: Uuid,
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub role: GroupRoleDb,
    pub invited_by: Option<Uuid>,
    pub joined_at: DateTime<Utc>,
    // User fields
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
