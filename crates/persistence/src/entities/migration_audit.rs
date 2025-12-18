//! Migration audit log entity.
//!
//! Tracks registration group to authenticated group migrations.
//! Story UGM-2.1: Create Migration Audit Log Infrastructure

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Migration status enum matching database enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "migration_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MigrationStatusDb {
    #[default]
    Success,
    Failed,
    Partial,
}

impl std::fmt::Display for MigrationStatusDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failed => write!(f, "failed"),
            Self::Partial => write!(f, "partial"),
        }
    }
}

/// Migration audit log entity mapping to database row.
#[derive(Debug, Clone, FromRow)]
pub struct MigrationAuditLogEntity {
    /// Unique migration ID
    pub id: Uuid,
    /// User who initiated the migration
    pub user_id: Uuid,
    /// Source registration group ID (string)
    pub registration_group_id: String,
    /// Target authenticated group UUID
    pub authenticated_group_id: Uuid,
    /// Number of devices migrated
    pub devices_migrated: i32,
    /// Array of device UUIDs that were migrated
    pub device_ids: Vec<Uuid>,
    /// Migration status
    pub status: MigrationStatusDb,
    /// Error message if migration failed
    pub error_message: Option<String>,
    /// Timestamp of migration
    pub created_at: DateTime<Utc>,
}

/// Migration audit log entity with user email for admin queries.
#[derive(Debug, Clone, FromRow)]
pub struct MigrationAuditLogWithUserEntity {
    /// Unique migration ID
    pub id: Uuid,
    /// User who initiated the migration
    pub user_id: Uuid,
    /// User email (joined from users table)
    pub user_email: Option<String>,
    /// Source registration group ID (string)
    pub registration_group_id: String,
    /// Target authenticated group UUID
    pub authenticated_group_id: Uuid,
    /// Target group name (joined from groups table)
    pub group_name: Option<String>,
    /// Number of devices migrated
    pub devices_migrated: i32,
    /// Array of device UUIDs that were migrated
    pub device_ids: Vec<Uuid>,
    /// Migration status
    pub status: MigrationStatusDb,
    /// Error message if migration failed
    pub error_message: Option<String>,
    /// Timestamp of migration
    pub created_at: DateTime<Utc>,
}
