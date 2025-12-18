//! Device-group membership entity definitions.
//!
//! Story UGM-3.1: Device-Group Membership Table

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A device-group membership record from the database.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DeviceGroupMembershipEntity {
    pub id: Uuid,
    pub device_id: Uuid,
    pub group_id: Uuid,
    pub added_by: Uuid,
    pub added_at: DateTime<Utc>,
}

/// Device with group membership info (for listing devices in a group).
#[derive(Debug, Clone, FromRow)]
pub struct DeviceInGroupEntity {
    // Device fields
    pub device_id: Uuid,
    pub display_name: String,
    pub last_seen_at: Option<DateTime<Utc>>,
    // Owner fields
    pub owner_user_id: Option<Uuid>,
    pub owner_display_name: Option<String>,
    // Membership fields
    pub membership_id: Uuid,
    pub added_by: Uuid,
    pub added_at: DateTime<Utc>,
}

/// Device with group membership and last location (for listing devices with location).
#[derive(Debug, Clone, FromRow)]
pub struct DeviceInGroupWithLocationEntity {
    // Device fields
    pub device_id: Uuid,
    pub display_name: String,
    pub last_seen_at: Option<DateTime<Utc>>,
    // Owner fields
    pub owner_user_id: Option<Uuid>,
    pub owner_display_name: Option<String>,
    // Membership fields
    pub membership_id: Uuid,
    pub added_by: Uuid,
    pub added_at: DateTime<Utc>,
    // Location fields (optional)
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub accuracy: Option<f32>,
    pub location_timestamp: Option<DateTime<Utc>>,
}

/// Group info for a device's membership (for listing device's groups).
#[derive(Debug, Clone, FromRow)]
pub struct DeviceGroupInfoEntity {
    pub group_id: Uuid,
    pub group_name: String,
    pub group_slug: String,
    pub user_role: String,
    pub membership_id: Uuid,
    pub added_at: DateTime<Utc>,
}
