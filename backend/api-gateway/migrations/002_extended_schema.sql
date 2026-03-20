-- Migration 002: Extended schema for full API support
-- Adds missing columns to existing tables and creates new tables

-- ============================================
-- Extend bounties table
-- ============================================
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS creator_address VARCHAR(42) DEFAULT '';
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS bounty_type VARCHAR(30) DEFAULT 'custom';
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS priority VARCHAR(20) DEFAULT 'medium';
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS total_reward VARCHAR(78) DEFAULT '0';
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS minimum_stake VARCHAR(78) DEFAULT '0';
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS distribution_method VARCHAR(30) DEFAULT 'proportional_stake';
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS max_participants INTEGER;
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS current_participants INTEGER DEFAULT 0;
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS required_consensus DECIMAL(5,2) DEFAULT 70.0;
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS minimum_reputation DECIMAL(5,2) DEFAULT 0.0;
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS auto_finalize BOOLEAN DEFAULT TRUE;
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS requires_human_analysis BOOLEAN DEFAULT FALSE;
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS file_types_allowed TEXT[] DEFAULT '{}';
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS max_file_size BIGINT;
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS tags TEXT[] DEFAULT '{}';
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS metadata JSONB DEFAULT '{}';
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS blockchain_tx_hash VARCHAR(66);
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS escrow_address VARCHAR(42);
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS started_at TIMESTAMPTZ;
ALTER TABLE bounties ADD COLUMN IF NOT EXISTS completed_at TIMESTAMPTZ;

-- Rename creator_id to creator (model uses 'creator')
ALTER TABLE bounties RENAME COLUMN creator_id TO creator;
-- Copy reward_amount data into total_reward, then drop old column
UPDATE bounties SET total_reward = COALESCE(reward_amount::text, '0') WHERE total_reward = '0' OR total_reward IS NULL;
ALTER TABLE bounties DROP COLUMN IF EXISTS reward_amount;
-- Rename file_hash to something that doesn't conflict (model doesn't use it directly)
-- The model stores hash info in metadata; keep file_hash for backward compat but don't require it
-- Rename expires_at references: initial schema used 'deadline' so no rename needed

-- ============================================
-- Extend analyses table
-- ============================================
ALTER TABLE analyses ADD COLUMN IF NOT EXISTS analysis_hash VARCHAR(128);
ALTER TABLE analyses ADD COLUMN IF NOT EXISTS stake_amount VARCHAR(78) DEFAULT '0';
ALTER TABLE analyses ADD COLUMN IF NOT EXISTS risk_score INTEGER;
ALTER TABLE analyses ADD COLUMN IF NOT EXISTS threat_types TEXT[] DEFAULT '{}';
ALTER TABLE analyses ADD COLUMN IF NOT EXISTS metadata JSONB DEFAULT '{}';
ALTER TABLE analyses ADD COLUMN IF NOT EXISTS started_at TIMESTAMPTZ;
ALTER TABLE analyses ADD COLUMN IF NOT EXISTS completed_at TIMESTAMPTZ;

-- ============================================
-- Wallet transactions table
-- ============================================
CREATE TABLE IF NOT EXISTS wallet_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    tx_hash VARCHAR(66),
    tx_type VARCHAR(30) NOT NULL, -- 'deposit', 'withdrawal', 'stake', 'unstake', 'reward', 'slash'
    amount VARCHAR(78) NOT NULL DEFAULT '0',
    from_address VARCHAR(42),
    to_address VARCHAR(42),
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'confirmed', 'failed'
    block_number BIGINT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_wallet_tx_user ON wallet_transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_wallet_tx_hash ON wallet_transactions(tx_hash);
CREATE INDEX IF NOT EXISTS idx_wallet_tx_status ON wallet_transactions(status);

-- ============================================
-- Reputation history table
-- ============================================
CREATE TABLE IF NOT EXISTS reputation_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL,
    event_type VARCHAR(30) NOT NULL, -- 'submission', 'bounty_win', 'penalty', 'bonus', 'slash'
    score_change DECIMAL(10,2) NOT NULL DEFAULT 0,
    new_score DECIMAL(10,2) NOT NULL DEFAULT 0,
    reason TEXT NOT NULL DEFAULT '',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rep_history_user ON reputation_history(user_id);
CREATE INDEX IF NOT EXISTS idx_rep_history_type ON reputation_history(event_type);

-- ============================================
-- Webhooks table
-- ============================================
CREATE TABLE IF NOT EXISTS webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL,
    url TEXT NOT NULL,
    events TEXT[] NOT NULL DEFAULT '{}',
    secret VARCHAR(128),
    is_active BOOLEAN DEFAULT TRUE,
    description TEXT,
    headers JSONB DEFAULT '{}',
    retry_max_attempts INTEGER DEFAULT 3,
    retry_interval_seconds INTEGER DEFAULT 60,
    exponential_backoff BOOLEAN DEFAULT TRUE,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_webhooks_user ON webhooks(user_id);
CREATE INDEX IF NOT EXISTS idx_webhooks_active ON webhooks(is_active);

-- ============================================
-- Webhook deliveries table
-- ============================================
CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID REFERENCES webhooks(id) ON DELETE CASCADE NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'success', 'failed', 'retrying'
    status_code SMALLINT,
    response_body TEXT,
    error_message TEXT,
    attempt_number INTEGER DEFAULT 1,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_webhook_del_webhook ON webhook_deliveries(webhook_id);
CREATE INDEX IF NOT EXISTS idx_webhook_del_status ON webhook_deliveries(status);

-- ============================================
-- Sync state table (for blockchain sync service)
-- ============================================
CREATE TABLE IF NOT EXISTS sync_state (
    service VARCHAR(50) PRIMARY KEY,
    block_number BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
