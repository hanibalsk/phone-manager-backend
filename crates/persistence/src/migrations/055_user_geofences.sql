-- Migration 055: User-level geofences and tracking control
-- Epic 9: Admin User Management
-- User geofences apply to ALL devices owned by the user

-- Add tracking_enabled column to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS tracking_enabled BOOLEAN NOT NULL DEFAULT true;

-- Index for filtering users by tracking status
CREATE INDEX IF NOT EXISTS idx_users_tracking_enabled ON users(tracking_enabled) WHERE is_active = true;

-- Create user_geofences table (user-level geofences)
CREATE TABLE user_geofences (
    id              BIGSERIAL PRIMARY KEY,
    geofence_id     UUID NOT NULL UNIQUE DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    name            VARCHAR(100) NOT NULL,
    latitude        DOUBLE PRECISION NOT NULL,
    longitude       DOUBLE PRECISION NOT NULL,
    radius_meters   REAL NOT NULL,
    event_types     TEXT[] NOT NULL DEFAULT ARRAY['enter', 'exit'],
    active          BOOLEAN NOT NULL DEFAULT TRUE,
    color           VARCHAR(7),
    metadata        JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Data validation constraints (same as device geofences)
    CONSTRAINT chk_user_geofence_latitude CHECK (latitude >= -90 AND latitude <= 90),
    CONSTRAINT chk_user_geofence_longitude CHECK (longitude >= -180 AND longitude <= 180),
    CONSTRAINT chk_user_geofence_radius CHECK (radius_meters >= 20 AND radius_meters <= 50000),
    CONSTRAINT chk_user_geofence_name_length CHECK (char_length(name) >= 1 AND char_length(name) <= 100),
    CONSTRAINT chk_user_geofence_event_types CHECK (
        event_types <@ ARRAY['enter', 'exit', 'dwell']::TEXT[]
        AND array_length(event_types, 1) > 0
    ),
    CONSTRAINT chk_user_geofence_color CHECK (
        color IS NULL OR color ~ '^#[0-9A-Fa-f]{6}$'
    )
);

-- Index for user geofence queries (primary access pattern)
CREATE INDEX idx_user_geofences_user_id ON user_geofences(user_id) WHERE active = TRUE;

-- Index for geofence lookup by UUID
CREATE INDEX idx_user_geofences_geofence_id ON user_geofences(geofence_id);

-- Trigger to auto-update updated_at
CREATE TRIGGER update_user_geofences_updated_at
    BEFORE UPDATE ON user_geofences
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comment documentation
COMMENT ON TABLE user_geofences IS 'User-level geofences that apply to all devices owned by the user';
COMMENT ON COLUMN user_geofences.user_id IS 'User this geofence applies to (all their devices)';
COMMENT ON COLUMN user_geofences.created_by IS 'Admin who created this geofence';
COMMENT ON COLUMN user_geofences.color IS 'Hex color code for map display (e.g., #FF5733)';
COMMENT ON COLUMN users.tracking_enabled IS 'Whether location tracking is enabled for this user';
