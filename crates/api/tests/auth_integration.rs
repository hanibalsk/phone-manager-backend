//! Integration tests for authentication flows.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test auth_integration

mod common;

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use common::{cleanup_test_data, create_test_pool, run_migrations, test_config, TestUser};
use serde_json::{json, Value};
use tower::ServiceExt;

/// Helper to create a JSON request.
fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

/// Helper to parse JSON response body.
async fn parse_response_body(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap_or(Value::Null)
}

// ============================================================================
// Registration Tests
// ============================================================================

#[tokio::test]
async fn test_register_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let app = common::create_test_app(config, pool.clone());

    let user = TestUser::new();
    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": user.email,
            "password": user.password,
            "display_name": user.display_name
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body.get("userId").is_some());
    assert_eq!(body["email"], user.email.to_lowercase());
    assert!(body.get("accessToken").is_some());
    assert!(body.get("refresh_token").is_some());
    assert!(!body["accessToken"].as_str().unwrap().is_empty());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let user = TestUser::new();

    // First registration
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": user.email,
            "password": user.password,
            "display_name": user.display_name
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Second registration with same email
    let app = common::create_test_app(config, pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": user.email,
            "password": user.password,
            "display_name": "Another User"
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = parse_response_body(response).await;
    assert!(body["error"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("email"));

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_register_weak_password() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let app = common::create_test_app(config, pool.clone());

    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": "test@example.com",
            "password": "weak",
            "display_name": "Test User"
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    // Should reject weak password
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_register_invalid_email() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let app = common::create_test_app(config, pool.clone());

    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": "not-an-email",
            "password": "SecureP@ss123!",
            "display_name": "Test User"
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    cleanup_test_data(&pool).await;
}

// ============================================================================
// Login Tests
// ============================================================================

#[tokio::test]
async fn test_login_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let user = TestUser::new();

    // Register user first
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": user.email,
            "password": user.password,
            "display_name": user.display_name
        }),
    );
    let _ = app.oneshot(request).await.unwrap();

    // Login
    let app = common::create_test_app(config, pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/login",
        json!({
            "email": user.email,
            "password": user.password
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body.get("userId").is_some());
    assert!(body.get("accessToken").is_some());
    assert!(body.get("refresh_token").is_some());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_login_invalid_password() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let user = TestUser::new();

    // Register user first
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": user.email,
            "password": user.password,
            "display_name": user.display_name
        }),
    );
    let _ = app.oneshot(request).await.unwrap();

    // Login with wrong password
    let app = common::create_test_app(config, pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/login",
        json!({
            "email": user.email,
            "password": "WrongP@ss123!"
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_login_nonexistent_user() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let app = common::create_test_app(config, pool.clone());

    let request = json_request(
        Method::POST,
        "/api/v1/auth/login",
        json!({
            "email": "nonexistent@example.com",
            "password": "SecureP@ss123!"
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_login_case_insensitive_email() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let user = TestUser::new();

    // Register with lowercase email
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": user.email.to_lowercase(),
            "password": user.password,
            "display_name": user.display_name
        }),
    );
    let _ = app.oneshot(request).await.unwrap();

    // Login with uppercase email
    let app = common::create_test_app(config, pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/login",
        json!({
            "email": user.email.to_uppercase(),
            "password": user.password
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    cleanup_test_data(&pool).await;
}

// ============================================================================
// Token Refresh Tests
// ============================================================================

#[tokio::test]
async fn test_refresh_token_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let user = TestUser::new();

    // Register to get tokens
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": user.email,
            "password": user.password,
            "display_name": user.display_name
        }),
    );
    let response = app.oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;
    let refresh_token = body["refresh_token"].as_str().unwrap();

    // Use refresh token
    let app = common::create_test_app(config, pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/refresh",
        json!({
            "refresh_token": refresh_token
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body.get("accessToken").is_some());
    assert!(body.get("refresh_token").is_some());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_refresh_token_invalid() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let app = common::create_test_app(config, pool.clone());

    let request = json_request(
        Method::POST,
        "/api/v1/auth/refresh",
        json!({
            "refresh_token": "invalid-token"
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_test_data(&pool).await;
}

// ============================================================================
// Logout Tests
// ============================================================================

#[tokio::test]
async fn test_logout_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let user = TestUser::new();

    // Register to get tokens
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": user.email,
            "password": user.password,
            "display_name": user.display_name
        }),
    );
    let response = app.oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;
    let access_token = body["accessToken"].as_str().unwrap();

    // Logout
    let app = common::create_test_app(config, pool.clone());
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/logout")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NO_CONTENT
    );

    cleanup_test_data(&pool).await;
}

// ============================================================================
// Protected Route Access Tests
// ============================================================================

#[tokio::test]
async fn test_access_protected_route_with_valid_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let user = TestUser::new();

    // Register to get tokens
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": user.email,
            "password": user.password,
            "display_name": user.display_name
        }),
    );
    let response = app.oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;
    let access_token = body["accessToken"].as_str().unwrap();

    // Access protected route (get current user)
    let app = common::create_test_app(config, pool.clone());
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/users/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["email"], user.email.to_lowercase());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_access_protected_route_without_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let app = common::create_test_app(config, pool.clone());

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/users/me")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_access_protected_route_with_invalid_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let app = common::create_test_app(config, pool.clone());

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/users/me")
        .header(header::AUTHORIZATION, "Bearer invalid-token")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_test_data(&pool).await;
}

// ============================================================================
// Session Management Tests
// ============================================================================

#[tokio::test]
async fn test_multiple_sessions_same_user() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let user = TestUser::new();

    // Register
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": user.email,
            "password": user.password,
            "display_name": user.display_name
        }),
    );
    let _ = app.oneshot(request).await.unwrap();

    // Login from "device 1"
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/login",
        json!({
            "email": user.email,
            "password": user.password
        }),
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_response_body(response).await;
    let token1 = body["accessToken"].as_str().unwrap().to_string();

    // Login from "device 2"
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/login",
        json!({
            "email": user.email,
            "password": user.password
        }),
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_response_body(response).await;
    let token2 = body["accessToken"].as_str().unwrap().to_string();

    // Both tokens should be different
    assert_ne!(token1, token2);

    // Both tokens should be valid
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/users/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", token1))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let app = common::create_test_app(config, pool.clone());
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/users/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", token2))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    cleanup_test_data(&pool).await;
}

// ============================================================================
// OAuth Tests
// ============================================================================

#[tokio::test]
async fn test_oauth_invalid_provider() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let app = common::create_test_app(config, pool.clone());

    let request = json_request(
        Method::POST,
        "/api/v1/auth/oauth",
        json!({
            "provider": "invalid_provider",
            "id_token": "some-token"
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    // Should reject unknown provider
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_oauth_missing_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let app = common::create_test_app(config, pool.clone());

    let request = json_request(
        Method::POST,
        "/api/v1/auth/oauth",
        json!({
            "provider": "google",
            "id_token": ""
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    cleanup_test_data(&pool).await;
}

// ============================================================================
// Password Reset Tests
// ============================================================================

#[tokio::test]
async fn test_forgot_password_existing_user() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let user = TestUser::new();

    // Register user first
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": user.email,
            "password": user.password,
            "display_name": user.display_name
        }),
    );
    let _ = app.oneshot(request).await.unwrap();

    // Request password reset
    let app = common::create_test_app(config, pool.clone());
    let request = json_request(
        Method::POST,
        "/api/v1/auth/forgot-password",
        json!({
            "email": user.email
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    // Should succeed (or at least not reveal user existence)
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::ACCEPTED
            || response.status() == StatusCode::NO_CONTENT
    );

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_forgot_password_nonexistent_user() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let app = common::create_test_app(config, pool.clone());

    let request = json_request(
        Method::POST,
        "/api/v1/auth/forgot-password",
        json!({
            "email": "nonexistent@example.com"
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    // Should not reveal that user doesn't exist (security best practice)
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::ACCEPTED
            || response.status() == StatusCode::NO_CONTENT
    );

    cleanup_test_data(&pool).await;
}

// ============================================================================
// Email Verification Tests
// ============================================================================

#[tokio::test]
async fn test_verify_email_invalid_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let config = test_config();
    let app = common::create_test_app(config, pool.clone());

    let request = json_request(
        Method::POST,
        "/api/v1/auth/verify-email",
        json!({
            "token": "invalid-verification-token"
        }),
    );

    let response = app.oneshot(request).await.unwrap();
    // Should reject invalid token
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::NOT_FOUND
    );

    cleanup_test_data(&pool).await;
}
