-- Initial database schema for Verdyx
-- Aligned with database/postgres/migrations/001_user_engine.sql + 002_bounty_system.sql
-- Uses IF NOT EXISTS so this is idempotent with the root schema

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users table (matches root 001_user_engine.sql)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    wallet_address VARCHAR(42) UNIQUE,
    reputation_score INTEGER DEFAULT 0,
    total_submissions INTEGER DEFAULT 0,
    successful_submissions INTEGER DEFAULT 0,
    is_verified BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Engines table (matches root 001_user_engine.sql)
CREATE TABLE IF NOT EXISTS engines (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    engine_type VARCHAR(20) NOT NULL,
    description TEXT,
    owner_id UUID REFERENCES users(id) ON DELETE SET NULL,
    api_endpoint VARCHAR(255),
    is_active BOOLEAN DEFAULT TRUE,
    accuracy_rate DECIMAL(5,4) DEFAULT 0.0000,
    total_analyses INTEGER DEFAULT 0,
    correct_analyses INTEGER DEFAULT 0,
    stake_amount DECIMAL(20,8) DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Submissions table (matches root 001_user_engine.sql)
CREATE TABLE IF NOT EXISTS submissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    submitter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_hash VARCHAR(64) UNIQUE,
    url TEXT,
    original_filename VARCHAR(255),
    file_size BIGINT,
    mime_type VARCHAR(100),
    category_id INTEGER,
    file_path TEXT,
    submission_type VARCHAR(10) NOT NULL,
    is_malicious BOOLEAN,
    confidence_score DECIMAL(5,4),
    analysis_status VARCHAR(20) DEFAULT 'pending',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Bounties table (matches root 002_bounty_system.sql)
CREATE TABLE IF NOT EXISTS bounties (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    title VARCHAR(200) NOT NULL,
    description TEXT,
    reward_amount DECIMAL(20,8) NOT NULL,
    min_stake_amount DECIMAL(20,8) NOT NULL DEFAULT 0,
    max_participants INTEGER,
    deadline TIMESTAMPTZ,
    bounty_status VARCHAR(20) DEFAULT 'active',
    requires_verification BOOLEAN DEFAULT FALSE,
    priority_level INTEGER DEFAULT 1,
    blockchain_tx_hash VARCHAR(66),
    smart_contract_address VARCHAR(42),
    total_staked DECIMAL(20,8) DEFAULT 0,
    participant_count INTEGER DEFAULT 0,
    consensus_threshold DECIMAL(3,2) DEFAULT 0.60,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Analysis results table (matches root 002_bounty_system.sql)
CREATE TABLE IF NOT EXISTS analysis_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    participation_id UUID,
    engine_id UUID NOT NULL REFERENCES engines(id) ON DELETE CASCADE,
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    verdict VARCHAR(20) NOT NULL,
    confidence_score DECIMAL(5,4) NOT NULL,
    threat_types TEXT[],
    analysis_duration INTEGER,
    detailed_report JSONB DEFAULT '{}',
    analysis_status VARCHAR(20) DEFAULT 'completed',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Consensus results table (matches root 002_bounty_system.sql)
CREATE TABLE IF NOT EXISTS consensus_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    bounty_id UUID NOT NULL REFERENCES bounties(id) ON DELETE CASCADE,
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    final_verdict VARCHAR(20) NOT NULL,
    confidence_score DECIMAL(5,4) NOT NULL,
    malicious_votes INTEGER DEFAULT 0,
    benign_votes INTEGER DEFAULT 0,
    suspicious_votes INTEGER DEFAULT 0,
    unknown_votes INTEGER DEFAULT 0,
    total_participants INTEGER NOT NULL,
    weighted_score DECIMAL(10,8),
    consensus_algorithm VARCHAR(50) DEFAULT 'majority_vote',
    calculation_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sessions table (matches root 001_user_engine.sql)
CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    user_agent TEXT
);

-- API keys table (matches root 001_user_engine.sql)
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    permissions TEXT[] DEFAULT ARRAY['read'],
    rate_limit INTEGER DEFAULT 1000,
    is_active BOOLEAN DEFAULT TRUE,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

-- Essential indexes
CREATE INDEX IF NOT EXISTS idx_users_wallet ON users(wallet_address);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_submissions_hash ON submissions(file_hash);
CREATE INDEX IF NOT EXISTS idx_submissions_submitter ON submissions(submitter_id);
CREATE INDEX IF NOT EXISTS idx_bounties_creator ON bounties(creator_id);
CREATE INDEX IF NOT EXISTS idx_bounties_status ON bounties(bounty_status);
CREATE INDEX IF NOT EXISTS idx_analysis_results_submission ON analysis_results(submission_id);
CREATE INDEX IF NOT EXISTS idx_consensus_bounty ON consensus_results(bounty_id);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id);
