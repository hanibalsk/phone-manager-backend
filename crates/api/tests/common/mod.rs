//! Common test utilities for integration tests.
//!
//! This module provides helper functions and fixtures for running integration tests
//! against a real PostgreSQL database.

use axum::Router;
use phone_manager_api::{app::create_app, config::Config};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

/// Create a test database pool.
///
/// Uses the `TEST_DATABASE_URL` environment variable, or falls back to a default
/// test database URL.
pub async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://phone_manager:phone_manager_dev@localhost:5432/phone_manager_test".to_string()
    });

    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(30))
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
            app_base_url: "http://localhost:3000".to_string(),
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
            export_rate_limit_per_hour: 0, // Disable export rate limiting for tests
            forgot_password_rate_limit_per_hour: 0, // Disable auth rate limiting for tests
            request_verification_rate_limit_per_hour: 0, // Disable auth rate limiting for tests
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
        email: phone_manager_api::config::EmailConfig {
            enabled: false,
            provider: "console".to_string(),
            smtp_host: String::new(),
            smtp_port: 587,
            smtp_username: String::new(),
            smtp_password: String::new(),
            smtp_use_tls: true,
            sendgrid_api_key: String::new(),
            ses_region: String::new(),
            sender_email: "test@example.com".to_string(),
            sender_name: "Test".to_string(),
            base_url: "https://test.example.com".to_string(),
            template_style: "html".to_string(),
        },
        oauth: phone_manager_api::config::OAuthConfig {
            google_client_id: String::new(),
            apple_client_id: String::new(),
            apple_team_id: String::new(),
        },
        fcm: phone_manager_api::config::FcmConfig {
            enabled: false,
            project_id: String::new(),
            credentials: String::new(),
            timeout_ms: 10000,
            max_retries: 3,
            high_priority: false,
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

/// Clean up ALL test data from the database.
///
/// This function truncates all tables to ensure a clean slate for tests.
/// Tables are truncated in order respecting foreign key constraints.
pub async fn cleanup_all_test_data(pool: &PgPool) {
    // Truncate all tables in reverse dependency order
    let tables = [
        // Audit and export
        "audit_export_jobs",
        "audit_logs",
        // Fleet and bulk operations
        "bulk_import_jobs",
        "device_commands",
        "device_tokens",
        // Enrollment
        "enrollment_tokens",
        // Device policies
        "device_policies",
        // Organizations
        "org_users",
        "organizations",
        // Device settings and unlock
        "unlock_requests",
        "device_settings",
        "setting_definitions",
        // Groups
        "group_invites",
        "group_memberships",
        "groups",
        // Trips and movement
        "trip_path_corrections",
        "trips",
        "movement_events",
        // Location tracking
        "proximity_alerts",
        "geofences",
        "locations",
        // Core
        "idempotency_keys",
        "api_keys",
        "devices",
        // Auth
        "user_sessions",
        "oauth_accounts",
        "users",
    ];

    for table in tables {
        sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table))
            .execute(pool)
            .await
            .ok();
    }
}

/// Test device data.
#[derive(Debug, Clone)]
pub struct TestDevice {
    pub device_id: String,
    pub display_name: String,
    pub group_id: String,
    pub platform: String,
    pub os_version: String,
    pub app_version: String,
}

impl TestDevice {
    pub fn new() -> Self {
        Self {
            device_id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Device".to_string(),
            group_id: format!("test-group-{}", uuid::Uuid::new_v4().simple()),
            platform: "android".to_string(),
            os_version: "14".to_string(),
            app_version: "1.0.0".to_string(),
        }
    }

    pub fn with_group(mut self, group_id: &str) -> Self {
        self.group_id = group_id.to_string();
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.display_name = name.to_string();
        self
    }
}

impl Default for TestDevice {
    fn default() -> Self {
        Self::new()
    }
}

/// Authenticated user context for tests.
pub struct AuthenticatedUser {
    pub user_id: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
}

/// Register a user and return authentication context.
///
/// Creates a new user via the API and returns their credentials.
pub async fn create_authenticated_user(app: &Router, user: &TestUser) -> AuthenticatedUser {
    use axum::{body::Body, http::{header, Method, Request}};
    use tower::ServiceExt;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&serde_json::json!({
            "email": user.email,
            "password": user.password,
            "display_name": user.display_name
        })).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    AuthenticatedUser {
        user_id: json["userId"].as_str().unwrap().to_string(),
        email: json["email"].as_str().unwrap().to_string(),
        access_token: json["accessToken"].as_str().unwrap().to_string(),
        refresh_token: json["refresh_token"].as_str().unwrap().to_string(),
    }
}

/// Test group data.
#[derive(Debug, Clone)]
pub struct TestGroup {
    pub name: String,
    pub description: Option<String>,
}

impl TestGroup {
    pub fn new() -> Self {
        Self {
            name: format!("Test Group {}", uuid::Uuid::new_v4().simple()),
            description: Some("A test group for integration tests".to_string()),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}

impl Default for TestGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Created group context.
pub struct CreatedGroup {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub invite_code: String,
}

/// Create a group via the API.
///
/// Requires an authenticated user. Returns the created group details.
pub async fn create_test_group(
    app: &Router,
    auth: &AuthenticatedUser,
    group: &TestGroup,
) -> CreatedGroup {
    use axum::{body::Body, http::{header, Method, Request}};
    use tower::ServiceExt;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/groups")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", auth.access_token))
        .body(Body::from(serde_json::to_string(&serde_json::json!({
            "name": group.name,
            "description": group.description
        })).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    CreatedGroup {
        id: json["id"].as_str().unwrap().to_string(),
        slug: json["slug"].as_str().unwrap().to_string(),
        name: json["name"].as_str().unwrap().to_string(),
        invite_code: json["inviteCode"].as_str().unwrap().to_string(),
    }
}

/// Build a JSON request with authentication.
pub fn json_request_with_auth(
    method: axum::http::Method,
    uri: &str,
    body: serde_json::Value,
    token: &str,
) -> axum::http::Request<axum::body::Body> {
    use axum::{body::Body, http::{header, Request}};

    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

/// Build a GET request with authentication.
pub fn get_request_with_auth(uri: &str, token: &str) -> axum::http::Request<axum::body::Body> {
    use axum::{body::Body, http::{header, Method, Request}};

    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap()
}

/// Build a DELETE request with authentication.
pub fn delete_request_with_auth(uri: &str, token: &str) -> axum::http::Request<axum::body::Body> {
    use axum::{body::Body, http::{header, Method, Request}};

    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap()
}

/// Helper to parse JSON response body.
pub async fn parse_response_body(response: axum::response::Response) -> serde_json::Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null)
}

/// Register a device via the API.
pub async fn register_test_device(
    app: &Router,
    auth: &AuthenticatedUser,
    device: &TestDevice,
) -> serde_json::Value {
    use axum::http::Method;
    use tower::ServiceExt;

    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/devices/register",
        serde_json::json!({
            "device_id": device.device_id,
            "display_name": device.display_name,
            "group_id": device.group_id,
            "platform": device.platform,
            "os_version": device.os_version,
            "app_version": device.app_version
        }),
        &auth.access_token,
    );

    let response = app.clone().oneshot(request).await.unwrap();
    parse_response_body(response).await
}

// =============================================================================
// API Key Authentication Helpers (for Admin Endpoints)
// =============================================================================

/// Create an admin API key for testing.
///
/// Returns the raw API key (unhashed) for use in requests.
pub async fn create_test_api_key(pool: &PgPool, name: &str) -> String {
    // Generate a test API key
    let api_key = format!("pm_test_{}", uuid::Uuid::new_v4().simple());
    let key_prefix = &api_key[..12];

    // Hash the key for storage using shared crypto utility
    let key_hash = shared::crypto::sha256_hex(&api_key);

    // Insert into database
    sqlx::query(
        r#"
        INSERT INTO api_keys (id, name, key_prefix, key_hash, is_active, created_at, last_used_at)
        VALUES ($1, $2, $3, $4, true, NOW(), NULL)
        "#
    )
    .bind(uuid::Uuid::new_v4())
    .bind(name)
    .bind(key_prefix)
    .bind(key_hash)
    .execute(pool)
    .await
    .expect("Failed to create test API key");

    api_key
}

/// Build a JSON request with API key authentication.
pub fn json_request_with_api_key(
    method: axum::http::Method,
    uri: &str,
    body: serde_json::Value,
    api_key: &str,
) -> axum::http::Request<axum::body::Body> {
    use axum::{body::Body, http::{header, Request}};

    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", api_key)
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

/// Build a GET request with API key authentication.
pub fn get_request_with_api_key(uri: &str, api_key: &str) -> axum::http::Request<axum::body::Body> {
    use axum::{body::Body, http::{Method, Request}};

    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("X-API-Key", api_key)
        .body(Body::empty())
        .unwrap()
}

/// Build a DELETE request with API key authentication.
pub fn delete_request_with_api_key(uri: &str, api_key: &str) -> axum::http::Request<axum::body::Body> {
    use axum::{body::Body, http::{Method, Request}};

    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("X-API-Key", api_key)
        .body(Body::empty())
        .unwrap()
}

/// Build a PUT request with API key authentication.
pub fn put_request_with_api_key(
    uri: &str,
    body: serde_json::Value,
    api_key: &str,
) -> axum::http::Request<axum::body::Body> {
    use axum::{body::Body, http::{header, Method, Request}};

    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", api_key)
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

// =============================================================================
// Organization Test Helpers
// =============================================================================

/// Test organization data.
#[derive(Debug, Clone)]
pub struct TestOrganization {
    pub name: String,
    pub slug: String,
    pub billing_email: String,
}

impl TestOrganization {
    pub fn new() -> Self {
        let unique_id = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
        Self {
            name: format!("Test Org {}", unique_id),
            slug: format!("test-org-{}", unique_id),
            billing_email: format!("billing-{}@example.com", unique_id),
        }
    }
}

impl Default for TestOrganization {
    fn default() -> Self {
        Self::new()
    }
}

/// Created organization context.
pub struct CreatedOrganization {
    pub id: String,
    pub name: String,
    pub slug: String,
}

/// Create an organization via the admin API.
pub async fn create_test_organization(
    app: &Router,
    api_key: &str,
    org: &TestOrganization,
) -> CreatedOrganization {
    use axum::http::Method;
    use tower::ServiceExt;

    let request = json_request_with_api_key(
        Method::POST,
        "/api/admin/v1/organizations",
        serde_json::json!({
            "name": org.name,
            "slug": org.slug,
            "billing_email": org.billing_email
        }),
        api_key,
    );

    let response = app.clone().oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;

    CreatedOrganization {
        id: body["organization"]["id"].as_str().unwrap().to_string(),
        name: body["organization"]["name"].as_str().unwrap().to_string(),
        slug: body["organization"]["slug"].as_str().unwrap().to_string(),
    }
}
