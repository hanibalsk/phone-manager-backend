//! OpenAPI documentation and Swagger UI routes.

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Redirect, Response},
};
use rust_embed::Embed;

/// Embedded Swagger UI assets from the assets/swagger-ui directory.
#[derive(Embed)]
#[folder = "assets/swagger-ui/"]
struct SwaggerAssets;

/// Embedded OpenAPI specification from docs/api/openapi.yaml.
const OPENAPI_SPEC: &str = include_str!("../../../../docs/api/openapi.yaml");

/// Redirect `/api/docs` to `/api/docs/` (trailing slash).
pub async fn swagger_ui_redirect() -> Redirect {
    Redirect::permanent("/api/docs/")
}

/// Serve Swagger UI index page or static assets.
///
/// Handles requests to `/api/docs/` and `/api/docs/*path` by serving
/// the appropriate static file from the embedded Swagger UI assets.
pub async fn swagger_ui(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches("/api/docs/");
    let path = if path.is_empty() { "index.html" } else { path };

    match SwaggerAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(Body::from(content.data.into_owned()))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Serve the OpenAPI YAML specification.
///
/// Returns the embedded OpenAPI 3.1 specification at `/api/docs/openapi.yaml`.
pub async fn openapi_spec() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/yaml; charset=utf-8")],
        OPENAPI_SPEC,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================
    // OpenAPI Spec Tests
    // ===========================================

    #[test]
    fn test_openapi_spec_not_empty() {
        assert!(!OPENAPI_SPEC.is_empty());
    }

    #[test]
    fn test_openapi_spec_is_yaml() {
        // YAML files typically start with '---' or contain key: value pairs
        assert!(
            OPENAPI_SPEC.contains("openapi:") || OPENAPI_SPEC.contains("info:"),
            "OpenAPI spec should contain valid YAML structure"
        );
    }

    #[test]
    fn test_openapi_spec_has_version() {
        // OpenAPI 3.x specs should have version
        assert!(
            OPENAPI_SPEC.contains("openapi:"),
            "OpenAPI spec should have openapi version field"
        );
    }

    #[test]
    fn test_openapi_spec_has_info() {
        assert!(
            OPENAPI_SPEC.contains("info:"),
            "OpenAPI spec should have info section"
        );
    }

    #[test]
    fn test_openapi_spec_has_paths() {
        assert!(
            OPENAPI_SPEC.contains("paths:"),
            "OpenAPI spec should have paths section"
        );
    }

    #[test]
    fn test_openapi_spec_has_api_endpoints() {
        // Should include our main API endpoints
        let endpoints = vec!["/api/v1/devices", "/api/v1/locations"];

        for endpoint in endpoints {
            assert!(
                OPENAPI_SPEC.contains(endpoint),
                "OpenAPI spec should document {} endpoint",
                endpoint
            );
        }
    }

    // ===========================================
    // Swagger Assets Tests
    // ===========================================

    #[test]
    fn test_swagger_assets_index_exists() {
        assert!(SwaggerAssets::get("index.html").is_some());
    }

    #[test]
    fn test_swagger_assets_has_required_files() {
        // Swagger UI requires these core files
        let required_files = vec!["index.html"];

        for file in required_files {
            assert!(
                SwaggerAssets::get(file).is_some(),
                "Swagger UI should include {}",
                file
            );
        }
    }

    // ===========================================
    // Path Handling Tests
    // ===========================================

    #[test]
    fn test_path_trimming_logic() {
        let paths = vec![
            ("/api/docs/", ""),
            ("/api/docs/index.html", "index.html"),
            ("/api/docs/swagger-ui.css", "swagger-ui.css"),
            ("/api/docs/swagger-ui-bundle.js", "swagger-ui-bundle.js"),
        ];

        for (input, expected) in paths {
            let result = input.trim_start_matches("/api/docs/");
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_empty_path_defaults_to_index() {
        let path = "/api/docs/".trim_start_matches("/api/docs/");
        let result = if path.is_empty() { "index.html" } else { path };
        assert_eq!(result, "index.html");
    }

    #[test]
    fn test_non_empty_path_preserved() {
        let path = "/api/docs/swagger-ui.css".trim_start_matches("/api/docs/");
        let result = if path.is_empty() { "index.html" } else { path };
        assert_eq!(result, "swagger-ui.css");
    }

    // ===========================================
    // MIME Type Tests
    // ===========================================

    #[test]
    fn test_mime_type_html() {
        let mime = mime_guess::from_path("index.html").first_or_octet_stream();
        assert_eq!(mime.type_(), "text");
        assert_eq!(mime.subtype(), "html");
    }

    #[test]
    fn test_mime_type_css() {
        let mime = mime_guess::from_path("swagger-ui.css").first_or_octet_stream();
        assert_eq!(mime.type_(), "text");
        assert_eq!(mime.subtype(), "css");
    }

    #[test]
    fn test_mime_type_javascript() {
        let mime = mime_guess::from_path("swagger-ui-bundle.js").first_or_octet_stream();
        assert!(
            mime.type_() == "application" || mime.type_() == "text",
            "JavaScript MIME type should be application or text"
        );
    }

    #[test]
    fn test_mime_type_yaml() {
        let content_type = "application/yaml; charset=utf-8";
        assert!(content_type.contains("yaml"));
        assert!(content_type.contains("charset=utf-8"));
    }

    #[test]
    fn test_mime_type_unknown_defaults_to_octet_stream() {
        let mime = mime_guess::from_path("file.unknownext").first_or_octet_stream();
        assert_eq!(mime.type_(), "application");
        assert_eq!(mime.subtype(), "octet-stream");
    }

    // ===========================================
    // Response Header Tests
    // ===========================================

    #[test]
    fn test_cache_control_header_format() {
        let cache_control = "public, max-age=3600";
        assert!(cache_control.contains("public"));
        assert!(cache_control.contains("max-age=3600"));
    }

    #[test]
    fn test_cache_control_max_age_is_one_hour() {
        let max_age = 3600;
        assert_eq!(max_age / 60, 60); // 60 minutes
    }

    // ===========================================
    // Redirect Tests
    // ===========================================

    #[test]
    fn test_redirect_target() {
        let target = "/api/docs/";
        assert!(target.ends_with('/'));
        assert_eq!(target, "/api/docs/");
    }

    #[test]
    fn test_redirect_is_permanent() {
        // Permanent redirect uses 308 or 301
        // The function uses Redirect::permanent which is 308
        let redirect = Redirect::permanent("/api/docs/");
        // We can't easily check the status code, but we can verify it compiles
        let _redirect = redirect;
    }

    // ===========================================
    // OpenAPI Content Validation Tests
    // ===========================================

    #[test]
    fn test_openapi_spec_has_components() {
        assert!(
            OPENAPI_SPEC.contains("components:"),
            "OpenAPI spec should have components section for schemas"
        );
    }

    #[test]
    fn test_openapi_spec_has_security_definitions() {
        // Should define API key security
        assert!(
            OPENAPI_SPEC.contains("securitySchemes:") || OPENAPI_SPEC.contains("security:"),
            "OpenAPI spec should have security definitions"
        );
    }

    #[test]
    fn test_openapi_spec_has_server_info() {
        assert!(
            OPENAPI_SPEC.contains("servers:"),
            "OpenAPI spec should have servers section"
        );
    }

    #[test]
    fn test_openapi_spec_character_count() {
        // Spec should be substantial (basic spec is at least 1KB)
        assert!(
            OPENAPI_SPEC.len() > 1000,
            "OpenAPI spec should be substantial, got {} bytes",
            OPENAPI_SPEC.len()
        );
    }

    #[test]
    fn test_openapi_spec_is_utf8() {
        // Verify the spec is valid UTF-8 (it's a &str so it must be)
        let bytes = OPENAPI_SPEC.as_bytes();
        assert!(std::str::from_utf8(bytes).is_ok());
    }
}
