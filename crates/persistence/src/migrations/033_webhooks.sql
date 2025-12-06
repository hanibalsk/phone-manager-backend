-- Migration: 033_webhooks.sql
-- Story 15.1: Webhook Registration and Management API
-- Create webhooks table for external system integrations

-- Create webhooks table
CREATE TABLE IF NOT EXISTS webhooks (
    id BIGSERIAL PRIMARY KEY,
    webhook_id UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    owner_device_id UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    target_url VARCHAR(2048) NOT NULL,
    secret VARCHAR(256) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unique constraint: prevent duplicate names per device
    CONSTRAINT webhooks_owner_device_id_name_key UNIQUE (owner_device_id, name),

    -- Security constraint: only HTTPS URLs allowed
    CONSTRAINT webhooks_target_url_https_check CHECK (target_url LIKE 'https://%')
);

-- Index for efficient lookups by owner device
CREATE INDEX IF NOT EXISTS idx_webhooks_owner_device_id
    ON webhooks(owner_device_id);

-- Index for ordering by creation date
CREATE INDEX IF NOT EXISTS idx_webhooks_created_at
    ON webhooks(created_at DESC);

-- Comment on table and columns
COMMENT ON TABLE webhooks IS 'Webhook endpoints for external system integrations (Home Assistant, n8n, etc.)';
COMMENT ON COLUMN webhooks.webhook_id IS 'Public UUID identifier for the webhook';
COMMENT ON COLUMN webhooks.owner_device_id IS 'Device that owns this webhook';
COMMENT ON COLUMN webhooks.name IS 'User-friendly name for the webhook (unique per device)';
COMMENT ON COLUMN webhooks.target_url IS 'HTTPS URL to send webhook events to';
COMMENT ON COLUMN webhooks.secret IS 'Secret key for HMAC-SHA256 signature verification';
COMMENT ON COLUMN webhooks.enabled IS 'Whether webhook deliveries are enabled';
