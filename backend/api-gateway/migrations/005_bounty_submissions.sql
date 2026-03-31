-- Migration 005: Bounty submissions and extended submissions tables
-- These tables back the BountySubmission and ExtendedSubmission Rust models
-- used by the submission API endpoints.

-- ============================================
-- Bounty submissions table
-- ============================================
CREATE TABLE IF NOT EXISTS bounty_submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bounty_id UUID NOT NULL REFERENCES bounties(id) ON DELETE CASCADE,
    engine_id UUID NOT NULL,
    engine_name VARCHAR(100) NOT NULL,
    engine_address VARCHAR(42) NOT NULL DEFAULT '0x0000000000000000000000000000000000000000',
    verdict VARCHAR(20) NOT NULL,
    confidence DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    stake_amount VARCHAR(78) NOT NULL DEFAULT '0',
    details JSONB NOT NULL DEFAULT '{}',
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_verified BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_bounty_submissions_bounty ON bounty_submissions(bounty_id);
CREATE INDEX IF NOT EXISTS idx_bounty_submissions_engine ON bounty_submissions(engine_id);
CREATE INDEX IF NOT EXISTS idx_bounty_submissions_verdict ON bounty_submissions(verdict);

-- ============================================
-- Extended submissions table (1:1 with bounty_submissions)
-- ============================================
CREATE TABLE IF NOT EXISTS extended_submissions (
    submission_id UUID PRIMARY KEY REFERENCES bounty_submissions(id) ON DELETE CASCADE,
    engine_version VARCHAR(50) NOT NULL DEFAULT '1.0',
    threat_types TEXT[] NOT NULL DEFAULT '{}',
    risk_score SMALLINT NOT NULL DEFAULT 0,
    analysis_summary TEXT NOT NULL DEFAULT '',
    signatures TEXT[] NOT NULL DEFAULT '{}',
    status VARCHAR(20) NOT NULL DEFAULT 'Pending',
    processing_metrics JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- Submission votes table (used by vote_on_submission handler)
-- ============================================
CREATE TABLE IF NOT EXISTS submission_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id UUID NOT NULL REFERENCES bounty_submissions(id) ON DELETE CASCADE,
    verdict VARCHAR(20) NOT NULL,
    confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_submission_votes_submission ON submission_votes(submission_id);
