-- Migration: Organization Member Invites
-- Purpose: Email-based invitations for new organization members

-- Organization member invites table
CREATE TABLE org_member_invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Organization this invite belongs to
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Secure token for invite URL (URL-safe, 32 characters)
    token VARCHAR(64) UNIQUE NOT NULL,

    -- Email address of the invitee (required)
    email VARCHAR(255) NOT NULL,

    -- Role to assign when accepted ("admin" or "member")
    role VARCHAR(50) NOT NULL DEFAULT 'member',

    -- Who created this invite
    invited_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Invite expiration (required)
    expires_at TIMESTAMPTZ NOT NULL,

    -- When the invite was accepted
    accepted_at TIMESTAMPTZ,

    -- User who accepted the invite
    accepted_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Optional note for admin tracking
    note VARCHAR(255)
);

-- Index for token lookup (primary use case for accepting invites)
CREATE INDEX idx_org_member_invites_token ON org_member_invites(token);

-- Index for listing invites by organization
CREATE INDEX idx_org_member_invites_org_id ON org_member_invites(organization_id);

-- Index for finding invites by email
CREATE INDEX idx_org_member_invites_email ON org_member_invites(email);

-- Index for finding pending (unused) invites per organization
CREATE INDEX idx_org_member_invites_pending ON org_member_invites(organization_id, expires_at)
    WHERE accepted_at IS NULL;

-- Prevent duplicate pending invites for the same email in the same org
CREATE UNIQUE INDEX idx_org_member_invites_unique_pending
    ON org_member_invites(organization_id, email)
    WHERE accepted_at IS NULL;

COMMENT ON TABLE org_member_invites IS 'Organization member invitations for controlled onboarding';
COMMENT ON COLUMN org_member_invites.token IS 'URL-safe invite token';
COMMENT ON COLUMN org_member_invites.email IS 'Email address of the invitee';
COMMENT ON COLUMN org_member_invites.role IS 'Role to assign: admin or member';
COMMENT ON COLUMN org_member_invites.expires_at IS 'Invite expiration timestamp';
COMMENT ON COLUMN org_member_invites.accepted_at IS 'When the invite was accepted';
COMMENT ON COLUMN org_member_invites.accepted_by IS 'User who accepted the invite';
