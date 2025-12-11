//! Integration tests for movement event endpoints.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test movement_events_integration

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_api_key, create_test_app,
    create_test_pool, get_request_with_api_key_and_jwt, json_request_with_api_key_and_jwt,
    parse_response_body, register_test_device, run_migrations, test_config, TestDevice, TestUser,
};
use serde_json::json;
use tower::ServiceExt;

// ============================================================================
// Movement Event Creation Tests
// ============================================================================

#[tokio::test]
async fn test_create_movement_event_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user
    let api_key = create_test_api_key(&pool, "test_create_movement_event_success").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a movement event
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/movement-events",
        json!({
            "device_id": device_id,
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "latitude": 37.7749,
            "longitude": -122.4194,
            "accuracy": 10.0,
            "transportation_mode": "WALKING",
            "confidence": 0.85,
            "detection_source": "ACTIVITY_RECOGNITION"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body.get("id").is_some());
    assert!(body.get("created_at").is_some());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_movement_event_device_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user but don't register device
    let api_key = create_test_api_key(&pool, "test_create_movement_event_device_not_found").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to create event for non-existent device
    let app = create_test_app(config, pool.clone());
    let fake_device_id = uuid::Uuid::new_v4();
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/movement-events",
        json!({
            "device_id": fake_device_id.to_string(),
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "latitude": 37.7749,
            "longitude": -122.4194,
            "accuracy": 10.0,
            "transportation_mode": "WALKING",
            "confidence": 0.85,
            "detection_source": "ACTIVITY_RECOGNITION"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_movement_event_invalid_event_type() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_create_movement_event_invalid_event_type").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create event with invalid transportation mode
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/movement-events",
        json!({
            "device_id": device_id,
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "latitude": 37.7749,
            "longitude": -122.4194,
            "accuracy": 10.0,
            "transportation_mode": "INVALID_MODE",
            "confidence": 0.85,
            "detection_source": "ACTIVITY_RECOGNITION"
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

// ============================================================================
// Batch Movement Event Creation Tests
// ============================================================================

#[tokio::test]
async fn test_create_movement_events_batch_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_create_movement_events_batch_success").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create batch of events
    let app = create_test_app(config, pool.clone());
    let now = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/movement-events/batch",
        json!({
            "device_id": device_id,
            "events": [
                {
                    "timestamp": now - 3000,
                    "latitude": 37.7749,
                    "longitude": -122.4194,
                    "accuracy": 10.0,
                    "transportation_mode": "WALKING",
                    "confidence": 0.85,
                    "detection_source": "ACTIVITY_RECOGNITION"
                },
                {
                    "timestamp": now - 2000,
                    "latitude": 37.7759,
                    "longitude": -122.4184,
                    "accuracy": 10.0,
                    "transportation_mode": "STATIONARY",
                    "confidence": 0.9,
                    "detection_source": "ACTIVITY_RECOGNITION"
                },
                {
                    "timestamp": now - 1000,
                    "latitude": 37.7769,
                    "longitude": -122.4174,
                    "accuracy": 10.0,
                    "transportation_mode": "WALKING",
                    "confidence": 0.85,
                    "detection_source": "ACTIVITY_RECOGNITION"
                }
            ]
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["processed_count"].as_i64().unwrap(), 3);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_movement_events_batch_empty() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_create_movement_events_batch_empty").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Try to create empty batch
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/movement-events/batch",
        json!({
            "device_id": device_id,
            "events": []
        }),
        &api_key,
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
// Get Device Movement Events Tests
// ============================================================================

#[tokio::test]
async fn test_get_device_movement_events_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_get_device_movement_events_success").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create some events
    let app = create_test_app(config.clone(), pool.clone());
    let now = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/movement-events/batch",
        json!({
            "device_id": device_id,
            "events": [
                {
                    "timestamp": now - 2000,
                    "latitude": 37.7749,
                    "longitude": -122.4194,
                    "accuracy": 10.0,
                    "transportation_mode": "WALKING",
                    "confidence": 0.85,
                    "detection_source": "ACTIVITY_RECOGNITION"
                },
                {
                    "timestamp": now - 1000,
                    "latitude": 37.7759,
                    "longitude": -122.4184,
                    "accuracy": 10.0,
                    "transportation_mode": "STATIONARY",
                    "confidence": 0.9,
                    "detection_source": "ACTIVITY_RECOGNITION"
                }
            ]
        }),
        &api_key,
        &auth.access_token,
    );
    let _upload_response = app.oneshot(request).await.unwrap();

    // Get device movement events
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/movement-events", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let events = body["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_device_movement_events_empty() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_get_device_movement_events_empty").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Get movement events (none exist)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/movement-events", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let events = body["events"].as_array().unwrap();
    assert!(events.is_empty());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_device_movement_events_with_pagination() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key =
        create_test_api_key(&pool, "test_get_device_movement_events_with_pagination").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create many events
    let app = create_test_app(config.clone(), pool.clone());
    let now = chrono::Utc::now().timestamp_millis();
    let events: Vec<serde_json::Value> = (0..10)
        .map(|i| {
            json!({
                "timestamp": now - ((10 - i) * 1000),
                "latitude": 37.7749 + (i as f64 * 0.001),
                "longitude": -122.4194,
                "accuracy": 10.0,
                "transportation_mode": if i % 2 == 0 { "WALKING" } else { "STATIONARY" },
                "confidence": 0.85,
                "detection_source": "ACTIVITY_RECOGNITION"
            })
        })
        .collect();

    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/movement-events/batch",
        json!({
            "device_id": device_id,
            "events": events
        }),
        &api_key,
        &auth.access_token,
    );
    let _upload_response = app.oneshot(request).await.unwrap();

    // Get first page
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/movement-events?limit=5", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let events = body["events"].as_array().unwrap();
    assert_eq!(events.len(), 5);

    cleanup_all_test_data(&pool).await;
}
