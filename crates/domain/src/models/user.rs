//! User authentication domain models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Represents a user account in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)] // Never serialize password hash to API responses
    pub password_hash: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// OAuth provider enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    Apple,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::Apple => "apple",
        }
    }
}

impl FromStr for OAuthProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "google" => Ok(OAuthProvider::Google),
            "apple" => Ok(OAuthProvider::Apple),
            _ => Err(format!("Invalid OAuth provider: {}", s)),
        }
    }
}

impl fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents an OAuth account linked to a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: OAuthProvider,
    pub provider_user_id: String,
    pub provider_email: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Represents an active user session with JWT tokens.
#[derive(Debug, Clone)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub refresh_token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_provider_as_str() {
        assert_eq!(OAuthProvider::Google.as_str(), "google");
        assert_eq!(OAuthProvider::Apple.as_str(), "apple");
    }

    #[test]
    fn test_oauth_provider_from_str() {
        assert_eq!(
            OAuthProvider::from_str("google").unwrap(),
            OAuthProvider::Google
        );
        assert_eq!(
            OAuthProvider::from_str("GOOGLE").unwrap(),
            OAuthProvider::Google
        );
        assert_eq!(
            OAuthProvider::from_str("apple").unwrap(),
            OAuthProvider::Apple
        );
        assert_eq!(
            OAuthProvider::from_str("APPLE").unwrap(),
            OAuthProvider::Apple
        );
        assert!(OAuthProvider::from_str("invalid").is_err());
    }

    #[test]
    fn test_oauth_provider_display() {
        assert_eq!(format!("{}", OAuthProvider::Google), "google");
        assert_eq!(format!("{}", OAuthProvider::Apple), "apple");
    }

    #[test]
    fn test_user_struct() {
        let user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: Some("hashed_password".to_string()),
            display_name: Some("Test User".to_string()),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            is_active: true,
            email_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
        };
        assert_eq!(user.email, "test@example.com");
        assert!(user.is_active);
        assert!(!user.email_verified);
    }

    #[test]
    fn test_user_password_hash_not_serialized() {
        let user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: Some("secret_hash".to_string()),
            display_name: Some("Test User".to_string()),
            avatar_url: None,
            is_active: true,
            email_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
        };
        let json = serde_json::to_string(&user).unwrap();
        // Password hash should not appear in JSON
        assert!(!json.contains("secret_hash"));
        assert!(!json.contains("passwordHash"));
        assert!(!json.contains("password_hash"));
    }

    #[test]
    fn test_oauth_account_struct() {
        let account = OAuthAccount {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            provider: OAuthProvider::Google,
            provider_user_id: "google_user_123".to_string(),
            provider_email: Some("test@gmail.com".to_string()),
            created_at: Utc::now(),
        };
        assert_eq!(account.provider, OAuthProvider::Google);
        assert_eq!(account.provider_user_id, "google_user_123");
    }

    #[test]
    fn test_user_session_struct() {
        let session = UserSession {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: "token_hash_123".to_string(),
            refresh_token_hash: "refresh_hash_123".to_string(),
            expires_at: Utc::now(),
            created_at: Utc::now(),
            last_used_at: Utc::now(),
        };
        assert_eq!(session.token_hash, "token_hash_123");
        assert_eq!(session.refresh_token_hash, "refresh_hash_123");
    }
}
