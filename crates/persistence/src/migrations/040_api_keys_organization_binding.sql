-- Migration: Add organization_id to api_keys for per-organization key management
-- This allows organizations to manage their own API keys for programmatic access

-- Add organization_id FK to api_keys
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Index for organization API key lookups
CREATE INDEX IF NOT EXISTS idx_api_keys_organization_id ON api_keys(organization_id) WHERE organization_id IS NOT NULL;

-- Composite index for org + active keys (efficient for listing active keys per org)
CREATE INDEX IF NOT EXISTS idx_api_keys_org_active ON api_keys(organization_id, is_active) WHERE organization_id IS NOT NULL;
