//! Static frontend file serving with hostname-based environment selection.
//!
//! Serves Next.js static export files from configurable directories based on
//! the Host header. Supports SPA fallback (serves index.html for non-file routes).

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, warn};

use crate::app::AppState;
use crate::config::FrontendConfig;

/// Determines the frontend directory based on the Host header.
pub fn resolve_frontend_dir(config: &FrontendConfig, host: Option<&str>) -> PathBuf {
    let environment = match host {
        Some(h) => {
            // Strip port if present (e.g., "admin.example.com:8080" -> "admin.example.com")
            let host_without_port = h.split(':').next().unwrap_or(h);

            if !config.staging_hostname.is_empty()
                && host_without_port == config.staging_hostname
            {
                "staging"
            } else if !config.production_hostname.is_empty()
                && host_without_port == config.production_hostname
            {
                "production"
            } else {
                debug!(
                    host = %h,
                    default = %config.default_environment,
                    "Unknown hostname, using default environment"
                );
                &config.default_environment
            }
        }
        None => {
            warn!("No Host header, using default environment");
            &config.default_environment
        }
    };

    PathBuf::from(&config.base_dir).join(environment)
}

/// Serve static files with SPA fallback.
pub async fn serve_frontend(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    let config = &state.config.frontend;

    // Get Host header for environment selection
    let host = headers.get(header::HOST).and_then(|h| h.to_str().ok());

    let base_dir = resolve_frontend_dir(config, host);

    // Check if directory exists
    if !base_dir.exists() {
        warn!(dir = %base_dir.display(), "Frontend directory does not exist");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Frontend not available",
        )
            .into_response();
    }

    let path = uri.path().trim_start_matches('/');

    // Try to serve the requested file
    let file_path = if path.is_empty() {
        base_dir.join("index.html")
    } else {
        base_dir.join(path)
    };

    // Security: prevent path traversal
    if !is_safe_path(&base_dir, &file_path) {
        warn!(
            requested_path = %file_path.display(),
            base_dir = %base_dir.display(),
            "Path traversal attempt detected"
        );
        return StatusCode::FORBIDDEN.into_response();
    }

    // Try to serve the exact file
    if let Ok(response) = serve_file(&file_path, config).await {
        return response;
    }

    // SPA fallback: serve index.html for non-file routes
    // (routes without file extensions are likely SPA routes)
    if !path.contains('.') {
        let index_path = base_dir.join("index.html");
        if let Ok(response) = serve_file(&index_path, config).await {
            return response;
        }
    }

    // File not found
    StatusCode::NOT_FOUND.into_response()
}

/// Serve a single file with appropriate cache headers.
async fn serve_file(path: &Path, config: &FrontendConfig) -> Result<Response, std::io::Error> {
    let content = fs::read(path).await?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    // Determine cache strategy based on file path
    let cache_control = if is_immutable_asset(path) {
        format!(
            "public, max-age={}, immutable",
            config.immutable_cache_max_age
        )
    } else {
        format!("public, max-age={}", config.mutable_cache_max_age)
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref())
        .header(header::CACHE_CONTROL, cache_control)
        .body(Body::from(content))
        .unwrap())
}

/// Check if path is within base directory (prevent path traversal).
fn is_safe_path(base: &Path, path: &Path) -> bool {
    // First check if the path exists - if it doesn't, we can't canonicalize it
    // so we do a simple prefix check on the normalized path
    if path.exists() {
        match (base.canonicalize(), path.canonicalize()) {
            (Ok(canonical_base), Ok(canonical_path)) => canonical_path.starts_with(canonical_base),
            _ => false,
        }
    } else {
        // For non-existent files, do a simple check:
        // Normalize the path by removing . and .. components
        let normalized = normalize_path(path);
        let base_normalized = normalize_path(base);
        normalized.starts_with(base_normalized)
    }
}

/// Normalize a path by removing . and .. components
fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                result.pop();
            }
            std::path::Component::CurDir => {}
            component => {
                result.push(component);
            }
        }
    }
    result
}

/// Check if file is an immutable asset (hashed filename).
/// Next.js static exports put hashed assets in _next/static/
fn is_immutable_asset(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("_next/static/") || path_str.contains("_next\\static\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> FrontendConfig {
        FrontendConfig {
            enabled: true,
            base_dir: "/app/frontend".to_string(),
            staging_hostname: "admin-staging.example.com".to_string(),
            production_hostname: "admin.example.com".to_string(),
            default_environment: "production".to_string(),
            immutable_cache_max_age: 31536000,
            mutable_cache_max_age: 60,
        }
    }

    #[test]
    fn test_resolve_frontend_dir_staging() {
        let config = test_config();
        let dir = resolve_frontend_dir(&config, Some("admin-staging.example.com"));
        assert_eq!(dir, PathBuf::from("/app/frontend/staging"));
    }

    #[test]
    fn test_resolve_frontend_dir_production() {
        let config = test_config();
        let dir = resolve_frontend_dir(&config, Some("admin.example.com"));
        assert_eq!(dir, PathBuf::from("/app/frontend/production"));
    }

    #[test]
    fn test_resolve_frontend_dir_with_port() {
        let config = test_config();
        let dir = resolve_frontend_dir(&config, Some("admin-staging.example.com:8080"));
        assert_eq!(dir, PathBuf::from("/app/frontend/staging"));
    }

    #[test]
    fn test_resolve_frontend_dir_unknown_host() {
        let config = test_config();
        let dir = resolve_frontend_dir(&config, Some("unknown.example.com"));
        assert_eq!(dir, PathBuf::from("/app/frontend/production")); // default
    }

    #[test]
    fn test_resolve_frontend_dir_no_host() {
        let config = test_config();
        let dir = resolve_frontend_dir(&config, None);
        assert_eq!(dir, PathBuf::from("/app/frontend/production")); // default
    }

    #[test]
    fn test_is_immutable_asset() {
        assert!(is_immutable_asset(Path::new(
            "/app/frontend/staging/_next/static/chunks/123.js"
        )));
        assert!(is_immutable_asset(Path::new(
            "/app/frontend/production/_next/static/css/abc.css"
        )));
        assert!(is_immutable_asset(Path::new(
            r"C:\frontend\production\_next\static\chunks\hash.js"
        )));
        assert!(!is_immutable_asset(Path::new(
            "/app/frontend/staging/index.html"
        )));
        assert!(!is_immutable_asset(Path::new(
            "/app/frontend/staging/favicon.ico"
        )));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path(Path::new("/app/frontend/../other")),
            PathBuf::from("/app/other")
        );
        assert_eq!(
            normalize_path(Path::new("/app/./frontend")),
            PathBuf::from("/app/frontend")
        );
        assert_eq!(
            normalize_path(Path::new("/app/frontend/staging")),
            PathBuf::from("/app/frontend/staging")
        );
    }

    #[test]
    fn test_is_safe_path_valid() {
        let base = Path::new("/app/frontend");
        let path = Path::new("/app/frontend/staging/index.html");
        // For non-existent paths, uses normalize check
        assert!(is_safe_path(base, path));
    }

    #[test]
    fn test_is_safe_path_traversal_attempt() {
        let base = Path::new("/app/frontend");
        let path = Path::new("/app/frontend/../etc/passwd");
        assert!(!is_safe_path(base, path));
    }
}
