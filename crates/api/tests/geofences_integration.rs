//! Integration tests for geofence management endpoints.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test geofences_integration

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
// Geofence Creation Tests
// ============================================================================

#[tokio::test]
async fn test_create_geofence_success() {
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

    // Create a geofence
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofences",
        json!({
            "device_id": device_id,
            "name": "Home",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius": 100.0,
            "event_types": ["enter", "exit"]
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body.get("id").is_some());
    assert_eq!(body["name"], "Home");
    assert_eq!(body["radius"], 100.0);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_geofence_invalid_radius_too_small() {
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

    // Try to create geofence with radius too small (< 20 meters)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofences",
        json!({
            "device_id": device_id,
            "name": "Tiny Zone",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius": 5.0,
            "event_types": ["enter"]
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
async fn test_create_geofence_invalid_radius_too_large() {
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

    // Try to create geofence with radius too large (> 50000 meters)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofences",
        json!({
            "device_id": device_id,
            "name": "Giant Zone",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius": 100000.0,
            "event_types": ["enter"]
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
async fn test_create_geofence_device_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user but don't register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to create geofence for non-existent device
    let fake_device_id = uuid::Uuid::new_v4();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofences",
        json!({
            "device_id": fake_device_id.to_string(),
            "name": "Home",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius": 100.0,
            "event_types": ["enter"]
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Geofence Listing Tests
// ============================================================================

#[tokio::test]
async fn test_list_geofences_success() {
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

    // Create two geofences
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofences",
        json!({
            "device_id": device_id,
            "name": "Home",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius": 100.0,
            "event_types": ["enter", "exit"]
        }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofences",
        json!({
            "device_id": device_id,
            "name": "Work",
            "latitude": 37.7849,
            "longitude": -122.4094,
            "radius": 200.0,
            "event_types": ["enter"]
        }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // List geofences for device
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/geofences?device_id={}", device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let geofences = body["geofences"].as_array().unwrap();
    assert_eq!(geofences.len(), 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_geofences_empty() {
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

    // List geofences for device (none exist)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/geofences?device_id={}", device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let geofences = body["geofences"].as_array().unwrap();
    assert!(geofences.is_empty());

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Get Single Geofence Tests
// ============================================================================

#[tokio::test]
async fn test_get_geofence_success() {
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

    // Create a geofence
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofences",
        json!({
            "device_id": device_id,
            "name": "Home",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius": 100.0,
            "event_types": ["enter", "exit"]
        }),
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let geofence_id = create_body["id"].as_str().unwrap();

    // Get the geofence
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/geofences/{}", geofence_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["id"], geofence_id);
    assert_eq!(body["name"], "Home");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_geofence_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to get non-existent geofence
    let fake_geofence_id = uuid::Uuid::new_v4();
    let request = get_request_with_auth(
        &format!("/api/v1/geofences/{}", fake_geofence_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Geofence Update Tests
// ============================================================================

#[tokio::test]
async fn test_update_geofence_success() {
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

    // Create a geofence
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofences",
        json!({
            "device_id": device_id,
            "name": "Home",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius": 100.0,
            "event_types": ["enter"]
        }),
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let geofence_id = create_body["id"].as_str().unwrap();

    // Update the geofence
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::PATCH,
        &format!("/api/v1/geofences/{}", geofence_id),
        json!({
            "name": "New Home",
            "radius": 150.0
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["name"], "New Home");
    assert_eq!(body["radius"], 150.0);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Geofence Deletion Tests
// ============================================================================

#[tokio::test]
async fn test_delete_geofence_success() {
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

    // Create a geofence
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofences",
        json!({
            "device_id": device_id,
            "name": "Home",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius": 100.0,
            "event_types": ["enter"]
        }),
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let geofence_id = create_body["id"].as_str().unwrap();

    // Delete the geofence
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_request_with_auth(
        &format!("/api/v1/geofences/{}", geofence_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify geofence is gone
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/geofences/{}", geofence_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_geofence_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to delete non-existent geofence
    let fake_geofence_id = uuid::Uuid::new_v4();
    let request = delete_request_with_auth(
        &format!("/api/v1/geofences/{}", fake_geofence_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}
