-- Migration: 036_webhook_circuit_breaker.sql
-- Story 15.3: AC 15.3.6 - Circuit Breaker for Webhooks
--
-- Adds circuit breaker functionality to automatically disable webhooks
-- after consecutive failures and re-enable after a cooldown period.

-- Add circuit breaker columns to webhooks table
ALTER TABLE webhooks
ADD COLUMN consecutive_failures INTEGER NOT NULL DEFAULT 0,
ADD COLUMN circuit_open_until TIMESTAMPTZ NULL;

-- Add comment for documentation
COMMENT ON COLUMN webhooks.consecutive_failures IS 'Number of consecutive delivery failures since last success';
COMMENT ON COLUMN webhooks.circuit_open_until IS 'When circuit breaker is open, this is when it will auto-close';

-- Create partial index for efficient lookup of webhooks with open circuits
-- Used by cleanup job or monitoring
CREATE INDEX IF NOT EXISTS idx_webhooks_circuit_open
ON webhooks(circuit_open_until)
WHERE circuit_open_until IS NOT NULL;
