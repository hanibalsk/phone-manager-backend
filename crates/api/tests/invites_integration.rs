//! Integration tests for group invite endpoints.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test invites_integration

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_app, create_test_pool,
    parse_response_body, run_migrations, test_config, TestUser,
};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a test group and add the user as owner.
async fn create_test_group_with_owner(pool: &PgPool, user_id: Uuid) -> Uuid {
    let group_id = Uuid::new_v4();
    let name = format!("Test Group {}", &group_id.to_string()[..8]);
    let slug = format!("test-group-{}", &group_id.to_string()[..8]);

    sqlx::query(
        r#"
        INSERT INTO groups (id, name, slug, created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        "#,
    )
    .bind(group_id)
    .bind(&name)
    .bind(&slug)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("Failed to create test group");

    // Add user as owner of the group
    sqlx::query(
        r#"
        INSERT INTO group_memberships (id, group_id, user_id, role, joined_at, updated_at)
        VALUES ($1, $2, $3, 'owner'::group_role, NOW(), NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(group_id)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("Failed to create group membership");

    group_id
}

/// Create a test group and add the user as a regular member (cannot create invites).
async fn create_test_group_with_member(pool: &PgPool, owner_id: Uuid, member_id: Uuid) -> Uuid {
    let group_id = Uuid::new_v4();
    let name = format!("Test Group {}", &group_id.to_string()[..8]);
    let slug = format!("test-group-{}", &group_id.to_string()[..8]);

    sqlx::query(
        r#"
        INSERT INTO groups (id, name, slug, created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        "#,
    )
    .bind(group_id)
    .bind(&name)
    .bind(&slug)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("Failed to create test group");

    // Add member as regular member (not admin/owner)
    sqlx::query(
        r#"
        INSERT INTO group_memberships (id, group_id, user_id, role, joined_at, updated_at)
        VALUES ($1, $2, $3, 'member'::group_role, NOW(), NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(group_id)
    .bind(member_id)
    .execute(pool)
    .await
    .expect("Failed to create group membership");

    group_id
}

/// Create a test invite directly in the database.
async fn create_test_invite(pool: &PgPool, group_id: Uuid, created_by: Uuid) -> (Uuid, String) {
    let invite_id = Uuid::new_v4();
    let code = format!(
        "TST-{}-{}",
        &Uuid::new_v4().to_string()[..3].to_uppercase(),
        &Uuid::new_v4().to_string()[..3].to_uppercase()
    );

    sqlx::query(
        r#"
        INSERT INTO group_invites (id, group_id, code, preset_role, max_uses, current_uses, expires_at, created_by, is_active, created_at)
        VALUES ($1, $2, $3, 'member'::group_role, 10, 0, NOW() + INTERVAL '24 hours', $4, true, NOW())
        "#,
    )
    .bind(invite_id)
    .bind(group_id)
    .bind(&code)
    .bind(created_by)
    .execute(pool)
    .await
    .expect("Failed to create test invite");

    (invite_id, code)
}

/// Create an expired test invite.
async fn create_expired_invite(pool: &PgPool, group_id: Uuid, created_by: Uuid) -> (Uuid, String) {
    let invite_id = Uuid::new_v4();
    let code = format!(
        "EXP-{}-{}",
        &Uuid::new_v4().to_string()[..3].to_uppercase(),
        &Uuid::new_v4().to_string()[..3].to_uppercase()
    );

    sqlx::query(
        r#"
        INSERT INTO group_invites (id, group_id, code, preset_role, max_uses, current_uses, expires_at, created_by, is_active, created_at)
        VALUES ($1, $2, $3, 'member'::group_role, 10, 0, NOW() - INTERVAL '1 hour', $4, true, NOW())
        "#,
    )
    .bind(invite_id)
    .bind(group_id)
    .bind(&code)
    .bind(created_by)
    .execute(pool)
    .await
    .expect("Failed to create expired invite");

    (invite_id, code)
}

/// Build a JSON request with JWT authentication.
fn json_request_with_jwt(
    method: Method,
    uri: &str,
    body: serde_json::Value,
    jwt: &str,
) -> axum::http::Request<axum::body::Body> {
    use axum::{
        body::Body,
        http::{header, Request},
    };

    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

/// Build a GET request with JWT authentication.
fn get_request_with_jwt(uri: &str, jwt: &str) -> axum::http::Request<axum::body::Body> {
    use axum::{
        body::Body,
        http::{header, Request},
    };

    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
        .body(Body::empty())
        .unwrap()
}

/// Build a DELETE request with JWT authentication.
fn delete_request_with_jwt(uri: &str, jwt: &str) -> axum::http::Request<axum::body::Body> {
    use axum::{
        body::Body,
        http::{header, Request},
    };

    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
        .body(Body::empty())
        .unwrap()
}

// ============================================================================
// Create Invite Tests
// ============================================================================

#[tokio::test]
async fn test_create_invite_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Parse user_id from the response
    let user_id = Uuid::parse_str(&auth.user_id).unwrap();

    // Create group with user as owner
    let group_id = create_test_group_with_owner(&pool, user_id).await;

    // Create invite
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_jwt(
        Method::POST,
        &format!("/api/v1/groups/{}/invites", group_id),
        json!({
            "expires_in_hours": 48,
            "max_uses": 5,
            "preset_role": "member"
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body["id"].is_string());
    assert!(body["code"].is_string());
    assert_eq!(body["max_uses"], 5);
    assert_eq!(body["current_uses"], 0);
    assert!(body["invite_url"].is_string());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_invite_forbidden_for_member() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create two users
    let owner = TestUser::new();
    let owner_auth = create_authenticated_user(&app, &owner).await;
    let owner_id = Uuid::parse_str(&owner_auth.user_id).unwrap();

    let app = create_test_app(config.clone(), pool.clone());
    let member = TestUser::new();
    let member_auth = create_authenticated_user(&app, &member).await;
    let member_id = Uuid::parse_str(&member_auth.user_id).unwrap();

    // Create group with member as just a member (not admin/owner)
    let group_id = create_test_group_with_member(&pool, owner_id, member_id).await;

    // Try to create invite as member (should fail)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_jwt(
        Method::POST,
        &format!("/api/v1/groups/{}/invites", group_id),
        json!({
            "expires_in_hours": 24
        }),
        &member_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_invite_owner_role_rejected() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let user_id = Uuid::parse_str(&auth.user_id).unwrap();

    // Create group with user as owner
    let group_id = create_test_group_with_owner(&pool, user_id).await;

    // Try to create invite with owner role (should fail)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_jwt(
        Method::POST,
        &format!("/api/v1/groups/{}/invites", group_id),
        json!({
            "preset_role": "owner"
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should be 400 or 422 for validation error
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// List Invites Tests
// ============================================================================

#[tokio::test]
async fn test_list_invites_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let user_id = Uuid::parse_str(&auth.user_id).unwrap();

    // Create group with user as owner
    let group_id = create_test_group_with_owner(&pool, user_id).await;

    // Create a few invites
    create_test_invite(&pool, group_id, user_id).await;
    create_test_invite(&pool, group_id, user_id).await;

    // List invites
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_jwt(
        &format!("/api/v1/groups/{}/invites", group_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_invites_empty() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let user_id = Uuid::parse_str(&auth.user_id).unwrap();

    // Create group with user as owner
    let group_id = create_test_group_with_owner(&pool, user_id).await;

    // List invites (should be empty)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_jwt(
        &format!("/api/v1/groups/{}/invites", group_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 0);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Revoke Invite Tests
// ============================================================================

#[tokio::test]
async fn test_revoke_invite_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let user_id = Uuid::parse_str(&auth.user_id).unwrap();

    // Create group with user as owner
    let group_id = create_test_group_with_owner(&pool, user_id).await;

    // Create an invite
    let (invite_id, _code) = create_test_invite(&pool, group_id, user_id).await;

    // Revoke invite
    let app = create_test_app(config, pool.clone());
    let request = delete_request_with_jwt(
        &format!("/api/v1/groups/{}/invites/{}", group_id, invite_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_revoke_invite_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let user_id = Uuid::parse_str(&auth.user_id).unwrap();

    // Create group with user as owner
    let group_id = create_test_group_with_owner(&pool, user_id).await;

    // Try to revoke non-existent invite
    let fake_invite_id = Uuid::new_v4();
    let app = create_test_app(config, pool.clone());
    let request = delete_request_with_jwt(
        &format!("/api/v1/groups/{}/invites/{}", group_id, fake_invite_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Get Invite Info (Public) Tests
// ============================================================================

#[tokio::test]
async fn test_get_invite_info_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let user_id = Uuid::parse_str(&auth.user_id).unwrap();

    // Create group with user as owner
    let group_id = create_test_group_with_owner(&pool, user_id).await;

    // Create an invite
    let (_invite_id, code) = create_test_invite(&pool, group_id, user_id).await;

    // Get invite info (no auth required)
    let app = create_test_app(config, pool.clone());
    let request = axum::http::Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/invites/{}", code))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["group"].is_object());
    assert!(body["group"]["name"].is_string());
    assert!(body["is_valid"].is_boolean());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_invite_info_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Try to get non-existent invite
    let request = axum::http::Request::builder()
        .method(Method::GET)
        .uri("/api/v1/invites/FAKE-INV-ITE")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_expired_invite_info() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let user_id = Uuid::parse_str(&auth.user_id).unwrap();

    // Create group with user as owner
    let group_id = create_test_group_with_owner(&pool, user_id).await;

    // Create an expired invite
    let (_invite_id, code) = create_expired_invite(&pool, group_id, user_id).await;

    // Get invite info (should show is_valid: false)
    let app = create_test_app(config, pool.clone());
    let request = axum::http::Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/invites/{}", code))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["is_valid"], false);

    cleanup_all_test_data(&pool).await;
}
