-- Add avatar_url column to users table for user profile feature
-- Story 9.11: Current User Profile Endpoints

ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(500);

-- Create index for future queries filtering by avatar presence
CREATE INDEX IF NOT EXISTS idx_users_avatar_url ON users (avatar_url) WHERE avatar_url IS NOT NULL;
