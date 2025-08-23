-- Add migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable the pgcrypto extension for gen_random_uuid()
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create bounty_manager specific tables
CREATE TABLE IF NOT EXISTS bounty_submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bounty_id UUID NOT NULL REFERENCES bounties(id),
    submitter_address TEXT NOT NULL,
    analysis_result JSONB NOT NULL,
    threat_verdict VARCHAR(20) NOT NULL CHECK (threat_verdict IN ('Malicious', 'Benign', 'Suspicious', 'Unknown')),
    confidence_score DECIMAL(3, 2) NOT NULL CHECK (confidence_score >= 0 AND confidence_score <= 1),
    metadata JSONB DEFAULT '{}',
    stakes JSONB DEFAULT '[]',
    status VARCHAR(20) NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'Accepted', 'Rejected', 'Under_Review')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS reputation_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    engine_address TEXT NOT NULL UNIQUE,
    overall_score DECIMAL(5, 2) NOT NULL DEFAULT 0,
    accuracy_score DECIMAL(5, 2) NOT NULL DEFAULT 0,
    consistency_score DECIMAL(5, 2) NOT NULL DEFAULT 0,
    response_time_score DECIMAL(5, 2) NOT NULL DEFAULT 0,
    total_submissions INTEGER NOT NULL DEFAULT 0,
    correct_submissions INTEGER NOT NULL DEFAULT 0,
    specialty_areas TEXT[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS payout_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bounty_id UUID NOT NULL REFERENCES bounties(id),
    recipient_address TEXT NOT NULL,
    amount DECIMAL(20, 8) NOT NULL,
    token_address TEXT NOT NULL,
    transaction_hash TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'Completed', 'Failed', 'Cancelled')),
    payout_type VARCHAR(20) NOT NULL CHECK (payout_type IN ('Winner', 'Participant', 'Stake_Return', 'Slashed')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS stake_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id UUID NOT NULL REFERENCES bounty_submissions(id),
    staker_address TEXT NOT NULL,
    amount DECIMAL(20, 8) NOT NULL,
    token_address TEXT NOT NULL,
    transaction_hash TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Returned', 'Slashed')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add indexes for performance
CREATE INDEX IF NOT EXISTS idx_bounty_submissions_bounty_id ON bounty_submissions(bounty_id);
CREATE INDEX IF NOT EXISTS idx_bounty_submissions_submitter ON bounty_submissions(submitter_address);
CREATE INDEX IF NOT EXISTS idx_bounty_submissions_status ON bounty_submissions(status);
CREATE INDEX IF NOT EXISTS idx_reputation_scores_address ON reputation_scores(engine_address);
CREATE INDEX IF NOT EXISTS idx_payout_records_bounty_id ON payout_records(bounty_id);
CREATE INDEX IF NOT EXISTS idx_payout_records_recipient ON payout_records(recipient_address);
CREATE INDEX IF NOT EXISTS idx_stake_records_submission_id ON stake_records(submission_id);
CREATE INDEX IF NOT EXISTS idx_stake_records_staker ON stake_records(staker_address);
