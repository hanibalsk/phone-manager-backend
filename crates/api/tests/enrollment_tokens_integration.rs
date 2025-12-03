//! Integration tests for enrollment token endpoints.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test enrollment_tokens_integration

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_admin_api_key, create_test_app,
    create_test_pool, delete_request_with_api_key, get_request_with_api_key,
    json_request_with_api_key_and_jwt, parse_response_body, run_migrations, test_config, TestUser,
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

// ============================================================================
// Create Token Tests
// ============================================================================

#[tokio::test]
async fn test_create_enrollment_token_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create organization
    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_create_token").await;

    // Create enrollment token
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens", org_id),
        json!({
            "group_id": "test-group-1",
            "max_uses": 10,
            "expires_in_days": 30,
            "auto_assign_user_by_email": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body["id"].is_string());
    assert!(body["token"].is_string());
    assert!(body["token_prefix"].is_string());
    assert_eq!(body["max_uses"], 10);
    assert!(body["expires_at"].is_string());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_enrollment_token_minimal() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create organization
    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_create_minimal").await;

    // Create minimal enrollment token (no optional fields)
    let app = create_test_app(config, pool.clone());
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
    assert!(body["id"].is_string());
    assert!(body["token"].is_string());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_enrollment_token_with_policy() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create organization
    let org_id = create_test_org(&pool).await;

    // Create a policy first
    let policy_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO device_policies (id, organization_id, name, priority, created_at, updated_at)
        VALUES ($1, $2, 'Test Policy', 0, NOW(), NOW())
        "#,
    )
    .bind(policy_id)
    .bind(org_id)
    .execute(&pool)
    .await
    .expect("Failed to create test policy");

    let api_key = create_test_admin_api_key(&pool, "test_create_with_policy").await;

    // Create enrollment token with policy
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens", org_id),
        json!({
            "policy_id": policy_id,
            "group_id": "policy-group"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert_eq!(body["policy_id"], policy_id.to_string());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_enrollment_token_requires_jwt() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_requires_jwt").await;

    // Try to create without JWT (only API key)
    use axum::{body::Body, http::{header, Request}};
    let request = Request::builder()
        .method(Method::POST)
        .uri(&format!("/api/admin/v1/organizations/{}/enrollment-tokens", org_id))
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", &api_key)
        .body(Body::from(r#"{}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Should fail due to missing JWT
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// List Tokens Tests
// ============================================================================

#[tokio::test]
async fn test_list_enrollment_tokens_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create organization
    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_list_tokens").await;

    // Create two tokens
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens", org_id),
        json!({"group_id": "group-1"}),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let app = create_test_app(config.clone(), pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_list_tokens2").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens", org_id),
        json!({"group_id": "group-2"}),
        &api_key2,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // List tokens
    let app = create_test_app(config, pool.clone());
    let api_key3 = create_test_admin_api_key(&pool, "test_list_tokens3").await;
    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens", org_id),
        &api_key3,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["data"].is_array());
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_enrollment_tokens_empty() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_list_empty").await;

    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens", org_id),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let data = body["data"].as_array().unwrap();
    assert!(data.is_empty());

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Get Token Tests
// ============================================================================

#[tokio::test]
async fn test_get_enrollment_token_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create organization
    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_get_token").await;

    // Create a token
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens", org_id),
        json!({"group_id": "get-test-group"}),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_response_body(response).await;
    let token_id = body["id"].as_str().unwrap();

    // Get the token
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_get_token2").await;
    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens/{}", org_id, token_id),
        &api_key2,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["id"], token_id);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_enrollment_token_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_get_not_found").await;
    let fake_token_id = Uuid::new_v4();

    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens/{}", org_id, fake_token_id),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_enrollment_token_wrong_org() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create two organizations
    let org1_id = create_test_org(&pool).await;
    let org2_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_wrong_org").await;

    // Create token in org1
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens", org1_id),
        json!({}),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_response_body(response).await;
    let token_id = body["id"].as_str().unwrap();

    // Try to get it from org2 (should fail)
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_wrong_org2").await;
    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens/{}", org2_id, token_id),
        &api_key2,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Revoke Token Tests
// ============================================================================

#[tokio::test]
async fn test_revoke_enrollment_token_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create organization
    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_revoke_token").await;

    // Create a token
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
    let token_id = body["id"].as_str().unwrap();

    // Revoke the token
    let app = create_test_app(config.clone(), pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_revoke_token2").await;
    let request = delete_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens/{}", org_id, token_id),
        &api_key2,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify token is revoked (check revoked_at is set)
    let app = create_test_app(config, pool.clone());
    let api_key3 = create_test_admin_api_key(&pool, "test_revoke_token3").await;
    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens/{}", org_id, token_id),
        &api_key3,
    );
    let response = app.oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;
    assert!(body["revoked_at"].is_string());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_revoke_enrollment_token_already_revoked() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create organization
    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_already_revoked").await;

    // Create a token
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
    let token_id = body["id"].as_str().unwrap();

    // Revoke the token first time
    let app = create_test_app(config.clone(), pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_already_revoked2").await;
    let request = delete_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens/{}", org_id, token_id),
        &api_key2,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Try to revoke again (should fail with conflict)
    let app = create_test_app(config, pool.clone());
    let api_key3 = create_test_admin_api_key(&pool, "test_already_revoked3").await;
    let request = delete_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens/{}", org_id, token_id),
        &api_key3,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_revoke_enrollment_token_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_revoke_not_found").await;
    let fake_token_id = Uuid::new_v4();

    let request = delete_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens/{}", org_id, fake_token_id),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// QR Code Tests
// ============================================================================

#[tokio::test]
async fn test_get_enrollment_token_qr_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create organization
    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_qr_token").await;

    // Create a token
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
    let token_id = body["id"].as_str().unwrap();

    // Get QR code
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_qr_token2").await;
    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens/{}/qr", org_id, token_id),
        &api_key2,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["qr_data"].is_string());
    assert!(body["enrollment_url"].is_string());
    // Enrollment URL should contain the token
    let enrollment_url = body["enrollment_url"].as_str().unwrap();
    assert!(enrollment_url.contains("enroll?token="));

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_enrollment_token_qr_revoked_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create organization
    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_qr_revoked").await;

    // Create a token
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
    let token_id = body["id"].as_str().unwrap();

    // Revoke the token
    let app = create_test_app(config.clone(), pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_qr_revoked2").await;
    let request = delete_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens/{}", org_id, token_id),
        &api_key2,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Try to get QR for revoked token (should fail)
    let app = create_test_app(config, pool.clone());
    let api_key3 = create_test_admin_api_key(&pool, "test_qr_revoked3").await;
    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/enrollment-tokens/{}/qr", org_id, token_id),
        &api_key3,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}
