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
