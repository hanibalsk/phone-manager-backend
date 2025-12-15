-- Migration: Setting Changes History
-- Epic 12: Settings Control
-- Story 12.x: Settings History Tracking

-- Create setting_change_type enum
CREATE TYPE setting_change_type AS ENUM ('value_changed', 'locked', 'unlocked', 'reset');

-- Table: setting_changes
-- Stores history of setting changes for devices
CREATE TABLE setting_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    setting_key VARCHAR(100) NOT NULL,
    old_value JSONB,
    new_value JSONB,
    changed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    change_type setting_change_type NOT NULL
);

-- Index for efficient queries by device and time (most common query pattern)
CREATE INDEX idx_setting_changes_device_time ON setting_changes(device_id, changed_at DESC);

-- Index for queries filtering by setting key within a device
CREATE INDEX idx_setting_changes_device_key ON setting_changes(device_id, setting_key);

-- Index for queries by user who made changes
CREATE INDEX idx_setting_changes_changed_by ON setting_changes(changed_by);

-- Comments for documentation
COMMENT ON TABLE setting_changes IS 'Audit log of device setting changes';
COMMENT ON COLUMN setting_changes.device_id IS 'Device whose setting was changed';
COMMENT ON COLUMN setting_changes.setting_key IS 'Key of the setting that was changed';
COMMENT ON COLUMN setting_changes.old_value IS 'Previous value before change (null for new settings)';
COMMENT ON COLUMN setting_changes.new_value IS 'New value after change (null for unlocks)';
COMMENT ON COLUMN setting_changes.changed_by IS 'User who made the change';
COMMENT ON COLUMN setting_changes.changed_at IS 'Timestamp when change occurred';
COMMENT ON COLUMN setting_changes.change_type IS 'Type of change: value_changed, locked, unlocked, reset';
