//! Webhook delivery entity definitions.
//!
//! Story 15.3: Webhook Delivery Logging and Retry
//! Maps to the webhook_deliveries table for tracking delivery attempts.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Database entity for webhook_deliveries table.
#[derive(Debug, Clone, FromRow)]
pub struct WebhookDeliveryEntity {
    pub id: i64,
    pub delivery_id: Uuid,
    pub webhook_id: Uuid,
    pub event_id: Option<Uuid>,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub attempts: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub response_code: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Delivery status values.
pub const STATUS_PENDING: &str = "pending";
pub const STATUS_SUCCESS: &str = "success";
pub const STATUS_FAILED: &str = "failed";

/// Maximum number of retry attempts.
pub const MAX_RETRY_ATTEMPTS: i32 = 4;

/// Backoff intervals in seconds for each retry attempt.
/// Attempt 1: immediate, Attempt 2: 60s, Attempt 3: 300s, Attempt 4: 900s
pub const RETRY_BACKOFF_SECONDS: [i64; 4] = [0, 60, 300, 900];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_constants() {
        assert_eq!(STATUS_PENDING, "pending");
        assert_eq!(STATUS_SUCCESS, "success");
        assert_eq!(STATUS_FAILED, "failed");
    }

    #[test]
    fn test_retry_constants() {
        assert_eq!(MAX_RETRY_ATTEMPTS, 4);
        assert_eq!(RETRY_BACKOFF_SECONDS[0], 0); // Immediate
        assert_eq!(RETRY_BACKOFF_SECONDS[1], 60); // 1 minute
        assert_eq!(RETRY_BACKOFF_SECONDS[2], 300); // 5 minutes
        assert_eq!(RETRY_BACKOFF_SECONDS[3], 900); // 15 minutes
    }
}
