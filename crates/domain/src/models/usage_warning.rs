//! Usage warning models for limit monitoring.
//!
//! Provides types for tracking resource usage against configured limits
//! and generating warnings when usage approaches thresholds.

use serde::{Deserialize, Serialize};

/// A warning about resource usage approaching configured limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWarning {
    /// Type of resource (e.g., "webhooks", "geofences", "devices")
    pub resource_type: String,
    /// Current count of resources
    pub current: i64,
    /// Maximum allowed limit
    pub limit: i64,
    /// Remaining capacity
    pub remaining: i64,
    /// Usage percentage (0.0 - 100.0)
    pub percentage: f64,
    /// Human-readable warning message
    pub message: String,
}

impl UsageWarning {
    /// Create a new usage warning.
    pub fn new(resource_type: &str, current: i64, limit: i64) -> Self {
        let remaining = (limit - current).max(0);
        let percentage = if limit > 0 {
            (current as f64 / limit as f64) * 100.0
        } else {
            0.0
        };

        let message = format!(
            "You have used {}/{} {} ({}% of limit). {} remaining.",
            current,
            limit,
            resource_type,
            percentage.round() as i64,
            remaining
        );

        Self {
            resource_type: resource_type.to_string(),
            current,
            limit,
            remaining,
            percentage,
            message,
        }
    }

    /// Check if usage exceeds the warning threshold.
    pub fn exceeds_threshold(&self, threshold_percent: u32) -> bool {
        self.percentage >= threshold_percent as f64
    }
}

/// Calculate usage percentage and optionally return a warning.
///
/// Returns `Some(UsageWarning)` if usage percentage >= threshold, otherwise `None`.
pub fn check_usage_warning(
    resource_type: &str,
    current: i64,
    limit: i64,
    threshold_percent: u32,
) -> Option<UsageWarning> {
    if limit <= 0 {
        return None;
    }

    let warning = UsageWarning::new(resource_type, current, limit);

    if warning.exceeds_threshold(threshold_percent) {
        Some(warning)
    } else {
        None
    }
}

/// A generic response wrapper that can include usage warnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseWithWarnings<T> {
    /// The main response data
    #[serde(flatten)]
    pub data: T,
    /// Optional usage warnings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<UsageWarning>>,
}

impl<T> ResponseWithWarnings<T> {
    /// Create a response without warnings.
    pub fn new(data: T) -> Self {
        Self {
            data,
            warnings: None,
        }
    }

    /// Create a response with a single warning.
    pub fn with_warning(data: T, warning: UsageWarning) -> Self {
        Self {
            data,
            warnings: Some(vec![warning]),
        }
    }

    /// Create a response with multiple warnings.
    pub fn with_warnings(data: T, warnings: Vec<UsageWarning>) -> Self {
        let warnings = if warnings.is_empty() {
            None
        } else {
            Some(warnings)
        };
        Self { data, warnings }
    }

    /// Add a warning if the condition is met.
    pub fn maybe_with_warning(data: T, warning: Option<UsageWarning>) -> Self {
        match warning {
            Some(w) => Self::with_warning(data, w),
            None => Self::new(data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_warning_new() {
        let warning = UsageWarning::new("webhooks", 8, 10);
        assert_eq!(warning.resource_type, "webhooks");
        assert_eq!(warning.current, 8);
        assert_eq!(warning.limit, 10);
        assert_eq!(warning.remaining, 2);
        assert!((warning.percentage - 80.0).abs() < 0.01);
        assert!(warning.message.contains("8/10"));
    }

    #[test]
    fn test_usage_warning_at_limit() {
        let warning = UsageWarning::new("geofences", 50, 50);
        assert_eq!(warning.remaining, 0);
        assert!((warning.percentage - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_usage_warning_over_limit() {
        // Should handle edge case gracefully
        let warning = UsageWarning::new("devices", 25, 20);
        assert_eq!(warning.remaining, 0);
        assert!(warning.percentage > 100.0);
    }

    #[test]
    fn test_exceeds_threshold() {
        let warning = UsageWarning::new("alerts", 8, 10);
        assert!(warning.exceeds_threshold(80));
        assert!(warning.exceeds_threshold(75));
        assert!(!warning.exceeds_threshold(85));
    }

    #[test]
    fn test_check_usage_warning_triggers() {
        let warning = check_usage_warning("webhooks", 8, 10, 80);
        assert!(warning.is_some());
        let w = warning.unwrap();
        assert_eq!(w.current, 8);
    }

    #[test]
    fn test_check_usage_warning_below_threshold() {
        let warning = check_usage_warning("webhooks", 5, 10, 80);
        assert!(warning.is_none());
    }

    #[test]
    fn test_check_usage_warning_zero_limit() {
        let warning = check_usage_warning("webhooks", 5, 0, 80);
        assert!(warning.is_none());
    }

    #[test]
    fn test_response_with_warnings_new() {
        #[derive(Debug, Clone, Serialize)]
        struct TestData {
            id: String,
        }

        let data = TestData {
            id: "test".to_string(),
        };
        let response = ResponseWithWarnings::new(data);
        assert!(response.warnings.is_none());
    }

    #[test]
    fn test_response_with_warnings_single() {
        #[derive(Debug, Clone, Serialize)]
        struct TestData {
            id: String,
        }

        let data = TestData {
            id: "test".to_string(),
        };
        let warning = UsageWarning::new("webhooks", 9, 10);
        let response = ResponseWithWarnings::with_warning(data, warning);
        assert!(response.warnings.is_some());
        assert_eq!(response.warnings.unwrap().len(), 1);
    }

    #[test]
    fn test_response_with_warnings_maybe() {
        #[derive(Debug, Clone, Serialize)]
        struct TestData {
            id: String,
        }

        let data = TestData {
            id: "test".to_string(),
        };

        // With warning
        let warning = Some(UsageWarning::new("webhooks", 9, 10));
        let response = ResponseWithWarnings::maybe_with_warning(data.clone(), warning);
        assert!(response.warnings.is_some());

        // Without warning
        let response = ResponseWithWarnings::maybe_with_warning(data, None);
        assert!(response.warnings.is_none());
    }
}
