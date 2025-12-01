-- Migration: 030_bulk_device_import.sql
-- Story 13.8: Bulk Device Import Endpoint
-- Add external_id and metadata columns for bulk import support

-- Add external_id column to devices (unique within organization)
ALTER TABLE devices
    ADD COLUMN IF NOT EXISTS external_id VARCHAR(255);

-- Add metadata JSONB column to devices
ALTER TABLE devices
    ADD COLUMN IF NOT EXISTS metadata JSONB;

-- Create unique index for external_id within organization
CREATE UNIQUE INDEX IF NOT EXISTS idx_devices_org_external_id
    ON devices(organization_id, external_id)
    WHERE external_id IS NOT NULL;

-- Create index for faster lookups by external_id
CREATE INDEX IF NOT EXISTS idx_devices_external_id
    ON devices(external_id)
    WHERE external_id IS NOT NULL;

-- Create bulk_import_jobs table for async import tracking (future use)
CREATE TABLE IF NOT EXISTS bulk_import_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    initiated_by UUID NOT NULL REFERENCES users(id),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    total_devices INT NOT NULL DEFAULT 0,
    processed_devices INT NOT NULL DEFAULT 0,
    created_devices INT NOT NULL DEFAULT 0,
    updated_devices INT NOT NULL DEFAULT 0,
    skipped_devices INT NOT NULL DEFAULT 0,
    error_count INT NOT NULL DEFAULT 0,
    errors JSONB,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on organization_id for bulk import jobs
CREATE INDEX IF NOT EXISTS idx_bulk_import_jobs_org_id ON bulk_import_jobs(organization_id);
CREATE INDEX IF NOT EXISTS idx_bulk_import_jobs_status ON bulk_import_jobs(status);
