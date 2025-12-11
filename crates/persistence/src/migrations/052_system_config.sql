-- System configuration storage for dynamic settings
-- AP-9: System Configuration mutation APIs

-- System settings overrides (key-value store for runtime settings)
CREATE TABLE IF NOT EXISTS system_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    setting_key VARCHAR(100) NOT NULL UNIQUE,
    setting_value JSONB NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL,
    is_sensitive BOOLEAN NOT NULL DEFAULT FALSE,
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Feature flags with database-backed overrides
CREATE TABLE IF NOT EXISTS feature_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flag_key VARCHAR(100) NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    description TEXT,
    category VARCHAR(50) NOT NULL DEFAULT 'general',
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Rate limit overrides
CREATE TABLE IF NOT EXISTS rate_limit_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    limit_key VARCHAR(100) NOT NULL UNIQUE,
    requests_per_period INTEGER NOT NULL,
    period_seconds INTEGER NOT NULL DEFAULT 60,
    description TEXT,
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Notification templates
CREATE TABLE IF NOT EXISTS notification_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_key VARCHAR(100) NOT NULL UNIQUE,
    template_type VARCHAR(50) NOT NULL, -- 'push', 'in_app', 'sms'
    title_template TEXT NOT NULL,
    body_template TEXT NOT NULL,
    data_schema JSONB, -- JSON schema for template variables
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Email templates
CREATE TABLE IF NOT EXISTS email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_key VARCHAR(100) NOT NULL UNIQUE,
    subject_template TEXT NOT NULL,
    body_html_template TEXT NOT NULL,
    body_text_template TEXT,
    data_schema JSONB, -- JSON schema for template variables
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_system_settings_category ON system_settings(category);
CREATE INDEX IF NOT EXISTS idx_feature_flags_category ON feature_flags(category);
CREATE INDEX IF NOT EXISTS idx_notification_templates_type ON notification_templates(template_type);
CREATE INDEX IF NOT EXISTS idx_notification_templates_active ON notification_templates(is_active);
CREATE INDEX IF NOT EXISTS idx_email_templates_active ON email_templates(is_active);

-- Insert default feature flags (matching config file defaults)
INSERT INTO feature_flags (flag_key, enabled, description, category) VALUES
    ('geofences_enabled', true, 'Enable geofence functionality', 'features'),
    ('proximity_alerts_enabled', true, 'Enable proximity alert functionality', 'features'),
    ('webhooks_enabled', true, 'Enable webhook functionality', 'features'),
    ('movement_tracking_enabled', true, 'Enable movement tracking (trips)', 'features'),
    ('b2b_enabled', true, 'Enable B2B organization features', 'features'),
    ('geofence_events_enabled', true, 'Enable geofence event endpoints', 'features'),
    ('registration_enabled', true, 'Enable user registration', 'auth'),
    ('invite_only', false, 'Require invite token for registration', 'auth'),
    ('oauth_only', false, 'Disable password authentication', 'auth')
ON CONFLICT (flag_key) DO NOTHING;

-- Insert default rate limit configurations
INSERT INTO rate_limit_configs (limit_key, requests_per_period, period_seconds, description) VALUES
    ('api_general', 100, 60, 'General API rate limit per minute'),
    ('export', 10, 3600, 'Export rate limit per hour'),
    ('forgot_password', 5, 3600, 'Forgot password rate limit per hour'),
    ('request_verification', 3, 3600, 'Email verification request rate limit per hour'),
    ('map_matching', 30, 60, 'Map matching API rate limit per minute')
ON CONFLICT (limit_key) DO NOTHING;

-- Insert default email templates
INSERT INTO email_templates (template_key, subject_template, body_html_template, body_text_template) VALUES
    ('welcome', 'Welcome to {{app_name}}!', '<h1>Welcome to {{app_name}}</h1><p>Hi {{user_name}},</p><p>Thank you for joining!</p>', 'Welcome to {{app_name}}!\n\nHi {{user_name}},\n\nThank you for joining!'),
    ('password_reset', 'Reset your {{app_name}} password', '<h1>Password Reset</h1><p>Hi {{user_name}},</p><p>Click <a href="{{reset_link}}">here</a> to reset your password.</p>', 'Password Reset\n\nHi {{user_name}},\n\nClick this link to reset your password: {{reset_link}}'),
    ('email_verification', 'Verify your email for {{app_name}}', '<h1>Email Verification</h1><p>Hi {{user_name}},</p><p>Click <a href="{{verify_link}}">here</a> to verify your email.</p>', 'Email Verification\n\nHi {{user_name}},\n\nClick this link to verify your email: {{verify_link}}'),
    ('invitation', 'You''ve been invited to join {{org_name}}', '<h1>Invitation</h1><p>Hi {{user_name}},</p><p>You''ve been invited to join {{org_name}}. Click <a href="{{invite_link}}">here</a> to accept.</p>', 'Invitation\n\nHi {{user_name}},\n\nYou''ve been invited to join {{org_name}}. Click this link to accept: {{invite_link}}')
ON CONFLICT (template_key) DO NOTHING;

-- Insert default notification templates
INSERT INTO notification_templates (template_key, template_type, title_template, body_template) VALUES
    ('geofence_enter', 'push', '{{device_name}} entered {{geofence_name}}', '{{device_name}} has entered the geofence "{{geofence_name}}" at {{time}}'),
    ('geofence_exit', 'push', '{{device_name}} left {{geofence_name}}', '{{device_name}} has left the geofence "{{geofence_name}}" at {{time}}'),
    ('proximity_alert', 'push', '{{source_device}} is near {{target_device}}', '{{source_device}} is now within {{distance}}m of {{target_device}}'),
    ('device_low_battery', 'push', '{{device_name}} has low battery', '{{device_name}} battery is at {{battery_level}}%'),
    ('trip_started', 'push', '{{device_name}} started a trip', '{{device_name}} began a new trip at {{time}}'),
    ('trip_ended', 'push', '{{device_name}} completed a trip', '{{device_name}} completed a trip. Distance: {{distance}}km')
ON CONFLICT (template_key) DO NOTHING;
