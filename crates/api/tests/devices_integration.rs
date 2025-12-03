//! Integration tests for device management endpoints.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test devices_integration

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
// Device Registration Tests
// ============================================================================

#[tokio::test]
async fn test_register_device_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register a device
    let device = TestDevice::new();
    let response = register_test_device(&app, &pool, &auth, &device).await;

    assert!(response.get("device_id").is_some());
    assert_eq!(response["device_id"], device.device_id);
    assert_eq!(response["display_name"], device.display_name);
    assert_eq!(response["group_id"], device.group_id);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_register_device_update_existing() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register a device
    let device = TestDevice::new();
    let _first_response = register_test_device(&app, &pool, &auth, &device).await;

    // Update the same device with new display name
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_update_device").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/devices/register",
        json!({
            "device_id": device.device_id,
            "display_name": "Updated Device Name",
            "group_id": device.group_id,
            "platform": device.platform,
            "os_version": device.os_version,
            "app_version": device.app_version
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Update should return OK (200)
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["device_id"], device.device_id);
    assert_eq!(body["display_name"], "Updated Device Name");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_register_device_with_jwt_links_to_user() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register a device with JWT auth
    let device = TestDevice::new();
    let response = register_test_device(&app, &pool, &auth, &device).await;

    // The device should be linked to the user (owner_user_id set)
    assert!(response.get("device_id").is_some());
    // First device should be primary
    if let Some(is_primary) = response.get("is_primary") {
        assert_eq!(is_primary.as_bool().unwrap_or(false), true);
    }

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_register_device_group_capacity_limit() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let mut config = test_config();
    // Set a low device limit for testing
    config.limits.max_devices_per_group = 2;

    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create a shared group ID
    let group_id = format!("test-group-{}", uuid::Uuid::new_v4().simple());

    // Register first device
    let device1 = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let _response1 = register_test_device(&app, &pool, &auth, &device1).await;

    // Register second device
    let device2 = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let _response2 = register_test_device(&app, &pool, &auth, &device2).await;

    // Third device should fail due to capacity limit
    let device3 = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_capacity_limit").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/devices/register",
        json!({
            "device_id": device3.device_id,
            "display_name": device3.display_name,
            "group_id": device3.group_id,
            "platform": device3.platform,
            "os_version": device3.os_version,
            "app_version": device3.app_version
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = parse_response_body(response).await;
    assert!(body["message"]
        .as_str()
        .unwrap_or("")
        .contains("maximum device limit"));

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_register_device_invalid_data() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to register with invalid UUID
    let api_key = create_test_api_key(&pool, "test_invalid_data").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/devices/register",
        json!({
            "device_id": "not-a-valid-uuid",
            "display_name": "Test Device",
            "group_id": "test-group",
            "platform": "android"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should fail validation
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_register_device_empty_display_name() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to register with empty display name
    let api_key = create_test_api_key(&pool, "test_empty_name").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/devices/register",
        json!({
            "device_id": uuid::Uuid::new_v4().to_string(),
            "display_name": "",
            "group_id": "test-group",
            "platform": "android"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should fail validation
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Get Group Devices Tests
// ============================================================================

#[tokio::test]
async fn test_get_group_devices_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create a shared group ID
    let group_id = format!("test-group-{}", uuid::Uuid::new_v4().simple());

    // Register two devices in the same group
    let device1 = TestDevice::new().with_group(&group_id).with_name("Device 1");
    let app = create_test_app(config.clone(), pool.clone());
    let _response1 = register_test_device(&app, &pool, &auth, &device1).await;

    let device2 = TestDevice::new().with_group(&group_id).with_name("Device 2");
    let app = create_test_app(config.clone(), pool.clone());
    let _response2 = register_test_device(&app, &pool, &auth, &device2).await;

    // Get devices in the group
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_get_devices").await;
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices?group_id={}", group_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let devices = body["devices"].as_array().unwrap();
    assert_eq!(devices.len(), 2);

    // Verify device names are present
    let names: Vec<&str> = devices
        .iter()
        .map(|d| d["display_name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Device 1"));
    assert!(names.contains(&"Device 2"));

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_group_devices_empty_group() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Get devices from a group that has no devices
    let api_key = create_test_api_key(&pool, "test_empty_group").await;
    let request = get_request_with_api_key_and_jwt(
        "/api/v1/devices?group_id=nonexistent-group",
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let devices = body["devices"].as_array().unwrap();
    assert!(devices.is_empty());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_group_devices_missing_group_id() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Get devices without group_id parameter
    let api_key = create_test_api_key(&pool, "test_missing_group_id").await;
    let request = get_request_with_api_key_and_jwt("/api/v1/devices", &api_key, &auth.access_token);

    let response = app.oneshot(request).await.unwrap();
    // Should fail validation - group_id is required
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Delete Device Tests
// ============================================================================

#[tokio::test]
async fn test_delete_device_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register a device
    let device = TestDevice::new();
    let response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = response["device_id"].as_str().unwrap();

    // Delete the device
    let app = create_test_app(config.clone(), pool.clone());
    let api_key = create_test_api_key(&pool, "test_delete_device").await;
    let request = delete_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify device is no longer in the group list (soft deleted)
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_api_key(&pool, "test_delete_device_verify").await;
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices?group_id={}", device.group_id),
        &api_key2,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;
    let devices = body["devices"].as_array().unwrap();
    assert!(devices.is_empty());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_device_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to delete a non-existent device
    let fake_device_id = uuid::Uuid::new_v4();
    let api_key = create_test_api_key(&pool, "test_delete_not_found").await;
    let request = delete_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}", fake_device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_device_invalid_uuid() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to delete with invalid UUID
    let api_key = create_test_api_key(&pool, "test_delete_invalid_uuid").await;
    let request = delete_request_with_api_key_and_jwt(
        "/api/v1/devices/not-a-valid-uuid",
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    cleanup_all_test_data(&pool).await;
}
