//! Common test utilities for integration tests.
//!
//! This module provides helper functions and fixtures for running integration tests
//! against a real PostgreSQL database.

// Allow dead code in this module - these are helper utilities that may not be used
// by all integration tests but are intentionally available for future use.
#![allow(dead_code)]

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
    // Test RSA keys in PKCS#8 format (generated with openssl)
    let private_key = r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC1+DkLQQl+TPdV
ui3DgGa/pT+x+JhG57LUNVRyxZ+t5IVnZPkJxG8eT2LDnXt/bl5cY0NJUrKCP92k
C+RS7To/n3wwmNHj5wYJALQ1rNtnRLomkIxrIGNO7WNfwhurqiDsRksSIlbUTNT0
q3p+1ajxbIDtIEW9b0zo3WD4+arIkD1gCjBel4lXT0cgUzt2Mmv+5IeI4MXI+8Ek
mZzm+fl/JVrNuE2PrplIJb+owHVODosT2xFikihG3cJkpMUtzbLR0OxwjVwV8Uf8
1Cmaiw7Q9fcF8N+0C0DfekEQW2JOmdQKQ2W1JWV5NUn7FOCd+0QLf14BvQ8lcu5m
ksnQOXdhAgMBAAECggEAA7IV3n+kpLcFcu1EDqtl6tB9Waz10sLT4/FtVKNk2dBB
UVdAo40kwJXWKKjjIDRqoC+35x5R18laRAGl0nVU8IPZrtb7tEg13CryfgCTuCYy
LaRT5b0Tpz+0+/XiP/tFjebjkWu3HbqtvIZbB4ZpVvXgLHCyWeWPx07vsD7J1Cbo
+L1d/0R9eDcl3HhOTKHuLhqxETvhEMUR/h61pFf8TX2nKokmnk/CjZ6zfO7G+MOh
PeDIQkPQRixZV6gKSDi0PTqcJTp2Iqa4jIRKLVOClIefJIYYNtTu3OUisgnNq2QJ
8lxr2PIriV8+LpVyiF1WKQDm+3HepuatO3eapNJqDQKBgQDuaf/NiRyCYaF3h+eg
c5MCLgiN2aGdB2zSJyAizxWv2xzLAKlTh/SPEPU1JQ3eM5zD37VaZGCpfg13ERyJ
l/Ut4iT+gWuheKtyMvwm7c17zdQQawLJOfXTwverS4O1brpRYnorBsxTU0pHirtb
MWyVQeicHlid1Kv5DFEsPqFBjwKBgQDDZGBpQFN01yvG0kgRTyDkU917JDKZiGiD
DX7oe/p5cOFkGrOWT5Z70D2ZZRCpRWmBrCkmigITp83jFC4J6YPNdcJcXc0H6Xc6
JHchtv6aHvt/GaJbijYuopGqggF38dEFLM/rwJ3VpnD2KaQgGUz+u+vF3E3rr4kx
VXq31j9gDwKBgQDBEXXlrDM6InXvpk8c0HssOLsUpDkMQQcO6EBN8AVP89DNVCvL
ST3y3Xi1INyqJIG+3VqvaLoeh8W/tku14Sjbj1cGAyh2CpJMWJ15qPnOWFBzOzV2
X0mDw09tmCmAs7qOTYFBdq/gioKMjPxMTSnxdP457xk0NxVNCXxyqAVOYQKBgQCx
UZ+ZBNJ4H2lP9reGVcwgyecegJwW708BV7cLHrARk5pIMV83EqUbWcD9O1WieCam
kmmJ2wbFdayH3mFlh3CgfbTUBCA0hPA5aKxggWSO030jPE02S7ieG9Sb632Pr3kj
/CX46gWSxYiQLPwQUUWpizsNhb+FGvkjN1K2EQ3UiwKBgAY/m2QhNi1noHa8GMfi
/8zO0llSOw4XkeJNOvQUAUczG4I27TX3Pg38Wlwa6LLjtvKwvjBC6g6CRTF3i7oS
pwmeRGTwuh6dQ+3qLlgTrbZ3OnfiD1pmpqWiaQHZgqycT0EMB3U6CsPsANOfP5qz
U3lyhj2Z6dpCN9rMuUGrQjzy
-----END PRIVATE KEY-----"#;

    let public_key = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAtfg5C0EJfkz3Vbotw4Bm
v6U/sfiYRuey1DVUcsWfreSFZ2T5CcRvHk9iw517f25eXGNDSVKygj/dpAvkUu06
P598MJjR4+cGCQC0NazbZ0S6JpCMayBjTu1jX8Ibq6og7EZLEiJW1EzU9Kt6ftWo
8WyA7SBFvW9M6N1g+PmqyJA9YAowXpeJV09HIFM7djJr/uSHiODFyPvBJJmc5vn5
fyVazbhNj66ZSCW/qMB1Tg6LE9sRYpIoRt3CZKTFLc2y0dDscI1cFfFH/NQpmosO
0PX3BfDftAtA33pBEFtiTpnUCkNltSVleTVJ+xTgnftEC39eAb0PJXLuZpLJ0Dl3
YQIDAQAB
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
            max_webhooks_per_device: Some(10),
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
        frontend: phone_manager_api::config::FrontendConfig::default(),
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
        // Note: setting_definitions is reference data seeded in migrations, don't delete it
        "unlock_requests",
        "device_settings",
        // Groups
        "group_invites",
        "group_memberships",
        "groups",
        // Trips and movement
        "trip_path_corrections",
        "trips",
        "movement_events",
        // Webhooks (must come before webhooks and geofence_events)
        "webhook_deliveries",
        // Geofence events (must come before geofences)
        "geofence_events",
        // Location tracking
        "proximity_alerts",
        // Webhooks (must come before devices)
        "webhooks",
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
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or_else(|_| {
        panic!("Failed to parse response body. Status: {}, Body: {:?}", status, String::from_utf8_lossy(&body));
    });

    if !status.is_success() {
        panic!("Registration failed with status: {}, body: {}", status, json);
    }

    AuthenticatedUser {
        user_id: json["user"]["id"].as_str().unwrap_or_else(|| {
            panic!("Missing user.id in response. Full response: {}", json);
        }).to_string(),
        email: json["user"]["email"].as_str().unwrap_or_else(|| {
            panic!("Missing user.email in response. Full response: {}", json);
        }).to_string(),
        access_token: json["tokens"]["access_token"].as_str().unwrap_or_else(|| {
            panic!("Missing tokens.access_token in response. Full response: {}", json);
        }).to_string(),
        refresh_token: json["tokens"]["refresh_token"].as_str().unwrap_or_else(|| {
            panic!("Missing tokens.refresh_token in response. Full response: {}", json);
        }).to_string(),
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
/// Also creates a default invite code for the group.
pub async fn create_test_group(
    app: &Router,
    auth: &AuthenticatedUser,
    group: &TestGroup,
) -> CreatedGroup {
    use axum::{body::Body, http::{header, Method, Request}};
    use tower::ServiceExt;

    // Step 1: Create the group
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

    let group_id = json["id"].as_str().unwrap().to_string();
    let slug = json["slug"].as_str().unwrap().to_string();
    let name = json["name"].as_str().unwrap().to_string();

    // Step 2: Create an invite for the group
    let invite_request = Request::builder()
        .method(Method::POST)
        .uri(&format!("/api/v1/groups/{}/invites", group_id))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", auth.access_token))
        .body(Body::from(serde_json::to_string(&serde_json::json!({
            "max_uses": 100,
            "expires_in_hours": 24
        })).unwrap()))
        .unwrap();

    let invite_response = app.clone().oneshot(invite_request).await.unwrap();
    let invite_body = axum::body::to_bytes(invite_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let invite_json: serde_json::Value = serde_json::from_slice(&invite_body).unwrap();
    let invite_code = invite_json["code"].as_str().unwrap_or("").to_string();

    CreatedGroup {
        id: group_id,
        slug,
        name,
        invite_code,
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
///
/// Requires both API key (for the middleware) and JWT (for user linking).
/// The pool parameter is used to create a test API key.
pub async fn register_test_device(
    app: &Router,
    pool: &PgPool,
    auth: &AuthenticatedUser,
    device: &TestDevice,
) -> serde_json::Value {
    use axum::http::Method;
    use tower::ServiceExt;

    // Create an API key for this test
    let api_key = create_test_api_key(pool, "test_device_registration").await;

    let request = json_request_with_api_key_and_jwt(
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
        &api_key,
        &auth.access_token,
    );

    let response = app.clone().oneshot(request).await.unwrap();
    parse_response_body(response).await
}

// =============================================================================
// API Key Authentication Helpers (for Admin Endpoints)
// =============================================================================

/// Create an API key for testing (non-admin).
///
/// Returns the raw API key (unhashed) for use in requests.
pub async fn create_test_api_key(pool: &PgPool, name: &str) -> String {
    // Generate a test API key
    let api_key = format!("pm_test_{}", uuid::Uuid::new_v4().simple());
    // key_prefix is first 8 chars after "pm_" (chars 3..11), matching shared::crypto::extract_key_prefix
    let key_prefix = shared::crypto::extract_key_prefix(&api_key)
        .expect("Test API key should have valid format");

    // Hash the key for storage using shared crypto utility
    let key_hash = shared::crypto::sha256_hex(&api_key);

    // Insert into database (id is BIGSERIAL, auto-generated)
    sqlx::query(
        r#"
        INSERT INTO api_keys (name, key_prefix, key_hash, is_active, is_admin, created_at, last_used_at)
        VALUES ($1, $2, $3, true, false, NOW(), NULL)
        "#
    )
    .bind(name)
    .bind(key_prefix)
    .bind(key_hash)
    .execute(pool)
    .await
    .expect("Failed to create test API key");

    api_key
}

/// Create an admin API key for testing (with is_admin = true).
///
/// Returns the raw API key (unhashed) for use in requests.
pub async fn create_test_admin_api_key(pool: &PgPool, name: &str) -> String {
    // Generate a test API key
    let api_key = format!("pm_admin_{}", uuid::Uuid::new_v4().simple());
    // key_prefix is first 8 chars after "pm_" (chars 3..11), matching shared::crypto::extract_key_prefix
    let key_prefix = shared::crypto::extract_key_prefix(&api_key)
        .expect("Test admin API key should have valid format");

    // Hash the key for storage using shared crypto utility
    let key_hash = shared::crypto::sha256_hex(&api_key);

    // Insert into database with is_admin = true
    sqlx::query(
        r#"
        INSERT INTO api_keys (name, key_prefix, key_hash, is_active, is_admin, created_at, last_used_at)
        VALUES ($1, $2, $3, true, true, NOW(), NULL)
        "#
    )
    .bind(name)
    .bind(key_prefix)
    .bind(key_hash)
    .execute(pool)
    .await
    .expect("Failed to create test admin API key");

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

/// Build a JSON request with both API key and JWT authentication.
/// This is needed for endpoints that require API key (via middleware)
/// but also support optional JWT for user linking.
pub fn json_request_with_api_key_and_jwt(
    method: axum::http::Method,
    uri: &str,
    body: serde_json::Value,
    api_key: &str,
    jwt_token: &str,
) -> axum::http::Request<axum::body::Body> {
    use axum::{body::Body, http::{header, Request}};

    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", api_key)
        .header(header::AUTHORIZATION, format!("Bearer {}", jwt_token))
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

/// Build a GET request with both API key and JWT authentication.
pub fn get_request_with_api_key_and_jwt(
    uri: &str,
    api_key: &str,
    jwt_token: &str,
) -> axum::http::Request<axum::body::Body> {
    use axum::{body::Body, http::{header, Method, Request}};

    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("X-API-Key", api_key)
        .header(header::AUTHORIZATION, format!("Bearer {}", jwt_token))
        .body(Body::empty())
        .unwrap()
}

/// Build a DELETE request with both API key and JWT authentication.
pub fn delete_request_with_api_key_and_jwt(
    uri: &str,
    api_key: &str,
    jwt_token: &str,
) -> axum::http::Request<axum::body::Body> {
    use axum::{body::Body, http::{header, Method, Request}};

    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("X-API-Key", api_key)
        .header(header::AUTHORIZATION, format!("Bearer {}", jwt_token))
        .body(Body::empty())
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
    let status = response.status();
    let body = parse_response_body(response).await;

    // Verify the request succeeded
    assert_eq!(status, axum::http::StatusCode::CREATED, "Failed to create organization: {:?}", body);

    // Organization is returned directly in body (not wrapped in "organization" key)
    CreatedOrganization {
        id: body["id"]
            .as_str()
            .unwrap_or_else(|| panic!("Missing 'id' in response body: {:?}", body))
            .to_string(),
        name: body["name"]
            .as_str()
            .unwrap_or_else(|| panic!("Missing 'name' in response body: {:?}", body))
            .to_string(),
        slug: body["slug"]
            .as_str()
            .unwrap_or_else(|| panic!("Missing 'slug' in response body: {:?}", body))
            .to_string(),
    }
}

/// Seed setting definitions if they don't exist.
/// This is reference data that should persist across tests.
pub async fn seed_setting_definitions(pool: &PgPool) {
    sqlx::query(
        r#"
        INSERT INTO setting_definitions (key, display_name, description, data_type, default_value, is_lockable, category, sort_order)
        VALUES
            ('tracking_enabled', 'Location Tracking', 'Enable or disable location tracking', 'boolean', 'true', true, 'tracking', 1),
            ('tracking_interval_minutes', 'Tracking Interval', 'Minutes between location updates', 'integer', '5', true, 'tracking', 2),
            ('movement_detection_enabled', 'Movement Detection', 'Enable automatic movement detection', 'boolean', 'true', true, 'tracking', 3),
            ('secret_mode_enabled', 'Secret Mode', 'Hide device location from other group members', 'boolean', 'false', true, 'privacy', 10),
            ('battery_optimization_enabled', 'Battery Optimization', 'Reduce tracking frequency when battery is low', 'boolean', 'true', false, 'battery', 20),
            ('notification_sounds_enabled', 'Notification Sounds', 'Play sounds for notifications', 'boolean', 'true', false, 'notifications', 30),
            ('geofence_notifications_enabled', 'Geofence Alerts', 'Receive notifications for geofence events', 'boolean', 'true', true, 'notifications', 31),
            ('sos_enabled', 'SOS Feature', 'Enable emergency SOS functionality', 'boolean', 'true', true, 'privacy', 11)
        ON CONFLICT (key) DO NOTHING
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed setting definitions");
}
