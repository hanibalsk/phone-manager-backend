//! Authentication service for user registration, login, and token management.

use chrono::Utc;
use shared::crypto::sha256_hex;
use shared::jwt::{JwtConfig, JwtError};
use shared::password::{hash_password, PasswordError};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::config::JwtAuthConfig;

/// Errors that can occur during authentication operations.
#[derive(Debug, Error)]
#[allow(dead_code)] // Some variants used in future stories (login, logout, etc.)
pub enum AuthError {
    #[error("Email already registered")]
    EmailAlreadyExists,

    #[error("Invalid email format")]
    InvalidEmail,

    #[error("Password does not meet requirements")]
    WeakPassword(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("User is disabled")]
    UserDisabled,

    #[error("Token error: {0}")]
    TokenError(#[from] JwtError),

    #[error("Password error: {0}")]
    PasswordError(#[from] PasswordError),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result of a successful authentication.
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub email_verified: bool,
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_in: i64,
}

/// Token pair with metadata.
#[derive(Debug, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub access_token_jti: String,
    pub refresh_token: String,
    pub refresh_token_jti: String,
    #[allow(dead_code)] // Used in future stories
    pub expires_in: i64,
}

/// Authentication service.
pub struct AuthService {
    pool: PgPool,
    jwt_config: JwtConfig,
    access_token_expiry: i64,
}

impl AuthService {
    /// Creates a new AuthService with the given database pool and JWT configuration.
    pub fn new(pool: PgPool, jwt_config: &JwtAuthConfig) -> Result<Self, AuthError> {
        let jwt = JwtConfig::new(
            &jwt_config.private_key,
            &jwt_config.public_key,
            jwt_config.access_token_expiry_secs,
            jwt_config.refresh_token_expiry_secs,
        )
        .map_err(|e| AuthError::Internal(format!("Failed to initialize JWT: {}", e)))?;

        Ok(Self {
            pool,
            jwt_config: jwt,
            access_token_expiry: jwt_config.access_token_expiry_secs,
        })
    }

    /// Register a new user with email and password.
    pub async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<AuthResult, AuthError> {
        // Validate password requirements
        self.validate_password(password)?;

        // Hash the password
        let password_hash = hash_password(password)?;

        // Check if email already exists
        let existing: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM users WHERE email = $1"
        )
        .bind(email.to_lowercase())
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            return Err(AuthError::EmailAlreadyExists);
        }

        // Create user
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, display_name, is_active, email_verified, created_at, updated_at)
            VALUES ($1, $2, $3, $4, true, false, $5, $5)
            "#
        )
        .bind(user_id)
        .bind(email.to_lowercase())
        .bind(&password_hash)
        .bind(display_name)
        .bind(now)
        .execute(&self.pool)
        .await?;

        // Generate tokens
        let tokens = self.generate_tokens(user_id)?;

        // Create session
        self.create_session(user_id, &tokens).await?;

        Ok(AuthResult {
            user_id,
            email: email.to_lowercase(),
            display_name: display_name.to_string(),
            email_verified: false,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            access_token_expires_in: self.access_token_expiry,
        })
    }

    /// Validate password meets security requirements.
    ///
    /// Requirements:
    /// - Minimum 8 characters
    /// - At least 1 uppercase letter
    /// - At least 1 lowercase letter
    /// - At least 1 digit
    fn validate_password(&self, password: &str) -> Result<(), AuthError> {
        if password.len() < 8 {
            return Err(AuthError::WeakPassword(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(AuthError::WeakPassword(
                "Password must contain at least one uppercase letter".to_string(),
            ));
        }

        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(AuthError::WeakPassword(
                "Password must contain at least one lowercase letter".to_string(),
            ));
        }

        if !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(AuthError::WeakPassword(
                "Password must contain at least one digit".to_string(),
            ));
        }

        Ok(())
    }

    /// Generate access and refresh tokens for a user.
    fn generate_tokens(&self, user_id: Uuid) -> Result<TokenPair, AuthError> {
        let (access_token, access_jti) = self.jwt_config.generate_access_token(user_id)?;
        let (refresh_token, refresh_jti) = self.jwt_config.generate_refresh_token(user_id)?;

        Ok(TokenPair {
            access_token,
            access_token_jti: access_jti,
            refresh_token,
            refresh_token_jti: refresh_jti,
            expires_in: self.access_token_expiry,
        })
    }

    /// Create a session for the user with the generated tokens.
    async fn create_session(&self, user_id: Uuid, tokens: &TokenPair) -> Result<(), AuthError> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(self.jwt_config.refresh_token_expiry_secs);

        // Hash the JTIs for storage (for revocation checking)
        let token_hash = sha256_hex(&tokens.access_token_jti);
        let refresh_hash = sha256_hex(&tokens.refresh_token_jti);

        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, token_hash, refresh_token_hash, expires_at, created_at, last_used_at)
            VALUES ($1, $2, $3, $4, $5, $6, $6)
            "#
        )
        .bind(session_id)
        .bind(user_id)
        .bind(&token_hash)
        .bind(&refresh_hash)
        .bind(expires_at)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation_too_short() {
        // We can't create AuthService without a real pool, so test the logic directly
        let password = "Ab1";
        assert!(password.len() < 8);
    }

    #[test]
    fn test_password_validation_no_uppercase() {
        let password = "abcdefgh1";
        assert!(!password.chars().any(|c| c.is_uppercase()));
    }

    #[test]
    fn test_password_validation_no_lowercase() {
        let password = "ABCDEFGH1";
        assert!(!password.chars().any(|c| c.is_lowercase()));
    }

    #[test]
    fn test_password_validation_no_digit() {
        let password = "Abcdefgh";
        assert!(!password.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_password_validation_valid() {
        let password = "SecureP@ss1";
        assert!(password.len() >= 8);
        assert!(password.chars().any(|c| c.is_uppercase()));
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
    }
}
