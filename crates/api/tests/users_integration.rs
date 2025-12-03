//! Integration tests for user profile and device binding endpoints.
//!
//! Tests cover:
//! - GET /api/v1/users/me (get current user profile)
//! - PUT /api/v1/users/me (update current user profile)
//! - GET /api/v1/users/:user_id/devices (list user's devices)
//! - POST /api/v1/users/:user_id/devices/:device_id/link (link device to user)
//! - DELETE /api/v1/users/:user_id/devices/:device_id/unlink (unlink device from user)
//! - POST /api/v1/users/:user_id/devices/:device_id/transfer (transfer device ownership)

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_app, create_test_pool,
    delete_request_with_auth, get_request_with_auth, json_request_with_auth,
    parse_response_body, register_test_device, run_migrations, test_config, TestDevice, TestUser,
};
use serde_json::json;
use tower::ServiceExt;

// =============================================================================
// GET /api/v1/users/me Tests
// =============================================================================

#[tokio::test]
async fn test_get_current_user_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Get current user profile
    let request = get_request_with_auth("/api/v1/users/me", &auth.access_token);
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["email"].as_str().unwrap(), user.email);
    assert!(body["id"].as_str().is_some());
    assert!(body["created_at"].as_str().is_some());
    assert!(body["updated_at"].as_str().is_some());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_current_user_missing_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Request without auth token
    use axum::{body::Body, http::Request};
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/users/me")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_current_user_invalid_token() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Request with invalid token
    let request = get_request_with_auth("/api/v1/users/me", "invalid_token_here");
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_all_test_data(&pool).await;
}

// =============================================================================
// PUT /api/v1/users/me Tests
// =============================================================================

#[tokio::test]
async fn test_update_current_user_display_name() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Update display name
    let request = json_request_with_auth(
        Method::PUT,
        "/api/v1/users/me",
        json!({
            "display_name": "Updated Name"
        }),
        &auth.access_token,
    );
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["display_name"].as_str().unwrap(), "Updated Name");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_current_user_avatar_url() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Update avatar URL
    let request = json_request_with_auth(
        Method::PUT,
        "/api/v1/users/me",
        json!({
            "avatar_url": "https://example.com/avatar.png"
        }),
        &auth.access_token,
    );
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(
        body["avatar_url"].as_str().unwrap(),
        "https://example.com/avatar.png"
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_current_user_invalid_avatar_url() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Update with invalid avatar URL
    let request = json_request_with_auth(
        Method::PUT,
        "/api/v1/users/me",
        json!({
            "avatar_url": "not-a-valid-url"
        }),
        &auth.access_token,
    );
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_current_user_display_name_too_long() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Update with too long display name (over 100 chars)
    let request = json_request_with_auth(
        Method::PUT,
        "/api/v1/users/me",
        json!({
            "display_name": "A".repeat(101)
        }),
        &auth.access_token,
    );
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_current_user_empty_request() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Update with empty request (no-op, should return current profile)
    let request = json_request_with_auth(
        Method::PUT,
        "/api/v1/users/me",
        json!({}),
        &auth.access_token,
    );
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["email"].as_str().unwrap(), user.email);

    cleanup_all_test_data(&pool).await;
}

// =============================================================================
// GET /api/v1/users/:user_id/devices Tests
// =============================================================================

#[tokio::test]
async fn test_list_user_devices_empty() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // List devices (should be empty)
    let uri = format!("/api/v1/users/{}/devices", auth.user_id);
    let request = get_request_with_auth(&uri, &auth.access_token);
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["devices"].as_array().unwrap().is_empty());
    assert_eq!(body["count"].as_i64().unwrap(), 0);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_user_devices_with_devices() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register a device (this should auto-link to the user)
    let device = TestDevice::new();
    let _device_response = register_test_device(&app, &pool, &auth, &device).await;

    // List devices
    let uri = format!("/api/v1/users/{}/devices", auth.user_id);
    let request = get_request_with_auth(&uri, &auth.access_token);
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let devices = body["devices"].as_array().unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(body["count"].as_i64().unwrap(), 1);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_user_devices_forbidden_other_user() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create two authenticated users
    let user1 = TestUser::new();
    let auth1 = create_authenticated_user(&app, &user1).await;

    let user2 = TestUser::new();
    let auth2 = create_authenticated_user(&app, &user2).await;

    // User1 tries to list user2's devices
    let uri = format!("/api/v1/users/{}/devices", auth2.user_id);
    let request = get_request_with_auth(&uri, &auth1.access_token);
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_all_test_data(&pool).await;
}

// =============================================================================
// POST /api/v1/users/:user_id/devices/:device_id/link Tests
// =============================================================================

#[tokio::test]
async fn test_link_device_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // First register a device without auto-linking (use raw request)
    let device = TestDevice::new();

    // Register device without user context (via API key or similar)
    // For simplicity, register with user auth - it will auto-link
    // Then we test the link endpoint works when device is already linked to same user
    let _device_response = register_test_device(&app, &pool, &auth, &device).await;

    // Try to link again (should succeed since already linked to same user)
    let uri = format!(
        "/api/v1/users/{}/devices/{}/link",
        auth.user_id, device.device_id
    );
    let request = json_request_with_auth(
        Method::POST,
        &uri,
        json!({
            "display_name": "My Phone",
            "is_primary": true
        }),
        &auth.access_token,
    );
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["linked"].as_bool().unwrap());
    assert!(body["device"].is_object());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_link_device_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to link a non-existent device
    let fake_device_id = uuid::Uuid::new_v4();
    let uri = format!(
        "/api/v1/users/{}/devices/{}/link",
        auth.user_id, fake_device_id
    );
    let request = json_request_with_auth(
        Method::POST,
        &uri,
        json!({
            "is_primary": false
        }),
        &auth.access_token,
    );
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_link_device_forbidden_other_user() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create two authenticated users
    let user1 = TestUser::new();
    let auth1 = create_authenticated_user(&app, &user1).await;

    let user2 = TestUser::new();
    let auth2 = create_authenticated_user(&app, &user2).await;

    // Register device with user1
    let device = TestDevice::new();
    let _device_response = register_test_device(&app, &pool, &auth1, &device).await;

    // User2 tries to link to their account (but device is linked to user1)
    let uri = format!(
        "/api/v1/users/{}/devices/{}/link",
        auth2.user_id, device.device_id
    );
    let request = json_request_with_auth(
        Method::POST,
        &uri,
        json!({
            "is_primary": false
        }),
        &auth2.access_token,
    );
    let response = app.clone().oneshot(request).await.unwrap();

    // Should be CONFLICT because device is linked to another user
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}

// =============================================================================
// DELETE /api/v1/users/:user_id/devices/:device_id/unlink Tests
// =============================================================================

#[tokio::test]
async fn test_unlink_device_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register a device (auto-linked to user)
    let device = TestDevice::new();
    let _device_response = register_test_device(&app, &pool, &auth, &device).await;

    // Unlink the device
    let uri = format!(
        "/api/v1/users/{}/devices/{}/unlink",
        auth.user_id, device.device_id
    );
    let request = delete_request_with_auth(&uri, &auth.access_token);
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["unlinked"].as_bool().unwrap());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_unlink_device_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to unlink a non-existent device
    let fake_device_id = uuid::Uuid::new_v4();
    let uri = format!(
        "/api/v1/users/{}/devices/{}/unlink",
        auth.user_id, fake_device_id
    );
    let request = delete_request_with_auth(&uri, &auth.access_token);
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_unlink_device_forbidden_other_user() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create two authenticated users
    let user1 = TestUser::new();
    let auth1 = create_authenticated_user(&app, &user1).await;

    let user2 = TestUser::new();
    let auth2 = create_authenticated_user(&app, &user2).await;

    // Register device with user1
    let device = TestDevice::new();
    let _device_response = register_test_device(&app, &pool, &auth1, &device).await;

    // User2 tries to unlink user1's device via their own user path
    let uri = format!(
        "/api/v1/users/{}/devices/{}/unlink",
        auth2.user_id, device.device_id
    );
    let request = delete_request_with_auth(&uri, &auth2.access_token);
    let response = app.clone().oneshot(request).await.unwrap();

    // Should be FORBIDDEN because device is not linked to user2
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_all_test_data(&pool).await;
}

// =============================================================================
// POST /api/v1/users/:user_id/devices/:device_id/transfer Tests
// =============================================================================

#[tokio::test]
async fn test_transfer_device_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create two authenticated users
    let user1 = TestUser::new();
    let auth1 = create_authenticated_user(&app, &user1).await;

    let user2 = TestUser::new();
    let auth2 = create_authenticated_user(&app, &user2).await;

    // Register device with user1
    let device = TestDevice::new();
    let _device_response = register_test_device(&app, &pool, &auth1, &device).await;

    // Transfer device to user2
    let uri = format!(
        "/api/v1/users/{}/devices/{}/transfer",
        auth1.user_id, device.device_id
    );
    let request = json_request_with_auth(
        Method::POST,
        &uri,
        json!({
            "new_owner_id": auth2.user_id
        }),
        &auth1.access_token,
    );
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["transferred"].as_bool().unwrap());
    assert_eq!(body["new_owner_id"].as_str().unwrap(), auth2.user_id);
    assert_eq!(body["previous_owner_id"].as_str().unwrap(), auth1.user_id);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_transfer_device_to_self_fails() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register device with user
    let device = TestDevice::new();
    let _device_response = register_test_device(&app, &pool, &auth, &device).await;

    // Try to transfer to self
    let uri = format!(
        "/api/v1/users/{}/devices/{}/transfer",
        auth.user_id, device.device_id
    );
    let request = json_request_with_auth(
        Method::POST,
        &uri,
        json!({
            "new_owner_id": auth.user_id
        }),
        &auth.access_token,
    );
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_transfer_device_invalid_recipient() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register device with user
    let device = TestDevice::new();
    let _device_response = register_test_device(&app, &pool, &auth, &device).await;

    // Try to transfer to non-existent user
    let fake_user_id = uuid::Uuid::new_v4();
    let uri = format!(
        "/api/v1/users/{}/devices/{}/transfer",
        auth.user_id, device.device_id
    );
    let request = json_request_with_auth(
        Method::POST,
        &uri,
        json!({
            "new_owner_id": fake_user_id
        }),
        &auth.access_token,
    );
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_transfer_device_not_owner() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create three authenticated users
    let user1 = TestUser::new();
    let auth1 = create_authenticated_user(&app, &user1).await;

    let user2 = TestUser::new();
    let auth2 = create_authenticated_user(&app, &user2).await;

    let user3 = TestUser::new();
    let auth3 = create_authenticated_user(&app, &user3).await;

    // Register device with user1
    let device = TestDevice::new();
    let _device_response = register_test_device(&app, &pool, &auth1, &device).await;

    // User2 tries to transfer user1's device to user3 (via user2's path)
    let uri = format!(
        "/api/v1/users/{}/devices/{}/transfer",
        auth2.user_id, device.device_id
    );
    let request = json_request_with_auth(
        Method::POST,
        &uri,
        json!({
            "new_owner_id": auth3.user_id
        }),
        &auth2.access_token,
    );
    let response = app.clone().oneshot(request).await.unwrap();

    // Should be FORBIDDEN because user2 doesn't own the device
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_all_test_data(&pool).await;
}
