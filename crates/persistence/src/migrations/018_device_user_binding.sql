-- Migration 018: Device-User Binding Schema
-- Adds columns to link devices to user accounts
-- Story 10.1: Device table migration (owner_user_id, organization_id)

-- Add owner_user_id column: links device to a user account
-- NULL means device is not linked to any user (legacy anonymous device)
ALTER TABLE devices ADD COLUMN IF NOT EXISTS owner_user_id UUID;

-- Add organization_id column: for future B2B features (Epic 13)
-- No FK constraint yet since organizations table doesn't exist
ALTER TABLE devices ADD COLUMN IF NOT EXISTS organization_id UUID;

-- Add is_primary column: marks user's primary device
ALTER TABLE devices ADD COLUMN IF NOT EXISTS is_primary BOOLEAN NOT NULL DEFAULT false;

-- Add linked_at column: when the device was linked to the user
ALTER TABLE devices ADD COLUMN IF NOT EXISTS linked_at TIMESTAMPTZ;

-- Foreign key constraint to users table
-- ON DELETE SET NULL: if user is deleted, device becomes unlinked but keeps data
ALTER TABLE devices ADD CONSTRAINT fk_devices_owner_user
    FOREIGN KEY (owner_user_id) REFERENCES users(id) ON DELETE SET NULL;

-- Index for finding all devices owned by a user
CREATE INDEX IF NOT EXISTS idx_devices_owner_user_id ON devices(owner_user_id) WHERE owner_user_id IS NOT NULL;

-- Index for finding devices by organization (future B2B queries)
CREATE INDEX IF NOT EXISTS idx_devices_organization_id ON devices(organization_id) WHERE organization_id IS NOT NULL;

-- Index for finding a user's primary device
CREATE INDEX IF NOT EXISTS idx_devices_user_primary ON devices(owner_user_id, is_primary) WHERE owner_user_id IS NOT NULL AND is_primary = true;
