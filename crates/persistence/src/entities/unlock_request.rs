//! Unlock request entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database enum for unlock request status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "unlock_request_status", rename_all = "lowercase")]
pub enum UnlockRequestStatusDb {
    Pending,
    Approved,
    Denied,
    Expired,
}

impl From<UnlockRequestStatusDb> for String {
    fn from(status: UnlockRequestStatusDb) -> Self {
        match status {
            UnlockRequestStatusDb::Pending => "pending".to_string(),
            UnlockRequestStatusDb::Approved => "approved".to_string(),
            UnlockRequestStatusDb::Denied => "denied".to_string(),
            UnlockRequestStatusDb::Expired => "expired".to_string(),
        }
    }
}

/// Database row mapping for the unlock_requests table.
#[derive(Debug, Clone, FromRow)]
pub struct UnlockRequestEntity {
    pub id: Uuid,
    pub device_id: Uuid,
    pub setting_key: String,
    pub requested_by: Uuid,
    pub status: UnlockRequestStatusDb,
    pub reason: Option<String>,
    pub responded_by: Option<Uuid>,
    pub response_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
}

/// Extended unlock request with device and user details for listing.
#[derive(Debug, Clone, FromRow)]
pub struct UnlockRequestWithDetailsEntity {
    pub id: Uuid,
    pub device_id: Uuid,
    pub device_display_name: String,
    pub setting_key: String,
    pub setting_display_name: String,
    pub requested_by: Uuid,
    pub requester_display_name: Option<String>,
    pub status: UnlockRequestStatusDb,
    pub reason: Option<String>,
    pub responded_by: Option<Uuid>,
    pub responder_display_name: Option<String>,
    pub response_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
}
