//! Integration tests for device settings endpoints.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test device_settings_integration

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_api_key, create_test_app,
    create_test_pool, get_request_with_api_key_and_jwt, json_request_with_api_key_and_jwt,
    parse_response_body, register_test_device, run_migrations, seed_setting_definitions,
    test_config, TestDevice, TestUser,
};
use serde_json::json;
use tower::ServiceExt;

// ============================================================================
// Get Device Settings Tests
// ============================================================================

#[tokio::test]
async fn test_get_device_settings_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register a device
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Get device settings
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_get_settings").await;
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/settings", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["device_id"], device_id);
    assert!(body["settings"].is_object());

    // Check some default settings exist (seeded by migration)
    let settings = body["settings"].as_object().unwrap();
    assert!(settings.contains_key("tracking_enabled"));
    assert!(settings.contains_key("tracking_interval_minutes"));

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_device_settings_with_definitions() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Get settings with definitions
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_get_settings_def").await;
    let request = get_request_with_api_key_and_jwt(
        &format!(
            "/api/v1/devices/{}/settings?include_definitions=true",
            device_id
        ),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["definitions"].is_array());
    let definitions = body["definitions"].as_array().unwrap();
    assert!(!definitions.is_empty());

    // Verify definition structure
    let first_def = &definitions[0];
    assert!(first_def["key"].is_string());
    assert!(first_def["display_name"].is_string());
    assert!(first_def["data_type"].is_string());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_device_settings_device_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let fake_device_id = uuid::Uuid::new_v4();
    let api_key = create_test_api_key(&pool, "test_settings_not_found").await;
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/settings", fake_device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_device_settings_unauthorized() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();

    // Create first user and register a device
    let app = create_test_app(config.clone(), pool.clone());
    let user1 = TestUser::new();
    let auth1 = create_authenticated_user(&app, &user1).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth1, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create second user (different group)
    let app = create_test_app(config.clone(), pool.clone());
    let user2 = TestUser::new();
    let auth2 = create_authenticated_user(&app, &user2).await;

    // Try to get settings with second user (should fail - not authorized)
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_settings_unauth").await;
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/settings", device_id),
        &api_key,
        &auth2.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Update Device Settings Tests
// ============================================================================

#[tokio::test]
async fn test_update_device_settings_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Update settings
    let app = create_test_app(config.clone(), pool.clone());
    let api_key = create_test_api_key(&pool, "test_update_settings").await;
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/devices/{}/settings", device_id),
        json!({
            "settings": {
                "tracking_enabled": false,
                "tracking_interval_minutes": 10
            }
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let updated = body["updated"].as_array().unwrap();
    assert!(updated.iter().any(|v| v == "tracking_enabled"));
    assert!(updated.iter().any(|v| v == "tracking_interval_minutes"));

    // Verify settings were updated
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_api_key(&pool, "test_verify_update").await;
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/settings", device_id),
        &api_key2,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;
    assert_eq!(body["settings"]["tracking_enabled"]["value"], false);
    assert_eq!(body["settings"]["tracking_interval_minutes"]["value"], 10);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_single_setting_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Update single setting
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_update_single").await;
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/devices/{}/settings/tracking_enabled", device_id),
        json!({
            "value": false
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    // Response is SettingValue which has value field directly
    assert_eq!(body["value"], false);
    assert!(body["is_locked"].is_boolean());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_setting_invalid_key() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to update non-existent setting
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_invalid_key").await;
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/devices/{}/settings/nonexistent_setting", device_id),
        json!({
            "value": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Setting Lock Tests
// ============================================================================

#[tokio::test]
async fn test_lock_setting_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Lock a setting
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_lock_setting").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!(
            "/api/v1/devices/{}/settings/tracking_enabled/lock",
            device_id
        ),
        json!({
            "reason": "Parental control",
            "value": true
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["key"], "tracking_enabled");
    assert_eq!(body["is_locked"], true);
    assert_eq!(body["reason"], "Parental control");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_unlock_setting_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // First lock the setting
    let app = create_test_app(config.clone(), pool.clone());
    let api_key = create_test_api_key(&pool, "test_lock_for_unlock").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!(
            "/api/v1/devices/{}/settings/tracking_enabled/lock",
            device_id
        ),
        json!({
            "reason": "Test lock"
        }),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Now unlock it
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_api_key(&pool, "test_unlock_setting").await;
    let request = axum::http::Request::builder()
        .method(Method::DELETE)
        .uri(format!(
            "/api/v1/devices/{}/settings/tracking_enabled/lock",
            device_id
        ))
        .header("Content-Type", "application/json")
        .header("X-API-Key", &api_key2)
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["key"], "tracking_enabled");
    assert_eq!(body["is_locked"], false);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_lock_non_lockable_setting() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to lock a non-lockable setting (battery_optimization_enabled is not lockable)
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_lock_nonlockable").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!(
            "/api/v1/devices/{}/settings/battery_optimization_enabled/lock",
            device_id
        ),
        json!({
            "reason": "Should fail"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should fail because the setting is not lockable
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::CONFLICT
            || response.status() == StatusCode::FORBIDDEN
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_setting_locks() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Lock a setting first
    let app = create_test_app(config.clone(), pool.clone());
    let api_key = create_test_api_key(&pool, "test_lock_for_list").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!(
            "/api/v1/devices/{}/settings/tracking_enabled/lock",
            device_id
        ),
        json!({
            "reason": "Test lock"
        }),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Get all locks
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_api_key(&pool, "test_get_locks").await;
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/settings/locks", device_id),
        &api_key2,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    // Response is ListLocksResponse with locks as array
    assert!(body["locks"].is_array());
    let locks = body["locks"].as_array().unwrap();
    // Should have at least one locked setting
    assert!(!locks.is_empty());
    // Check that locked_count is present
    assert!(body["locked_count"].is_i64());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_bulk_update_locks() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Bulk update locks - format is locks: { key: bool }
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_bulk_locks").await;
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/devices/{}/settings/locks", device_id),
        json!({
            "locks": {
                "tracking_enabled": true,
                "secret_mode_enabled": true
            },
            "reason": "Bulk lock reason"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    // Response is BulkUpdateLocksResponse
    assert!(body["updated"].is_array() || body["skipped"].is_array());

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Update Locked Setting Tests
// ============================================================================

#[tokio::test]
async fn test_update_locked_setting_fails() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Lock the setting first
    let app = create_test_app(config.clone(), pool.clone());
    let api_key = create_test_api_key(&pool, "test_lock_first").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!(
            "/api/v1/devices/{}/settings/tracking_enabled/lock",
            device_id
        ),
        json!({
            "reason": "Locked for test",
            "value": true
        }),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Try to update the locked setting (should be skipped)
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_api_key(&pool, "test_update_locked").await;
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/devices/{}/settings", device_id),
        json!({
            "settings": {
                "tracking_enabled": false
            }
        }),
        &api_key2,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    // The locked setting should be in the locked array, not updated
    let locked = body["locked"].as_array().unwrap();
    assert!(locked.iter().any(|v| v == "tracking_enabled"));

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Sync Settings Tests
// ============================================================================

#[tokio::test]
async fn test_sync_settings_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Sync settings from device
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_sync_settings").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!("/api/v1/devices/{}/settings/sync", device_id),
        json!({
            "settings": {
                "tracking_enabled": true,
                "tracking_interval_minutes": 15
            },
            "device_timestamp": chrono::Utc::now().timestamp_millis()
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    // SyncSettingsResponse has synced_at, settings, changes_applied
    assert!(body["synced_at"].is_string());
    assert!(body["settings"].is_object());
    assert!(body["changes_applied"].is_array());

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Unlock Request Tests
// ============================================================================

#[tokio::test]
async fn test_create_unlock_request_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // First lock the setting
    let app = create_test_app(config.clone(), pool.clone());
    let api_key = create_test_api_key(&pool, "test_lock_for_request").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!(
            "/api/v1/devices/{}/settings/tracking_enabled/lock",
            device_id
        ),
        json!({
            "reason": "Parental control"
        }),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Create an unlock request
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_api_key(&pool, "test_create_unlock_req").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!(
            "/api/v1/devices/{}/settings/tracking_enabled/unlock-request",
            device_id
        ),
        json!({
            "reason": "Need to disable tracking temporarily"
        }),
        &api_key2,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    // CreateUnlockRequestResponse has id, device_id, setting_key, status, etc.
    assert!(body["id"].is_string());
    assert_eq!(body["setting_key"], "tracking_enabled");
    assert_eq!(body["status"], "pending");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_respond_to_unlock_request_approve() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Lock the setting
    let app = create_test_app(config.clone(), pool.clone());
    let api_key = create_test_api_key(&pool, "test_lock_approve").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!(
            "/api/v1/devices/{}/settings/tracking_enabled/lock",
            device_id
        ),
        json!({}),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Create unlock request
    let app = create_test_app(config.clone(), pool.clone());
    let api_key2 = create_test_api_key(&pool, "test_unlock_req_approve").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!(
            "/api/v1/devices/{}/settings/tracking_enabled/unlock-request",
            device_id
        ),
        json!({
            "reason": "Please unlock"
        }),
        &api_key2,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_response_body(response).await;
    let request_id = body["id"].as_str().unwrap();

    // Approve the request
    let app = create_test_app(config, pool.clone());
    let api_key3 = create_test_api_key(&pool, "test_approve_request").await;
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/unlock-requests/{}", request_id),
        json!({
            "status": "approved",
            "note": "Request approved"
        }),
        &api_key3,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["status"], "approved");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_respond_to_unlock_request_deny() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Lock the setting
    let app = create_test_app(config.clone(), pool.clone());
    let api_key = create_test_api_key(&pool, "test_lock_deny").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!(
            "/api/v1/devices/{}/settings/tracking_enabled/lock",
            device_id
        ),
        json!({}),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Create unlock request
    let app = create_test_app(config.clone(), pool.clone());
    let api_key2 = create_test_api_key(&pool, "test_unlock_req_deny").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!(
            "/api/v1/devices/{}/settings/tracking_enabled/unlock-request",
            device_id
        ),
        json!({
            "reason": "Please unlock"
        }),
        &api_key2,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_response_body(response).await;
    let request_id = body["id"].as_str().unwrap();

    // Deny the request
    let app = create_test_app(config, pool.clone());
    let api_key3 = create_test_api_key(&pool, "test_deny_request").await;
    let request = json_request_with_api_key_and_jwt(
        Method::PUT,
        &format!("/api/v1/unlock-requests/{}", request_id),
        json!({
            "status": "denied",
            "note": "Request denied for safety"
        }),
        &api_key3,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["status"], "denied");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_unlock_request_setting_not_locked() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;
    seed_setting_definitions(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create unlock request for unlocked setting
    let app = create_test_app(config, pool.clone());
    let api_key = create_test_api_key(&pool, "test_unlock_not_locked").await;
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        &format!(
            "/api/v1/devices/{}/settings/tracking_enabled/unlock-request",
            device_id
        ),
        json!({
            "reason": "Not locked"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should fail because setting is not locked
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::CONFLICT
    );

    cleanup_all_test_data(&pool).await;
}
