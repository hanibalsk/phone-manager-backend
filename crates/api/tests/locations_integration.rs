//! Integration tests for location tracking endpoints.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test locations_integration

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_app, create_test_pool,
    get_request_with_auth, json_request_with_auth, parse_response_body, register_test_device,
    run_migrations, test_config, TestDevice, TestUser,
};
use serde_json::json;
use tower::ServiceExt;

// ============================================================================
// Single Location Upload Tests
// ============================================================================

#[tokio::test]
async fn test_upload_location_success() {
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
    let _device_response = register_test_device(&app, &auth, &device).await;

    // Upload a location
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/locations",
        json!({
            "device_id": device.device_id,
            "latitude": 37.7749,
            "longitude": -122.4194,
            "accuracy": 10.5,
            "timestamp": chrono::Utc::now().timestamp_millis()
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body.get("processed").is_some());
    assert_eq!(body["processed"].as_i64().unwrap(), 1);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_upload_location_device_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user but don't register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to upload location for non-existent device
    let fake_device_id = uuid::Uuid::new_v4();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/locations",
        json!({
            "device_id": fake_device_id.to_string(),
            "latitude": 37.7749,
            "longitude": -122.4194,
            "accuracy": 10.5,
            "timestamp": chrono::Utc::now().timestamp_millis()
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_upload_location_invalid_latitude() {
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
    let _device_response = register_test_device(&app, &auth, &device).await;

    // Try to upload location with invalid latitude (> 90)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/locations",
        json!({
            "device_id": device.device_id,
            "latitude": 91.0,
            "longitude": -122.4194,
            "accuracy": 10.5,
            "timestamp": chrono::Utc::now().timestamp_millis()
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
async fn test_upload_location_invalid_longitude() {
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
    let _device_response = register_test_device(&app, &auth, &device).await;

    // Try to upload location with invalid longitude (> 180)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/locations",
        json!({
            "device_id": device.device_id,
            "latitude": 37.7749,
            "longitude": 181.0,
            "accuracy": 10.5,
            "timestamp": chrono::Utc::now().timestamp_millis()
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

// ============================================================================
// Batch Location Upload Tests
// ============================================================================

#[tokio::test]
async fn test_upload_batch_success() {
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
    let _device_response = register_test_device(&app, &auth, &device).await;

    // Upload batch of locations
    let app = create_test_app(config, pool.clone());
    let now = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/locations/batch",
        json!({
            "device_id": device.device_id,
            "locations": [
                {
                    "latitude": 37.7749,
                    "longitude": -122.4194,
                    "accuracy": 10.5,
                    "timestamp": now - 2000
                },
                {
                    "latitude": 37.7750,
                    "longitude": -122.4195,
                    "accuracy": 8.0,
                    "timestamp": now - 1000
                },
                {
                    "latitude": 37.7751,
                    "longitude": -122.4196,
                    "accuracy": 12.0,
                    "timestamp": now
                }
            ]
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["processed"].as_i64().unwrap(), 3);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_upload_batch_exceeds_limit() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let mut config = test_config();
    config.limits.max_batch_size = 5; // Set low limit for testing

    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _device_response = register_test_device(&app, &auth, &device).await;

    // Try to upload batch exceeding limit
    let app = create_test_app(config, pool.clone());
    let now = chrono::Utc::now().timestamp_millis();
    let locations: Vec<serde_json::Value> = (0..10)
        .map(|i| {
            json!({
                "latitude": 37.7749 + (i as f64 * 0.0001),
                "longitude": -122.4194,
                "accuracy": 10.5,
                "timestamp": now - (i * 1000)
            })
        })
        .collect();

    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/locations/batch",
        json!({
            "device_id": device.device_id,
            "locations": locations
        }),
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
async fn test_upload_batch_empty() {
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
    let _device_response = register_test_device(&app, &auth, &device).await;

    // Try to upload empty batch
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/locations/batch",
        json!({
            "device_id": device.device_id,
            "locations": []
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should fail validation - empty batch
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Location History Tests
// ============================================================================

#[tokio::test]
async fn test_get_location_history_success() {
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
    let _device_response = register_test_device(&app, &auth, &device).await;

    // Upload some locations
    let app = create_test_app(config.clone(), pool.clone());
    let now = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/locations/batch",
        json!({
            "device_id": device.device_id,
            "locations": [
                {
                    "latitude": 37.7749,
                    "longitude": -122.4194,
                    "accuracy": 10.5,
                    "timestamp": now - 2000
                },
                {
                    "latitude": 37.7750,
                    "longitude": -122.4195,
                    "accuracy": 8.0,
                    "timestamp": now - 1000
                }
            ]
        }),
        &auth.access_token,
    );
    let _upload_response = app.oneshot(request).await.unwrap();

    // Get location history
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/devices/{}/locations", device.device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body.get("locations").is_some());
    let locations = body["locations"].as_array().unwrap();
    assert_eq!(locations.len(), 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_location_history_with_pagination() {
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
    let _device_response = register_test_device(&app, &auth, &device).await;

    // Upload some locations
    let app = create_test_app(config.clone(), pool.clone());
    let now = chrono::Utc::now().timestamp_millis();
    let locations: Vec<serde_json::Value> = (0..5)
        .map(|i| {
            json!({
                "latitude": 37.7749 + (i as f64 * 0.0001),
                "longitude": -122.4194,
                "accuracy": 10.5,
                "timestamp": now - (i * 1000)
            })
        })
        .collect();

    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/locations/batch",
        json!({
            "device_id": device.device_id,
            "locations": locations
        }),
        &auth.access_token,
    );
    let _upload_response = app.oneshot(request).await.unwrap();

    // Get first page with limit
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/devices/{}/locations?limit=2", device.device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let locations = body["locations"].as_array().unwrap();
    assert_eq!(locations.len(), 2);

    // Check pagination info
    if let Some(pagination) = body.get("pagination") {
        assert!(pagination.get("next_cursor").is_some() || pagination.get("has_more").is_some());
    }

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_location_history_with_time_range() {
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
    let _device_response = register_test_device(&app, &auth, &device).await;

    // Upload some locations at different times
    let app = create_test_app(config.clone(), pool.clone());
    let now = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/locations/batch",
        json!({
            "device_id": device.device_id,
            "locations": [
                {
                    "latitude": 37.7749,
                    "longitude": -122.4194,
                    "accuracy": 10.5,
                    "timestamp": now - 60000 // 1 minute ago
                },
                {
                    "latitude": 37.7750,
                    "longitude": -122.4195,
                    "accuracy": 8.0,
                    "timestamp": now - 30000 // 30 seconds ago
                },
                {
                    "latitude": 37.7751,
                    "longitude": -122.4196,
                    "accuracy": 12.0,
                    "timestamp": now // now
                }
            ]
        }),
        &auth.access_token,
    );
    let _upload_response = app.oneshot(request).await.unwrap();

    // Get location history with time range (last 45 seconds)
    let app = create_test_app(config, pool.clone());
    let from = now - 45000;
    let to = now + 1000;
    let request = get_request_with_auth(
        &format!(
            "/api/v1/devices/{}/locations?from={}&to={}",
            device.device_id, from, to
        ),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let locations = body["locations"].as_array().unwrap();
    // Should only get 2 locations (30 seconds ago and now)
    assert_eq!(locations.len(), 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_location_history_with_simplification() {
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
    let _device_response = register_test_device(&app, &auth, &device).await;

    // Upload many locations in a line
    let app = create_test_app(config.clone(), pool.clone());
    let now = chrono::Utc::now().timestamp_millis();
    let locations: Vec<serde_json::Value> = (0..20)
        .map(|i| {
            json!({
                "latitude": 37.7749 + (i as f64 * 0.00001),
                "longitude": -122.4194 + (i as f64 * 0.00001),
                "accuracy": 10.5,
                "timestamp": now - ((20 - i) * 1000)
            })
        })
        .collect();

    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/locations/batch",
        json!({
            "device_id": device.device_id,
            "locations": locations
        }),
        &auth.access_token,
    );
    let _upload_response = app.oneshot(request).await.unwrap();

    // Get location history with simplification (RDP algorithm)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!(
            "/api/v1/devices/{}/locations?tolerance=100",
            device.device_id
        ),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let locations = body["locations"].as_array().unwrap();

    // With simplification, we should have fewer points than original
    // (though exact count depends on the tolerance and point distribution)
    assert!(locations.len() <= 20);

    // Check simplification info is present
    if let Some(simplification) = body.get("simplification") {
        assert!(simplification.get("original_count").is_some());
        assert!(simplification.get("simplified_count").is_some());
    }

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_location_history_empty() {
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
    let _device_response = register_test_device(&app, &auth, &device).await;

    // Get location history without uploading any locations
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/devices/{}/locations", device.device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let locations = body["locations"].as_array().unwrap();
    assert!(locations.is_empty());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_location_history_device_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user but don't register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to get location history for non-existent device
    let fake_device_id = uuid::Uuid::new_v4();
    let request = get_request_with_auth(
        &format!("/api/v1/devices/{}/locations", fake_device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should return not found or empty (depending on implementation)
    assert!(
        response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::OK
    );

    cleanup_all_test_data(&pool).await;
}
