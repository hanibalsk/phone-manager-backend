-- Migration: Device Settings Schema
-- Epic 12: Settings Control
-- Story 12.1: Device Settings Database Schema

-- Create setting_data_type enum
CREATE TYPE setting_data_type AS ENUM ('boolean', 'integer', 'string', 'float', 'json');

-- Create setting_category enum
CREATE TYPE setting_category AS ENUM ('tracking', 'privacy', 'notifications', 'battery', 'general');

-- Table: setting_definitions
-- Stores metadata about available settings
CREATE TABLE setting_definitions (
    key VARCHAR(100) PRIMARY KEY,
    display_name VARCHAR(200) NOT NULL,
    description TEXT,
    data_type setting_data_type NOT NULL,
    default_value JSONB NOT NULL,
    is_lockable BOOLEAN NOT NULL DEFAULT true,
    category setting_category NOT NULL DEFAULT 'general',
    validation_rules JSONB,  -- Optional: min, max, pattern, enum values
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table: device_settings
-- Stores per-device setting values and lock states
CREATE TABLE device_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    setting_key VARCHAR(100) NOT NULL REFERENCES setting_definitions(key),
    value JSONB NOT NULL,
    is_locked BOOLEAN NOT NULL DEFAULT false,
    locked_by UUID REFERENCES users(id) ON DELETE SET NULL,
    locked_at TIMESTAMPTZ,
    lock_reason VARCHAR(500),
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Composite unique constraint
    CONSTRAINT uq_device_setting UNIQUE (device_id, setting_key),

    -- Ensure lock fields are consistent
    CONSTRAINT chk_lock_consistency CHECK (
        (is_locked = false AND locked_by IS NULL AND locked_at IS NULL)
        OR (is_locked = true AND locked_by IS NOT NULL AND locked_at IS NOT NULL)
    )
);

-- Indexes for query performance
CREATE INDEX idx_device_settings_device_id ON device_settings(device_id);
CREATE INDEX idx_device_settings_setting_key ON device_settings(setting_key);
CREATE INDEX idx_device_settings_is_locked ON device_settings(is_locked) WHERE is_locked = true;
CREATE INDEX idx_setting_definitions_category ON setting_definitions(category);

-- Seed initial setting definitions
INSERT INTO setting_definitions (key, display_name, description, data_type, default_value, is_lockable, category, sort_order)
VALUES
    ('tracking_enabled', 'Location Tracking', 'Enable or disable location tracking', 'boolean', 'true', true, 'tracking', 1),
    ('tracking_interval_minutes', 'Tracking Interval', 'Minutes between location updates', 'integer', '5', true, 'tracking', 2),
    ('movement_detection_enabled', 'Movement Detection', 'Enable automatic movement detection', 'boolean', 'true', true, 'tracking', 3),
    ('secret_mode_enabled', 'Secret Mode', 'Hide device location from other group members', 'boolean', 'false', true, 'privacy', 10),
    ('battery_optimization_enabled', 'Battery Optimization', 'Reduce tracking frequency when battery is low', 'boolean', 'true', false, 'battery', 20),
    ('notification_sounds_enabled', 'Notification Sounds', 'Play sounds for notifications', 'boolean', 'true', false, 'notifications', 30),
    ('geofence_notifications_enabled', 'Geofence Alerts', 'Receive notifications for geofence events', 'boolean', 'true', true, 'notifications', 31),
    ('sos_enabled', 'SOS Feature', 'Enable emergency SOS functionality', 'boolean', 'true', true, 'privacy', 11);

-- Comment on tables
COMMENT ON TABLE setting_definitions IS 'Catalog of available device settings with metadata';
COMMENT ON TABLE device_settings IS 'Per-device setting values with lock states';
