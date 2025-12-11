-- Analytics and reporting infrastructure
-- AP-10: Dashboard & Analytics

-- Report jobs table for async report generation
CREATE TABLE IF NOT EXISTS report_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    report_type VARCHAR(50) NOT NULL, -- 'users', 'devices', 'api_usage'
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'processing', 'completed', 'failed'
    parameters JSONB NOT NULL DEFAULT '{}',
    file_path TEXT, -- Path to generated report file
    file_size_bytes BIGINT,
    error_message TEXT,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- API usage analytics tracking
CREATE TABLE IF NOT EXISTS api_usage_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    endpoint_path VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    status_code INTEGER NOT NULL,
    response_time_ms INTEGER NOT NULL,
    request_size_bytes INTEGER,
    response_size_bytes INTEGER,
    api_key_id UUID,
    user_id UUID,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Daily API usage aggregates for faster analytics queries
CREATE TABLE IF NOT EXISTS api_usage_daily (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    usage_date DATE NOT NULL,
    endpoint_path VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    total_requests BIGINT NOT NULL DEFAULT 0,
    success_count BIGINT NOT NULL DEFAULT 0,
    error_count BIGINT NOT NULL DEFAULT 0,
    avg_response_time_ms DOUBLE PRECISION,
    p95_response_time_ms INTEGER,
    total_request_bytes BIGINT DEFAULT 0,
    total_response_bytes BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, usage_date, endpoint_path, method)
);

-- User activity analytics for user analytics endpoint
CREATE TABLE IF NOT EXISTS user_activity_daily (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    activity_date DATE NOT NULL,
    active_users BIGINT NOT NULL DEFAULT 0,
    new_users BIGINT NOT NULL DEFAULT 0,
    returning_users BIGINT NOT NULL DEFAULT 0,
    total_sessions BIGINT NOT NULL DEFAULT 0,
    avg_session_duration_seconds DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, activity_date)
);

-- Device activity analytics for device analytics endpoint
CREATE TABLE IF NOT EXISTS device_activity_daily (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    activity_date DATE NOT NULL,
    active_devices BIGINT NOT NULL DEFAULT 0,
    new_enrollments BIGINT NOT NULL DEFAULT 0,
    unenrollments BIGINT NOT NULL DEFAULT 0,
    total_locations_reported BIGINT NOT NULL DEFAULT 0,
    total_geofence_events BIGINT NOT NULL DEFAULT 0,
    total_commands_issued BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, activity_date)
);

-- Indexes for analytics queries
CREATE INDEX IF NOT EXISTS idx_report_jobs_org_id ON report_jobs(organization_id);
CREATE INDEX IF NOT EXISTS idx_report_jobs_status ON report_jobs(status);
CREATE INDEX IF NOT EXISTS idx_report_jobs_expires ON report_jobs(expires_at) WHERE status = 'completed';

CREATE INDEX IF NOT EXISTS idx_api_usage_analytics_org_date ON api_usage_analytics(organization_id, recorded_at);
CREATE INDEX IF NOT EXISTS idx_api_usage_analytics_endpoint ON api_usage_analytics(endpoint_path, method);

CREATE INDEX IF NOT EXISTS idx_api_usage_daily_org_date ON api_usage_daily(organization_id, usage_date);
CREATE INDEX IF NOT EXISTS idx_user_activity_daily_org_date ON user_activity_daily(organization_id, activity_date);
CREATE INDEX IF NOT EXISTS idx_device_activity_daily_org_date ON device_activity_daily(organization_id, activity_date);

-- Auto-delete expired report jobs
CREATE INDEX IF NOT EXISTS idx_report_jobs_cleanup ON report_jobs(expires_at) WHERE status = 'completed';
