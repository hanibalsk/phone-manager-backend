-- Migration: 031_audit_logs.sql
-- Story 13.9: Audit Logging System
-- Comprehensive audit logging for all administrative actions

-- Create actor type enum
CREATE TYPE audit_actor_type AS ENUM ('user', 'system', 'api_key');

-- Create audit_logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actor_id UUID,
    actor_type audit_actor_type NOT NULL,
    actor_email VARCHAR(255),
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(30) NOT NULL,
    resource_id VARCHAR(255),
    resource_name VARCHAR(255),
    changes JSONB,
    metadata JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient querying by organization and timestamp
CREATE INDEX IF NOT EXISTS idx_audit_logs_org_timestamp
    ON audit_logs(organization_id, timestamp DESC);

-- Index for filtering by actor
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor
    ON audit_logs(organization_id, actor_id);

-- Index for filtering by action type
CREATE INDEX IF NOT EXISTS idx_audit_logs_action
    ON audit_logs(organization_id, action);

-- Index for filtering by resource
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource
    ON audit_logs(organization_id, resource_type, resource_id);

-- Index for timestamp range queries (useful for exports)
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp
    ON audit_logs(timestamp DESC);

-- Comment on table
COMMENT ON TABLE audit_logs IS 'Immutable audit log for all administrative actions in organizations';
COMMENT ON COLUMN audit_logs.actor_type IS 'Type of actor: user (human), system (automated), api_key (external integration)';
COMMENT ON COLUMN audit_logs.action IS 'Action format: resource.operation (e.g., device.assign, policy.update)';
COMMENT ON COLUMN audit_logs.changes IS 'JSON object with before/after values: {"field": {"old": "val1", "new": "val2"}}';
COMMENT ON COLUMN audit_logs.metadata IS 'Additional context: request_id, trace_id, etc.';
