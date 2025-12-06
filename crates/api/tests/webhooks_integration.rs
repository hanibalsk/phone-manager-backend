//! Integration tests for webhook management endpoints.
//!
//! Story 15.1: Webhook Registration and Management API
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test webhooks_integration

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_api_key, create_test_app,
    create_test_pool, delete_request_with_api_key_and_jwt, get_request_with_api_key_and_jwt,
    json_request_with_api_key_and_jwt, parse_response_body, register_test_device, run_migrations,
    test_config, TestDevice, TestUser,
};
use serde_json::json;
use tower::ServiceExt;

// ============================================================================
// Webhook Creation Tests (AC 15.1.2)
// ============================================================================

#[tokio::test]
async fn test_create_webhook_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_create_webhook").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/api/webhook/test",
            "secret": "my-secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body.get("webhook_id").is_some());
    assert_eq!(body["name"], "Home Assistant");
    assert_eq!(body["target_url"], "https://homeassistant.local/api/webhook/test");
    assert_eq!(body["enabled"], true);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_invalid_name_empty() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_invalid_name").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create webhook with empty name
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "",
            "target_url": "https://example.com/webhook",
            "secret": "my-secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_invalid_url_not_https() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_invalid_url").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create webhook with HTTP URL (not HTTPS)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Insecure Webhook",
            "target_url": "http://example.com/webhook",
            "secret": "my-secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_invalid_secret_too_short() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_short_secret").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create webhook with secret too short (< 8 characters)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Short Secret",
            "target_url": "https://example.com/webhook",
            "secret": "short",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_device_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user but don't register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_device_not_found").await;

    // Try to create webhook for non-existent device
    let app = create_test_app(config, pool.clone());
    let fake_device_id = uuid::Uuid::new_v4();
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": fake_device_id.to_string(),
            "name": "Orphan Webhook",
            "target_url": "https://example.com/webhook",
            "secret": "my-secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_duplicate_name_conflict() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_duplicate_name").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create first webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://example.com/webhook",
            "secret": "my-secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Try to create second webhook with same name
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://another.com/webhook",
            "secret": "different-secret-key",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Webhook Listing Tests (AC 15.1.3)
// ============================================================================

#[tokio::test]
async fn test_list_webhooks_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_list_webhooks").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create two webhooks
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "n8n Automation",
            "target_url": "https://n8n.local/webhook/test",
            "secret": "another-secret-key",
            "enabled": false
        }),
        &api_key,
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // List webhooks for device
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/webhooks?ownerDeviceId={}", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let webhooks = body["webhooks"].as_array().unwrap();
    assert_eq!(webhooks.len(), 2);
    assert_eq!(body["total"], 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_webhooks_empty() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_list_empty").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // List webhooks for device (none exist)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/webhooks?ownerDeviceId={}", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let webhooks = body["webhooks"].as_array().unwrap();
    assert!(webhooks.is_empty());
    assert_eq!(body["total"], 0);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_webhooks_missing_device_id() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_missing_device_id").await;

    // List webhooks without device ID
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        "/api/v1/webhooks",
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Get Single Webhook Tests (AC 15.1.4)
// ============================================================================

#[tokio::test]
async fn test_get_webhook_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_get_webhook").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Get the webhook
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/webhooks/{}", webhook_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["webhook_id"], webhook_id);
    assert_eq!(body["name"], "Home Assistant");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_webhook_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_get_not_found").await;

    // Try to get non-existent webhook
    let app = create_test_app(config, pool.clone());
    let fake_webhook_id = uuid::Uuid::new_v4();
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/webhooks/{}", fake_webhook_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Webhook Update Tests (AC 15.1.5)
// ============================================================================

#[tokio::test]
async fn test_update_webhook_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_update_webhook").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Update the webhook
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", webhook_id),
        json!({
            "name": "Updated Home Assistant",
            "enabled": false
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["name"], "Updated Home Assistant");
    assert_eq!(body["enabled"], false);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_webhook_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_update_not_found").await;

    // Try to update non-existent webhook
    let app = create_test_app(config, pool.clone());
    let fake_webhook_id = uuid::Uuid::new_v4();
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", fake_webhook_id),
        json!({
            "name": "Updated Name"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Webhook Deletion Tests (AC 15.1.6)
// ============================================================================

#[tokio::test]
async fn test_delete_webhook_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_delete_webhook").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Delete the webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_request_with_api_key_and_jwt(
        &format!("/api/v1/webhooks/{}", webhook_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify webhook is gone
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/webhooks/{}", webhook_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_webhook_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_delete_not_found").await;

    // Try to delete non-existent webhook
    let app = create_test_app(config, pool.clone());
    let fake_webhook_id = uuid::Uuid::new_v4();
    let request = delete_request_with_api_key_and_jwt(
        &format!("/api/v1/webhooks/{}", fake_webhook_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Webhook Limit Tests (AC 15.1.2 - max 10 webhooks per device)
// ============================================================================

#[tokio::test]
async fn test_create_webhook_limit_exceeded() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_webhook_limit").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create 10 webhooks (the limit)
    for i in 0..10 {
        let app = create_test_app(config.clone(), pool.clone());
        let request = json_request_with_api_key_and_jwt(
            Method::POST,
            "/api/v1/webhooks",
            json!({
                "owner_device_id": device_id,
                "name": format!("Webhook {}", i),
                "target_url": format!("https://example.com/webhook/{}", i),
                "secret": format!("secret-key-{:05}", i),
                "enabled": true
            }),
            &api_key,
            &auth.access_token,
        );
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED, "Failed to create webhook {}", i);
    }

    // Try to create 11th webhook (should fail)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Webhook 10 - Over Limit",
            "target_url": "https://example.com/webhook/overlimit",
            "secret": "secret-key-overlimit",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Additional Update Validation Tests (AC 15.1.5)
// ============================================================================

#[tokio::test]
async fn test_update_webhook_invalid_url_not_https() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_update_invalid_url").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Try to update with HTTP URL (should fail)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", webhook_id),
        json!({
            "target_url": "http://insecure.example.com/webhook"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_webhook_duplicate_name_conflict() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_update_dup_name").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create first webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // Create second webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "n8n Automation",
            "target_url": "https://n8n.local/webhook",
            "secret": "another-secret-key",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Try to rename second webhook to first webhook's name (should fail)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", webhook_id),
        json!({
            "name": "Home Assistant"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_webhook_only_target_url() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_update_url_only").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://old.homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Update only the target_url
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", webhook_id),
        json!({
            "target_url": "https://new.homeassistant.local/webhook"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["name"], "Home Assistant"); // Unchanged
    assert_eq!(body["target_url"], "https://new.homeassistant.local/webhook"); // Changed
    assert_eq!(body["enabled"], true); // Unchanged

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_webhook_only_secret() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_update_secret_only").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "old-secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Update only the secret
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", webhook_id),
        json!({
            "secret": "new-rotated-secret-key"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["name"], "Home Assistant"); // Unchanged
    assert_eq!(body["secret"], "new-rotated-secret-key"); // Changed

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_webhook_toggle_enabled() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_toggle_enabled").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook (enabled by default)
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Disable the webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", webhook_id),
        json!({
            "enabled": false
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_response_body(response).await;
    assert_eq!(body["enabled"], false);

    // Re-enable the webhook
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", webhook_id),
        json!({
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_response_body(response).await;
    assert_eq!(body["enabled"], true);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_create_webhook_with_disabled_status() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_create_disabled").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a disabled webhook
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Disabled Webhook",
            "target_url": "https://example.com/webhook",
            "secret": "secret-key-12345",
            "enabled": false
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert_eq!(body["enabled"], false);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_name_with_special_characters() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_special_chars").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create webhook with special characters in name
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant (Primary) - 🏠",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert_eq!(body["name"], "Home Assistant (Primary) - 🏠");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_name_at_max_length() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_max_name").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create webhook with 100 character name (max length)
    let max_name = "A".repeat(100);
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": max_name,
            "target_url": "https://example.com/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_name_exceeds_max_length() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_exceed_name").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create webhook with 101 character name (exceeds max)
    let too_long_name = "A".repeat(101);
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": too_long_name,
            "target_url": "https://example.com/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_webhooks_different_devices_isolation() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_device_isolation").await;

    // Register first device
    let device1 = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device1_response = register_test_device(&app, &pool, &auth, &device1).await;
    let device1_id = device1_response["device_id"].as_str().unwrap();

    // Register second device
    let device2 = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device2_response = register_test_device(&app, &pool, &auth, &device2).await;
    let device2_id = device2_response["device_id"].as_str().unwrap();

    // Create webhook for device 1
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device1_id,
            "name": "Device 1 Webhook",
            "target_url": "https://device1.example.com/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // Create webhook for device 2
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device2_id,
            "name": "Device 2 Webhook",
            "target_url": "https://device2.example.com/webhook",
            "secret": "another-secret-key",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // List webhooks for device 1 - should only see device 1's webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/webhooks?ownerDeviceId={}", device1_id),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_response_body(response).await;
    let webhooks = body["webhooks"].as_array().unwrap();
    assert_eq!(webhooks.len(), 1);
    assert_eq!(webhooks[0]["name"], "Device 1 Webhook");

    // List webhooks for device 2 - should only see device 2's webhook
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/webhooks?ownerDeviceId={}", device2_id),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_response_body(response).await;
    let webhooks = body["webhooks"].as_array().unwrap();
    assert_eq!(webhooks.len(), 1);
    assert_eq!(webhooks[0]["name"], "Device 2 Webhook");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_webhook_timestamps_updated_on_modification() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_timestamps").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();
    let created_at = create_body["created_at"].as_str().unwrap();
    let original_updated_at = create_body["updated_at"].as_str().unwrap();

    // Small delay to ensure timestamp difference
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update the webhook
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", webhook_id),
        json!({
            "name": "Updated Name"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let new_updated_at = body["updated_at"].as_str().unwrap();

    // created_at should be unchanged
    assert_eq!(body["created_at"].as_str().unwrap(), created_at);
    // updated_at should be different (later)
    assert_ne!(new_updated_at, original_updated_at);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Authentication Tests
// ============================================================================

#[tokio::test]
async fn test_create_webhook_without_api_key() {
    use axum::{body::Body, http::{header, Method, Request}};

    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_no_api_key").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create webhook WITHOUT API key header
    let app = create_test_app(config, pool.clone());
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/webhooks")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", auth.access_token))
        .body(Body::from(serde_json::to_string(&json!({
            "owner_device_id": device_id,
            "name": "Test Webhook",
            "target_url": "https://example.com/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        })).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Should be unauthorized without API key
    assert!(
        response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::FORBIDDEN
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_with_invalid_api_key() {
    use axum::{body::Body, http::{header, Method, Request}};

    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_invalid_api_key").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create webhook with invalid API key
    let app = create_test_app(config, pool.clone());
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/webhooks")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", "pm_invalid_key_12345678901234567890")
        .header(header::AUTHORIZATION, format!("Bearer {}", auth.access_token))
        .body(Body::from(serde_json::to_string(&json!({
            "owner_device_id": device_id,
            "name": "Test Webhook",
            "target_url": "https://example.com/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        })).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Should be unauthorized with invalid API key
    assert!(
        response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::FORBIDDEN
    );

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Secret Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_webhook_secret_at_max_length() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_max_secret").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create webhook with 256 character secret (max length)
    let max_secret = "S".repeat(256);
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Max Secret Webhook",
            "target_url": "https://example.com/webhook",
            "secret": max_secret,
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert_eq!(body["secret"].as_str().unwrap().len(), 256);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_secret_exceeds_max_length() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_exceed_secret").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create webhook with 257 character secret (exceeds max)
    let too_long_secret = "S".repeat(257);
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Too Long Secret Webhook",
            "target_url": "https://example.com/webhook",
            "secret": too_long_secret,
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_secret_at_min_length() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_min_secret").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create webhook with 8 character secret (min length)
    let min_secret = "12345678";
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Min Secret Webhook",
            "target_url": "https://example.com/webhook",
            "secret": min_secret,
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// URL Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_webhook_url_invalid_format() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_invalid_url_format").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create webhook with invalid URL format
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Invalid URL Webhook",
            "target_url": "not-a-valid-url",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_url_with_path_and_query() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_url_with_query").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create webhook with complex URL (path + query params)
    let complex_url = "https://example.com/api/v1/webhooks/receive?token=abc123&type=location";
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Complex URL Webhook",
            "target_url": complex_url,
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert_eq!(body["target_url"], complex_url);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_webhook_url_with_port() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_url_with_port").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create webhook with custom port
    let url_with_port = "https://homeassistant.local:8123/api/webhook/location";
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Port URL Webhook",
            "target_url": url_with_port,
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert_eq!(body["target_url"], url_with_port);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Update Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_update_webhook_same_name_no_conflict() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_same_name_update").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Update with the same name (should succeed - no actual change)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", webhook_id),
        json!({
            "name": "Home Assistant"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["name"], "Home Assistant");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_webhook_short_secret_validation() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_update_short_secret").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Try to update with too short secret
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", webhook_id),
        json!({
            "secret": "short"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_webhook_all_fields_at_once() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_update_all_fields").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Original Name",
            "target_url": "https://original.example.com/webhook",
            "secret": "original-secret-key",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Update all fields at once
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", webhook_id),
        json!({
            "name": "Updated Name",
            "target_url": "https://updated.example.com/webhook",
            "secret": "updated-secret-key",
            "enabled": false
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["name"], "Updated Name");
    assert_eq!(body["target_url"], "https://updated.example.com/webhook");
    assert_eq!(body["secret"], "updated-secret-key");
    assert_eq!(body["enabled"], false);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Delete and Recreate Tests
// ============================================================================

#[tokio::test]
async fn test_delete_and_recreate_webhook_same_name() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_delete_recreate").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Delete the webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_request_with_api_key_and_jwt(
        &format!("/api/v1/webhooks/{}", webhook_id),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Recreate webhook with the same name (should succeed)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://new-homeassistant.local/webhook",
            "secret": "new-secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert_eq!(body["name"], "Home Assistant");
    // Should be a new webhook with different ID
    assert_ne!(body["webhook_id"].as_str().unwrap(), webhook_id);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Response Format Verification Tests
// ============================================================================

#[tokio::test]
async fn test_webhook_response_contains_all_required_fields() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_response_fields").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Test Webhook",
            "target_url": "https://example.com/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;

    // Verify all required fields are present
    assert!(body.get("webhook_id").is_some(), "Missing webhook_id");
    assert!(body.get("owner_device_id").is_some(), "Missing owner_device_id");
    assert!(body.get("name").is_some(), "Missing name");
    assert!(body.get("target_url").is_some(), "Missing target_url");
    assert!(body.get("secret").is_some(), "Missing secret");
    assert!(body.get("enabled").is_some(), "Missing enabled");
    assert!(body.get("created_at").is_some(), "Missing created_at");
    assert!(body.get("updated_at").is_some(), "Missing updated_at");

    // Verify field types
    assert!(body["webhook_id"].is_string());
    assert!(body["owner_device_id"].is_string());
    assert!(body["name"].is_string());
    assert!(body["target_url"].is_string());
    assert!(body["secret"].is_string());
    assert!(body["enabled"].is_boolean());
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());

    // Verify webhook_id is a valid UUID
    let webhook_id = body["webhook_id"].as_str().unwrap();
    assert!(uuid::Uuid::parse_str(webhook_id).is_ok(), "webhook_id is not a valid UUID");

    // Verify owner_device_id matches
    assert_eq!(body["owner_device_id"].as_str().unwrap(), device_id);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_webhooks_response_format() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_list_format").await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Test Webhook",
            "target_url": "https://example.com/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &api_key,
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // List webhooks
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/webhooks?ownerDeviceId={}", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;

    // Verify list response structure
    assert!(body.get("webhooks").is_some(), "Missing webhooks array");
    assert!(body.get("total").is_some(), "Missing total count");
    assert!(body["webhooks"].is_array());
    assert!(body["total"].is_number());

    let webhooks = body["webhooks"].as_array().unwrap();
    assert_eq!(webhooks.len(), 1);
    assert_eq!(body["total"].as_i64().unwrap(), 1);

    // Verify each webhook in array has all fields
    let webhook = &webhooks[0];
    assert!(webhook.get("webhook_id").is_some());
    assert!(webhook.get("owner_device_id").is_some());
    assert!(webhook.get("name").is_some());
    assert!(webhook.get("target_url").is_some());
    assert!(webhook.get("secret").is_some());
    assert!(webhook.get("enabled").is_some());
    assert!(webhook.get("created_at").is_some());
    assert!(webhook.get("updated_at").is_some());

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Invalid UUID Tests
// ============================================================================

#[tokio::test]
async fn test_get_webhook_invalid_uuid() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_invalid_uuid").await;

    // Try to get webhook with invalid UUID
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        "/api/v1/webhooks/not-a-valid-uuid",
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_webhook_invalid_uuid() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_delete_invalid_uuid").await;

    // Try to delete webhook with invalid UUID
    let app = create_test_app(config, pool.clone());
    let request = delete_request_with_api_key_and_jwt(
        "/api/v1/webhooks/not-a-valid-uuid",
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_webhook_invalid_uuid() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_update_invalid_uuid").await;

    // Try to update webhook with invalid UUID
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        "/api/v1/webhooks/not-a-valid-uuid",
        json!({
            "name": "Updated Name"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_webhooks_invalid_device_uuid() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_api_key(&pool, "test_list_invalid_device").await;

    // Try to list webhooks with invalid device UUID
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        "/api/v1/webhooks?ownerDeviceId=not-a-valid-uuid",
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    cleanup_all_test_data(&pool).await;
}
