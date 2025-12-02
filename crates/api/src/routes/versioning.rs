//! API versioning handlers.
//!
//! Provides redirect handlers for legacy unversioned API endpoints.

use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};

/// Redirect handler that converts legacy `/api/...` paths to `/api/v1/...`.
///
/// Returns 301 Moved Permanently with the versioned location.
#[allow(dead_code)] // Available for future use with catch-all routes
pub async fn redirect_to_v1(uri: Uri) -> Response {
    let path = uri.path();

    // Extract the path after /api/
    let new_path = if let Some(rest) = path.strip_prefix("/api/") {
        format!("/api/v1/{}", rest)
    } else {
        // Fallback - just prepend v1
        format!("/api/v1{}", path.strip_prefix("/api").unwrap_or(path))
    };

    // Include query string if present
    let new_uri = if let Some(query) = uri.query() {
        format!("{}?{}", new_path, query)
    } else {
        new_path
    };

    (
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, new_uri)],
        "Moved to versioned API",
    )
        .into_response()
}

/// Redirect POST requests for devices/register.
pub async fn redirect_devices_register() -> Response {
    (
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, "/api/v1/devices/register")],
        "Moved to versioned API",
    )
        .into_response()
}

/// Redirect GET requests for devices listing.
pub async fn redirect_devices_list(uri: Uri) -> Response {
    let new_uri = if let Some(query) = uri.query() {
        format!("/api/v1/devices?{}", query)
    } else {
        "/api/v1/devices".to_string()
    };

    (
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, new_uri)],
        "Moved to versioned API",
    )
        .into_response()
}

/// Redirect POST requests for single location upload.
pub async fn redirect_locations() -> Response {
    (
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, "/api/v1/locations")],
        "Moved to versioned API",
    )
        .into_response()
}

/// Redirect POST requests for batch location upload.
pub async fn redirect_locations_batch() -> Response {
    (
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, "/api/v1/locations/batch")],
        "Moved to versioned API",
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redirect_devices_register() {
        let response = redirect_devices_register().await;
        assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/api/v1/devices/register"
        );
    }

    #[tokio::test]
    async fn test_redirect_locations() {
        let response = redirect_locations().await;
        assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/api/v1/locations"
        );
    }

    #[tokio::test]
    async fn test_redirect_locations_batch() {
        let response = redirect_locations_batch().await;
        assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/api/v1/locations/batch"
        );
    }

    #[tokio::test]
    async fn test_redirect_devices_list_without_query() {
        let uri: Uri = "/api/devices".parse().unwrap();
        let response = redirect_devices_list(uri).await;
        assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/api/v1/devices"
        );
    }

    #[tokio::test]
    async fn test_redirect_devices_list_with_query() {
        let uri: Uri = "/api/devices?group_id=family".parse().unwrap();
        let response = redirect_devices_list(uri).await;
        assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/api/v1/devices?group_id=family"
        );
    }

    #[tokio::test]
    async fn test_redirect_to_v1_simple_path() {
        let uri: Uri = "/api/users".parse().unwrap();
        let response = redirect_to_v1(uri).await;
        assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/api/v1/users"
        );
    }

    #[tokio::test]
    async fn test_redirect_to_v1_with_query() {
        let uri: Uri = "/api/users?page=1&limit=10".parse().unwrap();
        let response = redirect_to_v1(uri).await;
        assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/api/v1/users?page=1&limit=10"
        );
    }
}
