-- Migration 007: Performance Indexes and Optimizations
-- Additional indexes for optimal query performance

-- Composite index for devices listing by group with all needed columns (covering index)
-- This allows the query to be satisfied entirely from the index
CREATE INDEX IF NOT EXISTS idx_devices_group_active_name
    ON devices(group_id, display_name)
    WHERE active = TRUE;

-- Index on idempotency keys for expiration cleanup
CREATE INDEX IF NOT EXISTS idx_idempotency_keys_expires_at
    ON idempotency_keys(expires_at)
    WHERE expires_at IS NOT NULL;

-- Analyze tables to ensure query planner has accurate statistics
ANALYZE devices;
ANALYZE locations;
ANALYZE api_keys;
ANALYZE idempotency_keys;

-- Performance targets documented (for reference):
-- 1. Device registration query: <20ms (uses idx_devices_device_id)
-- 2. Group device listing: <50ms for 20 devices (uses idx_devices_group_id + LATERAL join)
-- 3. Single location insert: <10ms (uses idx_locations_device_captured)
-- 4. Batch location insert (50): <100ms (transactional batch)
-- 5. Device with last location: <50ms (uses devices_with_last_location view)
-- 6. Location cleanup: Uses batch deletion with idx_locations_created_at
