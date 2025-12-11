-- Migration 045: Organization Suspension Support
-- Story AP-2.7: Suspend/Reactivate Organization
-- Adds suspension tracking fields to organizations table

-- Add suspension fields
ALTER TABLE organizations
    ADD COLUMN IF NOT EXISTS suspended_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS suspended_by UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS suspension_reason TEXT;

-- Create index for suspended organizations lookup
CREATE INDEX IF NOT EXISTS idx_organizations_suspended ON organizations(suspended_at) WHERE suspended_at IS NOT NULL;

-- Add comments
COMMENT ON COLUMN organizations.suspended_at IS 'Timestamp when organization was suspended (null = not suspended)';
COMMENT ON COLUMN organizations.suspended_by IS 'User who performed the suspension action';
COMMENT ON COLUMN organizations.suspension_reason IS 'Optional reason for the suspension';
