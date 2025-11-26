-- Migration: 010_drop_recent_index
-- Description: Remove problematic partial index that uses non-immutable NOW()
--
-- The idx_locations_recent partial index was created in migration 003 with a
-- WHERE clause using NOW(), which is not IMMUTABLE. This can cause issues with
-- query planning and index maintenance. The idx_locations_device_captured index
-- already provides efficient lookups for device location queries.

DROP INDEX IF EXISTS idx_locations_recent;
