-- Migration: 017_email_verification_columns
-- Description: Add email verification token columns to users table
-- Created: 2025-12-01

-- Add email verification columns
ALTER TABLE users
ADD COLUMN email_verification_token VARCHAR(64),
ADD COLUMN email_verification_expires_at TIMESTAMPTZ;

-- Index for token lookup (only on non-null values)
CREATE INDEX idx_users_email_verification_token ON users(email_verification_token) WHERE email_verification_token IS NOT NULL;
