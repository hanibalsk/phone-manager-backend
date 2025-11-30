-- Migration 014: Trip Path Corrections table
-- Stores original and map-matched path coordinates for trips
-- Part of Epic 8: Intelligent Path Detection

-- Create trip_path_corrections table
CREATE TABLE trip_path_corrections (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trip_id             UUID NOT NULL UNIQUE REFERENCES trips(id) ON DELETE CASCADE,
    original_path       GEOGRAPHY(LINESTRING, 4326) NOT NULL,
    corrected_path      GEOGRAPHY(LINESTRING, 4326),
    correction_quality  REAL,
    correction_status   VARCHAR(20) NOT NULL DEFAULT 'PENDING',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Quality must be between 0.0 and 1.0 when set
    CONSTRAINT chk_correction_quality CHECK (
        correction_quality IS NULL OR (correction_quality >= 0.0 AND correction_quality <= 1.0)
    ),

    -- Status must be valid
    CONSTRAINT chk_correction_status CHECK (
        correction_status IN ('PENDING', 'COMPLETED', 'FAILED', 'SKIPPED')
    )
);

-- Index for finding corrections by status (for processing queue)
CREATE INDEX idx_trip_path_corrections_status ON trip_path_corrections(correction_status);

-- Index for finding correction by trip_id (for retrieval)
-- Note: trip_id already has unique constraint which creates an index,
-- but explicit index clarifies intent

-- Trigger function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_trip_path_corrections_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for updated_at
CREATE TRIGGER trg_trip_path_corrections_updated_at
    BEFORE UPDATE ON trip_path_corrections
    FOR EACH ROW
    EXECUTE FUNCTION update_trip_path_corrections_updated_at();

-- Comments for documentation
COMMENT ON TABLE trip_path_corrections IS 'Stores original and map-snapped path coordinates for trips';
COMMENT ON COLUMN trip_path_corrections.trip_id IS 'Reference to the trip (one-to-one relationship)';
COMMENT ON COLUMN trip_path_corrections.original_path IS 'Original GPS trace as ordered sequence of points (WGS84)';
COMMENT ON COLUMN trip_path_corrections.corrected_path IS 'Map-matched path snapped to road network (WGS84)';
COMMENT ON COLUMN trip_path_corrections.correction_quality IS 'Confidence metric from map-matching service (0.0-1.0)';
COMMENT ON COLUMN trip_path_corrections.correction_status IS 'Status: PENDING, COMPLETED, FAILED, or SKIPPED';
