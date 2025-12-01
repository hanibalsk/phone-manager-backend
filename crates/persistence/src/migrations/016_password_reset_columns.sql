-- Migration: 016_password_reset_columns
-- Description: Add password reset token columns to users table
-- Created: 2025-12-01

-- Add password reset columns
ALTER TABLE users
ADD COLUMN password_reset_token VARCHAR(64),
ADD COLUMN password_reset_expires_at TIMESTAMPTZ;

-- Index for token lookup (only on non-null values)
CREATE INDEX idx_users_password_reset_token ON users(password_reset_token) WHERE password_reset_token IS NOT NULL;
