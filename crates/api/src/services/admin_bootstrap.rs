//! Admin bootstrap service for initial setup.
//!
//! Creates the first admin user on startup if configured via environment variables.
//! This is a one-time operation that checks if an admin already exists.

use shared::crypto::{extract_key_prefix, sha256_hex};
use shared::password::{hash_password, PasswordError};
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::AdminBootstrapConfig;

/// Error types for admin bootstrap.
#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Password hashing error: {0}")]
    PasswordHash(#[from] PasswordError),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Bootstrap admin user if configured and not already done.
///
/// This function should be called after migrations on startup.
/// It is idempotent - if an admin user already exists, it does nothing.
pub async fn bootstrap_admin(
    pool: &PgPool,
    config: &AdminBootstrapConfig,
) -> Result<(), BootstrapError> {
    // Skip if not configured
    if config.bootstrap_email.is_empty() {
        return Ok(());
    }

    if config.bootstrap_password.is_empty() {
        warn!(
            "PM__ADMIN__BOOTSTRAP_EMAIL is set but PM__ADMIN__BOOTSTRAP_PASSWORD is empty - skipping bootstrap"
        );
        return Ok(());
    }

    // Check if any admin already exists (user with admin API key, or with the bootstrap email)
    let admin_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM users u
            WHERE u.email = $1
        ) OR EXISTS(
            SELECT 1 FROM api_keys ak
            WHERE ak.is_admin = true
        )
        "#,
    )
    .bind(&config.bootstrap_email)
    .fetch_one(pool)
    .await?;

    if admin_exists {
        info!("Admin user or bootstrap email already exists - skipping bootstrap");
        return Ok(());
    }

    // Hash the password using argon2 (same as auth service)
    let password_hash = hash_password(&config.bootstrap_password)?;

    // Generate admin API key before transaction (no side effects)
    let api_key = generate_api_key();
    let key_hash = sha256_hex(&api_key);
    // extract_key_prefix always succeeds for our generated keys (pm_ + 32 chars)
    let key_prefix = extract_key_prefix(&api_key).unwrap_or(&api_key[..8]);

    // Use a transaction to ensure user and API key are created atomically.
    // This prevents the scenario where user is created but API key insert fails,
    // leaving the system with no admin access (since the email exists, future
    // boots would skip bootstrap).
    let mut tx = pool.begin().await?;

    // Create admin user
    let user_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (email, password_hash, display_name, is_active, email_verified)
        VALUES ($1, $2, 'System Administrator', true, true)
        RETURNING id
        "#,
    )
    .bind(&config.bootstrap_email)
    .bind(&password_hash)
    .fetch_one(&mut *tx)
    .await?;

    // Create admin API key
    sqlx::query(
        r#"
        INSERT INTO api_keys (key_hash, key_prefix, user_id, is_admin, description)
        VALUES ($1, $2, $3, true, 'Bootstrap admin key')
        "#,
    )
    .bind(&key_hash)
    .bind(&key_prefix)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Commit the transaction - if this fails, both inserts are rolled back
    tx.commit().await?;

    info!(
        email = %config.bootstrap_email,
        user_id = %user_id,
        api_key_prefix = %key_prefix,
        "Bootstrap admin user created successfully"
    );

    warn!(
        "SECURITY: Remove PM__ADMIN__BOOTSTRAP_EMAIL and PM__ADMIN__BOOTSTRAP_PASSWORD \
         from configuration after initial setup. Admin API key: {}",
        api_key
    );

    Ok(())
}

/// Generate a new API key with the pm_ prefix.
fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    let key: String = (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("pm_{}", key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key_format() {
        let key = generate_api_key();
        assert!(key.starts_with("pm_"));
        assert_eq!(key.len(), 35); // "pm_" + 32 chars
    }

    #[test]
    fn test_generate_api_key_has_valid_prefix() {
        let key = generate_api_key();
        // Should be extractable by shared crypto
        assert!(extract_key_prefix(&key).is_some());
        assert_eq!(extract_key_prefix(&key).unwrap().len(), 8);
    }

    #[test]
    fn test_sha256_hex_works() {
        let hash = sha256_hex("test");
        assert_eq!(hash.len(), 64); // SHA-256 produces 32 bytes = 64 hex chars
    }
}
