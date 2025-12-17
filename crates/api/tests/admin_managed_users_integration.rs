//! Integration tests for Admin Managed Users endpoints (Epic 9).
//!
//! Tests the following functionality:
//! - List managed users (org admin vs non-org admin)
//! - Get user location
//! - User geofences CRUD
//! - Update tracking status
//! - Authorization boundaries

mod common;

use axum::http::StatusCode;
use common::{
    add_user_to_organization, cleanup_all_test_data, create_authenticated_user,
    create_test_admin_api_key, create_test_app, create_test_organization, create_test_pool,
    delete_admin_request_with_jwt, get_admin_request_with_jwt, insert_device_location,
    parse_response_body, post_admin_request_with_jwt, put_admin_request_with_jwt,
    register_test_device, run_migrations, test_config, TestDevice, TestOrganization, TestUser,
};
use serde_json::json;
use tower::ServiceExt;

// ============================================================================
// List Managed Users Tests
// ============================================================================

#[tokio::test]
async fn test_list_managed_users_as_org_admin() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin user (the one who will make requests)
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;

    // Create a target user (the one to be managed)
    let app = create_test_app(config.clone(), pool.clone());
    let target_user = TestUser::new();
    let _target_auth = create_authenticated_user(&app, &target_user).await;

    // Add admin as org admin
    add_user_to_organization(&pool, &admin_auth.user_id, &org.id, "admin").await;

    // Add target user to the org as member
    add_user_to_organization(&pool, &_target_auth.user_id, &org.id, "member").await;

    // List managed users
    let app = create_test_app(config, pool.clone());
    let request =
        get_admin_request_with_jwt("/api/admin/v1/users", &admin_api_key, &admin_auth.access_token);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;

    assert!(body.get("users").is_some());
    assert!(body.get("pagination").is_some());

    let users = body["users"].as_array().unwrap();
    // Should find the target user (admin themselves may or may not be included depending on implementation)
    assert!(!users.is_empty());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_managed_users_with_search() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin user
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;

    // Create a target user with a specific email pattern
    let app = create_test_app(config.clone(), pool.clone());
    let mut target_user = TestUser::new();
    target_user.email = format!("searchable_user_{}@example.com", uuid::Uuid::new_v4());
    let target_auth = create_authenticated_user(&app, &target_user).await;

    // Add both to org
    add_user_to_organization(&pool, &admin_auth.user_id, &org.id, "admin").await;
    add_user_to_organization(&pool, &target_auth.user_id, &org.id, "member").await;

    // Search for the target user
    let app = create_test_app(config, pool.clone());
    let request = get_admin_request_with_jwt(
        "/api/admin/v1/users?search=searchable_user",
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let users = body["users"].as_array().unwrap();
    assert_eq!(users.len(), 1);
    assert!(users[0]["email"]
        .as_str()
        .unwrap()
        .contains("searchable_user"));

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_managed_users_without_auth() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Try to list without authentication
    let request = axum::http::Request::builder()
        .method(axum::http::Method::GET)
        .uri("/api/admin/v1/users")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Get User Location Tests
// ============================================================================

#[tokio::test]
async fn test_get_user_location_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin user
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;

    // Create target user
    let app = create_test_app(config.clone(), pool.clone());
    let target_user = TestUser::new();
    let target_auth = create_authenticated_user(&app, &target_user).await;

    // Add both to org
    add_user_to_organization(&pool, &admin_auth.user_id, &org.id, "admin").await;
    add_user_to_organization(&pool, &target_auth.user_id, &org.id, "member").await;

    // Register a device for the target user
    let device = TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let device_response = register_test_device(&app, &pool, &target_auth, &device).await;
    let device_id = device_response["device_id"].as_str().unwrap();

    // Insert a location for the device
    insert_device_location(&pool, device_id, 37.7749, -122.4194, 10.0).await;

    // Get user location
    let app = create_test_app(config, pool.clone());
    let request = get_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/location", target_auth.user_id),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body.get("device_id").is_some());
    assert!(body.get("latitude").is_some());
    assert!(body.get("longitude").is_some());

    // Verify location values
    let lat = body["latitude"].as_f64().unwrap();
    let lon = body["longitude"].as_f64().unwrap();
    assert!((lat - 37.7749).abs() < 0.0001);
    assert!((lon - (-122.4194)).abs() < 0.0001);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_user_location_no_location_data() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin and target users
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;

    let app = create_test_app(config.clone(), pool.clone());
    let target_user = TestUser::new();
    let target_auth = create_authenticated_user(&app, &target_user).await;

    // Add both to org
    add_user_to_organization(&pool, &admin_auth.user_id, &org.id, "admin").await;
    add_user_to_organization(&pool, &target_auth.user_id, &org.id, "member").await;

    // Get location without any device/location data
    let app = create_test_app(config, pool.clone());
    let request = get_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/location", target_auth.user_id),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// User Geofences CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_user_geofence_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin and target users
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;

    let app = create_test_app(config.clone(), pool.clone());
    let target_user = TestUser::new();
    let target_auth = create_authenticated_user(&app, &target_user).await;

    // Add both to org
    add_user_to_organization(&pool, &admin_auth.user_id, &org.id, "admin").await;
    add_user_to_organization(&pool, &target_auth.user_id, &org.id, "member").await;

    // Create a geofence
    let app = create_test_app(config, pool.clone());
    let request = post_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/geofences", target_auth.user_id),
        json!({
            "name": "Test Geofence",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius_meters": 500,
            "event_types": ["enter", "exit"],
            "color": "#FF5733"
        }),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body.get("geofence").is_some());
    let geofence = &body["geofence"];
    assert_eq!(geofence["name"].as_str().unwrap(), "Test Geofence");
    assert_eq!(geofence["radius_meters"].as_f64().unwrap() as i64, 500);
    assert_eq!(geofence["color"].as_str().unwrap(), "#FF5733");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_user_geofence_invalid_color() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin and target users
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;

    let app = create_test_app(config.clone(), pool.clone());
    let target_user = TestUser::new();
    let target_auth = create_authenticated_user(&app, &target_user).await;

    // Add both to org
    add_user_to_organization(&pool, &admin_auth.user_id, &org.id, "admin").await;
    add_user_to_organization(&pool, &target_auth.user_id, &org.id, "member").await;

    // Try to create a geofence with invalid color (non-hex characters)
    let app = create_test_app(config, pool.clone());
    let request = post_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/geofences", target_auth.user_id),
        json!({
            "name": "Test Geofence",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius_meters": 500,
            "event_types": ["enter"],
            "color": "#GGGGGG"  // Invalid hex color
        }),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_user_geofences() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin and target users
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;

    let app = create_test_app(config.clone(), pool.clone());
    let target_user = TestUser::new();
    let target_auth = create_authenticated_user(&app, &target_user).await;

    // Add both to org
    add_user_to_organization(&pool, &admin_auth.user_id, &org.id, "admin").await;
    add_user_to_organization(&pool, &target_auth.user_id, &org.id, "member").await;

    // Create two geofences
    for i in 1..=2 {
        let app = create_test_app(config.clone(), pool.clone());
        let request = post_admin_request_with_jwt(
            &format!("/api/admin/v1/users/{}/geofences", target_auth.user_id),
            json!({
                "name": format!("Geofence {}", i),
                "latitude": 37.7749 + (i as f64 * 0.01),
                "longitude": -122.4194,
                "radius_meters": 100 * i,
                "event_types": ["enter"]
            }),
            &admin_api_key,
            &admin_auth.access_token,
        );
        let _ = app.oneshot(request).await.unwrap();
    }

    // List geofences
    let app = create_test_app(config, pool.clone());
    let request = get_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/geofences", target_auth.user_id),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body.get("geofences").is_some());
    let geofences = body["geofences"].as_array().unwrap();
    assert_eq!(geofences.len(), 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_user_geofence() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin and target users
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;

    let app = create_test_app(config.clone(), pool.clone());
    let target_user = TestUser::new();
    let target_auth = create_authenticated_user(&app, &target_user).await;

    // Add both to org
    add_user_to_organization(&pool, &admin_auth.user_id, &org.id, "admin").await;
    add_user_to_organization(&pool, &target_auth.user_id, &org.id, "member").await;

    // Create a geofence
    let app = create_test_app(config.clone(), pool.clone());
    let request = post_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/geofences", target_auth.user_id),
        json!({
            "name": "Original Name",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius_meters": 500,
            "event_types": ["enter"]
        }),
        &admin_api_key,
        &admin_auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;
    let geofence_id = body["geofence"]["id"].as_str().unwrap();

    // Update the geofence
    let app = create_test_app(config, pool.clone());
    let request = put_admin_request_with_jwt(
        &format!(
            "/api/admin/v1/users/{}/geofences/{}",
            target_auth.user_id, geofence_id
        ),
        json!({
            "name": "Updated Name",
            "radius_meters": 1000
        }),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["geofence"]["name"].as_str().unwrap(), "Updated Name");
    assert_eq!(
        body["geofence"]["radius_meters"].as_f64().unwrap() as i64,
        1000
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_user_geofence() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin and target users
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;

    let app = create_test_app(config.clone(), pool.clone());
    let target_user = TestUser::new();
    let target_auth = create_authenticated_user(&app, &target_user).await;

    // Add both to org
    add_user_to_organization(&pool, &admin_auth.user_id, &org.id, "admin").await;
    add_user_to_organization(&pool, &target_auth.user_id, &org.id, "member").await;

    // Create a geofence
    let app = create_test_app(config.clone(), pool.clone());
    let request = post_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/geofences", target_auth.user_id),
        json!({
            "name": "To Be Deleted",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius_meters": 500,
            "event_types": ["enter"]
        }),
        &admin_api_key,
        &admin_auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;
    let geofence_id = body["geofence"]["id"].as_str().unwrap();

    // Delete the geofence
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_admin_request_with_jwt(
        &format!(
            "/api/admin/v1/users/{}/geofences/{}",
            target_auth.user_id, geofence_id
        ),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify it's deleted by trying to list
    let app = create_test_app(config, pool.clone());
    let request = get_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/geofences", target_auth.user_id),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;
    let geofences = body["geofences"].as_array().unwrap();
    assert_eq!(geofences.len(), 0);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Update Tracking Status Tests
// ============================================================================

#[tokio::test]
async fn test_update_tracking_status() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin and target users
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;

    let app = create_test_app(config.clone(), pool.clone());
    let target_user = TestUser::new();
    let target_auth = create_authenticated_user(&app, &target_user).await;

    // Add both to org
    add_user_to_organization(&pool, &admin_auth.user_id, &org.id, "admin").await;
    add_user_to_organization(&pool, &target_auth.user_id, &org.id, "member").await;

    // Disable tracking
    let app = create_test_app(config.clone(), pool.clone());
    let request = put_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/tracking", target_auth.user_id),
        json!({ "enabled": false }),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(!body["tracking_enabled"].as_bool().unwrap());

    // Re-enable tracking
    let app = create_test_app(config, pool.clone());
    let request = put_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/tracking", target_auth.user_id),
        json!({ "enabled": true }),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["tracking_enabled"].as_bool().unwrap());

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Authorization Boundary Tests
// ============================================================================

#[tokio::test]
async fn test_org_admin_cannot_access_other_org_users() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create two organizations
    let app = create_test_app(config.clone(), pool.clone());
    let org1 = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;
    let app = create_test_app(config.clone(), pool.clone());
    let org2 = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin in org1
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;
    add_user_to_organization(&pool, &admin_auth.user_id, &org1.id, "admin").await;

    // Create user in org2 (different org)
    let app = create_test_app(config.clone(), pool.clone());
    let other_org_user = TestUser::new();
    let other_org_auth = create_authenticated_user(&app, &other_org_user).await;
    add_user_to_organization(&pool, &other_org_auth.user_id, &org2.id, "member").await;

    // Try to access the other org's user location
    let app = create_test_app(config.clone(), pool.clone());
    let request = get_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/location", other_org_auth.user_id),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Try to create a geofence for the other org's user
    let app = create_test_app(config, pool.clone());
    let request = post_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/geofences", other_org_auth.user_id),
        json!({
            "name": "Unauthorized Geofence",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius_meters": 500,
            "event_types": ["enter"]
        }),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_org_member_cannot_manage_users() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create a regular member (not admin)
    let app = create_test_app(config.clone(), pool.clone());
    let member = TestUser::new();
    let member_auth = create_authenticated_user(&app, &member).await;
    add_user_to_organization(&pool, &member_auth.user_id, &org.id, "member").await;

    // Create another user in the same org
    let app = create_test_app(config.clone(), pool.clone());
    let target_user = TestUser::new();
    let target_auth = create_authenticated_user(&app, &target_user).await;
    add_user_to_organization(&pool, &target_auth.user_id, &org.id, "member").await;

    // Try to access user location as non-admin member
    let app = create_test_app(config, pool.clone());
    let request = get_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/location", target_auth.user_id),
        &admin_api_key,
        &member_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should be forbidden because member is not an admin
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_remove_managed_user() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin and target users
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;

    let app = create_test_app(config.clone(), pool.clone());
    let target_user = TestUser::new();
    let target_auth = create_authenticated_user(&app, &target_user).await;

    // Add both to org
    add_user_to_organization(&pool, &admin_auth.user_id, &org.id, "admin").await;
    add_user_to_organization(&pool, &target_auth.user_id, &org.id, "member").await;

    // Remove the user from management
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}", target_auth.user_id),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["removed"].as_bool().unwrap());

    // Verify user is no longer in the list
    let app = create_test_app(config, pool.clone());
    let request =
        get_admin_request_with_jwt("/api/admin/v1/users", &admin_api_key, &admin_auth.access_token);

    let response = app.oneshot(request).await.unwrap();
    let body = parse_response_body(response).await;
    let users = body["users"].as_array().unwrap();

    // Target user should not be in the list anymore
    let target_found = users
        .iter()
        .any(|u| u["id"].as_str() == Some(&target_auth.user_id));
    assert!(!target_found);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Geofence Limit Tests
// ============================================================================

#[tokio::test]
async fn test_geofence_limit_enforced() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let admin_api_key = create_test_admin_api_key(&pool, "test-admin-key").await;

    // Create an organization
    let app = create_test_app(config.clone(), pool.clone());
    let org = create_test_organization(&app, &admin_api_key, &TestOrganization::new()).await;

    // Create admin and target users
    let app = create_test_app(config.clone(), pool.clone());
    let admin = TestUser::new();
    let admin_auth = create_authenticated_user(&app, &admin).await;

    let app = create_test_app(config.clone(), pool.clone());
    let target_user = TestUser::new();
    let target_auth = create_authenticated_user(&app, &target_user).await;

    // Add both to org
    add_user_to_organization(&pool, &admin_auth.user_id, &org.id, "admin").await;
    add_user_to_organization(&pool, &target_auth.user_id, &org.id, "member").await;

    // Create 50 geofences (the limit)
    for i in 1..=50 {
        let app = create_test_app(config.clone(), pool.clone());
        let request = post_admin_request_with_jwt(
            &format!("/api/admin/v1/users/{}/geofences", target_auth.user_id),
            json!({
                "name": format!("Geofence {}", i),
                "latitude": 37.7749 + (i as f64 * 0.001),
                "longitude": -122.4194,
                "radius_meters": 100,
                "event_types": ["enter"]
            }),
            &admin_api_key,
            &admin_auth.access_token,
        );
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            "Failed to create geofence {}",
            i
        );
    }

    // Try to create one more (should fail)
    let app = create_test_app(config, pool.clone());
    let request = post_admin_request_with_jwt(
        &format!("/api/admin/v1/users/{}/geofences", target_auth.user_id),
        json!({
            "name": "One Too Many",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "radius_meters": 100,
            "event_types": ["enter"]
        }),
        &admin_api_key,
        &admin_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::CONFLICT
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}
