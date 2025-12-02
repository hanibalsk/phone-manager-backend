//! Integration tests for admin operation endpoints.
//!
//! Tests administrative functionality like stats, inactive device cleanup, and reactivation.

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_api_key, create_test_app,
    create_test_pool, delete_request_with_api_key, get_request_with_api_key,
    json_request_with_api_key, json_request_with_auth, parse_response_body, register_test_device,
    run_migrations, test_config, TestDevice, TestUser,
};
use serde_json::json;
use tower::ServiceExt;

// ============================================================================
// Admin Stats Tests
// ============================================================================

#[tokio::test]
async fn test_get_admin_stats_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create a user and some devices
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _ = register_test_device(&app, &auth, &device).await;

    // Get admin stats
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key("/api/v1/admin/stats", &api_key);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body.get("total_devices").is_some());
    assert!(body.get("active_devices").is_some());
    assert!(body.get("inactive_devices").is_some());
    assert!(body.get("total_locations").is_some());
    assert!(body.get("total_groups").is_some());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_admin_stats_without_api_key() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Try to get stats without API key
    let request = axum::http::Request::builder()
        .method(Method::GET)
        .uri("/api/v1/admin/stats")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_admin_stats_with_invalid_api_key() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Try to get stats with invalid API key
    let request = get_request_with_api_key("/api/v1/admin/stats", "invalid_key");

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Delete Inactive Devices Tests
// ============================================================================

#[tokio::test]
async fn test_delete_inactive_devices_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create a user and register a device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Deactivate the device first (soft delete)
    let app = create_test_app(config.clone(), pool.clone());
    let request = axum::http::Request::builder()
        .method(Method::DELETE)
        .uri(&format!("/api/v1/devices/{}", device_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .body(axum::body::Body::empty())
        .unwrap();
    let _ = app.oneshot(request).await.unwrap();

    // Delete inactive devices (should not delete anything since threshold is too high)
    let app = create_test_app(config, pool.clone());
    let request = delete_request_with_api_key(
        "/api/v1/admin/devices/inactive?older_than_days=365",
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["success"].as_bool().unwrap(), true);
    assert!(body.get("affected_count").is_some());
    assert!(body.get("message").is_some());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_inactive_devices_invalid_threshold_too_low() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    let app = create_test_app(config, pool.clone());
    let request = delete_request_with_api_key(
        "/api/v1/admin/devices/inactive?older_than_days=0",
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should fail validation - threshold must be at least 1
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_inactive_devices_invalid_threshold_too_high() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    let app = create_test_app(config, pool.clone());
    let request = delete_request_with_api_key(
        "/api/v1/admin/devices/inactive?older_than_days=500",
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should fail validation - threshold cannot exceed 365
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_inactive_devices_without_api_key() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let request = axum::http::Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/admin/devices/inactive?older_than_days=30")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Reactivate Device Tests
// ============================================================================

#[tokio::test]
async fn test_reactivate_device_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create a user and register a device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Deactivate the device (soft delete)
    let app = create_test_app(config.clone(), pool.clone());
    let request = axum::http::Request::builder()
        .method(Method::DELETE)
        .uri(&format!("/api/v1/devices/{}", device_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .body(axum::body::Body::empty())
        .unwrap();
    let _ = app.oneshot(request).await.unwrap();

    // Reactivate the device via admin API
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/v1/admin/devices/{}/reactivate", device_id),
        json!({}),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["success"].as_bool().unwrap(), true);
    assert_eq!(body["device_id"].as_str().unwrap(), device_id);
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("reactivated"));

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_reactivate_device_already_active() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create a user and register a device (already active)
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to reactivate already active device
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/v1/admin/devices/{}/reactivate", device_id),
        json!({}),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["success"].as_bool().unwrap(), true);
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("already active"));

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_reactivate_device_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    let app = create_test_app(config, pool.clone());
    let fake_device_id = uuid::Uuid::new_v4();
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/v1/admin/devices/{}/reactivate", fake_device_id),
        json!({}),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_reactivate_device_without_api_key() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let fake_device_id = uuid::Uuid::new_v4();
    let request = axum::http::Request::builder()
        .method(Method::POST)
        .uri(&format!(
            "/api/v1/admin/devices/{}/reactivate",
            fake_device_id
        ))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_all_test_data(&pool).await;
}
