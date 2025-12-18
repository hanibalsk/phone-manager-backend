-- Migration 056: Migration Audit Logs Schema
-- Creates table for tracking registration group to authenticated group migrations
-- Story UGM-2.1: Create Migration Audit Log Infrastructure

-- MigrationStatus enum type for migration status tracking
DO $$ BEGIN
    CREATE TYPE migration_status AS ENUM ('success', 'failed', 'partial');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- Migration audit logs table: Tracks all group migration events
CREATE TABLE IF NOT EXISTS migration_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- User who initiated the migration
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL,
    -- Source registration group ID (string, not FK as it's not in groups table)
    registration_group_id VARCHAR(255) NOT NULL,
    -- Target authenticated group UUID
    authenticated_group_id UUID NOT NULL REFERENCES groups(id) ON DELETE SET NULL,
    -- Number of devices migrated
    devices_migrated INTEGER NOT NULL DEFAULT 0,
    -- Array of device UUIDs that were migrated
    device_ids UUID[] NOT NULL DEFAULT '{}',
    -- Migration status
    status migration_status NOT NULL DEFAULT 'success',
    -- Error message if migration failed
    error_message TEXT,
    -- Timestamp of migration
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Validate devices_migrated is non-negative
    CONSTRAINT chk_migration_devices_count CHECK (devices_migrated >= 0),

    -- Ensure registration_group_id is not empty
    CONSTRAINT chk_migration_reg_group_not_empty CHECK (length(registration_group_id) > 0)
);

-- Index for finding migrations by user
CREATE INDEX IF NOT EXISTS idx_migration_audit_user_id ON migration_audit_logs(user_id);

-- Index for finding migrations by registration group
CREATE INDEX IF NOT EXISTS idx_migration_audit_reg_group ON migration_audit_logs(registration_group_id);

-- Index for finding migrations by authenticated group
CREATE INDEX IF NOT EXISTS idx_migration_audit_auth_group ON migration_audit_logs(authenticated_group_id);

-- Index for filtering by status
CREATE INDEX IF NOT EXISTS idx_migration_audit_status ON migration_audit_logs(status);

-- Index for pagination by creation date
CREATE INDEX IF NOT EXISTS idx_migration_audit_created_at ON migration_audit_logs(created_at DESC);

-- Composite index for efficient admin queries with status filter
CREATE INDEX IF NOT EXISTS idx_migration_audit_status_created ON migration_audit_logs(status, created_at DESC);

-- Comment on table
COMMENT ON TABLE migration_audit_logs IS 'Audit log for registration group to authenticated group migrations (UGM-2.1)';
