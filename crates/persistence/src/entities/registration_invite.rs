//! Registration invite entity (database row mapping).
//!
//! Used for invite-only registration mode.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row mapping for the registration_invites table.
#[derive(Debug, Clone, FromRow)]
pub struct RegistrationInviteEntity {
    pub id: Uuid,
    pub token: String,
    pub email: Option<String>,
    pub created_by: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub used_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub note: Option<String>,
}

impl RegistrationInviteEntity {
    /// Check if this invite is valid (not expired, not used).
    pub fn is_valid(&self) -> bool {
        self.used_at.is_none() && self.expires_at > Utc::now()
    }

    /// Check if this invite can be used by the given email.
    /// Returns true if the invite has no email restriction, or if the email matches.
    pub fn can_be_used_by(&self, email: &str) -> bool {
        match &self.email {
            Some(restricted_email) => restricted_email.eq_ignore_ascii_case(email),
            None => true,
        }
    }
}
