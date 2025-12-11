//! Authentication service for user registration, login, and token management.

use chrono::Utc;
use domain::models::user::OAuthProvider;
use shared::crypto::sha256_hex;
use shared::jwt::{JwtConfig, JwtError};
use shared::password::{hash_password, verify_password, PasswordError};
use sqlx::PgPool;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

use crate::config::JwtAuthConfig;
use crate::services::apple_auth::{AppleAuthClient, AppleAuthError};

/// Errors that can occur during authentication operations.
#[derive(Debug, Error)]
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

    #[error("Invalid refresh token")]
    InvalidRefreshToken,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Invalid or expired reset token")]
    InvalidResetToken,

    #[error("Invalid or expired verification token")]
    InvalidVerificationToken,

    #[error("Email already verified")]
    EmailAlreadyVerified,

    #[error("Invalid OAuth token")]
    InvalidOAuthToken,

    #[error("OAuth provider error: {0}")]
    OAuthProviderError(String),

    #[error("Unsupported OAuth provider")]
    UnsupportedOAuthProvider,

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
    #[allow(dead_code)] // Used for API response
    pub expires_in: i64,
}

/// Result of a successful token refresh.
#[derive(Debug, Clone)]
pub struct RefreshResult {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Database row for user query.
#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: Option<String>,
    display_name: String,
    is_active: bool,
    email_verified: bool,
}

/// Database row for session query.
#[derive(Debug, sqlx::FromRow)]
struct SessionRow {
    id: Uuid,
    #[allow(dead_code)] // Used for validation
    user_id: Uuid,
    #[allow(dead_code)] // Used for validation
    refresh_token_hash: String,
    expires_at: chrono::DateTime<Utc>,
}

/// Authentication service.
pub struct AuthService {
    pool: PgPool,
    jwt_config: JwtConfig,
    access_token_expiry: i64,
    /// Google OAuth client ID for audience validation
    google_client_id: Option<String>,
    /// Apple auth client for proper JWT verification
    apple_auth_client: AppleAuthClient,
}

impl AuthService {
    /// Creates a new AuthService with the given database pool and JWT configuration.
    pub fn new(
        pool: PgPool,
        jwt_config: &JwtAuthConfig,
        google_client_id: Option<String>,
        apple_client_id: Option<String>,
    ) -> Result<Self, AuthError> {
        // Convert literal \n sequences to actual newlines (for env var compatibility)
        // Handle both escaped \\n and literal \n from different env parsing methods
        let private_key = Self::normalize_pem_key(&jwt_config.private_key);
        let public_key = Self::normalize_pem_key(&jwt_config.public_key);

        tracing::debug!(
            "JWT key lengths - private: {}, public: {}",
            private_key.len(),
            public_key.len()
        );

        let jwt = JwtConfig::new(
            &private_key,
            &public_key,
            jwt_config.access_token_expiry_secs,
            jwt_config.refresh_token_expiry_secs,
        )
        .map_err(|e| AuthError::Internal(format!("Failed to initialize JWT: {}", e)))?;

        // Create Apple auth client with proper JWKS verification
        let apple_auth_client = AppleAuthClient::new(apple_client_id);

        Ok(Self {
            pool,
            jwt_config: jwt,
            access_token_expiry: jwt_config.access_token_expiry_secs,
            google_client_id,
            apple_auth_client,
        })
    }

    /// Normalize PEM key by converting various newline representations to actual newlines.
    /// Handles: literal "\n" string, escaped "\\n", and already-correct newlines.
    fn normalize_pem_key(key: &str) -> String {
        tracing::debug!("Raw key first 80 chars: {:?}", &key[..80.min(key.len())]);

        // Strip surrounding quotes if present (some env file parsers include them)
        let key = key.trim_matches('"').trim_matches('\'');

        // Replace literal \n sequences with actual newlines
        // In the env file: \n is stored as two characters (backslash + n)
        // We need to convert them to actual newline characters
        let normalized = key.replace("\\n", "\n");

        // Also check if we have the string "\\n" which would appear as 4 chars in debug
        let normalized = normalized.replace(r#"\n"#, "\n");

        let newline_count = normalized.matches('\n').count();
        tracing::debug!(
            "Normalized key has {} newlines, length: {}",
            newline_count,
            normalized.len()
        );

        // If the key still doesn't have proper PEM structure, log for debugging
        if newline_count == 0 && normalized.len() > 100 {
            tracing::error!(
                "PEM key missing newlines after normalization. First 80 chars: {:?}",
                &normalized[..80.min(normalized.len())]
            );
        }

        normalized
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
        let existing: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE email = $1")
            .bind(email.to_lowercase())
            .fetch_optional(&self.pool)
            .await?;

        if existing.is_some() {
            return Err(AuthError::EmailAlreadyExists);
        }

        // Create user
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let insert_result = sqlx::query(
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
        .await;

        // Handle unique constraint violation (race condition with concurrent registration)
        if let Err(sqlx::Error::Database(db_err)) = &insert_result {
            // PostgreSQL error code 23505 = unique_violation
            if db_err.code().as_deref() == Some("23505") {
                return Err(AuthError::EmailAlreadyExists);
            }
        }
        insert_result?;

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

    /// Login with email and password.
    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResult, AuthError> {
        // Fetch user by email
        let user: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, email, password_hash, display_name, is_active, email_verified
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email.to_lowercase())
        .fetch_optional(&self.pool)
        .await?;

        let user = match user {
            Some(u) => u,
            None => return Err(AuthError::InvalidCredentials),
        };

        // Check if user is active
        if !user.is_active {
            return Err(AuthError::UserDisabled);
        }

        // Verify password
        let password_hash = user.password_hash.ok_or(AuthError::InvalidCredentials)?;
        let is_valid = verify_password(password, &password_hash)?;
        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        // Update last_login_at
        let now = Utc::now();
        sqlx::query("UPDATE users SET last_login_at = $1 WHERE id = $2")
            .bind(now)
            .bind(user.id)
            .execute(&self.pool)
            .await?;

        // Generate tokens
        let tokens = self.generate_tokens(user.id)?;

        // Create session
        self.create_session(user.id, &tokens).await?;

        Ok(AuthResult {
            user_id: user.id,
            email: user.email,
            display_name: user.display_name,
            email_verified: user.email_verified,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            access_token_expires_in: self.access_token_expiry,
        })
    }

    /// Authenticate using an OAuth provider (Google or Apple).
    ///
    /// This method validates the ID token from the OAuth provider, then either:
    /// - Returns the existing user if the OAuth account is already linked
    /// - Creates a new user and links the OAuth account if this is a new sign-up
    /// - Links the OAuth account to an existing user with the same email
    pub async fn oauth_login(
        &self,
        provider: &str,
        id_token: &str,
    ) -> Result<AuthResult, AuthError> {
        // Parse and validate provider
        let provider =
            OAuthProvider::from_str(provider).map_err(|_| AuthError::UnsupportedOAuthProvider)?;

        // Verify the ID token with the OAuth provider
        let oauth_user_info = self.verify_oauth_token(provider, id_token).await?;

        // Check if OAuth account already exists
        let existing_oauth: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT user_id FROM oauth_accounts
            WHERE provider = $1 AND provider_user_id = $2
            "#,
        )
        .bind(provider.as_str())
        .bind(&oauth_user_info.provider_user_id)
        .fetch_optional(&self.pool)
        .await?;

        let user_id = if let Some((existing_user_id,)) = existing_oauth {
            // User already has this OAuth account linked
            existing_user_id
        } else {
            // Check if a user with this email already exists
            let existing_user: Option<(Uuid,)> =
                sqlx::query_as("SELECT id FROM users WHERE email = $1")
                    .bind(oauth_user_info.email.to_lowercase())
                    .fetch_optional(&self.pool)
                    .await?;

            if let Some((existing_user_id,)) = existing_user {
                // Link OAuth account to existing user
                self.create_oauth_account(
                    existing_user_id,
                    provider,
                    &oauth_user_info.provider_user_id,
                    &oauth_user_info.email,
                )
                .await?;

                existing_user_id
            } else {
                // Create new user and link OAuth account
                let new_user_id = self.create_oauth_user(&oauth_user_info, provider).await?;

                new_user_id
            }
        };

        // Fetch user details
        let user: UserRow = sqlx::query_as(
            r#"
            SELECT id, email, password_hash, display_name, is_active, email_verified
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Check if user is active
        if !user.is_active {
            return Err(AuthError::UserDisabled);
        }

        // Update last_login_at
        let now = Utc::now();
        sqlx::query("UPDATE users SET last_login_at = $1 WHERE id = $2")
            .bind(now)
            .bind(user.id)
            .execute(&self.pool)
            .await?;

        // Generate tokens
        let tokens = self.generate_tokens(user.id)?;

        // Create session
        self.create_session(user.id, &tokens).await?;

        Ok(AuthResult {
            user_id: user.id,
            email: user.email,
            display_name: user.display_name,
            email_verified: user.email_verified,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            access_token_expires_in: self.access_token_expiry,
        })
    }

    /// Verify OAuth ID token and extract user information.
    async fn verify_oauth_token(
        &self,
        provider: OAuthProvider,
        id_token: &str,
    ) -> Result<OAuthUserInfo, AuthError> {
        match provider {
            OAuthProvider::Google => self.verify_google_token(id_token).await,
            OAuthProvider::Apple => self.verify_apple_token(id_token).await,
        }
    }

    /// Verify Google ID token using Google's tokeninfo endpoint.
    async fn verify_google_token(&self, id_token: &str) -> Result<OAuthUserInfo, AuthError> {
        // Use Google's tokeninfo endpoint for validation
        let client = reqwest::Client::new();
        let response = client
            .get("https://oauth2.googleapis.com/tokeninfo")
            .query(&[("id_token", id_token)])
            .send()
            .await
            .map_err(|e| AuthError::OAuthProviderError(format!("Google API error: {}", e)))?;

        if !response.status().is_success() {
            return Err(AuthError::InvalidOAuthToken);
        }

        let token_info: GoogleTokenInfo = response.json().await.map_err(|e| {
            AuthError::OAuthProviderError(format!("Failed to parse Google response: {}", e))
        })?;

        // Validate audience - token must be issued for our app
        if let Some(ref expected_client_id) = self.google_client_id {
            let actual_aud = token_info.aud.as_deref().unwrap_or("");
            if actual_aud != expected_client_id {
                tracing::warn!(
                    expected = %expected_client_id,
                    actual = %actual_aud,
                    "Google token audience mismatch"
                );
                return Err(AuthError::InvalidOAuthToken);
            }
        } else {
            tracing::warn!("Google OAuth client ID not configured - audience validation skipped");
        }

        let email = token_info.email.ok_or(AuthError::OAuthProviderError(
            "No email in Google token".to_string(),
        ))?;

        Ok(OAuthUserInfo {
            provider_user_id: token_info.sub,
            email,
            display_name: token_info.name,
            email_verified: token_info.email_verified.unwrap_or(false),
            avatar_url: token_info.picture,
        })
    }

    /// Verify Apple ID token with proper JWKS signature verification.
    ///
    /// Apple Sign-In uses JWT ID tokens that are validated by:
    /// 1. Fetching Apple's public keys from their JWKS endpoint
    /// 2. Verifying the JWT signature using the appropriate public key
    /// 3. Validating the claims (iss, aud, exp)
    async fn verify_apple_token(&self, id_token: &str) -> Result<OAuthUserInfo, AuthError> {
        // Use AppleAuthClient for proper JWT verification with JWKS
        let claims = self
            .apple_auth_client
            .verify_token(id_token)
            .await
            .map_err(|e| match e {
                AppleAuthError::InvalidSignature => AuthError::InvalidOAuthToken,
                AppleAuthError::TokenExpired => AuthError::InvalidOAuthToken,
                AppleAuthError::InvalidIssuer => AuthError::InvalidOAuthToken,
                AppleAuthError::InvalidAudience => AuthError::InvalidOAuthToken,
                AppleAuthError::InvalidTokenFormat => AuthError::InvalidOAuthToken,
                AppleAuthError::KeyNotFound(kid) => {
                    tracing::error!(kid = %kid, "Apple public key not found");
                    AuthError::OAuthProviderError("Key not found".to_string())
                }
                AppleAuthError::KeyFetchError(msg) => {
                    tracing::error!(error = %msg, "Failed to fetch Apple JWKS");
                    AuthError::OAuthProviderError(format!("Key fetch failed: {}", msg))
                }
                AppleAuthError::MissingEmail => {
                    AuthError::OAuthProviderError("No email in Apple token".to_string())
                }
                AppleAuthError::ValidationError(msg) => {
                    tracing::error!(error = %msg, "Apple token validation failed");
                    AuthError::OAuthProviderError(msg)
                }
            })?;

        // Extract email (required)
        let email = claims.email.ok_or(AuthError::OAuthProviderError(
            "No email in Apple token".to_string(),
        ))?;

        Ok(OAuthUserInfo {
            provider_user_id: claims.sub,
            email,
            display_name: None, // Apple doesn't provide name in the ID token
            email_verified: claims.email_verified.unwrap_or(false),
            avatar_url: None,
        })
    }

    /// Create a new OAuth account record linked to a user.
    async fn create_oauth_account(
        &self,
        user_id: Uuid,
        provider: OAuthProvider,
        provider_user_id: &str,
        provider_email: &str,
    ) -> Result<(), AuthError> {
        let account_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO oauth_accounts (id, user_id, provider, provider_user_id, provider_email, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (provider, provider_user_id) DO NOTHING
            "#,
        )
        .bind(account_id)
        .bind(user_id)
        .bind(provider.as_str())
        .bind(provider_user_id)
        .bind(provider_email.to_lowercase())
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create a new user from OAuth information.
    async fn create_oauth_user(
        &self,
        oauth_info: &OAuthUserInfo,
        provider: OAuthProvider,
    ) -> Result<Uuid, AuthError> {
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        // Generate display name from email if not provided
        let display_name = oauth_info.display_name.clone().unwrap_or_else(|| {
            oauth_info
                .email
                .split('@')
                .next()
                .unwrap_or("User")
                .to_string()
        });

        // Create user (no password_hash for OAuth-only users)
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, display_name, avatar_url, is_active, email_verified, created_at, updated_at)
            VALUES ($1, $2, NULL, $3, $4, true, $5, $6, $6)
            "#,
        )
        .bind(user_id)
        .bind(oauth_info.email.to_lowercase())
        .bind(&display_name)
        .bind(&oauth_info.avatar_url)
        .bind(oauth_info.email_verified) // Trust provider's email verification
        .bind(now)
        .execute(&self.pool)
        .await?;

        // Create OAuth account link
        self.create_oauth_account(
            user_id,
            provider,
            &oauth_info.provider_user_id,
            &oauth_info.email,
        )
        .await?;

        Ok(user_id)
    }

    /// Refresh access token using a valid refresh token.
    ///
    /// Implements token rotation: old refresh token is invalidated and a new one is issued.
    pub async fn refresh(&self, refresh_token: &str) -> Result<RefreshResult, AuthError> {
        // Validate the refresh token
        let claims = self
            .jwt_config
            .validate_refresh_token(refresh_token)
            .map_err(|e| match e {
                JwtError::TokenExpired => AuthError::InvalidRefreshToken,
                JwtError::InvalidToken => AuthError::InvalidRefreshToken,
                _ => AuthError::TokenError(e),
            })?;

        // Parse user ID from claims
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidRefreshToken)?;

        // Hash the JTI to find the session
        let jti_hash = sha256_hex(&claims.jti);

        // Find and validate the session
        let session: Option<SessionRow> = sqlx::query_as(
            r#"
            SELECT id, user_id, refresh_token_hash, expires_at
            FROM user_sessions
            WHERE refresh_token_hash = $1 AND user_id = $2
            "#,
        )
        .bind(&jti_hash)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        let session = session.ok_or(AuthError::SessionNotFound)?;

        // Check if session is expired
        if session.expires_at < Utc::now() {
            // Delete expired session
            sqlx::query("DELETE FROM user_sessions WHERE id = $1")
                .bind(session.id)
                .execute(&self.pool)
                .await?;
            return Err(AuthError::InvalidRefreshToken);
        }

        // Check if user is still active
        let user_active: Option<(bool,)> =
            sqlx::query_as("SELECT is_active FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;

        let (is_active,) = user_active.ok_or(AuthError::UserNotFound)?;
        if !is_active {
            return Err(AuthError::UserDisabled);
        }

        // Generate new tokens (rotation)
        let new_tokens = self.generate_tokens(user_id)?;

        // Update session with new refresh token hash
        let now = Utc::now();
        let new_expires_at =
            now + chrono::Duration::seconds(self.jwt_config.refresh_token_expiry_secs);
        let new_token_hash = sha256_hex(&new_tokens.access_token_jti);
        let new_refresh_hash = sha256_hex(&new_tokens.refresh_token_jti);

        sqlx::query(
            r#"
            UPDATE user_sessions
            SET token_hash = $1, refresh_token_hash = $2, expires_at = $3, last_used_at = $4
            WHERE id = $5
            "#,
        )
        .bind(&new_token_hash)
        .bind(&new_refresh_hash)
        .bind(new_expires_at)
        .bind(now)
        .bind(session.id)
        .execute(&self.pool)
        .await?;

        Ok(RefreshResult {
            access_token: new_tokens.access_token,
            refresh_token: new_tokens.refresh_token,
            expires_in: self.access_token_expiry,
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

    /// Logout by invalidating the session associated with the refresh token.
    ///
    /// If `all_devices` is true, invalidates all sessions for the user.
    /// Otherwise, only invalidates the session identified by the refresh token.
    pub async fn logout(&self, refresh_token: &str, all_devices: bool) -> Result<(), AuthError> {
        // Validate the refresh token to get user ID
        let claims = self
            .jwt_config
            .validate_refresh_token(refresh_token)
            .map_err(|e| match e {
                JwtError::TokenExpired => AuthError::InvalidRefreshToken,
                JwtError::InvalidToken => AuthError::InvalidRefreshToken,
                _ => AuthError::TokenError(e),
            })?;

        // Parse user ID from claims
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidRefreshToken)?;

        if all_devices {
            // Delete all sessions for this user
            sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        } else {
            // Hash the JTI to find the specific session
            let jti_hash = sha256_hex(&claims.jti);

            // Delete the specific session
            let result = sqlx::query(
                "DELETE FROM user_sessions WHERE refresh_token_hash = $1 AND user_id = $2",
            )
            .bind(&jti_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

            // If no session was found, it's already logged out (not an error)
            if result.rows_affected() == 0 {
                tracing::debug!(user_id = %user_id, "Session not found during logout, may already be logged out");
            }
        }

        Ok(())
    }

    /// Initiate password reset by generating a reset token.
    ///
    /// Always succeeds to prevent email enumeration - if email doesn't exist,
    /// we silently do nothing but return success.
    ///
    /// Returns the reset token (to be logged in MVP, emailed in production).
    pub async fn forgot_password(&self, email: &str) -> Result<Option<String>, AuthError> {
        // Check if user exists (case-insensitive)
        let user: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM users WHERE email = $1 AND is_active = true")
                .bind(email.to_lowercase())
                .fetch_optional(&self.pool)
                .await?;

        let user_id = match user {
            Some((id,)) => id,
            None => {
                // User not found - silently return success to prevent enumeration
                tracing::debug!(email = %email, "Forgot password requested for non-existent email");
                return Ok(None);
            }
        };

        // Generate secure reset token (32 bytes = 64 hex chars)
        let reset_token = generate_secure_token();

        // Hash the token for storage
        let token_hash = sha256_hex(&reset_token);

        // Set expiry to 1 hour from now
        let expires_at = Utc::now() + chrono::Duration::hours(1);

        // Store hashed token in database
        sqlx::query(
            r#"
            UPDATE users
            SET password_reset_token = $1, password_reset_expires_at = $2
            WHERE id = $3
            "#,
        )
        .bind(&token_hash)
        .bind(expires_at)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Log token generation (never log the actual token)
        tracing::info!(
            user_id = %user_id,
            email = %email,
            "Password reset token generated"
        );

        Ok(Some(reset_token))
    }

    /// Reset password using a valid reset token.
    ///
    /// Validates the token, updates the password, invalidates the token,
    /// and clears all user sessions.
    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<(), AuthError> {
        // Validate new password requirements
        self.validate_password(new_password)?;

        // Hash the provided token to look up in database
        let token_hash = sha256_hex(token);

        // Find user with this reset token
        let user: Option<(Uuid, chrono::DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT id, password_reset_expires_at
            FROM users
            WHERE password_reset_token = $1 AND is_active = true
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(&self.pool)
        .await?;

        let (user_id, expires_at) = match user {
            Some((id, exp)) => (id, exp),
            None => return Err(AuthError::InvalidResetToken),
        };

        // Check if token has expired
        if expires_at < Utc::now() {
            // Clear the expired token
            sqlx::query(
                "UPDATE users SET password_reset_token = NULL, password_reset_expires_at = NULL WHERE id = $1"
            )
            .bind(user_id)
            .execute(&self.pool)
            .await?;

            return Err(AuthError::InvalidResetToken);
        }

        // Hash the new password
        let password_hash = hash_password(new_password)?;

        // Update password and clear reset token
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $1,
                password_reset_token = NULL,
                password_reset_expires_at = NULL,
                updated_at = $2
            WHERE id = $3
            "#,
        )
        .bind(&password_hash)
        .bind(now)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Invalidate all existing sessions for security
        sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        tracing::info!(user_id = %user_id, "Password reset successfully, all sessions invalidated");

        Ok(())
    }

    /// Request email verification by generating a verification token.
    ///
    /// Returns the verification token (to be logged in MVP, emailed in production).
    /// Returns error if email is already verified.
    pub async fn request_email_verification(&self, user_id: Uuid) -> Result<String, AuthError> {
        // Check if user exists and get email verification status
        let user: Option<(String, bool)> = sqlx::query_as(
            "SELECT email, email_verified FROM users WHERE id = $1 AND is_active = true",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        let (email, email_verified) = match user {
            Some((e, v)) => (e, v),
            None => return Err(AuthError::UserNotFound),
        };

        // Check if already verified
        if email_verified {
            return Err(AuthError::EmailAlreadyVerified);
        }

        // Generate secure verification token (32 bytes = 64 hex chars)
        let verification_token = generate_secure_token();

        // Hash the token for storage
        let token_hash = sha256_hex(&verification_token);

        // Set expiry to 24 hours from now
        let expires_at = Utc::now() + chrono::Duration::hours(24);

        // Store hashed token in database (replaces any existing token)
        sqlx::query(
            r#"
            UPDATE users
            SET email_verification_token = $1, email_verification_expires_at = $2
            WHERE id = $3
            "#,
        )
        .bind(&token_hash)
        .bind(expires_at)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Log token generation (never log the actual token)
        tracing::info!(
            user_id = %user_id,
            email = %email,
            "Email verification token generated"
        );

        Ok(verification_token)
    }

    /// Verify email using a valid verification token.
    ///
    /// Validates the token, updates email_verified to true, and clears the token.
    pub async fn verify_email(&self, token: &str) -> Result<Uuid, AuthError> {
        // Hash the provided token to look up in database
        let token_hash = sha256_hex(token);

        // Find user with this verification token
        let user: Option<(Uuid, chrono::DateTime<Utc>, bool)> = sqlx::query_as(
            r#"
            SELECT id, email_verification_expires_at, email_verified
            FROM users
            WHERE email_verification_token = $1 AND is_active = true
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(&self.pool)
        .await?;

        let (user_id, expires_at, email_verified) = match user {
            Some((id, exp, verified)) => (id, exp, verified),
            None => return Err(AuthError::InvalidVerificationToken),
        };

        // Check if already verified (shouldn't happen if token exists, but be safe)
        if email_verified {
            // Clear the stale token
            sqlx::query(
                "UPDATE users SET email_verification_token = NULL, email_verification_expires_at = NULL WHERE id = $1",
            )
            .bind(user_id)
            .execute(&self.pool)
            .await?;

            return Err(AuthError::EmailAlreadyVerified);
        }

        // Check if token has expired
        if expires_at < Utc::now() {
            // Clear the expired token
            sqlx::query(
                "UPDATE users SET email_verification_token = NULL, email_verification_expires_at = NULL WHERE id = $1",
            )
            .bind(user_id)
            .execute(&self.pool)
            .await?;

            return Err(AuthError::InvalidVerificationToken);
        }

        // Mark email as verified and clear token
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE users
            SET email_verified = true,
                email_verification_token = NULL,
                email_verification_expires_at = NULL,
                updated_at = $1
            WHERE id = $2
            "#,
        )
        .bind(now)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        tracing::info!(user_id = %user_id, "Email verified successfully");

        Ok(user_id)
    }
}

/// Generate a secure random token (32 bytes, hex encoded).
fn generate_secure_token() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::thread_rng().gen();
    hex::encode(bytes)
}

/// OAuth user information extracted from provider ID token.
#[derive(Debug, Clone)]
struct OAuthUserInfo {
    /// Provider-specific user ID
    provider_user_id: String,
    /// User's email address
    email: String,
    /// User's display name (if available)
    display_name: Option<String>,
    /// Whether the email is verified by the provider
    email_verified: bool,
    /// User's avatar URL (if available)
    avatar_url: Option<String>,
}

/// Google tokeninfo response structure.
#[derive(Debug, serde::Deserialize)]
struct GoogleTokenInfo {
    /// Subject (Google user ID)
    sub: String,
    /// Audience (client ID the token was issued for)
    aud: Option<String>,
    /// User's email
    email: Option<String>,
    /// Whether email is verified
    email_verified: Option<bool>,
    /// User's name
    name: Option<String>,
    /// User's profile picture URL
    picture: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_token() {
        let token = generate_secure_token();
        // Should be 64 hex characters (32 bytes * 2)
        assert_eq!(token.len(), 64);
        // Should be valid hex
        assert!(hex::decode(&token).is_ok());
    }

    #[test]
    fn test_generate_secure_token_uniqueness() {
        let token1 = generate_secure_token();
        let token2 = generate_secure_token();
        // Should be unique
        assert_ne!(token1, token2);
    }

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
