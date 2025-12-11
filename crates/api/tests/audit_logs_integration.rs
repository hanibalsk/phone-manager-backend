//! Integration tests for audit logs endpoints.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test audit_logs_integration

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_authenticated_user, create_test_admin_api_key, create_test_app,
    create_test_pool, get_request_with_api_key_and_jwt, parse_response_body, run_migrations,
    test_config, TestUser,
};
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

/// Create a test audit log entry directly in the database.
async fn create_test_audit_log(
    pool: &PgPool,
    org_id: Uuid,
    action: &str,
    resource_type: &str,
) -> Uuid {
    let log_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO audit_logs (id, organization_id, actor_id, actor_type, actor_email, action, resource_type, resource_id, resource_name, created_at)
        VALUES ($1, $2, $3, 'user'::audit_actor_type, 'test@example.com', $4, $5, $6, 'Test Resource', NOW())
        "#,
    )
    .bind(log_id)
    .bind(org_id)
    .bind(actor_id)
    .bind(action)
    .bind(resource_type)
    .bind(Uuid::new_v4().to_string())
    .execute(pool)
    .await
    .expect("Failed to create test audit log");

    log_id
}

/// Create multiple test audit log entries for pagination testing.
async fn create_multiple_audit_logs(pool: &PgPool, org_id: Uuid, count: i32) -> Vec<Uuid> {
    let mut log_ids = Vec::new();
    let actor_id = Uuid::new_v4();

    for i in 0..count {
        let log_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO audit_logs (id, organization_id, actor_id, actor_type, actor_email, action, resource_type, resource_id, resource_name, created_at)
            VALUES ($1, $2, $3, 'user'::audit_actor_type, 'test@example.com', $4, 'device', $5, $6, NOW() - INTERVAL '1 minute' * $7)
            "#,
        )
        .bind(log_id)
        .bind(org_id)
        .bind(actor_id)
        .bind(format!("device.action{}", i))
        .bind(Uuid::new_v4().to_string())
        .bind(format!("Resource {}", i))
        .bind(i)
        .execute(pool)
        .await
        .expect("Failed to create test audit log");

        log_ids.push(log_id);
    }

    log_ids
}

// ============================================================================
// List Audit Logs Tests
// ============================================================================

#[tokio::test]
async fn test_list_audit_logs_empty() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_admin_api_key(&pool, "audit_list_empty").await;

    // Create organization
    let org_id = create_test_org(&pool).await;

    // List audit logs (should be empty)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/admin/v1/organizations/{}/audit-logs", org_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert!(body["pagination"].is_object());

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_audit_logs_with_data() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_admin_api_key(&pool, "audit_list_data").await;

    // Create organization and audit log
    let org_id = create_test_org(&pool).await;
    let _log_id = create_test_audit_log(&pool, org_id, "device.create", "device").await;

    // List audit logs
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!("/api/admin/v1/organizations/{}/audit-logs", org_id),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["pagination"]["total"], 1);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_audit_logs_with_pagination() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_admin_api_key(&pool, "audit_list_page").await;

    // Create organization and multiple audit logs
    let org_id = create_test_org(&pool).await;
    create_multiple_audit_logs(&pool, org_id, 15).await;

    // List with pagination (page 1, 10 per page)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!(
            "/api/admin/v1/organizations/{}/audit-logs?page=1&per_page=10",
            org_id
        ),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 10);
    assert_eq!(body["pagination"]["total"], 15);
    assert_eq!(body["pagination"]["page"], 1);
    assert_eq!(body["pagination"]["per_page"], 10);
    assert_eq!(body["pagination"]["total_pages"], 2);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_audit_logs_with_action_filter() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_admin_api_key(&pool, "audit_filter").await;

    // Create organization and audit logs with different actions
    let org_id = create_test_org(&pool).await;
    create_test_audit_log(&pool, org_id, "device.create", "device").await;
    create_test_audit_log(&pool, org_id, "device.update", "device").await;
    create_test_audit_log(&pool, org_id, "policy.create", "policy").await;

    // Filter by action
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!(
            "/api/admin/v1/organizations/{}/audit-logs?action=device.create",
            org_id
        ),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Get Audit Log Tests
// ============================================================================

#[tokio::test]
async fn test_get_audit_log_success() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_admin_api_key(&pool, "audit_get_success").await;

    // Create organization and audit log
    let org_id = create_test_org(&pool).await;
    let log_id = create_test_audit_log(&pool, org_id, "device.create", "device").await;

    // Get audit log
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!(
            "/api/admin/v1/organizations/{}/audit-logs/{}",
            org_id, log_id
        ),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    assert_eq!(body["id"], log_id.to_string());
    assert_eq!(body["action"], "device.create");

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_audit_log_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_admin_api_key(&pool, "audit_get_notfound").await;

    // Create organization
    let org_id = create_test_org(&pool).await;
    let non_existent_id = Uuid::new_v4();

    // Get non-existent audit log
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!(
            "/api/admin/v1/organizations/{}/audit-logs/{}",
            org_id, non_existent_id
        ),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

// ============================================================================
// Export Audit Logs Tests
// ============================================================================

#[tokio::test]
async fn test_export_audit_logs_json_sync() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_admin_api_key(&pool, "audit_export_json").await;

    // Create organization and a few audit logs
    let org_id = create_test_org(&pool).await;
    create_test_audit_log(&pool, org_id, "device.create", "device").await;
    create_test_audit_log(&pool, org_id, "device.update", "device").await;

    // Export as JSON (sync - small dataset)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!(
            "/api/admin/v1/organizations/{}/audit-logs/export?format=json",
            org_id
        ),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    // For sync export, should have format and download_url
    assert!(body["format"].is_string() || body["job_id"].is_string()); // Either sync or async
    if body["format"].is_string() {
        assert_eq!(body["format"], "json");
        assert!(body["download_url"].is_string());
    }

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_export_audit_logs_csv_sync() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_admin_api_key(&pool, "audit_export_csv").await;

    // Create organization and audit logs
    let org_id = create_test_org(&pool).await;
    create_test_audit_log(&pool, org_id, "policy.create", "policy").await;

    // Export as CSV (sync - small dataset)
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!(
            "/api/admin/v1/organizations/{}/audit-logs/export?format=csv",
            org_id
        ),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body(response).await;
    if body["format"].is_string() {
        assert_eq!(body["format"], "csv");
        assert!(body["download_url"].is_string());
    }

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_export_job_status_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config.clone(), pool.clone());

    // Create authenticated user
    let user = TestUser::new();
    let auth = create_authenticated_user(&app, &user).await;
    let api_key = create_test_admin_api_key(&pool, "audit_job_notfound").await;

    // Create organization
    let org_id = create_test_org(&pool).await;

    // Get non-existent export job
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key_and_jwt(
        &format!(
            "/api/admin/v1/organizations/{}/audit-logs/export/nonexistent-job-id",
            org_id
        ),
        &api_key,
        &auth.access_token,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_audit_logs_unauthorized() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();
    let app = create_test_app(config, pool.clone());

    let org_id = create_test_org(&pool).await;

    // Try without authentication
    let request = axum::http::Request::builder()
        .method(Method::GET)
        .uri(format!("/api/admin/v1/organizations/{}/audit-logs", org_id))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Should be 401 or 403 without auth
    assert!(
        response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::FORBIDDEN
    );

    cleanup_all_test_data(&pool).await;
}
