-- Migration 039: Organization Admin Settings
-- Epic: Admin Portal Settings
-- Story: Per-organization admin settings (unlock PIN, daily limits, notifications)

-- Create organization_settings table for admin portal settings
CREATE TABLE organization_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL UNIQUE REFERENCES organizations(id) ON DELETE CASCADE,
    -- Security: unlock PIN is hashed with Argon2
    unlock_pin_hash VARCHAR(255),
    -- Daily screen time limit in minutes (0 = unlimited, max 1440 = 24 hours)
    default_daily_limit_minutes INTEGER NOT NULL DEFAULT 120,
    -- Enable/disable notifications for this organization
    notifications_enabled BOOLEAN NOT NULL DEFAULT true,
    -- Auto-approve unlock requests from devices
    auto_approve_unlock_requests BOOLEAN NOT NULL DEFAULT false,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Validation constraints
    CONSTRAINT chk_daily_limit_range CHECK (default_daily_limit_minutes >= 0 AND default_daily_limit_minutes <= 1440)
);

-- Index for fast lookup by organization
CREATE INDEX idx_organization_settings_org_id ON organization_settings(organization_id);

-- Trigger for updated_at
CREATE TRIGGER update_organization_settings_updated_at
    BEFORE UPDATE ON organization_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments
COMMENT ON TABLE organization_settings IS 'Per-organization admin settings for the admin portal';
COMMENT ON COLUMN organization_settings.unlock_pin_hash IS 'Argon2 hashed unlock PIN (write-only, cannot be retrieved)';
COMMENT ON COLUMN organization_settings.default_daily_limit_minutes IS 'Default daily screen time limit in minutes (0 = unlimited)';
COMMENT ON COLUMN organization_settings.notifications_enabled IS 'Enable push notifications for this organization';
COMMENT ON COLUMN organization_settings.auto_approve_unlock_requests IS 'Automatically approve device unlock requests';
