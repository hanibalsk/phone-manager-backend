//! Database metrics collection.
//!
//! Provides functions for recording database-related metrics.

use metrics::{gauge, histogram};
use sqlx::PgPool;
use std::time::Instant;

/// Record database query duration.
///
/// Call this function after executing a query to record its duration.
pub fn record_query_duration(query_name: &str, duration_secs: f64) {
    histogram!(
        "database_query_duration_seconds",
        "query" => query_name.to_string()
    )
    .record(duration_secs);
}

/// Record database connection pool metrics.
///
/// Call this function periodically to track pool health.
pub fn record_pool_metrics(pool: &PgPool) {
    let size = pool.size() as usize;
    let idle = pool.num_idle();
    let active = size.saturating_sub(idle);

    gauge!("database_connections_active").set(active as f64);
    gauge!("database_connections_idle").set(idle as f64);
    gauge!("database_connections_total").set(size as f64);
}

/// A helper to time database operations and record metrics.
///
/// Usage:
/// ```ignore
/// let timer = QueryTimer::new("find_device_by_id");
/// let result = sqlx::query_as::<_, DeviceEntity>(...).fetch_optional(&pool).await;
/// timer.record();
/// result
/// ```
pub struct QueryTimer {
    query_name: String,
    start: Instant,
}

impl QueryTimer {
    /// Create a new timer for the given query name.
    pub fn new(query_name: impl Into<String>) -> Self {
        Self {
            query_name: query_name.into(),
            start: Instant::now(),
        }
    }

    /// Record the elapsed duration to metrics.
    pub fn record(self) {
        let duration = self.start.elapsed().as_secs_f64();
        record_query_duration(&self.query_name, duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_timer_creation() {
        let timer = QueryTimer::new("test_query");
        assert_eq!(timer.query_name, "test_query");
    }

    #[test]
    fn test_query_timer_with_string() {
        let name = String::from("test_query");
        let timer = QueryTimer::new(name);
        assert_eq!(timer.query_name, "test_query");
    }
}
