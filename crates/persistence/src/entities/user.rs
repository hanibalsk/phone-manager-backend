//! User authentication entity (database row mapping).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::str::FromStr;
use uuid::Uuid;

/// Database row mapping for the users table.
#[derive(Debug, Clone, FromRow)]
pub struct UserEntity {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<UserEntity> for domain::models::User {
    fn from(entity: UserEntity) -> Self {
        Self {
            id: entity.id,
            email: entity.email,
            password_hash: entity.password_hash,
            display_name: entity.display_name,
            avatar_url: entity.avatar_url,
            is_active: entity.is_active,
            email_verified: entity.email_verified,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            last_login_at: entity.last_login_at,
        }
    }
}

/// Database row mapping for the oauth_accounts table.
#[derive(Debug, Clone, FromRow)]
pub struct OAuthAccountEntity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub provider_email: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<OAuthAccountEntity> for domain::models::OAuthAccount {
    fn from(entity: OAuthAccountEntity) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            provider: domain::models::OAuthProvider::from_str(&entity.provider)
                .unwrap_or(domain::models::OAuthProvider::Google), // Default fallback
            provider_user_id: entity.provider_user_id,
            provider_email: entity.provider_email,
            created_at: entity.created_at,
        }
    }
}

/// Database row mapping for the user_sessions table.
#[derive(Debug, Clone, FromRow)]
pub struct UserSessionEntity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub refresh_token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

impl From<UserSessionEntity> for domain::models::UserSession {
    fn from(entity: UserSessionEntity) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            token_hash: entity.token_hash,
            refresh_token_hash: entity.refresh_token_hash,
            expires_at: entity.expires_at,
            created_at: entity.created_at,
            last_used_at: entity.last_used_at,
        }
    }
}
