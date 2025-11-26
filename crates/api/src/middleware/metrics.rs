//! Prometheus metrics middleware.
//!
//! Provides HTTP request/response metrics collection and export.

use axum::{
    body::Body,
    extract::MatchedPath,
    http::{Method, Request},
    middleware::Next,
    response::{IntoResponse, Response},
};
use metrics::{counter, histogram};
use std::time::Instant;

/// Middleware to record HTTP request metrics.
///
/// Records the following metrics:
/// - `http_requests_total`: Counter with labels (method, path, status)
/// - `http_request_duration_seconds`: Histogram with labels (method, path)
pub async fn metrics_middleware(req: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());

    let response = next.run(req).await;

    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // Record metrics
    let method_str = method_to_str(&method);

    counter!(
        "http_requests_total",
        "method" => method_str.to_string(),
        "path" => path.clone(),
        "status" => status.clone()
    )
    .increment(1);

    histogram!(
        "http_request_duration_seconds",
        "method" => method_str.to_string(),
        "path" => path
    )
    .record(duration);

    response
}

/// Convert HTTP method to string for metric labels.
fn method_to_str(method: &Method) -> &'static str {
    match *method {
        Method::GET => "GET",
        Method::POST => "POST",
        Method::PUT => "PUT",
        Method::DELETE => "DELETE",
        Method::PATCH => "PATCH",
        Method::HEAD => "HEAD",
        Method::OPTIONS => "OPTIONS",
        _ => "OTHER",
    }
}

/// Record a custom business metric for locations uploaded.
#[allow(dead_code)] // Available for use in route handlers
pub fn record_locations_uploaded(count: usize) {
    counter!("locations_uploaded_total").increment(count as u64);
}

/// Record a custom business metric for devices registered.
#[allow(dead_code)] // Available for use in route handlers
pub fn record_device_registered() {
    counter!("devices_registered_total").increment(1);
}

/// Record database query duration.
#[allow(dead_code)] // Available for use in repositories
pub fn record_database_query_duration(query_name: &str, duration_secs: f64) {
    histogram!(
        "database_query_duration_seconds",
        "query" => query_name.to_string()
    )
    .record(duration_secs);
}

/// Record database connection pool metrics.
#[allow(dead_code)] // Available for use in background jobs
pub fn record_connection_pool_metrics(active: u32, idle: u32) {
    // Using gauge-style recording via counter with known value
    // In metrics 0.22+, we use gauge! macro
    metrics::gauge!("database_connections_active").set(active as f64);
    metrics::gauge!("database_connections_idle").set(idle as f64);
}

/// Handler for /metrics endpoint that returns Prometheus text format.
pub async fn metrics_handler() -> impl IntoResponse {
    // Get the handle from the global recorder
    // This is set up during app initialization
    if let Some(handle) = PROMETHEUS_HANDLE.get() {
        let output = handle.render();
        (
            axum::http::StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
            output,
        )
    } else {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [(axum::http::header::CONTENT_TYPE, "text/plain")],
            "Metrics not initialized".to_string(),
        )
    }
}

use std::sync::OnceLock;

static PROMETHEUS_HANDLE: OnceLock<metrics_exporter_prometheus::PrometheusHandle> = OnceLock::new();

/// Initialize the Prometheus metrics recorder.
///
/// Must be called once during application startup before any metrics are recorded.
/// Uses `install_recorder()` which sets up the global metrics recorder and returns
/// a handle that can be used to render Prometheus-formatted output.
pub fn init_metrics() {
    use metrics_exporter_prometheus::PrometheusBuilder;

    let builder = PrometheusBuilder::new();

    // Set histogram buckets for request duration
    let handle = builder
        .set_buckets(&[
            0.001, 0.005, 0.01, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0,
        ])
        .expect("Failed to set histogram buckets")
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    // Store the handle for the metrics endpoint
    if PROMETHEUS_HANDLE.set(handle).is_err() {
        panic!("Prometheus handle already initialized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_to_str() {
        assert_eq!(method_to_str(&Method::GET), "GET");
        assert_eq!(method_to_str(&Method::POST), "POST");
        assert_eq!(method_to_str(&Method::PUT), "PUT");
        assert_eq!(method_to_str(&Method::DELETE), "DELETE");
        assert_eq!(method_to_str(&Method::PATCH), "PATCH");
        assert_eq!(method_to_str(&Method::HEAD), "HEAD");
        assert_eq!(method_to_str(&Method::OPTIONS), "OPTIONS");
    }

    #[test]
    fn test_method_to_str_other() {
        // TRACE is not in our list, should return OTHER
        assert_eq!(method_to_str(&Method::TRACE), "OTHER");
    }
}
