-- Story 15.2: Geofence Events table
-- Stores geofence transition events from mobile devices with webhook delivery status

-- Create geofence_events table
CREATE TABLE IF NOT EXISTS geofence_events (
    id BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    device_id UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    geofence_id UUID NOT NULL REFERENCES geofences(geofence_id) ON DELETE CASCADE,
    event_type VARCHAR(20) NOT NULL,
    timestamp BIGINT NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    webhook_delivered BOOLEAN NOT NULL DEFAULT false,
    webhook_response_code INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraint: event_type must be one of the valid types
    CONSTRAINT geofence_events_event_type_check CHECK (event_type IN ('enter', 'exit', 'dwell')),

    -- Constraint: latitude must be valid (-90 to 90)
    CONSTRAINT geofence_events_latitude_check CHECK (latitude >= -90 AND latitude <= 90),

    -- Constraint: longitude must be valid (-180 to 180)
    CONSTRAINT geofence_events_longitude_check CHECK (longitude >= -180 AND longitude <= 180),

    -- Constraint: timestamp must be positive
    CONSTRAINT geofence_events_timestamp_check CHECK (timestamp > 0)
);

-- Index for device-based lookups
CREATE INDEX IF NOT EXISTS idx_geofence_events_device_id
    ON geofence_events(device_id);

-- Index for time-ordered queries by device
CREATE INDEX IF NOT EXISTS idx_geofence_events_device_id_timestamp
    ON geofence_events(device_id, timestamp DESC);

-- Index for geofence-based lookups
CREATE INDEX IF NOT EXISTS idx_geofence_events_geofence_id
    ON geofence_events(geofence_id);

-- Comment on table
COMMENT ON TABLE geofence_events IS 'Stores geofence transition events (enter, exit, dwell) from mobile devices';
COMMENT ON COLUMN geofence_events.event_id IS 'Unique identifier for the event (UUID)';
COMMENT ON COLUMN geofence_events.device_id IS 'Device that triggered the event';
COMMENT ON COLUMN geofence_events.geofence_id IS 'Geofence that was triggered';
COMMENT ON COLUMN geofence_events.event_type IS 'Type of transition: enter, exit, or dwell';
COMMENT ON COLUMN geofence_events.timestamp IS 'Event timestamp in milliseconds epoch';
COMMENT ON COLUMN geofence_events.webhook_delivered IS 'Whether webhook delivery was attempted and succeeded';
COMMENT ON COLUMN geofence_events.webhook_response_code IS 'HTTP response code from webhook delivery (null if not attempted)';
