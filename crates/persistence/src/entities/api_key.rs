//! API key entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database row mapping for the api_keys table.
#[derive(Debug, Clone, FromRow)]
pub struct ApiKeyEntity {
    pub id: i64,
    pub key_hash: String,
    pub key_prefix: String,
    pub name: String,
    pub is_active: bool,
    pub is_admin: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
