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
    cleanup_all_test_data, create_authenticated_user, create_test_app, create_test_pool,
    delete_request_with_auth, get_request_with_auth, json_request_with_auth, parse_response_body,
    register_test_device, run_migrations, test_config, TestDevice, TestUser,
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
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/api/webhook/test",
            "secret": "my-secret-key-12345",
            "enabled": true
        }),
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
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create webhook with empty name
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "",
            "target_url": "https://example.com/webhook",
            "secret": "my-secret-key-12345",
            "enabled": true
        }),
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
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create webhook with HTTP URL (not HTTPS)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Insecure Webhook",
            "target_url": "http://example.com/webhook",
            "secret": "my-secret-key-12345",
            "enabled": true
        }),
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
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create webhook with secret too short (< 8 characters)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Short Secret",
            "target_url": "https://example.com/webhook",
            "secret": "short",
            "enabled": true
        }),
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
    let app = create_test_app(config, pool.clone());

    // Create authenticated user but don't register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to create webhook for non-existent device
    let fake_device_id = uuid::Uuid::new_v4();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": fake_device_id.to_string(),
            "name": "Orphan Webhook",
            "target_url": "https://example.com/webhook",
            "secret": "my-secret-key-12345",
            "enabled": true
        }),
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
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create first webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://example.com/webhook",
            "secret": "my-secret-key-12345",
            "enabled": true
        }),
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Try to create second webhook with same name
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://another.com/webhook",
            "secret": "different-secret-key",
            "enabled": true
        }),
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
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create two webhooks
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "n8n Automation",
            "target_url": "https://n8n.local/webhook/test",
            "secret": "another-secret-key",
            "enabled": false
        }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // List webhooks for device
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/webhooks?ownerDeviceId={}", device_id),
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
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // List webhooks for device (none exist)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/webhooks?ownerDeviceId={}", device_id),
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
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // List webhooks without device ID
    let request = get_request_with_auth("/api/v1/webhooks", &auth.access_token);

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
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Get the webhook
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/webhooks/{}", webhook_id),
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
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to get non-existent webhook
    let fake_webhook_id = uuid::Uuid::new_v4();
    let request = get_request_with_auth(
        &format!("/api/v1/webhooks/{}", fake_webhook_id),
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
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Update the webhook
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", webhook_id),
        json!({
            "name": "Updated Home Assistant",
            "enabled": false
        }),
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
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to update non-existent webhook
    let fake_webhook_id = uuid::Uuid::new_v4();
    let request = json_request_with_auth(
        Method::PUT,
        &format!("/api/v1/webhooks/{}", fake_webhook_id),
        json!({
            "name": "Updated Name"
        }),
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
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Home Assistant",
            "target_url": "https://homeassistant.local/webhook",
            "secret": "secret-key-12345",
            "enabled": true
        }),
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let webhook_id = create_body["webhook_id"].as_str().unwrap();

    // Delete the webhook
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_request_with_auth(
        &format!("/api/v1/webhooks/{}", webhook_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify webhook is gone
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/webhooks/{}", webhook_id),
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
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to delete non-existent webhook
    let fake_webhook_id = uuid::Uuid::new_v4();
    let request = delete_request_with_auth(
        &format!("/api/v1/webhooks/{}", fake_webhook_id),
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
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create 10 webhooks (the limit)
    for i in 0..10 {
        let app = create_test_app(config.clone(), pool.clone());
        let request = json_request_with_auth(
            Method::POST,
            "/api/v1/webhooks",
            json!({
                "owner_device_id": device_id,
                "name": format!("Webhook {}", i),
                "target_url": format!("https://example.com/webhook/{}", i),
                "secret": format!("secret-key-{:05}", i),
                "enabled": true
            }),
            &auth.access_token,
        );
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED, "Failed to create webhook {}", i);
    }

    // Try to create 11th webhook (should fail)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/webhooks",
        json!({
            "owner_device_id": device_id,
            "name": "Webhook 10 - Over Limit",
            "target_url": "https://example.com/webhook/overlimit",
            "secret": "secret-key-overlimit",
            "enabled": true
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}
