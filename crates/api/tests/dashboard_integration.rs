//! Integration tests for dashboard metrics endpoint.
//!
//! These tests require a running PostgreSQL instance.
//! Set TEST_DATABASE_URL environment variable or use docker-compose.
//!
//! Run with: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --test dashboard_integration
//!
//! Note: Some dashboard features may require additional schema columns (organization_id,
//! enrollment_status, etc.) that are part of the B2B enterprise feature set.

mod common;

use axum::http::{Method, StatusCode};
use common::{
    cleanup_all_test_data, create_test_admin_api_key, create_test_api_key, create_test_app,
    create_test_pool, run_migrations, test_config,
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

/// Build a GET request with admin API key.
fn get_request_with_api_key(uri: &str, api_key: &str) -> axum::http::Request<axum::body::Body> {
    use axum::{body::Body, http::Request};

    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("X-API-Key", api_key)
        .body(Body::empty())
        .unwrap()
}

/// Build a GET request without any authentication.
fn get_request_no_auth(uri: &str) -> axum::http::Request<axum::body::Body> {
    use axum::{body::Body, http::Request};

    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

// ============================================================================
// Dashboard Authorization Tests
// ============================================================================

#[tokio::test]
async fn test_get_dashboard_metrics_org_not_found() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    // Create admin API key
    let api_key = create_test_admin_api_key(&pool, "dashboard_notfound_test").await;

    // Try to get dashboard for non-existent organization
    let fake_org_id = Uuid::new_v4();
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/dashboard", fake_org_id),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_dashboard_metrics_non_admin_key_rejected() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    // Create organization
    let org_id = create_test_org(&pool).await;

    // Create NON-admin API key
    let api_key = create_test_api_key(&pool, "dashboard_nonadmin_test").await;

    // Try to get dashboard with non-admin key
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key(
        &format!("/api/admin/v1/organizations/{}/dashboard", org_id),
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should be 403 Forbidden for non-admin API key
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_dashboard_metrics_no_api_key_rejected() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    // Create organization
    let org_id = create_test_org(&pool).await;

    // Try to get dashboard without API key
    let app = create_test_app(config, pool.clone());
    let request = get_request_no_auth(&format!(
        "/api/admin/v1/organizations/{}/dashboard",
        org_id
    ));

    let response = app.oneshot(request).await.unwrap();
    // Should be 401 Unauthorized for missing API key
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    cleanup_all_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_dashboard_metrics_invalid_org_id_format() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_all_test_data(&pool).await;

    let config = test_config();

    // Create admin API key
    let api_key = create_test_admin_api_key(&pool, "dashboard_invalid_uuid_test").await;

    // Try to get dashboard with invalid UUID format
    let app = create_test_app(config, pool.clone());
    let request = get_request_with_api_key(
        "/api/admin/v1/organizations/invalid-uuid/dashboard",
        &api_key,
    );

    let response = app.oneshot(request).await.unwrap();
    // Should be 400 Bad Request for invalid UUID format
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    cleanup_all_test_data(&pool).await;
}
