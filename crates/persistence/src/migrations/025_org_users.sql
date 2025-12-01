-- Migration 025: Organization Users table for B2B admin management
-- Epic 13: B2B Enterprise Features
-- Story 13.2: Organization users management endpoints

-- Create org_user_role enum
CREATE TYPE org_user_role AS ENUM ('owner', 'admin', 'member');

-- Create org_users table
CREATE TABLE org_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role org_user_role NOT NULL DEFAULT 'member',
    permissions JSONB NOT NULL DEFAULT '[]',
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Constraints
    CONSTRAINT org_users_unique UNIQUE (organization_id, user_id)
);

-- Indexes for common queries
CREATE INDEX idx_org_users_organization_id ON org_users(organization_id);
CREATE INDEX idx_org_users_user_id ON org_users(user_id);
CREATE INDEX idx_org_users_role ON org_users(organization_id, role);

-- Comments
COMMENT ON TABLE org_users IS 'Organization user memberships with roles and permissions';
COMMENT ON COLUMN org_users.role IS 'User role: owner (full access), admin (manage), member (view)';
COMMENT ON COLUMN org_users.permissions IS 'JSON array of permission strings';
