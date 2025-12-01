-- Migration: Device Enrollment
-- Story 13.5: Device Enrollment Endpoint
-- Adds managed device support with enrollment status and device tokens

-- Create enrollment_status enum
CREATE TYPE enrollment_status AS ENUM ('pending', 'enrolled', 'suspended', 'retired');

-- Add new columns to devices table for managed devices
ALTER TABLE devices
    ADD COLUMN IF NOT EXISTS is_managed BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS enrollment_status enrollment_status,
    ADD COLUMN IF NOT EXISTS policy_id UUID REFERENCES device_policies(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS enrolled_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS enrolled_via_token_id UUID REFERENCES enrollment_tokens(id) ON DELETE SET NULL;

-- Create index for managed devices lookup
CREATE INDEX IF NOT EXISTS idx_devices_managed ON devices(organization_id, is_managed) WHERE is_managed = true;
CREATE INDEX IF NOT EXISTS idx_devices_policy ON devices(policy_id) WHERE policy_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_devices_enrollment_status ON devices(enrollment_status) WHERE enrollment_status IS NOT NULL;

-- Create device_tokens table for managed device authentication
CREATE TABLE IF NOT EXISTS device_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id BIGINT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    token VARCHAR(100) NOT NULL,
    token_prefix VARCHAR(20) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    CONSTRAINT device_tokens_token_unique UNIQUE (token)
);

-- Create indexes for device token lookup
CREATE INDEX IF NOT EXISTS idx_device_tokens_device ON device_tokens(device_id);
CREATE INDEX IF NOT EXISTS idx_device_tokens_prefix ON device_tokens(token_prefix);
CREATE INDEX IF NOT EXISTS idx_device_tokens_org ON device_tokens(organization_id);
CREATE INDEX IF NOT EXISTS idx_device_tokens_valid ON device_tokens(token) WHERE revoked_at IS NULL;

-- Add comments
COMMENT ON COLUMN devices.is_managed IS 'Whether device is managed by an organization';
COMMENT ON COLUMN devices.enrollment_status IS 'Current enrollment status for managed devices';
COMMENT ON COLUMN devices.policy_id IS 'Active policy applied to this device';
COMMENT ON COLUMN devices.enrolled_at IS 'Timestamp when device was enrolled';
COMMENT ON COLUMN devices.enrolled_via_token_id IS 'Reference to enrollment token used for enrollment';

COMMENT ON TABLE device_tokens IS 'Long-lived authentication tokens for managed devices';
COMMENT ON COLUMN device_tokens.token IS 'The full device token (dt_<base64>)';
COMMENT ON COLUMN device_tokens.token_prefix IS 'First 8 chars for identification';
COMMENT ON COLUMN device_tokens.expires_at IS 'Token expiry (typically 90 days from creation)';
COMMENT ON COLUMN device_tokens.last_used_at IS 'Last time token was used for authentication';
COMMENT ON COLUMN device_tokens.revoked_at IS 'Timestamp when token was revoked (null = active)';
