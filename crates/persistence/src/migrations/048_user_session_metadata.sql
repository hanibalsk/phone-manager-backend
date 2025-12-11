-- Migration 048: User Session Metadata
-- Story AP-3.11-3.13: Session Management
-- Adds metadata columns for device info, IP address, and location

-- Add metadata columns to user_sessions table
ALTER TABLE user_sessions
    ADD COLUMN IF NOT EXISTS device_name VARCHAR(255),
    ADD COLUMN IF NOT EXISTS device_type VARCHAR(50), -- 'mobile', 'desktop', 'tablet', 'unknown'
    ADD COLUMN IF NOT EXISTS browser VARCHAR(100),
    ADD COLUMN IF NOT EXISTS os VARCHAR(100),
    ADD COLUMN IF NOT EXISTS ip_address INET,
    ADD COLUMN IF NOT EXISTS location VARCHAR(255), -- "City, Country" or "Unknown"
    ADD COLUMN IF NOT EXISTS user_agent TEXT;

-- Index for finding sessions by user
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id_active
    ON user_sessions(user_id, expires_at)
    WHERE expires_at > NOW();

-- Comments for documentation
COMMENT ON COLUMN user_sessions.device_name IS 'User-friendly device name (e.g., "iPhone 14", "Chrome on MacOS")';
COMMENT ON COLUMN user_sessions.device_type IS 'Device type: mobile, desktop, tablet, or unknown';
COMMENT ON COLUMN user_sessions.browser IS 'Browser name and version';
COMMENT ON COLUMN user_sessions.os IS 'Operating system name and version';
COMMENT ON COLUMN user_sessions.ip_address IS 'IP address at session creation';
COMMENT ON COLUMN user_sessions.location IS 'Geographic location based on IP';
COMMENT ON COLUMN user_sessions.user_agent IS 'Full user agent string for debugging';
