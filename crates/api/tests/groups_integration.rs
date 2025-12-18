//! Integration tests for group management endpoints.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test groups_integration

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_app, create_test_group,
    create_test_pool, delete_request_with_auth, get_request_with_auth, json_request_with_auth,
    parse_response_body, run_migrations, test_config, TestGroup, TestUser,
};
use serde_json::json;
use tower::ServiceExt;

// ============================================================================
// Group Creation Tests
// ============================================================================

#[tokio::test]
async fn test_create_group_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create a group
    let group = TestGroup::new();
    let created = create_test_group(&app, &auth, &group).await;

    assert!(!created.id.is_empty());
    assert!(!created.slug.is_empty());
    assert_eq!(created.name, group.name);
    assert!(!created.invite_code.is_empty());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_group_requires_auth() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Try to create a group without authentication
    let request = axum::http::Request::builder()
        .method(Method::POST)
        .uri("/api/v1/groups")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&json!({
                "name": "Test Group",
                "description": "A test group"
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_group_empty_name() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to create group with empty name
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/groups",
        json!({
            "name": "",
            "description": "A test group"
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
// Group Listing Tests
// ============================================================================

#[tokio::test]
async fn test_list_groups_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Create two groups
    let group1 = TestGroup::new().with_name("Group One");
    let _created1 = create_test_group(&app, &auth, &group1).await;

    let app = create_test_app(config.clone(), pool.clone());
    let group2 = TestGroup::new().with_name("Group Two");
    let _created2 = create_test_group(&app, &auth, &group2).await;

    // List user's groups
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth("/api/v1/groups", &auth.access_token);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let groups = body["data"].as_array().unwrap();
    assert_eq!(groups.len(), 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_groups_empty() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user (no groups)
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // List user's groups
    let request = get_request_with_auth("/api/v1/groups", &auth.access_token);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let groups = body["data"].as_array().unwrap();
    assert!(groups.is_empty());

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Get Single Group Tests
// ============================================================================

#[tokio::test]
async fn test_get_group_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and group
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &auth, &group).await;

    // Get the group by ID
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/groups/{}", created.id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["id"], created.id);
    assert_eq!(body["name"], group.name);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_group_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to get non-existent group
    let fake_group_id = uuid::Uuid::new_v4();
    let request = get_request_with_auth(
        &format!("/api/v1/groups/{}", fake_group_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Group Update Tests
// ============================================================================

#[tokio::test]
async fn test_update_group_as_owner() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and group
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &auth, &group).await;

    // Update the group
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::PUT,
        &format!("/api/v1/groups/{}", created.id),
        json!({
            "name": "Updated Group Name",
            "description": "Updated description"
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["name"], "Updated Group Name");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_group_as_member_forbidden() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create owner and group
    let owner = TestUser::new();
    let owner_auth = create_authenticated_user(&app, &owner).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &owner_auth, &group).await;

    // Create another user
    let app = create_test_app(config.clone(), pool.clone());
    let member = TestUser::new();
    let member_auth = create_authenticated_user(&app, &member).await;

    // Member joins the group
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/groups/join",
        json!({
            "code": created.invite_code
        }),
        &member_auth.access_token,
    );
    let _join_response = app.oneshot(request).await.unwrap();

    // Member tries to update group (should be forbidden)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::PUT,
        &format!("/api/v1/groups/{}", created.id),
        json!({
            "name": "Unauthorized Update"
        }),
        &member_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Group Deletion Tests
// ============================================================================

#[tokio::test]
async fn test_delete_group_as_owner() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and group
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &auth, &group).await;

    // Delete the group
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_request_with_auth(
        &format!("/api/v1/groups/{}", created.id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify group is gone
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/groups/{}", created.id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_group_as_member_forbidden() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create owner and group
    let owner = TestUser::new();
    let owner_auth = create_authenticated_user(&app, &owner).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &owner_auth, &group).await;

    // Create another user who joins
    let app = create_test_app(config.clone(), pool.clone());
    let member = TestUser::new();
    let member_auth = create_authenticated_user(&app, &member).await;

    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/groups/join",
        json!({
            "code": created.invite_code
        }),
        &member_auth.access_token,
    );
    let _join_response = app.oneshot(request).await.unwrap();

    // Member tries to delete group (should be forbidden)
    let app = create_test_app(config, pool.clone());
    let request = delete_request_with_auth(
        &format!("/api/v1/groups/{}", created.id),
        &member_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Join Group Tests
// ============================================================================

#[tokio::test]
async fn test_join_group_with_invite_code() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create owner and group
    let owner = TestUser::new();
    let owner_auth = create_authenticated_user(&app, &owner).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &owner_auth, &group).await;

    // Create another user
    let app = create_test_app(config.clone(), pool.clone());
    let new_member = TestUser::new();
    let new_member_auth = create_authenticated_user(&app, &new_member).await;

    // New user joins with invite code
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/groups/join",
        json!({
            "code": created.invite_code
        }),
        &new_member_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["group"]["id"], created.id);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_join_group_invalid_invite_code() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Try to join with invalid code
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/groups/join",
        json!({
            "code": "INV-ALI-DXX"
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_join_group_already_member() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create owner and group
    let owner = TestUser::new();
    let owner_auth = create_authenticated_user(&app, &owner).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &owner_auth, &group).await;

    // Owner tries to join their own group (should conflict)
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/groups/join",
        json!({
            "code": created.invite_code
        }),
        &owner_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Group Members Tests
// ============================================================================

#[tokio::test]
async fn test_list_group_members() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create owner and group
    let owner = TestUser::new();
    let owner_auth = create_authenticated_user(&app, &owner).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &owner_auth, &group).await;

    // Add another member
    let app = create_test_app(config.clone(), pool.clone());
    let member = TestUser::new();
    let member_auth = create_authenticated_user(&app, &member).await;

    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/groups/join",
        json!({
            "code": created.invite_code
        }),
        &member_auth.access_token,
    );
    let _join_response = app.oneshot(request).await.unwrap();

    // List members
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/groups/{}/members", created.id),
        &owner_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let members = body["data"].as_array().unwrap();
    assert_eq!(members.len(), 2); // Owner + member

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_remove_member_as_owner() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create owner and group
    let owner = TestUser::new();
    let owner_auth = create_authenticated_user(&app, &owner).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &owner_auth, &group).await;

    // Add another member
    let app = create_test_app(config.clone(), pool.clone());
    let member = TestUser::new();
    let member_auth = create_authenticated_user(&app, &member).await;

    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/groups/join",
        json!({
            "code": created.invite_code
        }),
        &member_auth.access_token,
    );
    let _join_response = app.oneshot(request).await.unwrap();

    // Owner removes member
    let app = create_test_app(config, pool.clone());
    let request = delete_request_with_auth(
        &format!(
            "/api/v1/groups/{}/members/{}",
            created.id, member_auth.user_id
        ),
        &owner_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Transfer Ownership Tests
// ============================================================================

#[tokio::test]
async fn test_transfer_ownership_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create owner and group
    let owner = TestUser::new();
    let owner_auth = create_authenticated_user(&app, &owner).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &owner_auth, &group).await;

    // Add another member
    let app = create_test_app(config.clone(), pool.clone());
    let new_owner = TestUser::new();
    let new_owner_auth = create_authenticated_user(&app, &new_owner).await;

    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        "/api/v1/groups/join",
        json!({
            "code": created.invite_code
        }),
        &new_owner_auth.access_token,
    );
    let join_response = app.oneshot(request).await.unwrap();
    assert_eq!(join_response.status(), StatusCode::OK);

    // Transfer ownership
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/transfer", created.id),
        json!({
            "new_owner_id": new_owner_auth.user_id
        }),
        &owner_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_transfer_ownership_non_member() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create owner and group
    let owner = TestUser::new();
    let owner_auth = create_authenticated_user(&app, &owner).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &owner_auth, &group).await;

    // Create non-member user
    let app = create_test_app(config.clone(), pool.clone());
    let non_member = TestUser::new();
    let non_member_auth = create_authenticated_user(&app, &non_member).await;

    // Try to transfer to non-member
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/transfer", created.id),
        json!({
            "new_owner_id": non_member_auth.user_id
        }),
        &owner_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should fail - can't transfer to non-member
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND
    );

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// UGM-3: Multi-Group Device Management Tests
// ============================================================================

#[tokio::test]
async fn test_add_device_to_group_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and group
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &auth, &group).await;

    // Register a device for the user
    let device = common::TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _device_response = common::register_test_device(&app, &pool, &auth, &device).await;

    // Add the device to the group
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/devices", created.id),
        json!({
            "device_id": device.device_id
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert_eq!(body["group_id"], created.id);
    assert_eq!(body["device_id"], device.device_id);
    assert!(body["added_at"].is_string());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_add_device_to_group_not_member() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create owner and their group
    let owner = TestUser::new();
    let owner_auth = create_authenticated_user(&app, &owner).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &owner_auth, &group).await;

    // Create non-member user with a device
    let app = create_test_app(config.clone(), pool.clone());
    let non_member = TestUser::new();
    let non_member_auth = create_authenticated_user(&app, &non_member).await;
    let device = common::TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _device_response = common::register_test_device(&app, &pool, &non_member_auth, &device).await;

    // Non-member tries to add their device to the group
    // Returns 404 to avoid leaking group existence to non-members
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/devices", created.id),
        json!({
            "device_id": device.device_id
        }),
        &non_member_auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_add_device_already_in_group() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and group
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &auth, &group).await;

    // Register a device for the user
    let device = common::TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _device_response = common::register_test_device(&app, &pool, &auth, &device).await;

    // Add the device to the group
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/devices", created.id),
        json!({
            "device_id": device.device_id
        }),
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Try to add the same device again
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/devices", created.id),
        json!({
            "device_id": device.device_id
        }),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_remove_device_from_group_as_owner() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and group
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &auth, &group).await;

    // Register a device for the user
    let device = common::TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _device_response = common::register_test_device(&app, &pool, &auth, &device).await;

    // Add the device to the group
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/devices", created.id),
        json!({
            "device_id": device.device_id
        }),
        &auth.access_token,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Remove the device from the group
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_request_with_auth(
        &format!("/api/v1/groups/{}/devices/{}", created.id, device.device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_remove_device_not_in_group() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and group
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &auth, &group).await;

    // Register a device for the user (but don't add to group)
    let device = common::TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _device_response = common::register_test_device(&app, &pool, &auth, &device).await;

    // Try to remove device that's not in the group
    let app = create_test_app(config.clone(), pool.clone());
    let request = delete_request_with_auth(
        &format!("/api/v1/groups/{}/devices/{}", created.id, device.device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_group_devices_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and group
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &auth, &group).await;

    // Register devices and add them to the group
    let device1 = common::TestDevice::new().with_name("Device 1");
    let app = create_test_app(config.clone(), pool.clone());
    let _device1_response = common::register_test_device(&app, &pool, &auth, &device1).await;

    let device2 = common::TestDevice::new().with_name("Device 2");
    let app = create_test_app(config.clone(), pool.clone());
    let _device2_response = common::register_test_device(&app, &pool, &auth, &device2).await;

    // Add devices to group
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/devices", created.id),
        json!({ "device_id": device1.device_id }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/devices", created.id),
        json!({ "device_id": device2.device_id }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // List devices in the group
    let app = create_test_app(config.clone(), pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/groups/{}/devices/members", created.id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let devices = body["data"].as_array().unwrap();
    assert_eq!(devices.len(), 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_group_devices_with_location() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user and group
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let group = TestGroup::new();
    let created = create_test_group(&app, &auth, &group).await;

    // Register a device
    let device = common::TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _device_response = common::register_test_device(&app, &pool, &auth, &device).await;

    // Add device to group
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/devices", created.id),
        json!({ "device_id": device.device_id }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // Insert a location for the device
    common::insert_device_location(&pool, &device.device_id, 48.8566, 2.3522, 10.0).await;

    // List devices with location
    let app = create_test_app(config.clone(), pool.clone());
    let request = get_request_with_auth(
        &format!(
            "/api/v1/groups/{}/devices/members?include_location=true",
            created.id
        ),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let devices = body["data"].as_array().unwrap();
    assert_eq!(devices.len(), 1);
    assert!(devices[0]["last_location"].is_object());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_device_groups_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register a device
    let device = common::TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _device_response = common::register_test_device(&app, &pool, &auth, &device).await;

    // Create two groups and add the device to both
    let group1 = TestGroup::new().with_name("Group A");
    let app = create_test_app(config.clone(), pool.clone());
    let created1 = create_test_group(&app, &auth, &group1).await;

    let group2 = TestGroup::new().with_name("Group B");
    let app = create_test_app(config.clone(), pool.clone());
    let created2 = create_test_group(&app, &auth, &group2).await;

    // Add device to both groups
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/devices", created1.id),
        json!({ "device_id": device.device_id }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/devices", created2.id),
        json!({ "device_id": device.device_id }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // List groups for the device
    let app = create_test_app(config.clone(), pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/devices/{}/groups", device.device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let groups = body["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 2);

    // Verify pagination info is present
    let pagination = &body["pagination"];
    assert_eq!(pagination["page"], 1);
    assert_eq!(pagination["total"], 2);
    assert!(pagination["per_page"].as_i64().unwrap() > 0);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_device_groups_not_device_owner() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create user 1 with a device
    let user1 = TestUser::new();
    let auth1 = create_authenticated_user(&app, &user1).await;
    let device = common::TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _device_response = common::register_test_device(&app, &pool, &auth1, &device).await;

    // Create user 2
    let app = create_test_app(config.clone(), pool.clone());
    let user2 = TestUser::new();
    let auth2 = create_authenticated_user(&app, &user2).await;

    // User 2 tries to list groups for user 1's device
    let app = create_test_app(config.clone(), pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/devices/{}/groups", device.device_id),
        &auth2.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_has_current_device_in_group_listing() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register a device
    let device = common::TestDevice::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _device_response = common::register_test_device(&app, &pool, &auth, &device).await;

    // Create two groups
    let group1 = TestGroup::new().with_name("Group With Device");
    let app = create_test_app(config.clone(), pool.clone());
    let created1 = create_test_group(&app, &auth, &group1).await;

    let group2 = TestGroup::new().with_name("Group Without Device");
    let app = create_test_app(config.clone(), pool.clone());
    let _created2 = create_test_group(&app, &auth, &group2).await;

    // Add device only to group 1
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/devices", created1.id),
        json!({ "device_id": device.device_id }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // List groups - should show has_current_device true for group1
    let app = create_test_app(config.clone(), pool.clone());
    let request = get_request_with_auth("/api/v1/groups", &auth.access_token);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let groups = body["data"].as_array().unwrap();
    assert_eq!(groups.len(), 2);

    // Find group1 and group2 in the response
    let group1_data = groups
        .iter()
        .find(|g| g["name"] == "Group With Device")
        .expect("Group with device not found");
    let group2_data = groups
        .iter()
        .find(|g| g["name"] == "Group Without Device")
        .expect("Group without device not found");

    assert_eq!(group1_data["has_current_device"], true);
    assert_eq!(group2_data["has_current_device"], false);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_has_current_device_with_device_id_filter() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;

    // Register two devices
    let device1 = common::TestDevice::new().with_name("Device 1");
    let app = create_test_app(config.clone(), pool.clone());
    let _device1_response = common::register_test_device(&app, &pool, &auth, &device1).await;

    let device2 = common::TestDevice::new().with_name("Device 2");
    let app = create_test_app(config.clone(), pool.clone());
    let _device2_response = common::register_test_device(&app, &pool, &auth, &device2).await;

    // Create a group and add only device1
    let group = TestGroup::new();
    let app = create_test_app(config.clone(), pool.clone());
    let created = create_test_group(&app, &auth, &group).await;

    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_auth(
        Method::POST,
        &format!("/api/v1/groups/{}/devices", created.id),
        json!({ "device_id": device1.device_id }),
        &auth.access_token,
    );
    let _response = app.oneshot(request).await.unwrap();

    // Query with device1 ID - should show has_current_device = true
    let app = create_test_app(config.clone(), pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/groups?device_id={}", device1.device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let groups = body["data"].as_array().unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["has_current_device"], true);

    // Query with device2 ID - should show has_current_device = false
    let app = create_test_app(config.clone(), pool.clone());
    let request = get_request_with_auth(
        &format!("/api/v1/groups?device_id={}", device2.device_id),
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let groups = body["data"].as_array().unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["has_current_device"], false);

    cleanup_all_test_data(&pool).await;
}
