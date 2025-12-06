# Story 15.3: Webhook Delivery Logging and Retry

## Story Overview

**As a** backend system
**I want** to log webhook deliveries and retry failures
**So that** webhook reliability is maintained

## Status: Ready for Review

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
