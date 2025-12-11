//! Integration tests for device policy endpoints.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test device_policies_integration

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_test_admin_api_key, create_test_app, create_test_pool,
    delete_request_with_api_key, get_request_with_api_key, json_request_with_api_key,
    parse_response_body, put_request_with_api_key, run_migrations, test_config,
};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a test organization directly in the database.
async fn create_test_org(pool: &PgPool) -> Uuid {
    let org_id = Uuid::new_v4();
    let name = format!("Test Org {}", &org_id.to_string()[..8]);
    let slug = format!("test-org-{}", &org_id.to_string()[..8]);

    sqlx::query(
        r#"
        INSERT INTO organizations (id, name, slug, billing_email, created_at, updated_at)
        VALUES ($1, $2, $3, 'billing@test.com', NOW(), NOW())
        "#,
    )
    .bind(org_id)
    .bind(&name)
    .bind(&slug)
    .execute(pool)
    .await
    .expect("Failed to create test organization");

    org_id
}

/// Create a test user for group creation.
async fn create_test_user_for_group(pool: &PgPool) -> Uuid {
    let user_id = Uuid::new_v4();
    let email = format!("group-creator-{}@test.com", &user_id.to_string()[..8]);

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, display_name, email_verified, is_active, created_at, updated_at)
        VALUES ($1, $2, 'hashed_password', 'Test User', true, true, NOW(), NOW())
        "#,
    )
    .bind(user_id)
    .bind(&email)
    .execute(pool)
    .await
    .expect("Failed to create test user for group");

    user_id
}

/// Create a test group (groups require a created_by user).
async fn create_test_group(pool: &PgPool, _org_id: Uuid) -> Uuid {
    // First create a user to be the group creator
    let user_id = create_test_user_for_group(pool).await;

    let group_id = Uuid::new_v4();
    let name = format!("Test Group {}", &group_id.to_string()[..8]);
    let slug = format!("test-group-{}", &group_id.to_string()[..8]);

    sqlx::query(
        r#"
        INSERT INTO groups (id, name, slug, created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        "#,
    )
    .bind(group_id)
    .bind(&name)
    .bind(&slug)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("Failed to create test group");

    group_id
}

// ============================================================================
// Create Policy Tests
// ============================================================================

#[tokio::test]
async fn test_create_policy_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_create_policy").await;

    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Test Policy",
            "description": "A test policy for tracking",
            "is_default": false,
            "settings": {
                "tracking_enabled": true,
                "tracking_interval_minutes": 5
            },
            "locked_settings": ["tracking_enabled"],
            "priority": 10
        }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body["id"].is_string());
    assert_eq!(body["name"], "Test Policy");
    assert_eq!(body["description"], "A test policy for tracking");
    assert_eq!(body["is_default"], false);
    assert_eq!(body["priority"], 10);
    assert!(body["locked_settings"]
        .as_array()
        .unwrap()
        .contains(&json!("tracking_enabled")));

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_policy_duplicate_name() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_dup_policy").await;

    // Create first policy
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Duplicate Policy",
            "priority": 0
        }),
        &api_key,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Try to create with same name
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_dup_policy2").await;
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Duplicate Policy",
            "priority": 0
        }),
        &api_key2,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_policy_invalid_priority() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_invalid_priority").await;

    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Invalid Priority Policy",
            "priority": 5000  // Over the max of 1000
        }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    // Validation errors can return either BAD_REQUEST or UNPROCESSABLE_ENTITY
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_policy_empty_name() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_empty_name").await;

    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "",
            "priority": 0
        }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    // Validation errors can return either BAD_REQUEST or UNPROCESSABLE_ENTITY
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// List Policies Tests
// ============================================================================

#[tokio::test]
async fn test_list_policies_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_list_policies").await;

    // Create a couple of policies
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Policy One",
            "priority": 10
        }),
        &api_key,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let app = create_test_app(config.clone(), pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_list_policies2").await;
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Policy Two",
            "priority": 20
        }),
        &api_key2,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // List policies
    let app = create_test_app(config, pool.clone());
    let api_key3 = create_test_admin_api_key(&pool, "test_list_policies3").await;
    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        &api_key3,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["data"].is_array());
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    assert!(body["pagination"]["total"].as_i64().unwrap() >= 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_policies_empty() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_list_empty").await;

    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let data = body["data"].as_array().unwrap();
    assert!(data.is_empty());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_policies_filter_default() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_filter_default").await;

    // Create a default policy
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Default Policy",
            "is_default": true,
            "priority": 0
        }),
        &api_key,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Create a non-default policy
    let app = create_test_app(config.clone(), pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_filter_default2").await;
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Other Policy",
            "is_default": false,
            "priority": 0
        }),
        &api_key2,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Filter for default only
    let app = create_test_app(config, pool.clone());
    let api_key3 = create_test_admin_api_key(&pool, "test_filter_default3").await;
    let request = get_request_with_api_key(
        &format!(
            "/api/admin/v1/organizations/{}/policies?is_default=true",
            org_id
        ),
        &api_key3,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["name"], "Default Policy");

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Get Policy Tests
// ============================================================================

#[tokio::test]
async fn test_get_policy_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_get_policy").await;

    // Create a policy
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Get Me Policy",
            "description": "Description here",
            "priority": 15
        }),
        &api_key,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_response_body(response).await;
    let policy_id = body["id"].as_str().unwrap();

    // Get the policy
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_get_policy2").await;
    let request = get_request_with_api_key(
        &format!(
            "/api/admin/v1/organizations/{}/policies/{}",
            org_id, policy_id
        ),
        &api_key2,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["id"], policy_id);
    assert_eq!(body["name"], "Get Me Policy");
    assert_eq!(body["description"], "Description here");
    assert_eq!(body["priority"], 15);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_policy_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_get_not_found").await;
    let fake_policy_id = Uuid::new_v4();

    let request = get_request_with_api_key(
        &format!(
            "/api/admin/v1/organizations/{}/policies/{}",
            org_id, fake_policy_id
        ),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_policy_wrong_org() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    // Create two organizations
    let org1_id = create_test_org(&pool).await;
    let org2_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_wrong_org").await;

    // Create policy in org1
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org1_id),
        json!({
            "name": "Org1 Policy",
            "priority": 0
        }),
        &api_key,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_response_body(response).await;
    let policy_id = body["id"].as_str().unwrap();

    // Try to get it from org2 (should fail)
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_wrong_org2").await;
    let request = get_request_with_api_key(
        &format!(
            "/api/admin/v1/organizations/{}/policies/{}",
            org2_id, policy_id
        ),
        &api_key2,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Update Policy Tests
// ============================================================================

#[tokio::test]
async fn test_update_policy_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_update_policy").await;

    // Create a policy
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Original Name",
            "description": "Original description",
            "priority": 10
        }),
        &api_key,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_response_body(response).await;
    let policy_id = body["id"].as_str().unwrap();

    // Update the policy
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_update_policy2").await;
    let request = put_request_with_api_key(
        &format!(
            "/api/admin/v1/organizations/{}/policies/{}",
            org_id, policy_id
        ),
        json!({
            "name": "Updated Name",
            "description": "Updated description",
            "priority": 20
        }),
        &api_key2,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["name"], "Updated Name");
    assert_eq!(body["description"], "Updated description");
    assert_eq!(body["priority"], 20);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_policy_name_conflict() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_name_conflict").await;

    // Create first policy
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Existing Name",
            "priority": 0
        }),
        &api_key,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Create second policy
    let app = create_test_app(config.clone(), pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_name_conflict2").await;
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Another Name",
            "priority": 0
        }),
        &api_key2,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_response_body(response).await;
    let policy_id = body["id"].as_str().unwrap();

    // Try to update second policy to have the same name as first
    let app = create_test_app(config, pool.clone());
    let api_key3 = create_test_admin_api_key(&pool, "test_name_conflict3").await;
    let request = put_request_with_api_key(
        &format!(
            "/api/admin/v1/organizations/{}/policies/{}",
            org_id, policy_id
        ),
        json!({
            "name": "Existing Name"
        }),
        &api_key3,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Delete Policy Tests
// ============================================================================

#[tokio::test]
async fn test_delete_policy_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_delete_policy").await;

    // Create a policy
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "To Be Deleted",
            "priority": 0
        }),
        &api_key,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_response_body(response).await;
    let policy_id = body["id"].as_str().unwrap();

    // Delete the policy
    let app = create_test_app(config.clone(), pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_delete_policy2").await;
    let request = delete_request_with_api_key(
        &format!(
            "/api/admin/v1/organizations/{}/policies/{}",
            org_id, policy_id
        ),
        &api_key2,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let app = create_test_app(config, pool.clone());
    let api_key3 = create_test_admin_api_key(&pool, "test_delete_policy3").await;
    let request = get_request_with_api_key(
        &format!(
            "/api/admin/v1/organizations/{}/policies/{}",
            org_id, policy_id
        ),
        &api_key3,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_policy_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_delete_not_found").await;
    let fake_policy_id = Uuid::new_v4();

    let request = delete_request_with_api_key(
        &format!(
            "/api/admin/v1/organizations/{}/policies/{}",
            org_id, fake_policy_id
        ),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Apply/Unapply Policy Tests
// ============================================================================

#[tokio::test]
async fn test_apply_policy_to_group() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    let org_id = create_test_org(&pool).await;
    let group_id = create_test_group(&pool, org_id).await;
    let api_key = create_test_admin_api_key(&pool, "test_apply_policy").await;

    // Create a policy
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Apply Test Policy",
            "priority": 0
        }),
        &api_key,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_response_body(response).await;
    let policy_id = body["id"].as_str().unwrap();

    // Apply the policy to a group
    let app = create_test_app(config, pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_apply_policy2").await;
    let request = json_request_with_api_key(
        Method::POST,
        &format!(
            "/api/admin/v1/organizations/{}/policies/{}/apply",
            org_id, policy_id
        ),
        json!({
            "targets": [
                {"type": "group", "id": group_id}
            ],
            "replace_existing": false
        }),
        &api_key2,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["policy_id"], policy_id);
    assert_eq!(body["applied_to"]["groups"], 1);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_unapply_policy_from_group() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    let org_id = create_test_org(&pool).await;
    let group_id = create_test_group(&pool, org_id).await;
    let api_key = create_test_admin_api_key(&pool, "test_unapply_policy").await;

    // Create a policy
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/policies", org_id),
        json!({
            "name": "Unapply Test Policy",
            "priority": 0
        }),
        &api_key,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_response_body(response).await;
    let policy_id = body["id"].as_str().unwrap();

    // Apply the policy first
    let app = create_test_app(config.clone(), pool.clone());
    let api_key2 = create_test_admin_api_key(&pool, "test_unapply_policy2").await;
    let request = json_request_with_api_key(
        Method::POST,
        &format!(
            "/api/admin/v1/organizations/{}/policies/{}/apply",
            org_id, policy_id
        ),
        json!({
            "targets": [
                {"type": "group", "id": group_id}
            ]
        }),
        &api_key2,
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Unapply the policy
    let app = create_test_app(config, pool.clone());
    let api_key3 = create_test_admin_api_key(&pool, "test_unapply_policy3").await;
    let request = json_request_with_api_key(
        Method::POST,
        &format!(
            "/api/admin/v1/organizations/{}/policies/{}/unapply",
            org_id, policy_id
        ),
        json!({
            "targets": [
                {"type": "group", "id": group_id}
            ]
        }),
        &api_key3,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["policy_id"], policy_id);
    assert_eq!(body["unapplied_from"]["groups"], 1);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_apply_policy_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let org_id = create_test_org(&pool).await;
    let api_key = create_test_admin_api_key(&pool, "test_apply_not_found").await;
    let fake_policy_id = Uuid::new_v4();
    let fake_group_id = Uuid::new_v4();

    let request = json_request_with_api_key(
        Method::POST,
        &format!(
            "/api/admin/v1/organizations/{}/policies/{}/apply",
            org_id, fake_policy_id
        ),
        json!({
            "targets": [
                {"type": "group", "id": fake_group_id}
            ]
        }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}
