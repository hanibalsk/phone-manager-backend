//! Integration tests for organization admin endpoints.
//!
//! Tests organization CRUD operations and org user management.

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_api_key, create_test_app,
    create_test_organization, create_test_pool, delete_request_with_api_key,
    get_request_with_api_key, json_request_with_api_key, parse_response_body,
    put_request_with_api_key, run_migrations, test_config, TestOrganization, TestUser,
};
use serde_json::json;
use tower::ServiceExt;

// ============================================================================
// Organization CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_organization_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;
    let org = TestOrganization::new();

    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        "/api/admin/v1/organizations",
        json!({
            "name": org.name,
            "slug": org.slug,
            "billing_email": org.billing_email
        }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body.get("organization").is_some());
    assert_eq!(body["organization"]["name"].as_str().unwrap(), org.name);
    assert_eq!(body["organization"]["slug"].as_str().unwrap(), org.slug);
    assert!(body["organization"]["id"].as_str().is_some());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_organization_with_plan_type() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;
    let org = TestOrganization::new();

    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        "/api/admin/v1/organizations",
        json!({
            "name": org.name,
            "slug": org.slug,
            "billing_email": org.billing_email,
            "plan_type": "business"
        }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert_eq!(
        body["organization"]["plan_type"].as_str().unwrap(),
        "business"
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_organization_duplicate_slug() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;
    let org = TestOrganization::new();

    // Create first organization
    let app = create_test_app(config.clone(), pool.clone());
    let _ = create_test_organization(&app, &api_key, &org).await;

    // Try to create with same slug
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        "/api/admin/v1/organizations",
        json!({
            "name": "Different Name",
            "slug": org.slug,
            "billing_email": "different@example.com"
        }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_organization_invalid_slug() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        "/api/admin/v1/organizations",
        json!({
            "name": "Test Org",
            "slug": "Invalid Slug With Spaces!",
            "billing_email": "test@example.com"
        }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_organizations_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create multiple organizations
    let app = create_test_app(config.clone(), pool.clone());
    let _ = create_test_organization(&app, &api_key, &TestOrganization::new()).await;
    let app = create_test_app(config.clone(), pool.clone());
    let _ = create_test_organization(&app, &api_key, &TestOrganization::new()).await;

    // List organizations
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key("/api/admin/v1/organizations", &api_key);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body.get("data").is_some());
    assert!(body.get("pagination").is_some());
    let orgs = body["data"].as_array().unwrap();
    assert!(orgs.len() >= 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_organizations_with_pagination() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create 5 organizations
    for _ in 0..5 {
        let app = create_test_app(config.clone(), pool.clone());
        let _ = create_test_organization(&app, &api_key, &TestOrganization::new()).await;
    }

    // Get first page with limit 2
    let app = create_test_app(config, pool.clone());
    let request =
        get_request_with_api_key("/api/admin/v1/organizations?page=1&per_page=2", &api_key);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    let orgs = body["data"].as_array().unwrap();
    assert_eq!(orgs.len(), 2);
    assert_eq!(body["pagination"]["page"].as_i64().unwrap(), 1);
    assert_eq!(body["pagination"]["per_page"].as_i64().unwrap(), 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_organization_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;
    let org = TestOrganization::new();

    let app = create_test_app(config.clone(), pool.clone());
    let created_org = create_test_organization(&app, &api_key, &org).await;

    // Get organization
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}", created_org.id),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["id"].as_str().unwrap(), created_org.id);
    assert_eq!(body["name"].as_str().unwrap(), org.name);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_organization_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    let app = create_test_app(config, pool.clone());
    let fake_id = uuid::Uuid::new_v4();
    let request =
        get_request_with_api_key(&format!("/api/admin/v1/organizations/{}", fake_id), &api_key);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_organization_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;
    let org = TestOrganization::new();

    let app = create_test_app(config.clone(), pool.clone());
    let created_org = create_test_organization(&app, &api_key, &org).await;

    // Update organization
    let app = create_test_app(config, pool.clone());
    let request = put_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}", created_org.id),
        json!({
            "name": "Updated Name",
            "max_users": 200
        }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["name"].as_str().unwrap(), "Updated Name");
    assert_eq!(body["max_users"].as_i64().unwrap(), 200);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_organization_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    let app = create_test_app(config, pool.clone());
    let fake_id = uuid::Uuid::new_v4();
    let request = put_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}", fake_id),
        json!({ "name": "New Name" }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_organization_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;
    let org = TestOrganization::new();

    let app = create_test_app(config.clone(), pool.clone());
    let created_org = create_test_organization(&app, &api_key, &org).await;

    // Delete organization (soft delete)
    let app = create_test_app(config, pool.clone());
    let request = delete_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}", created_org.id),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_organization_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    let app = create_test_app(config, pool.clone());
    let fake_id = uuid::Uuid::new_v4();
    let request = delete_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}", fake_id),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_organization_usage_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;
    let org = TestOrganization::new();

    let app = create_test_app(config.clone(), pool.clone());
    let created_org = create_test_organization(&app, &api_key, &org).await;

    // Get organization usage
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/usage", created_org.id),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body.get("users").is_some() || body.get("devices").is_some());

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Organization Users Tests
// ============================================================================

#[tokio::test]
async fn test_add_org_user_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create organization
    let org = TestOrganization::new();
    let app = create_test_app(config.clone(), pool.clone());
    let created_org = create_test_organization(&app, &api_key, &org).await;

    // Create a user to add
    let user = TestUser::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _ = create_authenticated_user(&app, &user).await;

    // Add user to organization
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/users", created_org.id),
        json!({
            "email": user.email,
            "role": "admin"
        }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_response_body(response).await;
    assert!(body.get("org_user").is_some());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_add_org_user_already_member() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create organization
    let org = TestOrganization::new();
    let app = create_test_app(config.clone(), pool.clone());
    let created_org = create_test_organization(&app, &api_key, &org).await;

    // Create a user to add
    let user = TestUser::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _ = create_authenticated_user(&app, &user).await;

    // Add user to organization
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/users", created_org.id),
        json!({
            "email": user.email,
            "role": "admin"
        }),
        &api_key,
    );
    let _ = app.oneshot(request).await.unwrap();

    // Try to add same user again
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/users", created_org.id),
        json!({
            "email": user.email,
            "role": "member"
        }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_add_org_user_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create organization
    let org = TestOrganization::new();
    let app = create_test_app(config.clone(), pool.clone());
    let created_org = create_test_organization(&app, &api_key, &org).await;

    // Try to add non-existent user
    let app = create_test_app(config, pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/users", created_org.id),
        json!({
            "email": "nonexistent@example.com",
            "role": "admin"
        }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_org_users_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create organization
    let org = TestOrganization::new();
    let app = create_test_app(config.clone(), pool.clone());
    let created_org = create_test_organization(&app, &api_key, &org).await;

    // Create and add a user
    let user = TestUser::new();
    let app = create_test_app(config.clone(), pool.clone());
    let _ = create_authenticated_user(&app, &user).await;
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/users", created_org.id),
        json!({
            "email": user.email,
            "role": "admin"
        }),
        &api_key,
    );
    let _ = app.oneshot(request).await.unwrap();

    // List org users
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/users", created_org.id),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body.get("data").is_some());
    assert!(body.get("pagination").is_some());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_org_user_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create organization
    let org = TestOrganization::new();
    let app = create_test_app(config.clone(), pool.clone());
    let created_org = create_test_organization(&app, &api_key, &org).await;

    // Create and add a user
    let user = TestUser::new();
    let app = create_test_app(config.clone(), pool.clone());
    let auth = create_authenticated_user(&app, &user).await;
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/users", created_org.id),
        json!({
            "email": user.email,
            "role": "member"
        }),
        &api_key,
    );
    let _ = app.oneshot(request).await.unwrap();

    // Update user role
    let app = create_test_app(config, pool.clone());
    let request = put_request_with_api_key(
        &format!(
            "/api/admin/v1/organizations/{}/users/{}",
            created_org.id, auth.user_id
        ),
        json!({ "role": "admin" }),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_remove_org_user_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create organization
    let org = TestOrganization::new();
    let app = create_test_app(config.clone(), pool.clone());
    let created_org = create_test_organization(&app, &api_key, &org).await;

    // Create and add two users (so we have an owner and can remove one)
    let user1 = TestUser::new();
    let app = create_test_app(config.clone(), pool.clone());
    let auth1 = create_authenticated_user(&app, &user1).await;
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/users", created_org.id),
        json!({
            "email": user1.email,
            "role": "owner"
        }),
        &api_key,
    );
    let _ = app.oneshot(request).await.unwrap();

    let user2 = TestUser::new();
    let app = create_test_app(config.clone(), pool.clone());
    let auth2 = create_authenticated_user(&app, &user2).await;
    let app = create_test_app(config.clone(), pool.clone());
    let request = json_request_with_api_key(
        Method::POST,
        &format!("/api/admin/v1/organizations/{}/users", created_org.id),
        json!({
            "email": user2.email,
            "role": "member"
        }),
        &api_key,
    );
    let _ = app.oneshot(request).await.unwrap();

    // Remove user2
    let app = create_test_app(config, pool.clone());
    let request = delete_request_with_api_key(
        &format!(
            "/api/admin/v1/organizations/{}/users/{}",
            created_org.id, auth2.user_id
        ),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_remove_org_user_not_member() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let api_key = create_test_api_key(&pool, "test-admin-key").await;

    // Create organization
    let org = TestOrganization::new();
    let app = create_test_app(config.clone(), pool.clone());
    let created_org = create_test_organization(&app, &api_key, &org).await;

    // Try to remove non-member user
    let fake_user_id = uuid::Uuid::new_v4();
    let app = create_test_app(config, pool.clone());
    let request = delete_request_with_api_key(
        &format!(
            "/api/admin/v1/organizations/{}/users/{}",
            created_org.id, fake_user_id
        ),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}
