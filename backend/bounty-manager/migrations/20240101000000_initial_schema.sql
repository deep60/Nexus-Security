-- Initial database schema for bounty-manager

-- Bounties table
CREATE TABLE IF NOT EXISTS bounties (
    id UUID PRIMARY KEY,
    creator VARCHAR(255) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    artifact_type VARCHAR(50) NOT NULL,
    artifact_hash VARCHAR(255),
    artifact_url TEXT,
    file_name VARCHAR(255),
    file_size BIGINT,
    mime_type VARCHAR(100),
    upload_path VARCHAR(500),
    reward_amount BIGINT NOT NULL,
    currency VARCHAR(100) NOT NULL,
    min_stake BIGINT NOT NULL,
    max_participants INTEGER,
    deadline TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'Active',
    consensus_threshold REAL NOT NULL DEFAULT 0.75,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB
);

CREATE INDEX idx_bounties_creator ON bounties(creator);
CREATE INDEX idx_bounties_status ON bounties(status);
CREATE INDEX idx_bounties_deadline ON bounties(deadline);

-- Submissions table
CREATE TABLE IF NOT EXISTS submissions (
    id UUID PRIMARY KEY,
    bounty_id UUID NOT NULL REFERENCES bounties(id) ON DELETE CASCADE,
    engine_id VARCHAR(255) NOT NULL,
    engine_type VARCHAR(50) NOT NULL,
    verdict VARCHAR(50) NOT NULL,
    confidence REAL NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    stake_amount BIGINT NOT NULL,
    analysis_details JSONB NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'Pending',
    transaction_hash VARCHAR(255),
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMP WITH TIME ZONE,
    accuracy_score REAL CHECK (accuracy_score >= 0 AND accuracy_score <= 1)
);

CREATE INDEX idx_submissions_bounty ON submissions(bounty_id);
CREATE INDEX idx_submissions_engine ON submissions(engine_id);
CREATE INDEX idx_submissions_status ON submissions(status);

-- Payouts table
CREATE TABLE IF NOT EXISTS payouts (
    id UUID PRIMARY KEY,
    bounty_id UUID NOT NULL REFERENCES bounties(id),
    submission_id UUID REFERENCES submissions(id),
    recipient VARCHAR(255) NOT NULL,
    amount BIGINT NOT NULL,
    currency VARCHAR(100) NOT NULL,
    payout_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'Pending',
    transaction_hash VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB
);

CREATE INDEX idx_payouts_bounty ON payouts(bounty_id);
CREATE INDEX idx_payouts_recipient ON payouts(recipient);
CREATE INDEX idx_payouts_status ON payouts(status);

-- Reputations table
CREATE TABLE IF NOT EXISTS reputations (
    engine_id VARCHAR(255) PRIMARY KEY,
    reputation_score REAL NOT NULL DEFAULT 1.0 CHECK (reputation_score >= 0),
    total_submissions INTEGER NOT NULL DEFAULT 0,
    correct_submissions INTEGER NOT NULL DEFAULT 0,
    accuracy_rate REAL NOT NULL DEFAULT 0.0 CHECK (accuracy_rate >= 0 AND accuracy_rate <= 1),
    average_confidence REAL NOT NULL DEFAULT 0.0 CHECK (average_confidence >= 0 AND average_confidence <= 1),
    total_stake BIGINT NOT NULL DEFAULT 0,
    rewards_earned BIGINT NOT NULL DEFAULT 0,
    penalties_incurred BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_reputations_score ON reputations(reputation_score DESC);

-- Disputes table (optional, for future use)
CREATE TABLE IF NOT EXISTS disputes (
    id UUID PRIMARY KEY,
    bounty_id UUID NOT NULL REFERENCES bounties(id),
    submission_id UUID REFERENCES submissions(id),
    disputer_id VARCHAR(255) NOT NULL,
    dispute_type VARCHAR(50) NOT NULL,
    reason TEXT NOT NULL,
    evidence JSONB,
    status VARCHAR(50) NOT NULL DEFAULT 'Open',
    severity VARCHAR(50) NOT NULL,
    stake_amount BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMP WITH TIME ZONE,
    resolver_id VARCHAR(255),
    resolution JSONB,
    metadata JSONB
);

CREATE INDEX idx_disputes_bounty ON disputes(bounty_id);
CREATE INDEX idx_disputes_status ON disputes(status);
