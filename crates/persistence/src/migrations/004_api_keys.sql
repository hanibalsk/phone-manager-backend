-- Migration 004: API Keys table
-- Stores hashed API keys for authentication

CREATE TABLE api_keys (
    id              BIGSERIAL PRIMARY KEY,
    key_hash        VARCHAR(128) NOT NULL UNIQUE,
    key_prefix      VARCHAR(8) NOT NULL,
    name            VARCHAR(100) NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    is_admin        BOOLEAN NOT NULL DEFAULT FALSE,
    last_used_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ
);

-- Index for active key lookup (primary authentication query)
CREATE INDEX idx_api_keys_active ON api_keys(key_hash) WHERE is_active = TRUE;

-- Index for key prefix identification (admin operations)
CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix);
