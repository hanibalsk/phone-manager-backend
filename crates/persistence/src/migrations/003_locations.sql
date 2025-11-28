-- Migration 003: Locations table
-- Stores device location history with validation constraints

CREATE TABLE locations (
    id              BIGSERIAL PRIMARY KEY,
    device_id       UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    latitude        DOUBLE PRECISION NOT NULL,
    longitude       DOUBLE PRECISION NOT NULL,
    accuracy        REAL NOT NULL,
    altitude        DOUBLE PRECISION,
    bearing         REAL,
    speed           REAL,
    provider        VARCHAR(50),
    battery_level   SMALLINT,
    network_type    VARCHAR(50),
    captured_at     TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints for data validation
    CONSTRAINT chk_latitude CHECK (latitude >= -90 AND latitude <= 90),
    CONSTRAINT chk_longitude CHECK (longitude >= -180 AND longitude <= 180),
    CONSTRAINT chk_accuracy CHECK (accuracy >= 0),
    CONSTRAINT chk_bearing CHECK (bearing IS NULL OR (bearing >= 0 AND bearing <= 360)),
    CONSTRAINT chk_speed CHECK (speed IS NULL OR speed >= 0),
    CONSTRAINT chk_battery CHECK (battery_level IS NULL OR (battery_level >= 0 AND battery_level <= 100))
);

-- Index for device location history (primary query pattern)
CREATE INDEX idx_locations_device_captured ON locations(device_id, captured_at DESC);

-- Index for time-based cleanup
CREATE INDEX idx_locations_created_at ON locations(created_at);

-- Note: Partial index for recent locations removed because NOW() is not IMMUTABLE
-- The idx_locations_device_captured index handles these queries efficiently
