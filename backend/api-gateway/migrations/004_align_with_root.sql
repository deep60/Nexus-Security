-- Migration 004: Align api-gateway schema with root migrations 005 + 003
-- Adds columns that the Rust models expect but 001_initial_schema.sql didn't create.
-- All use IF NOT EXISTS / ADD COLUMN IF NOT EXISTS for idempotency.

-- ============================================
-- Users: columns from root 005_fix_user_schema.sql
-- ============================================
ALTER TABLE users
ADD COLUMN IF NOT EXISTS total_stakes BIGINT DEFAULT 0,
ADD COLUMN IF NOT EXISTS successful_analyses INTEGER DEFAULT 0,
ADD COLUMN IF NOT EXISTS failed_analyses INTEGER DEFAULT 0,
ADD COLUMN IF NOT EXISTS is_engine BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS api_key VARCHAR(255) UNIQUE,
ADD COLUMN IF NOT EXISTS last_login TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_users_api_key ON users(api_key) WHERE api_key IS NOT NULL;

-- ============================================
-- Bounties: extra columns from root 005 + 003
-- ============================================
ALTER TABLE bounties
ADD COLUMN IF NOT EXISTS token_address VARCHAR(42),
ADD COLUMN IF NOT EXISTS on_chain_id BIGINT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_bounties_on_chain_id
    ON bounties(on_chain_id) WHERE on_chain_id IS NOT NULL;

-- ============================================
-- Analysis Results: columns from root 005
-- ============================================
ALTER TABLE analysis_results
ADD COLUMN IF NOT EXISTS bounty_id UUID REFERENCES bounties(id) ON DELETE CASCADE,
ADD COLUMN IF NOT EXISTS analyzer_id UUID REFERENCES engines(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_analysis_results_bounty ON analysis_results(bounty_id);
CREATE INDEX IF NOT EXISTS idx_analysis_results_analyzer ON analysis_results(analyzer_id);
