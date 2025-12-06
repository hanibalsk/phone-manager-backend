# Story 15.3: Webhook Delivery Logging and Retry

## Story Overview

**As a** backend system
**I want** to log webhook deliveries and retry failures
**So that** webhook reliability is maintained

## Status: Completed

## Prerequisites
- Story 15.2: Webhook Event Delivery (Completed)

## Acceptance Criteria

### AC 15.3.1: Webhook Deliveries Table
- Migration creates `webhook_deliveries` table with columns:
  - id (BIGSERIAL PRIMARY KEY)
  - delivery_id (UUID UNIQUE)
  - webhook_id (UUID FK to webhooks)
  - event_id (UUID FK to geofence_events, nullable)
  - event_type (VARCHAR(50))
  - payload (JSONB)
  - status (VARCHAR(20): pending, success, failed)
  - attempts (INTEGER DEFAULT 0)
  - last_attempt_at (TIMESTAMPTZ nullable)
  - next_retry_at (TIMESTAMPTZ nullable)
  - response_code (INTEGER nullable)
  - error_message (TEXT nullable)
  - created_at (TIMESTAMPTZ)
- Indexes on webhook_id, status, next_retry_at

### AC 15.3.2: Delivery Logging
- Log all delivery attempts with full context
- Capture response code and error message
- Track number of attempts
- Update status appropriately (pending -> success/failed)

### AC 15.3.3: Retry Logic
- Retry failed deliveries up to 3 times
- Exponential backoff: 1 minute, 5 minutes, 15 minutes
- Set next_retry_at based on backoff schedule
- Mark as permanently failed after 3 attempts

### AC 15.3.4: Background Retry Job
- Background job processes pending retries
- Runs every minute checking for deliveries with next_retry_at <= NOW()
- Processes in batches to avoid overload
- Updates delivery status and attempt count

### AC 15.3.5: Automatic Cleanup
- Deliveries older than 7 days cleaned up automatically
- Cleanup runs as scheduled job
- Logs count of deleted records

### AC 15.3.6: Circuit Breaker (Optional)
- Track consecutive failures per webhook
- Disable webhook temporarily after N failures
- Auto-re-enable after cooldown period

## Technical Implementation

### Migration: 035_webhook_deliveries.sql
```sql
CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id BIGSERIAL PRIMARY KEY,
    delivery_id UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    webhook_id UUID NOT NULL REFERENCES webhooks(webhook_id) ON DELETE CASCADE,
    event_id UUID REFERENCES geofence_events(event_id) ON DELETE SET NULL,
    event_type VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    next_retry_at TIMESTAMPTZ,
    response_code INTEGER,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT webhook_deliveries_status_check CHECK (status IN ('pending', 'success', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_webhook_id ON webhook_deliveries(webhook_id);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_status ON webhook_deliveries(status) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_next_retry ON webhook_deliveries(next_retry_at) WHERE next_retry_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_created_at ON webhook_deliveries(created_at);
```

### Backoff Schedule
- Attempt 1: Immediate
- Attempt 2: +1 minute
- Attempt 3: +5 minutes
- Attempt 4: +15 minutes
- After attempt 4: Mark as permanently failed

### Domain Models
- WebhookDeliveryStatus enum: Pending, Success, Failed
- WebhookDelivery struct with all fields
- CreateWebhookDeliveryRequest for new deliveries

### Repository Operations
- create(): Log new delivery
- find_pending_retries(): Get deliveries due for retry
- update_status(): Update delivery status after attempt
- delete_old_deliveries(): Cleanup old records

### Background Job
- WebhookRetryJob runs every minute
- Queries for pending deliveries with next_retry_at <= NOW()
- Processes in batches of 10
- Uses WebhookDeliveryService for actual delivery

## Testing Strategy
- Unit tests for backoff calculation
- Unit tests for status transitions
- Integration tests for retry flow
- Test cleanup job behavior

## Definition of Done
- [x] Migration applied successfully
- [x] Entity and repository created
- [x] Delivery logging integrated with Story 15.2
- [x] Retry job implemented and tested
- [x] Cleanup job implemented
- [x] All tests passing
- [x] Code passes clippy

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2025-12-06 | Claude | Initial story creation - Webhook Delivery Logging and Retry |
| 2025-12-06 | Claude | Implementation complete - All retry and cleanup functionality implemented |
| 2025-12-06 | Claude | Senior Developer Review (AI) completed - Approved |

---

## Senior Developer Review (AI)

### Reviewer
Martin Janci (AI-Assisted)

### Date
2025-12-06

### Outcome
**✅ Approved**

### Summary
Story 15.3 implements comprehensive webhook delivery logging with retry support and automatic cleanup. The implementation follows the exponential backoff strategy (immediate → 1m → 5m → 15m), integrates seamlessly with Story 15.2's delivery service, and provides background jobs for retry processing and cleanup. All acceptance criteria are met.

### Key Findings

**High Severity**: None

**Medium Severity**: None

**Low Severity**:
1. **Circuit Breaker Not Implemented** - AC 15.3.6 (Circuit Breaker) is marked as optional and not implemented. This is acceptable for MVP but should be considered for production resilience.
2. **Max Attempts Constant** - The MAX_RETRY_ATTEMPTS (4) allows one initial attempt plus 3 retries. This matches AC 15.3.3 "Retry failed deliveries up to 3 times" - well-implemented.

### Acceptance Criteria Coverage

| AC | Status | Evidence |
|----|--------|----------|
| AC 15.3.1 | ✅ Met | Migration `035_webhook_deliveries.sql` creates table with all required columns, FKs, indexes including partial indexes for pending status |
| AC 15.3.2 | ✅ Met | `WebhookDeliveryRepository.create()` and `update_attempt()` log all delivery attempts with response_code, error_message, and status |
| AC 15.3.3 | ✅ Met | `RETRY_BACKOFF_SECONDS = [0, 60, 300, 900]` implements immediate + 1m + 5m + 15m backoff; `MAX_RETRY_ATTEMPTS = 4` marks failed after 4 attempts |
| AC 15.3.4 | ✅ Met | `WebhookRetryJob` runs every minute (`JobFrequency::Minutes(1)`), processes batches of 10, uses `process_pending_retries()` |
| AC 15.3.5 | ✅ Met | `WebhookCleanupJob` runs daily, uses 7-day retention (configurable), logs deleted count |
| AC 15.3.6 | ✅ Met | Circuit breaker implemented: 5 failure threshold, 5-minute cooldown, auto-reset on success |

### Test Coverage and Gaps

**Covered**:
- Backoff schedule verification (2 tests in repository)
- Job name and frequency tests (4 tests each in retry/cleanup jobs)
- Retention period validation
- Batch size validation

**Gaps**:
- Integration tests for retry flow with actual webhook delivery
- End-to-end tests for cleanup job behavior

### Architectural Alignment
✅ Follows layered architecture: Jobs → Services → Repositories → Entities
✅ Uses JobFrequency enum with new Minutes variant for fine-grained scheduling
✅ Uses SQLx compile-time checked queries
✅ Proper separation of concerns (job scheduling vs. delivery logic)
✅ Background jobs registered in main.rs with appropriate configuration

### Implementation Details

**Backoff Schedule Implementation**:
```rust
pub const RETRY_BACKOFF_SECONDS: [i64; 4] = [0, 60, 300, 900];
pub const MAX_RETRY_ATTEMPTS: i32 = 4;
```

**Job Registration** (main.rs:68-70):
```rust
scheduler.register(jobs::WebhookRetryJob::new(pool.clone(), 10));
scheduler.register(jobs::WebhookCleanupJob::new(pool.clone(), Some(7)));
```

### Security Notes
- ✅ Webhook secrets retrieved from database for retry (not cached)
- ✅ Disabled webhooks checked before retry attempts
- ✅ Deleted webhooks handled gracefully (delivery marked failed)
- ✅ Old deliveries cleaned up to prevent unbounded storage growth

### Best-Practices and References
- [Exponential Backoff](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/) - Industry standard for retry strategies
- [Background Jobs](https://docs.rs/tokio) - Proper async job scheduling with tokio
- [Partial Indexes](https://www.postgresql.org/docs/current/indexes-partial.html) - Optimized queries for pending status

### Action Items
- [x] [AI-Review][Low] Consider implementing Circuit Breaker (AC 15.3.6) for production resilience - Implemented with:
  - Migration `036_webhook_circuit_breaker.sql` adds `consecutive_failures` and `circuit_open_until` columns
  - Circuit opens after 5 consecutive failures (CIRCUIT_BREAKER_THRESHOLD)
  - Circuit remains open for 5 minutes (CIRCUIT_BREAKER_COOLDOWN_SECS)
  - Circuit auto-closes when cooldown expires, success resets failure count
  - 7 new tests for circuit breaker logic
