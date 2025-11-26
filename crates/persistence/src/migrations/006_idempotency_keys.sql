-- Migration: Create idempotency_keys table for request deduplication
-- This table stores idempotency keys to prevent duplicate location uploads

-- Idempotency keys table
CREATE TABLE idempotency_keys (
    id              BIGSERIAL PRIMARY KEY,
    key_hash        VARCHAR(64) NOT NULL UNIQUE, -- SHA-256 hash of the idempotency key
    device_id       UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    response_body   JSONB NOT NULL,              -- Cached response to return
    response_status SMALLINT NOT NULL DEFAULT 200,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '24 hours')
);

-- Index for fast lookup by key hash
CREATE INDEX idx_idempotency_keys_key_hash ON idempotency_keys(key_hash);

-- Index for cleanup of expired keys
CREATE INDEX idx_idempotency_keys_expires_at ON idempotency_keys(expires_at);

-- Index for device-specific key lookups
CREATE INDEX idx_idempotency_keys_device_id ON idempotency_keys(device_id);

COMMENT ON TABLE idempotency_keys IS 'Stores idempotency keys for location upload deduplication';
COMMENT ON COLUMN idempotency_keys.key_hash IS 'SHA-256 hash of the Idempotency-Key header value';
COMMENT ON COLUMN idempotency_keys.response_body IS 'Cached JSON response body for duplicate requests';
COMMENT ON COLUMN idempotency_keys.expires_at IS 'Key expiration time (24 hours from creation)';
