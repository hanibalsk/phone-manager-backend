-- Data Subject Requests (GDPR/CCPA compliance)
-- Tracks requests for data access, deletion, portability, etc.

CREATE TYPE data_subject_request_type AS ENUM (
    'access',           -- Right to access personal data
    'deletion',         -- Right to erasure (right to be forgotten)
    'portability',      -- Right to data portability
    'rectification',    -- Right to rectification (correction)
    'restriction',      -- Right to restriction of processing
    'objection'         -- Right to object to processing
);

CREATE TYPE data_subject_request_status AS ENUM (
    'pending',          -- Request submitted, awaiting processing
    'in_progress',      -- Request being processed
    'completed',        -- Request fulfilled
    'rejected',         -- Request rejected (with reason)
    'cancelled'         -- Request cancelled by requester
);

CREATE TABLE data_subject_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Request details
    request_type data_subject_request_type NOT NULL,
    status data_subject_request_status NOT NULL DEFAULT 'pending',

    -- Subject identification
    subject_email VARCHAR(255) NOT NULL,
    subject_name VARCHAR(255),
    subject_user_id UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Request metadata
    description TEXT,
    rejection_reason TEXT,

    -- Processing information
    processed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    processed_at TIMESTAMPTZ,

    -- Result data (for access/portability requests)
    result_data JSONB,
    result_file_url TEXT,
    result_expires_at TIMESTAMPTZ,

    -- Compliance tracking
    due_date TIMESTAMPTZ NOT NULL,  -- GDPR requires response within 30 days

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_dsr_organization_id ON data_subject_requests(organization_id);
CREATE INDEX idx_dsr_status ON data_subject_requests(status);
CREATE INDEX idx_dsr_request_type ON data_subject_requests(request_type);
CREATE INDEX idx_dsr_subject_email ON data_subject_requests(subject_email);
CREATE INDEX idx_dsr_subject_user_id ON data_subject_requests(subject_user_id);
CREATE INDEX idx_dsr_created_at ON data_subject_requests(created_at DESC);
CREATE INDEX idx_dsr_due_date ON data_subject_requests(due_date);

-- Trigger for updated_at
CREATE TRIGGER set_dsr_updated_at
    BEFORE UPDATE ON data_subject_requests
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comment
COMMENT ON TABLE data_subject_requests IS 'GDPR/CCPA Data Subject Requests for compliance management';
