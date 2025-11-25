-- Migration 002: Devices table
-- Stores device registration and group membership

CREATE TABLE devices (
    id              BIGSERIAL PRIMARY KEY,
    device_id       UUID NOT NULL UNIQUE,
    display_name    VARCHAR(50) NOT NULL,
    group_id        VARCHAR(50) NOT NULL,
    platform        VARCHAR(20) NOT NULL DEFAULT 'android',
    fcm_token       TEXT,
    active          BOOLEAN NOT NULL DEFAULT TRUE,
    last_seen_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for group queries (only active devices)
CREATE INDEX idx_devices_group_id ON devices(group_id) WHERE active = TRUE;

-- Index for device lookup by UUID
CREATE INDEX idx_devices_device_id ON devices(device_id);

-- Index for FCM token lookup (push notifications)
CREATE INDEX idx_devices_fcm_token ON devices(fcm_token) WHERE fcm_token IS NOT NULL;

-- Trigger to auto-update updated_at
CREATE TRIGGER update_devices_updated_at
    BEFORE UPDATE ON devices
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
