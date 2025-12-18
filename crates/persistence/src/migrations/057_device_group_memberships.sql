-- Migration 057: Device-Group Memberships Table
-- Story UGM-3.1: Device-Group Membership Table
-- Allows devices to belong to multiple authenticated groups simultaneously

-- Create device_group_memberships table
CREATE TABLE IF NOT EXISTS device_group_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL,
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    added_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- A device can only be in each group once
    CONSTRAINT uq_device_group_membership UNIQUE (device_id, group_id)
);

-- Index for finding all devices in a group (primary query pattern)
CREATE INDEX IF NOT EXISTS idx_device_group_memberships_group_id
    ON device_group_memberships(group_id);

-- Index for finding all groups a device belongs to
CREATE INDEX IF NOT EXISTS idx_device_group_memberships_device_id
    ON device_group_memberships(device_id);

-- Index for finding which user added devices (audit queries)
CREATE INDEX IF NOT EXISTS idx_device_group_memberships_added_by
    ON device_group_memberships(added_by);

-- Index for ordering by added_at within a group
CREATE INDEX IF NOT EXISTS idx_device_group_memberships_group_added_at
    ON device_group_memberships(group_id, added_at DESC);
