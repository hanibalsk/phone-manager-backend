-- Migration 011: Movement Events table
-- Stores movement data with geospatial capabilities for trip tracking
-- Part of Epic 5: Movement Events API

-- Enable PostGIS extension for geospatial data types and functions
CREATE EXTENSION IF NOT EXISTS "postgis";

CREATE TABLE movement_events (
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id             UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    -- trip_id FK will be added in migration 012 after trips table is created
    trip_id               UUID,
    timestamp             BIGINT NOT NULL,
    location              GEOGRAPHY(POINT, 4326) NOT NULL,
    accuracy              REAL NOT NULL,
    speed                 REAL,
    bearing               REAL,
    altitude              DOUBLE PRECISION,
    transportation_mode   VARCHAR(20) NOT NULL,
    confidence            REAL NOT NULL,
    detection_source      VARCHAR(30) NOT NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Data validation constraints
    CONSTRAINT chk_movement_accuracy CHECK (accuracy >= 0),
    CONSTRAINT chk_movement_confidence CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT chk_movement_bearing CHECK (bearing IS NULL OR (bearing >= 0 AND bearing <= 360)),
    CONSTRAINT chk_movement_speed CHECK (speed IS NULL OR speed >= 0),
    CONSTRAINT chk_movement_mode CHECK (transportation_mode IN ('STATIONARY', 'WALKING', 'RUNNING', 'CYCLING', 'IN_VEHICLE', 'UNKNOWN')),
    CONSTRAINT chk_movement_source CHECK (detection_source IN ('ACTIVITY_RECOGNITION', 'BLUETOOTH_CAR', 'ANDROID_AUTO', 'MULTIPLE', 'NONE'))
);

-- Index for device-based queries with timestamp ordering (primary query pattern)
CREATE INDEX idx_movement_events_device_timestamp ON movement_events(device_id, timestamp DESC);

-- Index for trip-based lookups (used when retrieving events for a trip)
CREATE INDEX idx_movement_events_trip_id ON movement_events(trip_id) WHERE trip_id IS NOT NULL;

-- PostGIS GIST index for geospatial queries on location
CREATE INDEX idx_movement_events_location ON movement_events USING GIST (location);

-- Index for pagination with keyset (timestamp, id)
CREATE INDEX idx_movement_events_device_timestamp_id ON movement_events(device_id, timestamp DESC, id);

COMMENT ON TABLE movement_events IS 'Stores movement events with sensor telemetry and geospatial data';
COMMENT ON COLUMN movement_events.timestamp IS 'Event timestamp in milliseconds since epoch';
COMMENT ON COLUMN movement_events.location IS 'Geographic point in WGS84 (SRID 4326)';
COMMENT ON COLUMN movement_events.transportation_mode IS 'Detected mode: STATIONARY, WALKING, RUNNING, CYCLING, IN_VEHICLE, UNKNOWN';
COMMENT ON COLUMN movement_events.confidence IS 'Detection confidence score between 0.0 and 1.0';
COMMENT ON COLUMN movement_events.detection_source IS 'How mode was detected: ACTIVITY_RECOGNITION, BLUETOOTH_CAR, ANDROID_AUTO, MULTIPLE, NONE';
