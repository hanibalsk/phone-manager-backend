-- Migration: 009_proximity_alerts
-- Description: Create proximity_alerts table and Haversine distance function
-- Created: 2025-01-26

-- Create Haversine distance function for proximity calculations
-- Returns distance in meters between two lat/lon points
CREATE OR REPLACE FUNCTION haversine_distance(
    lat1 DOUBLE PRECISION,
    lon1 DOUBLE PRECISION,
    lat2 DOUBLE PRECISION,
    lon2 DOUBLE PRECISION
) RETURNS DOUBLE PRECISION AS $$
DECLARE
    R CONSTANT DOUBLE PRECISION := 6371000; -- Earth's radius in meters
    d_lat DOUBLE PRECISION;
    d_lon DOUBLE PRECISION;
    a DOUBLE PRECISION;
    c DOUBLE PRECISION;
BEGIN
    -- Convert degrees to radians
    d_lat := RADIANS(lat2 - lat1);
    d_lon := RADIANS(lon2 - lon1);

    -- Haversine formula
    a := SIN(d_lat / 2) * SIN(d_lat / 2) +
         COS(RADIANS(lat1)) * COS(RADIANS(lat2)) *
         SIN(d_lon / 2) * SIN(d_lon / 2);
    c := 2 * ATAN2(SQRT(a), SQRT(1 - a));

    RETURN R * c;
END;
$$ LANGUAGE plpgsql IMMUTABLE STRICT;

-- Create proximity_alerts table
CREATE TABLE proximity_alerts (
    id BIGSERIAL PRIMARY KEY,
    alert_id UUID NOT NULL UNIQUE DEFAULT uuid_generate_v4(),

    -- Source device that owns this alert
    source_device_id UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,

    -- Target device to monitor proximity to
    target_device_id UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,

    -- Alert configuration
    name VARCHAR(100),
    radius_meters INTEGER NOT NULL,

    -- Alert state
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_triggered BOOLEAN NOT NULL DEFAULT FALSE,
    last_triggered_at TIMESTAMPTZ,

    -- Metadata for client-side customization
    metadata JSONB,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT radius_valid CHECK (radius_meters >= 50 AND radius_meters <= 100000),
    CONSTRAINT different_devices CHECK (source_device_id != target_device_id)
);

-- Indexes for efficient queries
CREATE INDEX idx_proximity_alerts_source_device ON proximity_alerts(source_device_id);
CREATE INDEX idx_proximity_alerts_target_device ON proximity_alerts(target_device_id);
CREATE INDEX idx_proximity_alerts_active ON proximity_alerts(is_active) WHERE is_active = TRUE;

-- Unique constraint: only one alert per source-target pair
CREATE UNIQUE INDEX idx_proximity_alerts_unique_pair
    ON proximity_alerts(source_device_id, target_device_id);

-- Trigger to update updated_at timestamp
CREATE TRIGGER update_proximity_alerts_updated_at
    BEFORE UPDATE ON proximity_alerts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add comments for documentation
COMMENT ON TABLE proximity_alerts IS 'Proximity alerts between devices in the same group';
COMMENT ON COLUMN proximity_alerts.source_device_id IS 'Device that owns and receives this alert';
COMMENT ON COLUMN proximity_alerts.target_device_id IS 'Device whose proximity is being monitored';
COMMENT ON COLUMN proximity_alerts.radius_meters IS 'Distance threshold in meters (50-100000)';
COMMENT ON COLUMN proximity_alerts.is_triggered IS 'Whether the alert is currently triggered (devices within radius)';
COMMENT ON COLUMN proximity_alerts.last_triggered_at IS 'Timestamp when alert was last triggered';
COMMENT ON FUNCTION haversine_distance IS 'Calculate distance in meters between two lat/lon coordinates';
