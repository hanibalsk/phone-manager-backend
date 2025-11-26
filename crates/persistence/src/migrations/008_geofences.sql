-- Migration 008: Geofences table
-- Stores per-device geofence definitions with circular regions

CREATE TABLE geofences (
    id              BIGSERIAL PRIMARY KEY,
    geofence_id     UUID NOT NULL UNIQUE DEFAULT uuid_generate_v4(),
    device_id       UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    name            VARCHAR(100) NOT NULL,
    latitude        DOUBLE PRECISION NOT NULL,
    longitude       DOUBLE PRECISION NOT NULL,
    radius_meters   REAL NOT NULL,
    event_types     TEXT[] NOT NULL DEFAULT ARRAY['enter', 'exit'],
    active          BOOLEAN NOT NULL DEFAULT TRUE,
    metadata        JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Data validation constraints
    CONSTRAINT chk_geofence_latitude CHECK (latitude >= -90 AND latitude <= 90),
    CONSTRAINT chk_geofence_longitude CHECK (longitude >= -180 AND longitude <= 180),
    CONSTRAINT chk_geofence_radius CHECK (radius_meters >= 20 AND radius_meters <= 50000),
    CONSTRAINT chk_geofence_name_length CHECK (char_length(name) >= 1 AND char_length(name) <= 100),
    CONSTRAINT chk_geofence_event_types CHECK (
        event_types <@ ARRAY['enter', 'exit', 'dwell']::TEXT[]
        AND array_length(event_types, 1) > 0
    )
);

-- Index for device geofence queries (primary access pattern)
CREATE INDEX idx_geofences_device_id ON geofences(device_id) WHERE active = TRUE;

-- Index for geofence lookup by UUID
CREATE INDEX idx_geofences_geofence_id ON geofences(geofence_id);

-- Trigger to auto-update updated_at
CREATE TRIGGER update_geofences_updated_at
    BEFORE UPDATE ON geofences
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
