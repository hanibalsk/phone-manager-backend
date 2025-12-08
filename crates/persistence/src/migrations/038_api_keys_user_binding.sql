-- Migration 038: API Keys User Binding
-- Adds user_id FK and description column for admin bootstrap support
-- Makes name nullable since bootstrap uses description instead

-- Add user_id column with FK to users
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id) ON DELETE CASCADE;

-- Add description column for API key purpose
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS description VARCHAR(255);

-- Make name nullable (bootstrap doesn't use it, uses description instead)
ALTER TABLE api_keys ALTER COLUMN name DROP NOT NULL;

-- Set default for name to maintain backwards compatibility
ALTER TABLE api_keys ALTER COLUMN name SET DEFAULT 'API Key';

-- Index for user API key lookups
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id) WHERE user_id IS NOT NULL;
