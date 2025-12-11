-- Migration 046: Organization User Suspension Support
-- Story AP-3.5: Suspend User
-- Story AP-3.6: Reactivate User
-- Adds suspension tracking fields to org_users table

-- Add suspension fields
ALTER TABLE org_users
    ADD COLUMN IF NOT EXISTS suspended_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS suspended_by UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS suspension_reason TEXT;

-- Create index for suspended users lookup
CREATE INDEX IF NOT EXISTS idx_org_users_suspended ON org_users(organization_id, suspended_at) WHERE suspended_at IS NOT NULL;

-- Add comments
COMMENT ON COLUMN org_users.suspended_at IS 'Timestamp when user was suspended in this organization (null = not suspended)';
COMMENT ON COLUMN org_users.suspended_by IS 'Admin who performed the suspension action';
COMMENT ON COLUMN org_users.suspension_reason IS 'Optional reason for the suspension';
