//! Integration tests for proximity alert endpoints.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test proximity_alerts_integration

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
// Proximity Alert Creation Tests
// ============================================================================

#[tokio::test]
async fn test_create_proximity_alert_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create a shared group ID for both devices
    let group_id = format!("test-group-{}", uuid::Uuid::new_v4().simple());

    // Register source device
    let source_device = TestDevice::new().with_group(&group_id).with_name("Source Device");
    let app = create_test_app(config.clone(), pool.clone());
    let source_response = register_test_device(&app, &auth, &source_device).await;
    let source_device_id = source_response["device_id"].as_str().unwrap();

    // Register target device
    let target_device = TestDevice::new().with_group(&group_id).with_name("Target Device");
    let app = create_test_app(config.clone(), pool.clone());
    let target_response = register_test_device(&app, &auth, &target_device).await;
    let target_device_id = target_response["device_id"].as_str().unwrap();

    // Create a proximity alert
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/proximity-alerts",
        json!({
            "source_device_id": source_device_id,
            "target_device_id": target_device_id,
            "radius": 500.0,
            "name": "Near Target Alert"
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body.get("id").is_some());
    assert_eq!(body["radius"], 500.0);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_proximity_alert_invalid_radius_too_small() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let group_id = format!("test-group-{}", uuid::Uuid::new_v4().simple());

    // Register two devices
    let source_device = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let source_response = register_test_device(&app, &auth, &source_device).await;
    let source_device_id = source_response["device_id"].as_str().unwrap();

    let target_device = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let target_response = register_test_device(&app, &auth, &target_device).await;
    let target_device_id = target_response["device_id"].as_str().unwrap();

    // Try to create proximity alert with radius too small (< 50 meters)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/proximity-alerts",
        json!({
            "source_device_id": source_device_id,
            "target_device_id": target_device_id,
            "radius": 10.0,
            "name": "Too Close Alert"
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
async fn test_create_proximity_alert_same_device() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register single device
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create proximity alert with same source and target
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/proximity-alerts",
        json!({
            "source_device_id": device_id,
            "target_device_id": device_id,
            "radius": 500.0,
            "name": "Self Alert"
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should fail - can't create alert for same device
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_proximity_alert_different_groups() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register devices in different groups
    let source_device = TestDevice::new().with_group("group-a");
    let app = create_test_app(config.clone(), pool.clone());
    let source_response = register_test_device(&app, &auth, &source_device).await;
    let source_device_id = source_response["device_id"].as_str().unwrap();

    let target_device = TestDevice::new().with_group("group-b");
    let app = create_test_app(config.clone(), pool.clone());
    let target_response = register_test_device(&app, &auth, &target_device).await;
    let target_device_id = target_response["device_id"].as_str().unwrap();

    // Try to create proximity alert between devices in different groups
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/proximity-alerts",
        json!({
            "source_device_id": source_device_id,
            "target_device_id": target_device_id,
            "radius": 500.0,
            "name": "Cross Group Alert"
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should fail - devices must be in same group
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::FORBIDDEN
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Proximity Alert Listing Tests
// ============================================================================

#[tokio::test]
async fn test_list_proximity_alerts_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let group_id = format!("test-group-{}", uuid::Uuid::new_v4().simple());

    // Register three devices
    let source_device = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let source_response = register_test_device(&app, &auth, &source_device).await;
    let source_device_id = source_response["device_id"].as_str().unwrap();

    let target1 = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let target1_response = register_test_device(&app, &auth, &target1).await;
    let target1_id = target1_response["device_id"].as_str().unwrap();

    let target2 = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let target2_response = register_test_device(&app, &auth, &target2).await;
    let target2_id = target2_response["device_id"].as_str().unwrap();

    // Create two proximity alerts from same source
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/proximity-alerts",
        json!({
            "source_device_id": source_device_id,
            "target_device_id": target1_id,
            "radius": 500.0,
            "name": "Alert 1"
        }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/proximity-alerts",
        json!({
            "source_device_id": source_device_id,
            "target_device_id": target2_id,
            "radius": 1000.0,
            "name": "Alert 2"
        }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // List alerts for source device
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/proximity-alerts?source_device_id={}", source_device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let alerts = body["alerts"].as_array().unwrap();
    assert_eq!(alerts.len(), 2);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Get Single Proximity Alert Tests
// ============================================================================

#[tokio::test]
async fn test_get_proximity_alert_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let group_id = format!("test-group-{}", uuid::Uuid::new_v4().simple());

    // Register two devices
    let source_device = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let source_response = register_test_device(&app, &auth, &source_device).await;
    let source_device_id = source_response["device_id"].as_str().unwrap();

    let target_device = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let target_response = register_test_device(&app, &auth, &target_device).await;
    let target_device_id = target_response["device_id"].as_str().unwrap();

    // Create a proximity alert
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/proximity-alerts",
        json!({
            "source_device_id": source_device_id,
            "target_device_id": target_device_id,
            "radius": 500.0,
            "name": "Test Alert"
        }),
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let alert_id = create_body["id"].as_str().unwrap();

    // Get the alert
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/proximity-alerts/{}", alert_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["id"], alert_id);
    assert_eq!(body["radius"], 500.0);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_proximity_alert_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to get non-existent alert
    let fake_alert_id = uuid::Uuid::new_v4();
    let request = get_request_with_auth(
        &format!("/api/v1/proximity-alerts/{}", fake_alert_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Proximity Alert Update Tests
// ============================================================================

#[tokio::test]
async fn test_update_proximity_alert_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let group_id = format!("test-group-{}", uuid::Uuid::new_v4().simple());

    // Register two devices
    let source_device = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let source_response = register_test_device(&app, &auth, &source_device).await;
    let source_device_id = source_response["device_id"].as_str().unwrap();

    let target_device = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let target_response = register_test_device(&app, &auth, &target_device).await;
    let target_device_id = target_response["device_id"].as_str().unwrap();

    // Create a proximity alert
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/proximity-alerts",
        json!({
            "source_device_id": source_device_id,
            "target_device_id": target_device_id,
            "radius": 500.0,
            "name": "Original Name"
        }),
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let alert_id = create_body["id"].as_str().unwrap();

    // Update the alert
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::PATCH,
        &format!("/api/v1/proximity-alerts/{}", alert_id),
        json!({
            "radius": 1000.0,
            "name": "Updated Name"
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["radius"], 1000.0);
    assert_eq!(body["name"], "Updated Name");

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Proximity Alert Deletion Tests
// ============================================================================

#[tokio::test]
async fn test_delete_proximity_alert_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    let group_id = format!("test-group-{}", uuid::Uuid::new_v4().simple());

    // Register two devices
    let source_device = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let source_response = register_test_device(&app, &auth, &source_device).await;
    let source_device_id = source_response["device_id"].as_str().unwrap();

    let target_device = TestDevice::new().with_group(&group_id);
    let app = create_test_app(config.clone(), pool.clone());
    let target_response = register_test_device(&app, &auth, &target_device).await;
    let target_device_id = target_response["device_id"].as_str().unwrap();

    // Create a proximity alert
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/proximity-alerts",
        json!({
            "source_device_id": source_device_id,
            "target_device_id": target_device_id,
            "radius": 500.0,
            "name": "To Delete"
        }),
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let alert_id = create_body["id"].as_str().unwrap();

    // Delete the alert
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_request_with_auth(
        &format!("/api/v1/proximity-alerts/{}", alert_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify alert is gone
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/proximity-alerts/{}", alert_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}
