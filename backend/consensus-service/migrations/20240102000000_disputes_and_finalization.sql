-- Disputes table + consensus finalization columns for consensus-service.

-- Disputes raised against a consensus result.
CREATE TABLE IF NOT EXISTS consensus_disputes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bounty_id UUID NOT NULL,
    submission_id UUID,
    initiator_id UUID NOT NULL,
    disputed_verdict VARCHAR(50) NOT NULL,
    claimed_verdict VARCHAR(50) NOT NULL,
    reason TEXT NOT NULL,
    evidence JSONB,
    status VARCHAR(32) NOT NULL DEFAULT 'open',
    resolution TEXT,
    resolved_by UUID,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_disputes_bounty_id ON consensus_disputes(bounty_id);
CREATE INDEX IF NOT EXISTS idx_disputes_status ON consensus_disputes(status);

-- Extend consensus_results with finalization + agreement metadata so a result
-- is a complete record of a consensus calculation.
ALTER TABLE consensus_results
    ADD COLUMN IF NOT EXISTS agreement_score DECIMAL(5,4) NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS is_disputed BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS is_finalized BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS finalized_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS participating_engines TEXT[] NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS verdict_distribution JSONB NOT NULL DEFAULT '{}'::jsonb;

-- A bounty has at most one current consensus result.
CREATE UNIQUE INDEX IF NOT EXISTS uq_consensus_bounty ON consensus_results(bounty_id);
