-- Initial database schema for Nexus Security
-- This is a placeholder migration

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    wallet_address VARCHAR(42) UNIQUE NOT NULL,
    username VARCHAR(50) UNIQUE,
    email VARCHAR(255) UNIQUE,
    password_hash VARCHAR(255),
    reputation_score INTEGER DEFAULT 0,
    role VARCHAR(20) DEFAULT 'user',
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Bounties table
CREATE TABLE IF NOT EXISTS bounties (
    id UUID PRIMARY KEY,
    creator_id UUID REFERENCES users(id),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    reward_amount DECIMAL(20, 8),
    status VARCHAR(20) DEFAULT 'active',
    file_hash VARCHAR(64),
    deadline TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Analyses table
CREATE TABLE IF NOT EXISTS analyses (
    id UUID PRIMARY KEY,
    bounty_id UUID REFERENCES bounties(id),
    analyst_id UUID REFERENCES users(id),
    file_hash VARCHAR(64),
    verdict VARCHAR(20),
    confidence DECIMAL(5, 2),
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Submissions table
CREATE TABLE IF NOT EXISTS submissions (
    id UUID PRIMARY KEY,
    bounty_id UUID REFERENCES bounties(id),
    analyst_id UUID REFERENCES users(id),
    analysis_id UUID REFERENCES analyses(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
