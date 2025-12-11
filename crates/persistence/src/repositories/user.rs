//! User repository for database operations.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{OAuthAccountEntity, UserEntity, UserSessionEntity};
use crate::metrics::QueryTimer;

/// Repository for user-related database operations.
#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    /// Creates a new UserRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Find a user by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<UserEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_user_by_id");
        let result = sqlx::query_as::<_, UserEntity>(
            r#"
            SELECT id, email, password_hash, display_name, avatar_url, is_active, email_verified,
                   created_at, updated_at, last_login_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find a user by email address.
    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_user_by_email");
        let result = sqlx::query_as::<_, UserEntity>(
            r#"
            SELECT id, email, password_hash, display_name, avatar_url, is_active, email_verified,
                   created_at, updated_at, last_login_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Create a new user account.
    pub async fn create_user(
        &self,
        email: &str,
        password_hash: Option<&str>,
        display_name: Option<&str>,
    ) -> Result<UserEntity, sqlx::Error> {
        let timer = QueryTimer::new("create_user");
        let result = sqlx::query_as::<_, UserEntity>(
            r#"
            INSERT INTO users (email, password_hash, display_name, is_active, email_verified)
            VALUES ($1, $2, $3, true, false)
            RETURNING id, email, password_hash, display_name, avatar_url, is_active, email_verified,
                      created_at, updated_at, last_login_at
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(display_name)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Update user's last login timestamp.
    pub async fn update_last_login(
        &self,
        user_id: Uuid,
        last_login_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let timer = QueryTimer::new("update_user_last_login");
        sqlx::query(
            r#"
            UPDATE users
            SET last_login_at = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(last_login_at)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(())
    }

    /// Find an OAuth account by provider and provider user ID.
    pub async fn find_oauth_account(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Option<OAuthAccountEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_oauth_account");
        let result = sqlx::query_as::<_, OAuthAccountEntity>(
            r#"
            SELECT id, user_id, provider, provider_user_id, provider_email, created_at
            FROM oauth_accounts
            WHERE provider = $1 AND provider_user_id = $2
            "#,
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Create an OAuth account linking.
    pub async fn create_oauth_account(
        &self,
        user_id: Uuid,
        provider: &str,
        provider_user_id: &str,
        provider_email: Option<&str>,
    ) -> Result<OAuthAccountEntity, sqlx::Error> {
        let timer = QueryTimer::new("create_oauth_account");
        let result = sqlx::query_as::<_, OAuthAccountEntity>(
            r#"
            INSERT INTO oauth_accounts (user_id, provider, provider_user_id, provider_email)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, provider, provider_user_id, provider_email, created_at
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .bind(provider_user_id)
        .bind(provider_email)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Create a new user session.
    pub async fn create_session(
        &self,
        user_id: Uuid,
        token_hash: &str,
        refresh_token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<UserSessionEntity, sqlx::Error> {
        let timer = QueryTimer::new("create_user_session");
        let result = sqlx::query_as::<_, UserSessionEntity>(
            r#"
            INSERT INTO user_sessions (user_id, token_hash, refresh_token_hash, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, token_hash, refresh_token_hash, expires_at, created_at, last_used_at
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(refresh_token_hash)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find a session by token hash.
    pub async fn find_session_by_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<UserSessionEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_session_by_token");
        let result = sqlx::query_as::<_, UserSessionEntity>(
            r#"
            SELECT id, user_id, token_hash, refresh_token_hash, expires_at, created_at, last_used_at
            FROM user_sessions
            WHERE token_hash = $1 AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Delete a session by token hash (logout).
    pub async fn delete_session_by_token(&self, token_hash: &str) -> Result<(), sqlx::Error> {
        let timer = QueryTimer::new("delete_session_by_token");
        sqlx::query(
            r#"
            DELETE FROM user_sessions
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(())
    }

    /// Update session last_used_at timestamp.
    pub async fn update_session_last_used(
        &self,
        token_hash: &str,
        last_used_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let timer = QueryTimer::new("update_session_last_used");
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET last_used_at = $1
            WHERE token_hash = $2
            "#,
        )
        .bind(last_used_at)
        .bind(token_hash)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(())
    }

    /// Get MFA status for a user.
    pub async fn get_mfa_status(&self, user_id: Uuid) -> Result<Option<MfaStatusRow>, sqlx::Error> {
        let timer = QueryTimer::new("get_mfa_status");
        let result = sqlx::query_as::<_, MfaStatusRow>(
            r#"
            SELECT id, mfa_enabled, mfa_method, mfa_enrolled_at, mfa_forced, mfa_forced_at, mfa_forced_by
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Set MFA forced requirement for a user.
    pub async fn force_mfa(
        &self,
        user_id: Uuid,
        forced_by: Uuid,
    ) -> Result<Option<MfaStatusRow>, sqlx::Error> {
        let timer = QueryTimer::new("force_mfa");
        let now = Utc::now();
        let result = sqlx::query_as::<_, MfaStatusRow>(
            r#"
            UPDATE users
            SET mfa_forced = true, mfa_forced_at = $2, mfa_forced_by = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING id, mfa_enabled, mfa_method, mfa_enrolled_at, mfa_forced, mfa_forced_at, mfa_forced_by
            "#,
        )
        .bind(user_id)
        .bind(now)
        .bind(forced_by)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Reset MFA for a user (remove all MFA configuration).
    pub async fn reset_mfa(&self, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let timer = QueryTimer::new("reset_mfa");
        let result = sqlx::query(
            r#"
            UPDATE users
            SET mfa_enabled = false,
                mfa_method = NULL,
                mfa_secret = NULL,
                mfa_enrolled_at = NULL,
                mfa_forced = false,
                mfa_forced_at = NULL,
                mfa_forced_by = NULL,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(result.rows_affected() > 0)
    }

    /// List active sessions for a user.
    pub async fn list_user_sessions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserSessionRow>, sqlx::Error> {
        let timer = QueryTimer::new("list_user_sessions");
        let result = sqlx::query_as::<_, UserSessionRow>(
            r#"
            SELECT id, user_id, token_hash, expires_at, created_at, last_used_at,
                   device_name, device_type, browser, os,
                   ip_address::text as ip_address, location, user_agent
            FROM user_sessions
            WHERE user_id = $1 AND expires_at > NOW()
            ORDER BY last_used_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Count active sessions for a user.
    pub async fn count_user_sessions(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_user_sessions");
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM user_sessions
            WHERE user_id = $1 AND expires_at > NOW()
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        timer.record();
        Ok(count.0)
    }

    /// Revoke a specific session by ID.
    pub async fn revoke_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let timer = QueryTimer::new("revoke_session");
        let result = sqlx::query(
            r#"
            DELETE FROM user_sessions
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(result.rows_affected() > 0)
    }

    /// Revoke all sessions for a user.
    pub async fn revoke_all_sessions(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("revoke_all_sessions");
        let result = sqlx::query(
            r#"
            DELETE FROM user_sessions
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        timer.record();
        Ok(result.rows_affected() as i64)
    }
}

/// Row for MFA status queries.
#[derive(Debug, sqlx::FromRow)]
pub struct MfaStatusRow {
    pub id: Uuid,
    pub mfa_enabled: bool,
    pub mfa_method: Option<String>,
    pub mfa_enrolled_at: Option<DateTime<Utc>>,
    pub mfa_forced: bool,
    pub mfa_forced_at: Option<DateTime<Utc>>,
    pub mfa_forced_by: Option<Uuid>,
}

/// Row for user session queries with metadata.
#[derive(Debug, sqlx::FromRow)]
pub struct UserSessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub ip_address: Option<String>,
    pub location: Option<String>,
    pub user_agent: Option<String>,
}

#[cfg(test)]
mod tests {
    // Note: UserRepository tests require database connection and are covered by integration tests
}
