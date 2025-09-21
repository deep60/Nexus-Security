-- Fix user schema to match Rust models
-- This migration adds missing columns that the Rust User struct expects

-- Add missing columns to users table
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS total_stakes BIGINT DEFAULT 0,
ADD COLUMN IF NOT EXISTS successful_analyses INTEGER DEFAULT 0,
ADD COLUMN IF NOT EXISTS failed_analyses INTEGER DEFAULT 0,
ADD COLUMN IF NOT EXISTS is_engine BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS api_key VARCHAR(255) UNIQUE,
ADD COLUMN IF NOT EXISTS last_login TIMESTAMP WITH TIME ZONE;

-- Populate new fields from existing data where possible
UPDATE users SET 
    successful_analyses = successful_submissions,
    failed_analyses = GREATEST(0, total_submissions - successful_submissions),
    total_stakes = 0;

-- Add index for api_key lookups
CREATE INDEX IF NOT EXISTS idx_users_api_key ON users(api_key) WHERE api_key IS NOT NULL;

-- Add missing columns to bounties table that the model expects
ALTER TABLE bounties
ADD COLUMN IF NOT EXISTS token_address VARCHAR(42),
ADD COLUMN IF NOT EXISTS status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'completed', 'cancelled', 'expired'));

-- Add missing columns to analysis_results table
ALTER TABLE analysis_results
ADD COLUMN IF NOT EXISTS bounty_id UUID REFERENCES bounties(id) ON DELETE CASCADE,
ADD COLUMN IF NOT EXISTS analyzer_id UUID REFERENCES engines(id) ON DELETE SET NULL;

-- Create index for new foreign keys
CREATE INDEX IF NOT EXISTS idx_analysis_results_bounty ON analysis_results(bounty_id);
CREATE INDEX IF NOT EXISTS idx_analysis_results_analyzer ON analysis_results(analyzer_id);
