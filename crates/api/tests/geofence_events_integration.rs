//! Integration tests for geofence event endpoints.
//!
//! Story 15.2: Webhook Event Delivery
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test geofence_events_integration

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_app, create_test_pool,
    get_request_with_auth, json_request_with_auth, parse_response_body,
    register_test_device, run_migrations, test_config, TestDevice, TestUser,
};
use serde_json::json;
use tower::ServiceExt;

/// Helper to create a geofence for testing.
async fn create_test_geofence(
    _app: &axum::Router,
    pool: &sqlx::PgPool,
    config: &phone_manager_api::config::Config,
    auth: &common::AuthenticatedUser,
    device_id: &str,
    name: &str,
) -> String {
    let app = common::create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofences",
        json!({
            "device_id": device_id,
            "name": name,
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius": 100.0,
            "event_types": ["enter", "exit", "dwell"]
        }),
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;
    body["id"].as_str().unwrap().to_string()
}

// ============================================================================
// Geofence Event Creation Tests (AC 15.2.2)
// ============================================================================

#[tokio::test]
async fn test_create_geofence_event_success() {
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
    let geofence_id = create_test_geofence(&app, &pool, &config, &auth, device_id, "Home").await;

    // Create a geofence event
    let app = create_test_app(config, pool.clone());
    let timestamp = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofence-events",
        json!({
            "device_id": device_id,
            "geofence_id": geofence_id,
            "event_type": "enter",
            "timestamp": timestamp.to_string(),
            "latitude": 37.7749,
            "longitude": -122.4194
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body.get("event_id").is_some());
    assert_eq!(body["device_id"], device_id);
    assert_eq!(body["geofence_id"], geofence_id);
    assert_eq!(body["event_type"], "enter");
    assert!(body.get("webhook_delivered").is_some());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_geofence_event_exit_type() {
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
    let geofence_id = create_test_geofence(&app, &pool, &config, &auth, device_id, "Work").await;

    // Create an exit event
    let app = create_test_app(config, pool.clone());
    let timestamp = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofence-events",
        json!({
            "device_id": device_id,
            "geofence_id": geofence_id,
            "event_type": "exit",
            "timestamp": timestamp.to_string(),
            "latitude": 37.7849,
            "longitude": -122.4094
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert_eq!(body["event_type"], "exit");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_geofence_event_dwell_type() {
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
    let geofence_id = create_test_geofence(&app, &pool, &config, &auth, device_id, "Office").await;

    // Create a dwell event
    let app = create_test_app(config, pool.clone());
    let timestamp = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofence-events",
        json!({
            "device_id": device_id,
            "geofence_id": geofence_id,
            "event_type": "dwell",
            "timestamp": timestamp.to_string(),
            "latitude": 37.7749,
            "longitude": -122.4194
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert_eq!(body["event_type"], "dwell");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_geofence_event_invalid_latitude() {
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
    let geofence_id = create_test_geofence(&app, &pool, &config, &auth, device_id, "Test").await;

    // Try to create event with invalid latitude (> 90)
    let app = create_test_app(config, pool.clone());
    let timestamp = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofence-events",
        json!({
            "device_id": device_id,
            "geofence_id": geofence_id,
            "event_type": "enter",
            "timestamp": timestamp.to_string(),
            "latitude": 100.0,
            "longitude": -122.4194
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
async fn test_create_geofence_event_invalid_longitude() {
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
    let geofence_id = create_test_geofence(&app, &pool, &config, &auth, device_id, "Test").await;

    // Try to create event with invalid longitude (> 180)
    let app = create_test_app(config, pool.clone());
    let timestamp = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofence-events",
        json!({
            "device_id": device_id,
            "geofence_id": geofence_id,
            "event_type": "enter",
            "timestamp": timestamp.to_string(),
            "latitude": 37.7749,
            "longitude": 200.0
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
async fn test_create_geofence_event_device_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user but don't register device
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to create event for non-existent device
    let fake_device_id = uuid::Uuid::new_v4();
    let fake_geofence_id = uuid::Uuid::new_v4();
    let timestamp = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofence-events",
        json!({
            "device_id": fake_device_id.to_string(),
            "geofence_id": fake_geofence_id.to_string(),
            "event_type": "enter",
            "timestamp": timestamp.to_string(),
            "latitude": 37.7749,
            "longitude": -122.4194
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_geofence_event_geofence_not_found() {
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

    // Try to create event for non-existent geofence
    let app = create_test_app(config, pool.clone());
    let fake_geofence_id = uuid::Uuid::new_v4();
    let timestamp = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofence-events",
        json!({
            "device_id": device_id,
            "geofence_id": fake_geofence_id.to_string(),
            "event_type": "enter",
            "timestamp": timestamp.to_string(),
            "latitude": 37.7749,
            "longitude": -122.4194
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Geofence Event Listing Tests (AC 15.2.3)
// ============================================================================

#[tokio::test]
async fn test_list_geofence_events_success() {
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
    let geofence_id = create_test_geofence(&app, &pool, &config, &auth, device_id, "Home").await;

    // Create two events
    let timestamp1 = chrono::Utc::now().timestamp_millis();
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofence-events",
        json!({
            "device_id": device_id,
            "geofence_id": geofence_id,
            "event_type": "enter",
            "timestamp": timestamp1.to_string(),
            "latitude": 37.7749,
            "longitude": -122.4194
        }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    let timestamp2 = chrono::Utc::now().timestamp_millis() + 1000;
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofence-events",
        json!({
            "device_id": device_id,
            "geofence_id": geofence_id,
            "event_type": "exit",
            "timestamp": timestamp2.to_string(),
            "latitude": 37.7849,
            "longitude": -122.4094
        }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // List events for device
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/geofence-events?deviceId={}", device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let events = body["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(body["total"], 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_geofence_events_empty() {
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

    // List events for device (none exist)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/geofence-events?deviceId={}", device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let events = body["events"].as_array().unwrap();
    assert!(events.is_empty());
    assert_eq!(body["total"], 0);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_geofence_events_with_limit() {
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
    let geofence_id = create_test_geofence(&app, &pool, &config, &auth, device_id, "Home").await;

    // Create three events
    for i in 0..3 {
        let timestamp = chrono::Utc::now().timestamp_millis() + (i * 1000);
        let app = create_test_app(config.clone(), pool.clone());
        let request = json_request_with_auth(
            Method::POST,
            "/api/v1/geofence-events",
            json!({
                "device_id": device_id,
                "geofence_id": geofence_id,
                "event_type": "enter",
                "timestamp": timestamp.to_string(),
                "latitude": 37.7749,
                "longitude": -122.4194
            }),
            &auth.access_token,
        );
        let _response = app.oneshot(request).await.unwrap();
    }

    // List events with limit=2
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/geofence-events?deviceId={}&limit=2", device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let events = body["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(body["total"], 3); // Total is 3, but only 2 returned

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Get Single Geofence Event Tests (AC 15.2.4)
// ============================================================================

#[tokio::test]
async fn test_get_geofence_event_success() {
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
    let geofence_id = create_test_geofence(&app, &pool, &config, &auth, device_id, "Home").await;

    // Create an event
    let app = create_test_app(config.clone(), pool.clone());
    let timestamp = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofence-events",
        json!({
            "device_id": device_id,
            "geofence_id": geofence_id,
            "event_type": "enter",
            "timestamp": timestamp.to_string(),
            "latitude": 37.7749,
            "longitude": -122.4194
        }),
        &auth.access_token,
    );
    let create_response = app.oneshot(request).await.unwrap();
    let create_body = parse_response_body(create_response).await;
    let event_id = create_body["event_id"].as_str().unwrap();

    // Get the event
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/geofence-events/{}", event_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["event_id"], event_id);
    assert_eq!(body["device_id"], device_id);
    assert_eq!(body["event_type"], "enter");
    assert!(body.get("webhook_delivered").is_some());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_geofence_event_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to get non-existent event
    let fake_event_id = uuid::Uuid::new_v4();
    let request = get_request_with_auth(
        &format!("/api/v1/geofence-events/{}", fake_event_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Geofence Event with Webhook Status Tests (AC 15.2.5, 15.2.6)
// ============================================================================

#[tokio::test]
async fn test_geofence_event_includes_webhook_status() {
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
    let geofence_id = create_test_geofence(&app, &pool, &config, &auth, device_id, "Home").await;

    // Create an event (no webhooks configured, so webhook_delivered should be false)
    let app = create_test_app(config, pool.clone());
    let timestamp = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/geofence-events",
        json!({
            "device_id": device_id,
            "geofence_id": geofence_id,
            "event_type": "enter",
            "timestamp": timestamp.to_string(),
            "latitude": 37.7749,
            "longitude": -122.4194
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    // Webhook status should be included in response
    assert!(body.get("webhook_delivered").is_some());
    // Since no webhooks are configured, it should be false initially
    // (Note: async delivery might not have completed yet, so we just check the field exists)

    cleanup_all_test_data(&pool).await;
}
