//! Common test utilities for integration tests.
//!
//! This module provides helper functions and fixtures for running integration tests
//! against a real PostgreSQL database.

use axum::Router;
use phone_manager_api::{app::create_app, config::Config};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Once;
use std::time::Duration;

static INIT: Once = Once::new();

/// Initialize tracing for tests (only once).
pub fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("phone_manager_api=debug,sqlx=warn")
            .with_test_writer()
            .init();
    });
}

/// Create a test database pool.
///
/// Uses the `TEST_DATABASE_URL` environment variable, or falls back to a default
/// test database URL.
pub async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://phone_manager:phone_manager_dev@localhost:5432/phone_manager_test".to_string()
    });

    PgPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Run migrations on the test database.
pub async fn run_migrations(pool: &PgPool) {
    // Read all migration files in order
    let migration_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("persistence/src/migrations");

    let mut entries: Vec<_> = std::fs::read_dir(&migration_dir)
        .expect("Failed to read migrations directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "sql").unwrap_or(false))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let sql = std::fs::read_to_string(entry.path())
            .expect("Failed to read migration file");

        // Execute migration
        sqlx::raw_sql(&sql)
            .execute(pool)
            .await
            .unwrap_or_else(|_| {
                // Migration might already be applied, ignore errors
                sqlx::postgres::PgQueryResult::default()
            });
    }
}

/// Clean up test data from the database.
///
/// This function truncates all user-related tables to ensure a clean slate for tests.
pub async fn cleanup_test_data(pool: &PgPool) {
    // Truncate tables in order respecting foreign keys
    sqlx::query("TRUNCATE TABLE user_sessions CASCADE")
        .execute(pool)
        .await
        .ok();

    sqlx::query("TRUNCATE TABLE oauth_accounts CASCADE")
        .execute(pool)
        .await
        .ok();

    sqlx::query("TRUNCATE TABLE users CASCADE")
        .execute(pool)
        .await
        .ok();
}

/// Test configuration with valid RSA keys for JWT.
pub fn test_config() -> Config {
    // Generate or use test RSA keys
    let private_key = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA0Z3VS5JJcds3xfn/ygWyF8PbnGy0AHB7MxszTjFfOC8JoRv8
qVg5e9GfLgJlNpRuF9VKVOsGEX6GXOP3YrXaJXQGjITHRHPBE5rIKmWvLRU0D6hL
F0x0z2EhPFXJbqXq5MQLF8FNXzRDvKwMHvCkG6cPj/FMUqPvEwUrmZmBPLVZ3T3r
qG6KOJjNpXKpU0gU8LxK+LFfDMH0i0Aq8BXfT8kLsFD0FvcqDKNOOWFfjGKq4VcS
sJmCFPWEvEFb1EfPXvGk6MYkuPpE3fRLzNvXweGewHLd/dVKi4YCF3FVLNsFEe4p
BLkB1qLW6hoNhnWgU1fZlDmGykdHqHLN1H1MdwIDAQABAoIBABnBHzE9K+m7S3wt
jOJVqfIGUZCFh6FUHLjTqqCEhY9K1W9mWCH0l5l+EPx5yG6JwoJWTP/ByVpzJ+3W
vK3p9Y5BCLlLf0lF5ciASjWqwDNLnJHdD6bf8L+pXYI1BKi4M0vy6HOhWS7aFz+p
rAk3zs9jvJQWM7cDIa6FxXIOMNpxJpGL2BE4CBGO3gjSjBSt8g/8W4wC2VKJF7L1
j0ZD+LazgWTjThTMIBOmNnAX5lKdjbYWxuTlqXTFVNsB+5VwVPNHxwB8gHdIQLvi
vHQqNJF/3MAfbKTPKVdl82F0EqLe1fPVHvCZPmxgYHdPHF3jZMF+gwk/oNJIRcaQ
qH7cQmECgYEA5/YcyQ6QXEZ5wLnqvCNe8l7H0BVqqFSVqZvPbJPB2LHa9MHCp2gN
m0cMr5l6IDXg+FupDqgRKR6KAT9FU/lC5DcVpFNlr6xfqDq75sT5TT1MvqJfNWKe
6f0sXPNF/4pCdKb8F5FNvLFs3DEKkJl4xXbZ8GWP9hNFj9YRaEPyYlECgYEA5xrx
yD/A6Or4DUAU5LWRih3h7EqM8RnMvPsXQPiAh3DqYmnYDNnf9WP8BlcKQUwLIcLB
PIiGg0zBYT9PAqhF/TH6MVZZh1cPyS5JBfph7bT2gOJX4M/0g3K5qQ3LrSlnmmmV
FT0n8Ty1H7EfxlCPdLHN4ERPjGw3XVEKkLDdecCgYBzgEMKfGXr7fzT1T4X8GJW
qVdEBOxT3lY9g5Lbxkp1iCmMy6Nqc9NYol8YKJ/M7MH/p3yNDJFt9jneaHKDbQ+M
ZLV3W6R/EXKsxWH8oLxMsZJ5HqEqMxcE9s+eqLNxpjqj1y9FINzR+85YfP5PBhVe
q0c7/K8EBb3Q3vPjPLvwoQKBgFPFMJcGKDLlZvQDT+FGhe3p+Ipo5GEbMp3q5qrB
evdV1C9Dx2d7f1lDfPl8O3hF7oCqUB3WzZkM7L0vNYrM7KJZhTyRM6ld6F0jKM0M
wLKkOT4ygHn0DI7i+txvAmsZRubWTrTyFs8y2e4CgMLOy1+qLj5oUPs4v5AfmMVJ
FuU3AoGBAMxBgz6M7P5W/tVkCxOcnGsNZTp0Svn2Z7pZy6MFMWz5VE+/q3Qp8IFb
GCKqGb0RkpYhbPJaNVWvN5KuqbMBnrt2NTvicuqPyA8u7GZ3W3YPZPRQpS6j1D8f
pTXV7nVx3tCe3j3YC0YnBwCg+PZqLzHBWC0v67JKzLG7Z0VgAWkV
-----END RSA PRIVATE KEY-----"#;

    let public_key = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0Z3VS5JJcds3xfn/ygWy
F8PbnGy0AHB7MxszTjFfOC8JoRv8qVg5e9GfLgJlNpRuF9VKVOsGEX6GXOP3YrXa
JXQGjITHRHPBE5rIKmWvLRU0D6hLF0x0z2EhPFXJbqXq5MQLF8FNXzRDvKwMHvCk
G6cPj/FMUqPvEwUrmZmBPLVZ3T3rqG6KOJjNpXKpU0gU8LxK+LFfDMH0i0Aq8BXf
T8kLsFD0FvcqDKNOOWFfjGKq4VcSsJmCFPWEvEFb1EfPXvGk6MYkuPpE3fRLzNvX
weGewHLd/dVKi4YCF3FVLNsFEe4pBLkB1qLW6hoNhnWgU1fZlDmGykdHqHLN1H1M
dwIDAQAB
-----END PUBLIC KEY-----"#;

    Config {
        server: phone_manager_api::config::ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0, // Use random port
            request_timeout_secs: 30,
            max_body_size: 1048576,
        },
        database: phone_manager_api::config::DatabaseConfig {
            url: std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://phone_manager:phone_manager_dev@localhost:5432/phone_manager_test".to_string()
            }),
            max_connections: 5,
            min_connections: 1,
            connect_timeout_secs: 10,
            idle_timeout_secs: 600,
        },
        logging: phone_manager_api::config::LoggingConfig {
            level: "debug".to_string(),
            format: "pretty".to_string(),
        },
        security: phone_manager_api::config::SecurityConfig {
            cors_origins: vec![],
            rate_limit_per_minute: 0, // Disable rate limiting for tests
        },
        limits: phone_manager_api::config::LimitsConfig {
            max_devices_per_group: 20,
            max_batch_size: 50,
            location_retention_days: 30,
            max_display_name_length: 50,
            max_group_id_length: 50,
        },
        map_matching: phone_manager_api::config::MapMatchingConfig {
            provider: "osrm".to_string(),
            url: "".to_string(),
            timeout_ms: 30000,
            rate_limit_per_minute: 30,
            circuit_breaker_failures: 5,
            circuit_breaker_reset_secs: 60,
            enabled: false,
        },
        jwt: phone_manager_api::config::JwtAuthConfig {
            private_key: private_key.to_string(),
            public_key: public_key.to_string(),
            access_token_expiry_secs: 3600,
            refresh_token_expiry_secs: 86400 * 30,
            leeway_secs: 30,
        },
    }
}

/// Create a test application router.
pub fn create_test_app(config: Config, pool: PgPool) -> Router {
    create_app(config, pool)
}

/// Generate a unique email for testing.
pub fn unique_test_email() -> String {
    format!("test_{}@example.com", uuid::Uuid::new_v4())
}

/// Test user data.
pub struct TestUser {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

impl TestUser {
    pub fn new() -> Self {
        Self {
            email: unique_test_email(),
            password: "SecureP@ss123!".to_string(),
            display_name: "Test User".to_string(),
        }
    }
}

impl Default for TestUser {
    fn default() -> Self {
        Self::new()
    }
}
