-- Migration 013: Add context fields to locations table
-- Epic 7 - Enhanced Location Context (Story 7.1)
--
-- Adds transportation mode, detection source, and trip linkage to locations.
-- All columns nullable for backward compatibility with existing data.
-- Uses ADD COLUMN which is non-blocking in PostgreSQL.

-- Add transportation_mode column
-- Matches movement_events.transportation_mode for consistency
ALTER TABLE locations
ADD COLUMN transportation_mode VARCHAR(20);

-- Add detection_source column
-- Matches movement_events.detection_source for consistency
ALTER TABLE locations
ADD COLUMN detection_source VARCHAR(30);

-- Add trip_id foreign key column
-- Links location to an active trip (optional)
-- ON DELETE SET NULL preserves locations if trip is deleted
ALTER TABLE locations
ADD COLUMN trip_id UUID REFERENCES trips(id) ON DELETE SET NULL;

-- Index on trip_id for efficient trip-based location queries
-- Supports "get all locations for a trip" queries
CREATE INDEX idx_locations_trip_id ON locations(trip_id)
WHERE trip_id IS NOT NULL;

-- Comment documenting the context fields
COMMENT ON COLUMN locations.transportation_mode IS 'Transportation mode when location was captured (e.g., WALKING, IN_VEHICLE)';
COMMENT ON COLUMN locations.detection_source IS 'How the transportation mode was detected (e.g., ACTIVITY_RECOGNITION, BLUETOOTH_CAR)';
COMMENT ON COLUMN locations.trip_id IS 'Optional link to the active trip when this location was recorded';
