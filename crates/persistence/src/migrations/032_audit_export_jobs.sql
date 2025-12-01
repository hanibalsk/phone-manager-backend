-- Migration: 032_audit_export_jobs.sql
-- Story 13.10: Audit Query and Export Endpoints
-- Tracks export job status for async audit log exports

-- Create export job status enum
CREATE TYPE audit_export_job_status AS ENUM ('pending', 'processing', 'completed', 'failed', 'expired');

-- Create audit_export_jobs table
CREATE TABLE IF NOT EXISTS audit_export_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id VARCHAR(100) NOT NULL UNIQUE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    status audit_export_job_status NOT NULL DEFAULT 'pending',
    format VARCHAR(10) NOT NULL DEFAULT 'json',
    filters JSONB,
    record_count BIGINT,
    download_url TEXT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '24 hours'),
    completed_at TIMESTAMPTZ
);

-- Index for finding jobs by organization
CREATE INDEX IF NOT EXISTS idx_audit_export_jobs_org
    ON audit_export_jobs(organization_id, created_at DESC);

-- Index for finding jobs by status (for cleanup)
CREATE INDEX IF NOT EXISTS idx_audit_export_jobs_status
    ON audit_export_jobs(status, expires_at);

-- Index for looking up by job_id
CREATE INDEX IF NOT EXISTS idx_audit_export_jobs_job_id
    ON audit_export_jobs(job_id);

-- Comment on table
COMMENT ON TABLE audit_export_jobs IS 'Tracks async audit log export jobs with status and download URLs';
COMMENT ON COLUMN audit_export_jobs.job_id IS 'User-facing job identifier (export_<random>)';
COMMENT ON COLUMN audit_export_jobs.filters IS 'JSON object with filter parameters used for the export';
COMMENT ON COLUMN audit_export_jobs.download_url IS 'Data URL or external storage URL for completed exports';
