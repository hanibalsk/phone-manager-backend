-- Migration 043: System-level RBAC for admin panel
-- Implements system-wide roles that operate across organizations
-- Users can have multiple system roles, and org_admin/org_manager require explicit org assignments

-- Create system_role enum
CREATE TYPE system_role AS ENUM (
    'super_admin',   -- Full system access, manage all organizations
    'org_admin',     -- Full management for assigned organizations
    'org_manager',   -- Manage users/devices in assigned organizations
    'support',       -- View-only access for customer support (global read)
    'viewer'         -- Read-only system metrics
);

-- User system roles (many-to-many: users can have multiple system roles)
CREATE TABLE user_system_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role system_role NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Each user can only have each role once
    CONSTRAINT user_system_roles_unique UNIQUE (user_id, role)
);

-- Indexes for common queries
CREATE INDEX idx_user_system_roles_user ON user_system_roles(user_id);
CREATE INDEX idx_user_system_roles_role ON user_system_roles(role);

-- Admin organization assignments (for org_admin/org_manager roles)
-- These roles require explicit assignment to specific organizations
CREATE TABLE admin_org_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Each user can only be assigned to each organization once
    CONSTRAINT admin_org_assignments_unique UNIQUE (user_id, organization_id)
);

-- Indexes for common queries
CREATE INDEX idx_admin_org_assignments_user ON admin_org_assignments(user_id);
CREATE INDEX idx_admin_org_assignments_org ON admin_org_assignments(organization_id);

-- Comments for documentation
COMMENT ON TABLE user_system_roles IS 'System-level roles for admin panel access (users can have multiple roles)';
COMMENT ON COLUMN user_system_roles.role IS 'System role: super_admin (full), org_admin (assigned orgs), org_manager (assigned orgs), support (global read), viewer (metrics)';
COMMENT ON TABLE admin_org_assignments IS 'Organization assignments for org_admin and org_manager roles';
COMMENT ON COLUMN admin_org_assignments.organization_id IS 'Organization this admin user can access';
