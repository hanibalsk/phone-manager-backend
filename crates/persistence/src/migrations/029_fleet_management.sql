-- Migration: Fleet Management
-- Story 13.7: Fleet Management Endpoints
-- Adds assigned user support and device commands queue

-- Add assigned_user_id to devices (for user-device assignment)
ALTER TABLE devices
    ADD COLUMN IF NOT EXISTS assigned_user_id UUID REFERENCES users(id) ON DELETE SET NULL;

-- Create index for assigned user lookup
CREATE INDEX IF NOT EXISTS idx_devices_assigned_user ON devices(assigned_user_id) WHERE assigned_user_id IS NOT NULL;

-- Create device_command_type enum
CREATE TYPE device_command_type AS ENUM ('wipe', 'lock', 'unlock', 'restart', 'update_policy', 'sync_settings');

-- Create device_command_status enum
CREATE TYPE device_command_status AS ENUM ('pending', 'acknowledged', 'completed', 'failed', 'expired');

-- Create device_commands table for pending commands
CREATE TABLE IF NOT EXISTS device_commands (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id BIGINT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    command_type device_command_type NOT NULL,
    status device_command_status NOT NULL DEFAULT 'pending',
    payload JSONB,
    issued_by UUID NOT NULL REFERENCES users(id),
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    failure_reason TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for device commands
CREATE INDEX IF NOT EXISTS idx_device_commands_device ON device_commands(device_id);
CREATE INDEX IF NOT EXISTS idx_device_commands_org ON device_commands(organization_id);
CREATE INDEX IF NOT EXISTS idx_device_commands_status ON device_commands(status) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_device_commands_pending ON device_commands(device_id, status, expires_at) WHERE status = 'pending';

-- Create trigger for updated_at
CREATE OR REPLACE FUNCTION update_device_commands_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_device_commands_updated_at
    BEFORE UPDATE ON device_commands
    FOR EACH ROW
    EXECUTE FUNCTION update_device_commands_updated_at();

-- Add comments
COMMENT ON COLUMN devices.assigned_user_id IS 'User assigned to this managed device';
COMMENT ON TABLE device_commands IS 'Queue of pending commands to be executed by devices';
COMMENT ON COLUMN device_commands.command_type IS 'Type of command to execute';
COMMENT ON COLUMN device_commands.status IS 'Current status of the command';
COMMENT ON COLUMN device_commands.payload IS 'Additional data for the command';
COMMENT ON COLUMN device_commands.issued_by IS 'Admin who issued the command';
COMMENT ON COLUMN device_commands.expires_at IS 'Time after which command should be discarded';
