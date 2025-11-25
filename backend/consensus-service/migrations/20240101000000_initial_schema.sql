-- Initial schema for consensus-service
-- This migration can be expanded based on your database needs

-- Create consensus_results table if needed
CREATE TABLE IF NOT EXISTS consensus_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bounty_id UUID NOT NULL,
    final_verdict VARCHAR(50) NOT NULL,
    confidence DECIMAL(5,4) NOT NULL,
    total_submissions INTEGER NOT NULL,
    malicious_count INTEGER DEFAULT 0,
    benign_count INTEGER DEFAULT 0,
    suspicious_count INTEGER DEFAULT 0,
    unknown_count INTEGER DEFAULT 0,
    weighted_voting BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_consensus_bounty_id ON consensus_results(bounty_id);
CREATE INDEX IF NOT EXISTS idx_consensus_created_at ON consensus_results(created_at DESC);

-- Create submissions table if needed
CREATE TABLE IF NOT EXISTS consensus_submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bounty_id UUID NOT NULL,
    engine_id VARCHAR(255) NOT NULL,
    verdict VARCHAR(50) NOT NULL,
    confidence DECIMAL(5,4) NOT NULL,
    reputation_score INTEGER DEFAULT 0,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(bounty_id, engine_id)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_submissions_bounty_id ON consensus_submissions(bounty_id);
CREATE INDEX IF NOT EXISTS idx_submissions_engine_id ON consensus_submissions(engine_id);
