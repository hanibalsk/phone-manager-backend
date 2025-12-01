-- Migration 027: Enrollment Tokens Table
-- Story 13.4: Enrollment Tokens Management Endpoints
-- Creates enrollment_tokens table for device provisioning

-- Create enrollment_tokens table
CREATE TABLE enrollment_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    token VARCHAR(100) NOT NULL,
    token_prefix VARCHAR(20) NOT NULL,
    group_id VARCHAR(50),
    policy_id UUID REFERENCES device_policies(id) ON DELETE SET NULL,
    max_uses INTEGER,
    current_uses INTEGER NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ,
    auto_assign_user_by_email BOOLEAN NOT NULL DEFAULT false,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,
    CONSTRAINT enrollment_tokens_token_unique UNIQUE (token)
);

-- Create indexes
CREATE INDEX idx_enrollment_tokens_organization ON enrollment_tokens(organization_id);
CREATE INDEX idx_enrollment_tokens_token ON enrollment_tokens(token);
CREATE INDEX idx_enrollment_tokens_expires ON enrollment_tokens(expires_at) WHERE expires_at IS NOT NULL AND revoked_at IS NULL;
CREATE INDEX idx_enrollment_tokens_group ON enrollment_tokens(group_id) WHERE group_id IS NOT NULL;
CREATE INDEX idx_enrollment_tokens_policy ON enrollment_tokens(policy_id) WHERE policy_id IS NOT NULL;

-- Add comment
COMMENT ON TABLE enrollment_tokens IS 'Enrollment tokens for device provisioning into organizations';
COMMENT ON COLUMN enrollment_tokens.token IS 'Full enrollment token (enroll_<random>)';
COMMENT ON COLUMN enrollment_tokens.token_prefix IS 'First part of token for identification';
COMMENT ON COLUMN enrollment_tokens.max_uses IS 'Maximum number of times token can be used (NULL = unlimited)';
COMMENT ON COLUMN enrollment_tokens.current_uses IS 'Current number of times token has been used';
COMMENT ON COLUMN enrollment_tokens.auto_assign_user_by_email IS 'Automatically assign device to user matching email domain';
COMMENT ON COLUMN enrollment_tokens.revoked_at IS 'When token was revoked (soft delete)';
