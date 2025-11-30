-- Migration 012: Trips table
-- Stores trip lifecycle data with geospatial support
-- Part of Epic 6: Trip Lifecycle Management

-- Create trips table
CREATE TABLE trips (
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id             UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    local_trip_id         VARCHAR(100) NOT NULL,
    state                 VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    start_timestamp       BIGINT NOT NULL,
    end_timestamp         BIGINT,
    start_location        GEOGRAPHY(POINT, 4326) NOT NULL,
    end_location          GEOGRAPHY(POINT, 4326),
    transportation_mode   VARCHAR(20) NOT NULL,
    detection_source      VARCHAR(30) NOT NULL,
    distance_meters       DOUBLE PRECISION,
    duration_seconds      BIGINT,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unique constraint for idempotent trip creation
    CONSTRAINT uq_trips_device_local_id UNIQUE (device_id, local_trip_id),

    -- State must be valid
    CONSTRAINT chk_trips_state CHECK (state IN ('ACTIVE', 'COMPLETED', 'CANCELLED')),

    -- Distance must be non-negative when set
    CONSTRAINT chk_trips_distance CHECK (distance_meters IS NULL OR distance_meters >= 0),

    -- Duration must be non-negative when set
    CONSTRAINT chk_trips_duration CHECK (duration_seconds IS NULL OR duration_seconds >= 0),

    -- Transportation mode must be valid
    CONSTRAINT chk_trips_mode CHECK (transportation_mode IN ('STATIONARY', 'WALKING', 'RUNNING', 'CYCLING', 'IN_VEHICLE', 'UNKNOWN')),

    -- Detection source must be valid
    CONSTRAINT chk_trips_source CHECK (detection_source IN ('ACTIVITY_RECOGNITION', 'BLUETOOTH_CAR', 'ANDROID_AUTO', 'MULTIPLE', 'NONE'))
);

-- Index for finding active trips by device (common query pattern)
CREATE INDEX idx_trips_device_state ON trips(device_id, state);

-- Index for trip history queries (sorted by start time)
CREATE INDEX idx_trips_device_start_timestamp ON trips(device_id, start_timestamp DESC);

-- Index for pagination with keyset (start_timestamp, id)
CREATE INDEX idx_trips_device_start_timestamp_id ON trips(device_id, start_timestamp DESC, id);

-- Trigger function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_trips_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for updated_at
CREATE TRIGGER trg_trips_updated_at
    BEFORE UPDATE ON trips
    FOR EACH ROW
    EXECUTE FUNCTION update_trips_updated_at();

-- Add foreign key from movement_events.trip_id to trips.id
-- This was deferred from migration 011 until trips table exists
ALTER TABLE movement_events
    ADD CONSTRAINT fk_movement_events_trip_id
    FOREIGN KEY (trip_id) REFERENCES trips(id) ON DELETE SET NULL;

-- Comments for documentation
COMMENT ON TABLE trips IS 'Stores trip lifecycle data with start/end locations and statistics';
COMMENT ON COLUMN trips.local_trip_id IS 'Client-generated ID for idempotent trip creation';
COMMENT ON COLUMN trips.state IS 'Trip state: ACTIVE, COMPLETED, or CANCELLED';
COMMENT ON COLUMN trips.start_timestamp IS 'Trip start time in milliseconds since epoch';
COMMENT ON COLUMN trips.end_timestamp IS 'Trip end time in milliseconds since epoch (null for active trips)';
COMMENT ON COLUMN trips.start_location IS 'Geographic point where trip started (WGS84)';
COMMENT ON COLUMN trips.end_location IS 'Geographic point where trip ended (null for active/cancelled trips)';
COMMENT ON COLUMN trips.distance_meters IS 'Total trip distance calculated from movement events';
COMMENT ON COLUMN trips.duration_seconds IS 'Trip duration calculated from timestamps';
