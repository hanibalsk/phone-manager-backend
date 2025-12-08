-- Migration: Registration Invites
-- Purpose: Support invite-only registration mode

-- Registration invites table for controlled user onboarding
CREATE TABLE registration_invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Secure token for invite URL (URL-safe, 32 characters)
    token VARCHAR(64) UNIQUE NOT NULL,

    -- Optional: restrict invite to specific email
    -- NULL means any email can use this invite
    email VARCHAR(255),

    -- Who created this invite (NULL for system-generated)
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Invite expiration (required)
    expires_at TIMESTAMPTZ NOT NULL,

    -- Usage tracking
    used_at TIMESTAMPTZ,
    used_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Optional note for admin tracking
    note VARCHAR(500)
);

-- Index for token lookup (primary use case)
CREATE INDEX idx_registration_invites_token ON registration_invites(token);

-- Index for finding invites by email
CREATE INDEX idx_registration_invites_email ON registration_invites(email) WHERE email IS NOT NULL;

-- Index for finding unused invites
CREATE INDEX idx_registration_invites_unused ON registration_invites(expires_at)
    WHERE used_at IS NULL;

-- Index for finding invites created by a user
CREATE INDEX idx_registration_invites_created_by ON registration_invites(created_by)
    WHERE created_by IS NOT NULL;

COMMENT ON TABLE registration_invites IS 'Registration invites for invite-only mode';
COMMENT ON COLUMN registration_invites.token IS 'URL-safe invite token';
COMMENT ON COLUMN registration_invites.email IS 'Optional: restrict to specific email';
COMMENT ON COLUMN registration_invites.expires_at IS 'Invite expiration timestamp';
COMMENT ON COLUMN registration_invites.used_at IS 'When the invite was used';
COMMENT ON COLUMN registration_invites.used_by IS 'User who used the invite';
