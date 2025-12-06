//! Integration tests for trip management endpoints.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test trips_integration

mod common;

use axum::http::{Method, StatusCode};
use chrono::Utc;
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_api_key, create_test_app,
    create_test_pool, get_request_with_api_key_and_jwt, json_request_with_api_key_and_jwt,
    parse_response_body, register_test_device, run_migrations, test_config, TestDevice, TestUser,
};
use serde_json::json;
use tower::ServiceExt;

fn current_timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

// ============================================================================
// Trip Creation Tests
// ============================================================================

#[tokio::test]
async fn test_create_trip_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_create_trip_success").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a trip
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/trips",
        json!({
            "device_id": device_id,
            "local_trip_id": "morning-commute-001",
            "start_timestamp": current_timestamp_millis(),
            "start_latitude": 37.7749,
            "start_longitude": -122.4194,
            "transportation_mode": "WALKING",
            "detection_source": "ACTIVITY_RECOGNITION"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body.get("id").is_some());
    assert_eq!(body["local_trip_id"], "morning-commute-001");
    assert_eq!(body["state"], "ACTIVE");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_trip_idempotency() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_create_trip_idempotency").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    let idempotency_key = uuid::Uuid::new_v4().to_string();
    let start_timestamp = current_timestamp_millis();

    // Create first trip
    let app = create_test_app(config.clone(), pool.clone());
    let request = axum::http::Request::builder()
        .method(Method::POST)
        .uri("/api/v1/trips")
        .header("content-type", "application/json")
        .header("x-api-key", &api_key)
        .header("authorization", format!("Bearer {}", auth.access_token))
        .header("x-idempotency-key", &idempotency_key)
        .body(axum::body::Body::from(
            serde_json::to_string(&json!({
                "device_id": device_id,
                "local_trip_id": "idempotent-trip-001",
                "start_timestamp": start_timestamp,
                "start_latitude": 37.7749,
                "start_longitude": -122.4194,
                "transportation_mode": "WALKING",
                "detection_source": "ACTIVITY_RECOGNITION"
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let first_body = parse_response_body(response).await;
    let first_trip_id = first_body["id"].as_str().unwrap();

    // Try to create again with same idempotency key
    let app = create_test_app(config, pool.clone());
    let request = axum::http::Request::builder()
        .method(Method::POST)
        .uri("/api/v1/trips")
        .header("content-type", "application/json")
        .header("x-api-key", &api_key)
        .header("authorization", format!("Bearer {}", auth.access_token))
        .header("x-idempotency-key", &idempotency_key)
        .body(axum::body::Body::from(
            serde_json::to_string(&json!({
                "device_id": device_id,
                "local_trip_id": "idempotent-trip-001",
                "start_timestamp": start_timestamp,
                "start_latitude": 37.7749,
                "start_longitude": -122.4194,
                "transportation_mode": "WALKING",
                "detection_source": "ACTIVITY_RECOGNITION"
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Should return same trip (cached response)
    let second_body = parse_response_body(response).await;
    let second_trip_id = second_body["id"].as_str().unwrap();
    assert_eq!(first_trip_id, second_trip_id);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_trip_device_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user but don't register device
    let api_key = create_test_api_key(&pool, "test_create_trip_device_not_found").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to create trip for non-existent device
    let app = create_test_app(config, pool.clone());
    let fake_device_id = uuid::Uuid::new_v4();
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/trips",
        json!({
            "device_id": fake_device_id.to_string(),
            "local_trip_id": "ghost-trip-001",
            "start_timestamp": current_timestamp_millis(),
            "start_latitude": 37.7749,
            "start_longitude": -122.4194,
            "transportation_mode": "WALKING",
            "detection_source": "ACTIVITY_RECOGNITION"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Trip State Update Tests
// ============================================================================

#[tokio::test]
async fn test_complete_trip_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_complete_trip_success").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a trip
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/trips",
        json!({
            "device_id": device_id,
            "local_trip_id": "to-complete-001",
            "start_timestamp": current_timestamp_millis(),
            "start_latitude": 37.7749,
            "start_longitude": -122.4194,
            "transportation_mode": "WALKING",
            "detection_source": "ACTIVITY_RECOGNITION"
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let trip_id = create_body["id"].as_str().unwrap();

    // Complete the trip
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PATCH,
        &format!("/api/v1/trips/{}", trip_id),
        json!({
            "state": "COMPLETED",
            "end_timestamp": current_timestamp_millis(),
            "end_latitude": 37.7849,
            "end_longitude": -122.4094
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["state"], "COMPLETED");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_cancel_trip_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_cancel_trip_success").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a trip
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/trips",
        json!({
            "device_id": device_id,
            "local_trip_id": "to-cancel-001",
            "start_timestamp": current_timestamp_millis(),
            "start_latitude": 37.7749,
            "start_longitude": -122.4194,
            "transportation_mode": "WALKING",
            "detection_source": "ACTIVITY_RECOGNITION"
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let trip_id = create_body["id"].as_str().unwrap();

    // Cancel the trip
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PATCH,
        &format!("/api/v1/trips/{}", trip_id),
        json!({
            "state": "CANCELLED"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["state"], "CANCELLED");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_invalid_state_transition() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_invalid_state_transition").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a trip
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/trips",
        json!({
            "device_id": device_id,
            "local_trip_id": "state-machine-test-001",
            "start_timestamp": current_timestamp_millis(),
            "start_latitude": 37.7749,
            "start_longitude": -122.4194,
            "transportation_mode": "WALKING",
            "detection_source": "ACTIVITY_RECOGNITION"
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let trip_id = create_body["id"].as_str().unwrap();

    // Complete the trip first
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PATCH,
        &format!("/api/v1/trips/{}", trip_id),
        json!({
            "state": "COMPLETED",
            "end_timestamp": current_timestamp_millis(),
            "end_latitude": 37.7849,
            "end_longitude": -122.4094
        }),
        &api_key,
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // Try to cancel completed trip (invalid transition)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::PATCH,
        &format!("/api/v1/trips/{}", trip_id),
        json!({
            "state": "CANCELLED"
        }),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should fail - can't cancel completed trip
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::CONFLICT
    );

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Get Device Trips Tests
// ============================================================================

#[tokio::test]
async fn test_get_device_trips_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_get_device_trips_success").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create multiple trips (must complete each before creating next - only one active trip allowed)
    let base_timestamp = current_timestamp_millis();
    for i in 1..=3 {
        // Create trip
        let app = create_test_app(config.clone(), pool.clone());
        let request = json_request_with_api_key_and_jwt(
            Method::POST,
            "/api/v1/trips",
            json!({
                "device_id": device_id,
                "local_trip_id": format!("success-trip-{:03}", i),
                "start_timestamp": base_timestamp + (i as i64 * 1000),
                "start_latitude": 37.7749 + (i as f64 * 0.01),
                "start_longitude": -122.4194,
                "transportation_mode": "WALKING",
                "detection_source": "ACTIVITY_RECOGNITION"
            }),
            &api_key,
            &auth.access_token,
        );
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED, "Trip {} creation failed", i);
        let body = parse_response_body(response).await;
        let trip_id = body["id"].as_str().unwrap();

        // Complete the trip so we can create another
        // Use current time as end_timestamp (must be within 5 min of now for validation)
        let app = create_test_app(config.clone(), pool.clone());
        let request = json_request_with_api_key_and_jwt(
            Method::PATCH,
            &format!("/api/v1/trips/{}", trip_id),
            json!({
                "state": "COMPLETED",
                "end_timestamp": current_timestamp_millis(),
                "end_latitude": 37.7849 + (i as f64 * 0.01),
                "end_longitude": -122.4094
            }),
            &api_key,
            &auth.access_token,
        );
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "Trip {} completion failed", i);
    }

    // Get device trips
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/trips", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let trips = body["trips"].as_array().unwrap();
    assert_eq!(trips.len(), 3);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_device_trips_with_pagination() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_get_device_trips_with_pagination").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create 5 trips (must complete each before creating next - only one active trip allowed)
    let base_timestamp = current_timestamp_millis();
    for i in 1..=5 {
        // Create trip
        let app = create_test_app(config.clone(), pool.clone());
        let request = json_request_with_api_key_and_jwt(
            Method::POST,
            "/api/v1/trips",
            json!({
                "device_id": device_id,
                "local_trip_id": format!("paginated-trip-{:03}", i),
                "start_timestamp": base_timestamp + (i as i64 * 1000),
                "start_latitude": 37.7749,
                "start_longitude": -122.4194,
                "transportation_mode": "WALKING",
                "detection_source": "ACTIVITY_RECOGNITION"
            }),
            &api_key,
            &auth.access_token,
        );
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED, "Trip {} creation failed", i);
        let body = parse_response_body(response).await;
        let trip_id = body["id"].as_str().unwrap();

        // Complete the trip so we can create another
        // Use current time as end_timestamp (must be within 5 min of now for validation)
        let app = create_test_app(config.clone(), pool.clone());
        let request = json_request_with_api_key_and_jwt(
            Method::PATCH,
            &format!("/api/v1/trips/{}", trip_id),
            json!({
                "state": "COMPLETED",
                "end_timestamp": current_timestamp_millis(),
                "end_latitude": 37.7849,
                "end_longitude": -122.4094
            }),
            &api_key,
            &auth.access_token,
        );
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "Trip {} completion failed", i);
    }

    // Get first page
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/trips?limit=2", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let trips = body["trips"].as_array().unwrap();
    assert_eq!(trips.len(), 2);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Trip Movement Events Tests
// ============================================================================

#[tokio::test]
async fn test_get_trip_movement_events_empty() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_get_trip_movement_events_empty").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a trip
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/trips",
        json!({
            "device_id": device_id,
            "local_trip_id": "empty-trip-001",
            "start_timestamp": current_timestamp_millis(),
            "start_latitude": 37.7749,
            "start_longitude": -122.4194,
            "transportation_mode": "WALKING",
            "detection_source": "ACTIVITY_RECOGNITION"
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let trip_id = create_body["id"].as_str().unwrap();

    // Get movement events (none exist)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/trips/{}/movement-events", trip_id),
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

// ============================================================================
// Trip Path Tests
// ============================================================================

#[tokio::test]
async fn test_get_trip_path_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_get_trip_path_success").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Create a trip
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key_and_jwt(
        Method::POST,
        "/api/v1/trips",
        json!({
            "device_id": device_id,
            "local_trip_id": "path-test-trip-001",
            "start_timestamp": current_timestamp_millis(),
            "start_latitude": 37.7749,
            "start_longitude": -122.4194,
            "transportation_mode": "WALKING",
            "detection_source": "ACTIVITY_RECOGNITION"
        }),
        &api_key,
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let trip_id = create_body["id"].as_str().unwrap();

    // Get trip path - expect 404 since no path correction record exists for an active trip
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/trips/{}/path", trip_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Path correction record doesn't exist for active trips, so 404 is expected
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_trip_path_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user
    let api_key = create_test_api_key(&pool, "test_get_trip_path_not_found").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to get path for non-existent trip
    let app = create_test_app(config, pool.clone());
    let fake_trip_id = uuid::Uuid::new_v4();
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/trips/{}/path", fake_trip_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}
