//! Integration tests for privacy/GDPR endpoints.
//!
//! Tests data export and deletion functionality required for GDPR compliance.

mod common;

use axum::http::StatusCode;
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_api_key, create_test_app,
    create_test_pool, delete_request_with_api_key_and_jwt, get_request_with_api_key_and_jwt,
    json_request_with_api_key_and_jwt, parse_response_body, register_test_device, run_migrations,
    test_config, TestDevice, TestUser,
};
use serde_json::json;
use tower::ServiceExt;

// ============================================================================
// Data Export Tests (GDPR Article 20 - Right to Data Portability)
// ============================================================================

#[tokio::test]
async fn test_export_device_data_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_export_device_data_success").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Upload some locations
    let app = create_test_app(config.clone(), pool.clone());
    let now = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_api_key_and_jwt(
        axum::http::Method::POST,
        "/api/v1/locations/batch",
        json!({
            "device_id": device_id,
            "locations": [
                {
                    "latitude": 37.7749,
                    "longitude": -122.4194,
                    "accuracy": 10.0,
                    "timestamp": now - 2000
                },
                {
                    "latitude": 37.7759,
                    "longitude": -122.4184,
                    "accuracy": 15.0,
                    "timestamp": now - 1000
                }
            ]
        }),
        &api_key,
        &auth.access_token,
    );
    let upload_response = app.oneshot(request).await.unwrap();
    assert_eq!(upload_response.status(), StatusCode::OK);

    // Export device data
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/data-export", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body.get("device").is_some());
    assert!(body.get("locations").is_some());
    assert_eq!(body["location_count"].as_i64().unwrap(), 2);
    assert!(body.get("export_timestamp").is_some());

    // Verify device data
    let device_data = &body["device"];
    assert_eq!(device_data["display_name"], device.display_name);
    assert_eq!(device_data["platform"], device.platform);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_export_device_data_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user but don't register a device
    let api_key = create_test_api_key(&pool, "test_export_device_data_not_found").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to export non-existent device
    let app = create_test_app(config, pool.clone());
    let fake_device_id = uuid::Uuid::new_v4();
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/data-export", fake_device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_export_device_data_empty_locations() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device (no locations)
    let api_key = create_test_api_key(&pool, "test_export_device_data_empty_locations").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Export device data (should have empty locations)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/data-export", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["location_count"].as_i64().unwrap(), 0);
    assert!(body["locations"].as_array().unwrap().is_empty());

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Data Deletion Tests (GDPR Article 17 - Right to Erasure)
// ============================================================================

#[tokio::test]
async fn test_delete_device_data_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_delete_device_data_success").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Upload some locations
    let app = create_test_app(config.clone(), pool.clone());
    let now = chrono::Utc::now().timestamp_millis();
    let request = json_request_with_api_key_and_jwt(
        axum::http::Method::POST,
        "/api/v1/locations/batch",
        json!({
            "device_id": device_id,
            "locations": [
                {
                    "latitude": 37.7749,
                    "longitude": -122.4194,
                    "accuracy": 10.0,
                    "timestamp": now
                }
            ]
        }),
        &api_key,
        &auth.access_token,
    );
    let upload_response = app.oneshot(request).await.unwrap();
    assert_eq!(upload_response.status(), StatusCode::OK);

    // Delete device data (hard delete)
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/data", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify device is completely gone
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/data-export", device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_device_data_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user
    let api_key = create_test_api_key(&pool, "test_delete_device_data_not_found").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to delete non-existent device
    let app = create_test_app(config, pool.clone());
    let fake_device_id = uuid::Uuid::new_v4();
    let request = delete_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/data", fake_device_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_device_data_idempotent() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_delete_device_data_idempotent").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Delete device data
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/data", device_id),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Second deletion should return not found (data already deleted)
    let app = create_test_app(config, pool.clone());
    let request = delete_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/data", device_id),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_device_data_cascades_locations() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create API key and authenticated user and register device
    let api_key = create_test_api_key(&pool, "test_delete_device_data_cascades_locations").await;
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Upload many locations
    let app = create_test_app(config.clone(), pool.clone());
    let now = chrono::Utc::now().timestamp_millis();
    let locations: Vec<_> = (0..10)
        .map(|i| {
            json!({
                "latitude": 37.7749 + (i as f64 * 0.001),
                "longitude": -122.4194,
                "accuracy": 10.0,
                "timestamp": now - (i * 1000)
            })
        })
        .collect();

    let request = json_request_with_api_key_and_jwt(
        axum::http::Method::POST,
        "/api/v1/locations/batch",
        json!({
            "device_id": device_id,
            "locations": locations
        }),
        &api_key,
        &auth.access_token,
    );
    let upload_response = app.oneshot(request).await.unwrap();
    assert_eq!(upload_response.status(), StatusCode::OK);

    // Verify locations exist
    let app = create_test_app(config.clone(), pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/data-export", device_id),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;
    assert_eq!(body["location_count"].as_i64().unwrap(), 10);

    // Delete device data
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/data", device_id),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify device and all locations are gone
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/v1/devices/{}/data-export", device_id),
        &api_key,
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}
