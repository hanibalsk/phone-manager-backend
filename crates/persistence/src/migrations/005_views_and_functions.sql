-- Migration 005: Views and Functions
-- Materialized views, utility views, and maintenance functions

-- Materialized view for group member counts (refreshed periodically)
CREATE MATERIALIZED VIEW group_member_counts AS
SELECT
    group_id,
    COUNT(*) as member_count,
    MAX(last_seen_at) as last_activity
FROM devices
WHERE active = TRUE
GROUP BY group_id;

CREATE UNIQUE INDEX idx_group_member_counts ON group_member_counts(group_id);

-- View for devices with their last location (used for group device listings)
CREATE VIEW devices_with_last_location AS
SELECT
    d.id,
    d.device_id,
    d.display_name,
    d.group_id,
    d.platform,
    d.fcm_token,
    d.active,
    d.last_seen_at,
    d.created_at,
    d.updated_at,
    l.latitude as last_latitude,
    l.longitude as last_longitude,
    l.captured_at as last_location_time,
    l.accuracy as last_accuracy
FROM devices d
LEFT JOIN LATERAL (
    SELECT latitude, longitude, captured_at, accuracy
    FROM locations
    WHERE device_id = d.device_id
    ORDER BY captured_at DESC
    LIMIT 1
) l ON true;

-- Function to clean up old locations (called by background job)
CREATE OR REPLACE FUNCTION cleanup_old_locations(retention_days INTEGER DEFAULT 30)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM locations
    WHERE created_at < NOW() - (retention_days || ' days')::INTERVAL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to refresh group member counts materialized view
CREATE OR REPLACE FUNCTION refresh_group_member_counts()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY group_member_counts;
END;
$$ LANGUAGE plpgsql;
