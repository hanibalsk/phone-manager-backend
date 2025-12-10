-- Migration: 044_organization_roles
-- Description: Add custom organization roles table for RBAC
-- Story: AP-1.2, AP-1.3

-- Organization custom roles table
-- Supports user-defined roles with custom permission sets
CREATE TABLE IF NOT EXISTS organization_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(50) NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    -- Array of permission strings (e.g., ['device:read', 'user:manage'])
    permissions TEXT[] NOT NULL DEFAULT '{}',
    -- System roles cannot be deleted or modified
    is_system_role BOOLEAN NOT NULL DEFAULT FALSE,
    -- Priority for role hierarchy (higher = more privileged)
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Each role name must be unique within an organization
    CONSTRAINT uq_org_role_name UNIQUE (organization_id, name)
);

-- Index for fast lookups by organization
CREATE INDEX IF NOT EXISTS idx_org_roles_org_id ON organization_roles(organization_id);

-- Index for finding system roles
CREATE INDEX IF NOT EXISTS idx_org_roles_system ON organization_roles(organization_id, is_system_role);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_organization_roles_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_organization_roles_updated_at
    BEFORE UPDATE ON organization_roles
    FOR EACH ROW
    EXECUTE FUNCTION update_organization_roles_updated_at();

-- Function to initialize system roles for a new organization
CREATE OR REPLACE FUNCTION init_organization_system_roles(org_id UUID, creator_id UUID)
RETURNS void AS $$
BEGIN
    -- Insert default system roles for the organization
    INSERT INTO organization_roles (organization_id, name, display_name, description, permissions, is_system_role, priority, created_by)
    VALUES
        (org_id, 'owner', 'Owner', 'Full organization access with all permissions',
         ARRAY['device:read', 'device:manage', 'user:read', 'user:manage', 'policy:read', 'policy:manage', 'audit:read'],
         TRUE, 100, creator_id),
        (org_id, 'admin', 'Admin', 'Administrative access for managing devices and users',
         ARRAY['device:read', 'device:manage', 'user:read', 'user:manage', 'policy:read'],
         TRUE, 80, creator_id),
        (org_id, 'member', 'Member', 'Basic read access for organization members',
         ARRAY['device:read', 'user:read'],
         TRUE, 20, creator_id)
    ON CONFLICT (organization_id, name) DO NOTHING;
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE organization_roles IS 'Custom and system roles for organization RBAC';
COMMENT ON COLUMN organization_roles.name IS 'Internal role identifier (lowercase, no spaces)';
COMMENT ON COLUMN organization_roles.display_name IS 'Human-readable role name';
COMMENT ON COLUMN organization_roles.permissions IS 'Array of permission strings this role grants';
COMMENT ON COLUMN organization_roles.is_system_role IS 'System roles cannot be deleted or have permissions modified';
COMMENT ON COLUMN organization_roles.priority IS 'Higher priority roles can manage lower priority roles';
