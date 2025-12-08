-- Organization webhooks for organization-level event notifications
-- Similar to device webhooks but scoped to organizations

CREATE TABLE IF NOT EXISTS org_webhooks (
    id BIGSERIAL PRIMARY KEY,
    webhook_id UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    target_url VARCHAR(2048) NOT NULL,
    secret VARCHAR(256) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    event_types TEXT[] NOT NULL DEFAULT '{}',
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    circuit_open_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for organization webhook lookups
CREATE INDEX IF NOT EXISTS idx_org_webhooks_organization_id ON org_webhooks(organization_id);

-- Index for finding enabled webhooks for event delivery
CREATE INDEX IF NOT EXISTS idx_org_webhooks_org_enabled ON org_webhooks(organization_id, enabled) WHERE enabled = true;

-- Unique constraint: webhook name must be unique within organization
CREATE UNIQUE INDEX IF NOT EXISTS idx_org_webhooks_org_name ON org_webhooks(organization_id, LOWER(name));

-- Comment on table
COMMENT ON TABLE org_webhooks IS 'Organization-level webhooks for event notifications';
COMMENT ON COLUMN org_webhooks.event_types IS 'Array of event types to subscribe to (e.g., device.enrolled, member.joined)';
COMMENT ON COLUMN org_webhooks.consecutive_failures IS 'Counter for circuit breaker pattern';
COMMENT ON COLUMN org_webhooks.circuit_open_until IS 'When set, webhook is temporarily disabled until this time';
