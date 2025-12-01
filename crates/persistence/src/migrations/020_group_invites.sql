-- Migration: 020_group_invites
-- Description: Create group_invites table for invitation codes
-- Created: 2025-12-01

-- Group invites table
CREATE TABLE IF NOT EXISTS group_invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    code VARCHAR(11) NOT NULL,  -- Format: XXX-XXX-XXX
    preset_role group_role NOT NULL DEFAULT 'member',
    max_uses INTEGER NOT NULL DEFAULT 1,
    current_uses INTEGER NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT true,

    CONSTRAINT uq_group_invites_code UNIQUE (code),
    CONSTRAINT chk_group_invites_max_uses CHECK (max_uses BETWEEN 1 AND 100),
    CONSTRAINT chk_group_invites_current_uses CHECK (current_uses >= 0 AND current_uses <= max_uses)
);

-- Index for looking up invites by code (for join)
CREATE INDEX IF NOT EXISTS idx_group_invites_code ON group_invites(code) WHERE is_active = true;

-- Index for listing invites by group
CREATE INDEX IF NOT EXISTS idx_group_invites_group ON group_invites(group_id) WHERE is_active = true;

-- Index for cleanup of expired invites
CREATE INDEX IF NOT EXISTS idx_group_invites_expires ON group_invites(expires_at) WHERE is_active = true;

-- Comments
COMMENT ON TABLE group_invites IS 'Group invitation codes for joining groups';
COMMENT ON COLUMN group_invites.code IS 'Unique invite code in XXX-XXX-XXX format';
COMMENT ON COLUMN group_invites.preset_role IS 'Role assigned when joining with this invite';
COMMENT ON COLUMN group_invites.max_uses IS 'Maximum number of times this invite can be used';
COMMENT ON COLUMN group_invites.current_uses IS 'Current number of times this invite has been used';
COMMENT ON COLUMN group_invites.expires_at IS 'When this invite expires';
COMMENT ON COLUMN group_invites.is_active IS 'Soft delete flag';
