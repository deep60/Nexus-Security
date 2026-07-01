-- Initial schema for submission-service (database: verdyx_submissions).
--
-- Owns the `submissions` table that the service reads/writes at runtime.
-- Extensions (uuid-ossp, pgcrypto, pg_trgm) are installed by
-- /database/init/01-init-databases.sql on first volume initialization; this
-- migration only creates service-local tables.

-- ---------------------------------------------------------------------------
-- submissions: a file or URL submitted for analysis
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS submissions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submitter_id        UUID,
    file_hash           VARCHAR(128),
    url                 TEXT,
    original_filename   TEXT,
    file_size           BIGINT,
    mime_type           VARCHAR(255),
    file_path           TEXT,
    submission_type     VARCHAR(16) NOT NULL,
    is_malicious        BOOLEAN,
    confidence_score    DOUBLE PRECISION,
    analysis_status     VARCHAR(16) NOT NULL DEFAULT 'pending',
    metadata            JSONB,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_submissions_submitter_id ON submissions(submitter_id);
CREATE INDEX IF NOT EXISTS idx_submissions_file_hash ON submissions(file_hash);
CREATE INDEX IF NOT EXISTS idx_submissions_status ON submissions(analysis_status);
CREATE INDEX IF NOT EXISTS idx_submissions_type_created ON submissions(submission_type, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_submissions_created_at ON submissions(created_at DESC);
