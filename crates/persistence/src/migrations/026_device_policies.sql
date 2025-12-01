-- Migration 026: Device Policies Table
-- Story 13.3: Device Policies Table and CRUD Endpoints
-- Creates device_policies table for organization-wide device configuration management

-- Create device_policies table
CREATE TABLE device_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    settings JSONB NOT NULL DEFAULT '{}',
    locked_settings TEXT[] NOT NULL DEFAULT '{}',
    priority INTEGER NOT NULL DEFAULT 0,
    device_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_device_policies_organization ON device_policies(organization_id);
CREATE INDEX idx_device_policies_is_default ON device_policies(organization_id, is_default) WHERE is_default = true;
CREATE INDEX idx_device_policies_priority ON device_policies(organization_id, priority DESC);

-- Add unique constraint for policy name within organization
ALTER TABLE device_policies ADD CONSTRAINT device_policies_name_unique UNIQUE (organization_id, name);

-- Add policy_id column to devices table
ALTER TABLE devices ADD COLUMN policy_id UUID REFERENCES device_policies(id) ON DELETE SET NULL;

-- Create index for devices by policy
CREATE INDEX idx_devices_policy ON devices(policy_id) WHERE policy_id IS NOT NULL;

-- Create trigger function to ensure only one default policy per organization
CREATE OR REPLACE FUNCTION ensure_single_default_policy()
RETURNS TRIGGER AS $$
BEGIN
    -- If setting this policy as default, unset any existing default
    IF NEW.is_default = true THEN
        UPDATE device_policies
        SET is_default = false, updated_at = NOW()
        WHERE organization_id = NEW.organization_id
          AND id != NEW.id
          AND is_default = true;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for insert and update
CREATE TRIGGER trigger_ensure_single_default_policy
    BEFORE INSERT OR UPDATE ON device_policies
    FOR EACH ROW
    WHEN (NEW.is_default = true)
    EXECUTE FUNCTION ensure_single_default_policy();

-- Create trigger function to update device_count when devices are assigned/unassigned
CREATE OR REPLACE FUNCTION update_policy_device_count()
RETURNS TRIGGER AS $$
BEGIN
    -- Handle INSERT or UPDATE where policy_id changed
    IF TG_OP = 'INSERT' THEN
        IF NEW.policy_id IS NOT NULL THEN
            UPDATE device_policies
            SET device_count = device_count + 1, updated_at = NOW()
            WHERE id = NEW.policy_id;
        END IF;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.policy_id IS DISTINCT FROM NEW.policy_id THEN
            -- Decrement old policy count
            IF OLD.policy_id IS NOT NULL THEN
                UPDATE device_policies
                SET device_count = GREATEST(device_count - 1, 0), updated_at = NOW()
                WHERE id = OLD.policy_id;
            END IF;
            -- Increment new policy count
            IF NEW.policy_id IS NOT NULL THEN
                UPDATE device_policies
                SET device_count = device_count + 1, updated_at = NOW()
                WHERE id = NEW.policy_id;
            END IF;
        END IF;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        IF OLD.policy_id IS NOT NULL THEN
            UPDATE device_policies
            SET device_count = GREATEST(device_count - 1, 0), updated_at = NOW()
            WHERE id = OLD.policy_id;
        END IF;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create trigger on devices table
CREATE TRIGGER trigger_update_policy_device_count
    AFTER INSERT OR UPDATE OR DELETE ON devices
    FOR EACH ROW
    EXECUTE FUNCTION update_policy_device_count();

-- Add comment
COMMENT ON TABLE device_policies IS 'Organization device policies defining standard configurations and locks';
COMMENT ON COLUMN device_policies.settings IS 'JSON object with setting key-value pairs';
COMMENT ON COLUMN device_policies.locked_settings IS 'Array of setting keys that cannot be modified by users';
COMMENT ON COLUMN device_policies.priority IS 'Higher numbers take precedence in policy resolution';
COMMENT ON COLUMN device_policies.device_count IS 'Denormalized count of devices using this policy';
