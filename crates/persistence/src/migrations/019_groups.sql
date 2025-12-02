-- Migration 019: Groups and Group Memberships Schema
-- Creates tables for group management and membership tracking
-- Story 11.1: Group Database Schema & CRUD Endpoints

-- GroupRole enum type for membership roles
DO $$ BEGIN
    CREATE TYPE group_role AS ENUM ('owner', 'admin', 'member', 'viewer');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- Groups table: Core group information
CREATE TABLE IF NOT EXISTS groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(120) NOT NULL,
    description VARCHAR(500),
    icon_emoji VARCHAR(10),
    max_devices INTEGER NOT NULL DEFAULT 20,
    is_active BOOLEAN NOT NULL DEFAULT true,
    settings JSONB NOT NULL DEFAULT '{}',
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Slug must be unique for URL-friendly access
    CONSTRAINT uq_groups_slug UNIQUE (slug),

    -- Validate max_devices range
    CONSTRAINT chk_groups_max_devices CHECK (max_devices BETWEEN 1 AND 100),

    -- Validate name length
    CONSTRAINT chk_groups_name_length CHECK (length(name) >= 1)
);

-- Index for slug lookups (unique constraint already creates index)
-- Index for listing groups by creation date
CREATE INDEX IF NOT EXISTS idx_groups_created_at ON groups(created_at DESC);

-- Index for finding groups by creator
CREATE INDEX IF NOT EXISTS idx_groups_created_by ON groups(created_by);

-- Trigger to auto-update updated_at timestamp
CREATE TRIGGER trg_groups_updated_at
    BEFORE UPDATE ON groups
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Group memberships table: Links users to groups with roles
CREATE TABLE IF NOT EXISTS group_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role group_role NOT NULL DEFAULT 'member',
    invited_by UUID REFERENCES users(id) ON DELETE SET NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- User can only have one membership per group
    CONSTRAINT uq_group_membership UNIQUE (group_id, user_id)
);

-- Index for finding all members of a group
CREATE INDEX IF NOT EXISTS idx_group_memberships_group_id ON group_memberships(group_id);

-- Index for finding all groups a user belongs to
CREATE INDEX IF NOT EXISTS idx_group_memberships_user_id ON group_memberships(user_id);

-- Index for finding members by role within a group
CREATE INDEX IF NOT EXISTS idx_group_memberships_role ON group_memberships(group_id, role);

-- Trigger to auto-update updated_at timestamp
CREATE TRIGGER trg_group_memberships_updated_at
    BEFORE UPDATE ON group_memberships
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to ensure exactly one owner per group
CREATE OR REPLACE FUNCTION check_group_has_owner()
RETURNS TRIGGER AS $$
BEGIN
    -- On DELETE, ensure we're not deleting the last owner
    IF TG_OP = 'DELETE' AND OLD.role = 'owner' THEN
        IF NOT EXISTS (
            SELECT 1 FROM group_memberships
            WHERE group_id = OLD.group_id AND role = 'owner' AND id != OLD.id
        ) THEN
            RAISE EXCEPTION 'Cannot remove last owner from group';
        END IF;
    END IF;

    -- On UPDATE from owner to non-owner, ensure another owner exists
    IF TG_OP = 'UPDATE' AND OLD.role = 'owner' AND NEW.role != 'owner' THEN
        IF NOT EXISTS (
            SELECT 1 FROM group_memberships
            WHERE group_id = OLD.group_id AND role = 'owner' AND id != OLD.id
        ) THEN
            RAISE EXCEPTION 'Cannot change role of last owner';
        END IF;
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Trigger to prevent removing/demoting last owner
CREATE TRIGGER trg_check_group_owner
    BEFORE UPDATE OR DELETE ON group_memberships
    FOR EACH ROW
    EXECUTE FUNCTION check_group_has_owner();
