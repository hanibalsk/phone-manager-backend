//! Invite entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use super::group::GroupRoleDb;

/// Database row mapping for the group_invites table.
#[derive(Debug, Clone, FromRow)]
pub struct GroupInviteEntity {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub preset_role: GroupRoleDb,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Invite entity with creator info for listing.
#[derive(Debug, Clone, FromRow)]
pub struct InviteWithCreatorEntity {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub preset_role: GroupRoleDb,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    // Creator info
    pub creator_display_name: Option<String>,
}

/// Invite entity with group info for public lookup.
#[derive(Debug, Clone, FromRow)]
pub struct InviteWithGroupEntity {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub preset_role: GroupRoleDb,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
    // Group info
    pub group_name: String,
    pub group_icon_emoji: Option<String>,
    pub member_count: i64,
}
