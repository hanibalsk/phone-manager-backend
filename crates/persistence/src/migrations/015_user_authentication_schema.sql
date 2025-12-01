-- Migration 015: User Authentication Schema
-- Creates tables for user accounts, OAuth integrations, and sessions
-- Story 9.1: User Authentication Database Schema

-- Users table: Core user account information
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255), -- NULL for OAuth-only accounts
    display_name VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT true,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ
);

-- Index for fast email lookups during login
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Trigger to auto-update updated_at timestamp
CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

-- OAuth accounts table: Links users to external OAuth providers
CREATE TABLE IF NOT EXISTS oauth_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(20) NOT NULL, -- 'google', 'apple'
    provider_user_id VARCHAR(255) NOT NULL, -- User ID from OAuth provider
    provider_email VARCHAR(255), -- Email from OAuth provider
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure unique OAuth account per provider
    CONSTRAINT uq_oauth_provider_user UNIQUE (provider, provider_user_id),

    -- Validate provider values
    CONSTRAINT chk_oauth_provider CHECK (provider IN ('google', 'apple'))
);

-- Index for OAuth provider lookups
CREATE INDEX IF NOT EXISTS idx_oauth_accounts_provider_email ON oauth_accounts(provider, provider_email);
CREATE INDEX IF NOT EXISTS idx_oauth_accounts_user_id ON oauth_accounts(user_id);

-- User sessions table: Tracks active JWT sessions with refresh tokens
CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL, -- SHA-256 hash of access token
    refresh_token_hash VARCHAR(64) NOT NULL, -- SHA-256 hash of refresh token
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure unique token hashes for quick lookups
    CONSTRAINT uq_session_token_hash UNIQUE (token_hash)
);

-- Composite index for token validation queries
CREATE INDEX IF NOT EXISTS idx_user_sessions_token_expires ON user_sessions(token_hash, expires_at);

-- Index for user session lookups
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);

-- Index for session cleanup queries (finding expired sessions)
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions(expires_at);
