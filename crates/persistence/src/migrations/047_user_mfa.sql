-- Migration 047: User MFA (Multi-Factor Authentication) Support
-- Story AP-3.8-3.10: MFA Status, Force MFA Enrollment, Reset User MFA

-- Add MFA-related columns to users table
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS mfa_enabled BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS mfa_method VARCHAR(20), -- 'totp', 'sms', etc.
    ADD COLUMN IF NOT EXISTS mfa_secret VARCHAR(255), -- Encrypted TOTP secret
    ADD COLUMN IF NOT EXISTS mfa_enrolled_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS mfa_forced BOOLEAN NOT NULL DEFAULT false, -- Admin can force MFA enrollment
    ADD COLUMN IF NOT EXISTS mfa_forced_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS mfa_forced_by UUID REFERENCES users(id) ON DELETE SET NULL;

-- Index for finding users with MFA requirements
CREATE INDEX IF NOT EXISTS idx_users_mfa_forced ON users(mfa_forced) WHERE mfa_forced = true;

-- Constraint for MFA method values
ALTER TABLE users
    ADD CONSTRAINT chk_mfa_method CHECK (
        mfa_method IS NULL OR mfa_method IN ('totp', 'sms', 'email')
    );

-- Comments for documentation
COMMENT ON COLUMN users.mfa_enabled IS 'Whether MFA is currently enabled for this user';
COMMENT ON COLUMN users.mfa_method IS 'The MFA method being used (totp, sms, email)';
COMMENT ON COLUMN users.mfa_secret IS 'Encrypted TOTP secret for authenticator apps';
COMMENT ON COLUMN users.mfa_enrolled_at IS 'When MFA was successfully enrolled';
COMMENT ON COLUMN users.mfa_forced IS 'Whether admin has required MFA for this user';
COMMENT ON COLUMN users.mfa_forced_at IS 'When MFA requirement was set by admin';
COMMENT ON COLUMN users.mfa_forced_by IS 'Admin who set the MFA requirement';
