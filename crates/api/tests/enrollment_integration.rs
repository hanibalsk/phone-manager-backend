//! Integration tests for device enrollment endpoint.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test enrollment_integration

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_admin_api_key, create_test_app,
    create_test_pool, json_request_with_api_key_and_jwt, parse_response_body, run_migrations,
    test_config, TestUser,
};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a test organization directly in the database.
async fn create_test_org(pool: &PgPool) -> Uuid {
    let org_id = Uuid::new_v4();
    let name = format!("Test Org {}", &org_id.to_string()[..8]);
    let slug = format!("test-org-{}", &org_id.to_string()[..8]);

    sqlx::query(
        r#"
        INSERT INTO organizations (id, name, slug, billing_email, created_at, updated_at)
        VALUES ($1, $2, $3, 'billing@test.com', NOW(), NOW())
        "#,
    )
    .bind(org_id)
    .bind(&name)
    .bind(&slug)
    .execute(pool)
    .await
    .expect("Failed to create test organization");

    org_id
}

/// Create an enrollment token directly in the database and return the token string.
async fn create_enrollment_token(pool: &PgPool, org_id: Uuid) -> String {
    let token = format!("enroll_{}", Uuid::new_v4().simple());
    let token_prefix = &token[..8];

    sqlx::query(
        r#"
        INSERT INTO enrollment_tokens (id, organization_id, token, token_prefix, current_uses, created_at)
        VALUES ($1, $2, $3, $4, 0, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(org_id)
    .bind(&token)
    .bind(token_prefix)
    .execute(pool)
    .await
    .expect("Failed to create enrollment token");

    token
}

/// Create an enrollment token with max uses.
async fn create_enrollment_token_with_max_uses(
    pool: &PgPool,
    org_id: Uuid,
    max_uses: i32,
) -> String {
    let token = format!("enroll_{}", Uuid::new_v4().simple());
    let token_prefix = &token[..8];

    sqlx::query(
        r#"
        INSERT INTO enrollment_tokens (id, organization_id, token, token_prefix, max_uses, current_uses, created_at)
        VALUES ($1, $2, $3, $4, $5, 0, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(org_id)
    .bind(&token)
    .bind(token_prefix)
    .bind(max_uses)
    .execute(pool)
    .await
    .expect("Failed to create enrollment token");

    token
}

/// Create an expired enrollment token.
async fn create_expired_enrollment_token(pool: &PgPool, org_id: Uuid) -> String {
    let token = format!("enroll_{}", Uuid::new_v4().simple());
    let token_prefix = &token[..8];

    sqlx::query(
        r#"
        INSERT INTO enrollment_tokens (id, organization_id, token, token_prefix, expires_at, current_uses, created_at)
        VALUES ($1, $2, $3, $4, NOW() - INTERVAL '1 day', 0, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(org_id)
    .bind(&token)
    .bind(token_prefix)
    .execute(pool)
    .await
    .expect("Failed to create expired enrollment token");

    token
}

/// Create a revoked enrollment token.
async fn create_revoked_enrollment_token(pool: &PgPool, org_id: Uuid) -> String {
    let token = format!("enroll_{}", Uuid::new_v4().simple());
    let token_prefix = &token[..8];

    sqlx::query(
        r#"
        INSERT INTO enrollment_tokens (id, organization_id, token, token_prefix, revoked_at, current_uses, created_at)
        VALUES ($1, $2, $3, $4, NOW(), 0, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(org_id)
    .bind(&token)
    .bind(token_prefix)
    .execute(pool)
    .await
    .expect("Failed to create revoked enrollment token");

    token
}

/// Build a JSON request for enrollment (no auth required).
fn enroll_request(body: serde_json::Value) -> axum::http::Request<axum::body::Body> {
    use axum::{
        body::Body,
        http::{header, Request},
    };

    Request::builder()
        .method(Method::POST)
        .uri("/api/v1/devices/enroll")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

// ============================================================================
// Enrollment Tests
// ============================================================================

#[tokio::test]
async fn test_enroll_device_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create organization and enrollment token
    let org_id = create_test_org(&pool).await;
    let token = create_enrollment_token(&pool, org_id).await;

    let device_uuid = Uuid::new_v4();
    let request = enroll_request(json!({
        "enrollment_token": token,
        "device_uuid": device_uuid,
        "display_name": "Test Device",
        "platform": "android"
    }));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body["device"]["id"].is_number()); // id is i64, not string
    assert_eq!(body["device"]["device_uuid"], device_uuid.to_string());
    assert_eq!(body["device"]["display_name"], "Test Device");
    assert_eq!(body["device"]["organization_id"], org_id.to_string());
    assert_eq!(body["device"]["is_managed"], true);
    assert!(body["device_token"].is_string());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_enroll_device_with_fcm_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create organization and enrollment token
    let org_id = create_test_org(&pool).await;
    let token = create_enrollment_token(&pool, org_id).await;

    let device_uuid = Uuid::new_v4();
    let request = enroll_request(json!({
        "enrollment_token": token,
        "device_uuid": device_uuid,
        "display_name": "FCM Test Device",
        "platform": "android",
        "fcm_token": "test_fcm_token_123"
    }));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert_eq!(body["device"]["display_name"], "FCM Test Device");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_enroll_device_invalid_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let device_uuid = Uuid::new_v4();
    let request = enroll_request(json!({
        "enrollment_token": "invalid_token_that_does_not_exist",
        "device_uuid": device_uuid,
        "display_name": "Test Device",
        "platform": "android"
    }));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_enroll_device_expired_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create organization and expired enrollment token
    let org_id = create_test_org(&pool).await;
    let token = create_expired_enrollment_token(&pool, org_id).await;

    let device_uuid = Uuid::new_v4();
    let request = enroll_request(json!({
        "enrollment_token": token,
        "device_uuid": device_uuid,
        "display_name": "Test Device",
        "platform": "android"
    }));

    let response = app.oneshot(request).await.unwrap();
    // Expired token should return 410 GONE or 404 NOT FOUND
    assert!(response.status() == StatusCode::GONE || response.status() == StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_enroll_device_revoked_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create organization and revoked enrollment token
    let org_id = create_test_org(&pool).await;
    let token = create_revoked_enrollment_token(&pool, org_id).await;

    let device_uuid = Uuid::new_v4();
    let request = enroll_request(json!({
        "enrollment_token": token,
        "device_uuid": device_uuid,
        "display_name": "Test Device",
        "platform": "android"
    }));

    let response = app.oneshot(request).await.unwrap();
    // Revoked token should return 410 GONE or 404 NOT FOUND
    assert!(response.status() == StatusCode::GONE || response.status() == StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_enroll_device_exhausted_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    // Create organization and token with max_uses = 1
    let org_id = create_test_org(&pool).await;
    let token = create_enrollment_token_with_max_uses(&pool, org_id, 1).await;

    // First enrollment should succeed
    let app = create_test_app(config.clone(), pool.clone());
    let device_uuid1 = Uuid::new_v4();
    let request = enroll_request(json!({
        "enrollment_token": token.clone(),
        "device_uuid": device_uuid1,
        "display_name": "First Device",
        "platform": "android"
    }));
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Second enrollment should fail (token exhausted)
    let app = create_test_app(config, pool.clone());
    let device_uuid2 = Uuid::new_v4();
    let request = enroll_request(json!({
        "enrollment_token": token,
        "device_uuid": device_uuid2,
        "display_name": "Second Device",
        "platform": "android"
    }));
    let response = app.oneshot(request).await.unwrap();
    // Exhausted token should return 410 GONE or 404 NOT FOUND
    assert!(response.status() == StatusCode::GONE || response.status() == StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_enroll_device_missing_required_fields() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Missing enrollment_token
    let request = enroll_request(json!({
        "device_uuid": Uuid::new_v4(),
        "display_name": "Test Device",
        "platform": "android"
    }));

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_enroll_device_updates_use_count() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create organization and enrollment token with user for admin access
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_use_count").await;

    // Create token via API
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens", org_id),
        json!({}),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_response_body(response).await;
    let token = body["token"].as_str().unwrap().to_string();

    // Enroll a device
    let app = create_test_app(config, pool.clone());
    let device_uuid = Uuid::new_v4();
    let request = enroll_request(json!({
        "enrollment_token": token,
        "device_uuid": device_uuid,
        "display_name": "Use Count Test Device",
        "platform": "android"
    }));
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Check that current_uses was incremented
    let row: (i32,) = sqlx::query_as("SELECT current_uses FROM enrollment_tokens WHERE token = $1")
        .bind(&token)
        .fetch_one(&pool)
        .await
        .expect("Failed to query current_uses");
    assert_eq!(row.0, 1);

    cleanup_all_test_data(&pool).await;
}
