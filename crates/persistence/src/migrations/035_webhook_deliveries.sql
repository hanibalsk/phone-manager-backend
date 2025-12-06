-- Story 15.3: Webhook Delivery Logging and Retry
-- Migration to create webhook_deliveries table for tracking webhook delivery attempts

-- Create webhook_deliveries table
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

    CONSTRAINT webhook_deliveries_status_check CHECK (status IN ('pending', 'success', 'failed')),
    CONSTRAINT webhook_deliveries_attempts_check CHECK (attempts >= 0)
);

-- Index for looking up deliveries by webhook
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_webhook_id
    ON webhook_deliveries(webhook_id);

-- Partial index for pending deliveries (most common query for retries)
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_pending
    ON webhook_deliveries(status)
    WHERE status = 'pending';

-- Index for retry queue - find deliveries due for retry
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_next_retry
    ON webhook_deliveries(next_retry_at)
    WHERE next_retry_at IS NOT NULL AND status = 'pending';

-- Index for cleanup job - find old deliveries
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_created_at
    ON webhook_deliveries(created_at);

-- Index for looking up deliveries by event
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_event_id
    ON webhook_deliveries(event_id)
    WHERE event_id IS NOT NULL;

-- Comment on table
COMMENT ON TABLE webhook_deliveries IS 'Tracks webhook delivery attempts with retry support';
COMMENT ON COLUMN webhook_deliveries.status IS 'pending = awaiting delivery/retry, success = delivered, failed = permanently failed';
COMMENT ON COLUMN webhook_deliveries.next_retry_at IS 'When to attempt next retry (null if not scheduled)';
