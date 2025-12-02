-- Migration: Create unlock_requests table for setting unlock request workflow
-- Story 12.6: Unlock Request Workflow

-- Create status enum for unlock requests
CREATE TYPE unlock_request_status AS ENUM ('pending', 'approved', 'denied', 'expired');

-- Create unlock_requests table
CREATE TABLE unlock_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    setting_key VARCHAR(100) NOT NULL REFERENCES setting_definitions(key) ON DELETE CASCADE,
    requested_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status unlock_request_status NOT NULL DEFAULT 'pending',
    reason TEXT,
    responded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    response_note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '7 days'),
    responded_at TIMESTAMPTZ
);

-- Create indexes for efficient querying
CREATE INDEX idx_unlock_requests_device_id ON unlock_requests(device_id);
CREATE INDEX idx_unlock_requests_status ON unlock_requests(status);
CREATE INDEX idx_unlock_requests_requested_by ON unlock_requests(requested_by);
CREATE INDEX idx_unlock_requests_expires_at ON unlock_requests(expires_at);

-- Create unique constraint: only one pending request per device+setting
CREATE UNIQUE INDEX idx_unlock_requests_pending ON unlock_requests(device_id, setting_key)
    WHERE status = 'pending';

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_unlock_requests_updated_at
    BEFORE UPDATE ON unlock_requests
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
