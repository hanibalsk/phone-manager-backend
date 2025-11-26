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
