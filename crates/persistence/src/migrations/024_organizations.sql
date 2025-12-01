-- Migration 024: Organizations table for B2B multi-tenant support
-- Epic 13: B2B Enterprise Features
-- Story 13.1: Organizations table and CRUD endpoints

-- Create plan_type enum
CREATE TYPE plan_type AS ENUM ('free', 'starter', 'business', 'enterprise');

-- Create organizations table
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(50) NOT NULL,
    billing_email VARCHAR(255) NOT NULL,
    plan_type plan_type NOT NULL DEFAULT 'free',
    max_users INTEGER NOT NULL DEFAULT 5,
    max_devices INTEGER NOT NULL DEFAULT 10,
    max_groups INTEGER NOT NULL DEFAULT 5,
    settings JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT organizations_slug_unique UNIQUE (slug),
    CONSTRAINT organizations_slug_format CHECK (slug ~ '^[a-z0-9][a-z0-9-]*[a-z0-9]$' AND LENGTH(slug) >= 3 AND LENGTH(slug) <= 50),
    CONSTRAINT organizations_name_length CHECK (LENGTH(name) >= 2 AND LENGTH(name) <= 255),
    CONSTRAINT organizations_billing_email_format CHECK (billing_email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'),
    CONSTRAINT organizations_max_users_positive CHECK (max_users > 0),
    CONSTRAINT organizations_max_devices_positive CHECK (max_devices > 0),
    CONSTRAINT organizations_max_groups_positive CHECK (max_groups > 0)
);

-- Indexes for common queries
CREATE INDEX idx_organizations_slug ON organizations(slug);
CREATE INDEX idx_organizations_is_active ON organizations(is_active) WHERE is_active = true;
CREATE INDEX idx_organizations_plan_type ON organizations(plan_type);
CREATE INDEX idx_organizations_created_at ON organizations(created_at DESC);

-- Trigger for updated_at
CREATE TRIGGER update_organizations_updated_at
    BEFORE UPDATE ON organizations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add comment
COMMENT ON TABLE organizations IS 'B2B tenant organizations for enterprise device fleet management';
COMMENT ON COLUMN organizations.slug IS 'URL-friendly unique identifier (lowercase, alphanumeric, hyphens)';
COMMENT ON COLUMN organizations.settings IS 'Organization-specific configuration (JSON)';
