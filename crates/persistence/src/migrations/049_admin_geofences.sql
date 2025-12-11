-- Migration 049: Admin Geofences table
-- Organization-wide geofence definitions for administrative management
-- Separate from device-level geofences to support org-wide policies

CREATE TABLE admin_geofences (
    id              BIGSERIAL PRIMARY KEY,
    geofence_id     UUID NOT NULL UNIQUE DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            VARCHAR(100) NOT NULL,
    description     TEXT,
    latitude        DOUBLE PRECISION NOT NULL,
    longitude       DOUBLE PRECISION NOT NULL,
    radius_meters   REAL NOT NULL,
    event_types     TEXT[] NOT NULL DEFAULT ARRAY['enter', 'exit'],
    active          BOOLEAN NOT NULL DEFAULT TRUE,
    color           VARCHAR(7), -- Hex color code for UI display
    metadata        JSONB,
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Data validation constraints
    CONSTRAINT chk_admin_geofence_latitude CHECK (latitude >= -90 AND latitude <= 90),
    CONSTRAINT chk_admin_geofence_longitude CHECK (longitude >= -180 AND longitude <= 180),
    CONSTRAINT chk_admin_geofence_radius CHECK (radius_meters >= 20 AND radius_meters <= 50000),
    CONSTRAINT chk_admin_geofence_name_length CHECK (char_length(name) >= 1 AND char_length(name) <= 100),
    CONSTRAINT chk_admin_geofence_event_types CHECK (
        event_types <@ ARRAY['enter', 'exit', 'dwell']::TEXT[]
        AND array_length(event_types, 1) > 0
    ),
    CONSTRAINT chk_admin_geofence_color CHECK (
        color IS NULL OR color ~ '^#[0-9A-Fa-f]{6}$'
    )
);

-- Index for organization geofence queries (primary access pattern)
CREATE INDEX idx_admin_geofences_organization_id ON admin_geofences(organization_id) WHERE active = TRUE;

-- Index for geofence lookup by UUID
CREATE INDEX idx_admin_geofences_geofence_id ON admin_geofences(geofence_id);

-- Trigger to auto-update updated_at
CREATE TRIGGER update_admin_geofences_updated_at
    BEFORE UPDATE ON admin_geofences
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE admin_geofences IS 'Organization-wide geofences for administrative location policies';
COMMENT ON COLUMN admin_geofences.color IS 'Hex color code for map display (e.g., #FF5733)';
COMMENT ON COLUMN admin_geofences.event_types IS 'Array of event types to trigger: enter, exit, dwell';
