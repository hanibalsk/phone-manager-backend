//! Admin group entity definitions for database queries.
//!
//! Story 14.4: Group Management Admin Endpoints

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Entity for admin group list item.
#[derive(Debug, Clone, FromRow)]
pub struct AdminGroupEntity {
    pub group_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon_emoji: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub member_count: i64,
    pub device_count: i64,
    pub owner_id: Option<Uuid>,
    pub owner_email: Option<String>,
    pub owner_display_name: Option<String>,
}

/// Entity for admin group profile (detail view).
#[derive(Debug, Clone, FromRow)]
pub struct AdminGroupProfileEntity {
    pub group_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon_emoji: Option<String>,
    pub max_devices: i32,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub member_count: i64,
    pub device_count: i64,
}

/// Entity for group member info.
#[derive(Debug, Clone, FromRow)]
pub struct GroupMemberEntity {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

/// Entity for group device info.
#[derive(Debug, Clone, FromRow)]
pub struct GroupDeviceEntity {
    pub id: i64,
    pub device_id: Uuid,
    pub display_name: String,
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// Entity for group summary statistics.
#[derive(Debug, Clone, FromRow)]
pub struct AdminGroupSummaryEntity {
    pub total_groups: i64,
    pub active_groups: i64,
    pub total_members: i64,
    pub total_devices: i64,
}
