-- App Usage Tracking
-- AP-8.1-8.2, AP-8.7: App usage summary, history, and analytics

-- App usage records table
CREATE TABLE IF NOT EXISTS app_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    device_id UUID NOT NULL,
    package_name VARCHAR(255) NOT NULL,
    app_name VARCHAR(255),
    category VARCHAR(100),

    -- Usage metrics
    foreground_time_ms BIGINT NOT NULL DEFAULT 0,
    background_time_ms BIGINT NOT NULL DEFAULT 0,
    launch_count INTEGER NOT NULL DEFAULT 0,
    notification_count INTEGER NOT NULL DEFAULT 0,

    -- Time window for this record
    usage_date DATE NOT NULL,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unique constraint: one record per device/app/date
    UNIQUE (organization_id, device_id, package_name, usage_date)
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_app_usage_org_device ON app_usage(organization_id, device_id);
CREATE INDEX IF NOT EXISTS idx_app_usage_date ON app_usage(usage_date);
CREATE INDEX IF NOT EXISTS idx_app_usage_org_date ON app_usage(organization_id, usage_date);
CREATE INDEX IF NOT EXISTS idx_app_usage_package ON app_usage(package_name);
CREATE INDEX IF NOT EXISTS idx_app_usage_category ON app_usage(category);

-- App usage daily aggregates for faster analytics queries
CREATE TABLE IF NOT EXISTS app_usage_daily_aggregates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    usage_date DATE NOT NULL,

    -- Aggregate metrics
    total_devices INTEGER NOT NULL DEFAULT 0,
    total_foreground_time_ms BIGINT NOT NULL DEFAULT 0,
    total_background_time_ms BIGINT NOT NULL DEFAULT 0,
    total_launches INTEGER NOT NULL DEFAULT 0,
    unique_apps INTEGER NOT NULL DEFAULT 0,

    -- Top apps by foreground time (JSON array)
    top_apps_by_time JSONB,
    -- Top categories by usage (JSON array)
    top_categories JSONB,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (organization_id, usage_date)
);

CREATE INDEX IF NOT EXISTS idx_app_usage_daily_org_date ON app_usage_daily_aggregates(organization_id, usage_date);

-- Function to update app_usage updated_at
CREATE OR REPLACE FUNCTION update_app_usage_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for app_usage updated_at
DROP TRIGGER IF EXISTS trigger_app_usage_updated_at ON app_usage;
CREATE TRIGGER trigger_app_usage_updated_at
    BEFORE UPDATE ON app_usage
    FOR EACH ROW
    EXECUTE FUNCTION update_app_usage_updated_at();

-- Trigger for daily aggregates updated_at
DROP TRIGGER IF EXISTS trigger_app_usage_daily_updated_at ON app_usage_daily_aggregates;
CREATE TRIGGER trigger_app_usage_daily_updated_at
    BEFORE UPDATE ON app_usage_daily_aggregates
    FOR EACH ROW
    EXECUTE FUNCTION update_app_usage_updated_at();
