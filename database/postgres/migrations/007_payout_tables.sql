-- payout_tables.sql - Payout tracking originated in bounty-manager service
-- Promoted to root schema for cross-service visibility.

CREATE TABLE IF NOT EXISTS payouts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    bounty_id UUID NOT NULL REFERENCES bounties(id) ON DELETE CASCADE,
    submission_id UUID REFERENCES submissions(id) ON DELETE SET NULL,
    recipient VARCHAR(255) NOT NULL,
    amount BIGINT NOT NULL,
    currency VARCHAR(100) NOT NULL DEFAULT 'ETH',
    payout_type VARCHAR(30) NOT NULL, -- 'reward', 'stake_return', 'participation_bonus'
    status VARCHAR(20) NOT NULL DEFAULT 'Pending',
    transaction_hash VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    processed_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB
);

CREATE INDEX IF NOT EXISTS idx_payouts_bounty ON payouts(bounty_id);
CREATE INDEX IF NOT EXISTS idx_payouts_recipient ON payouts(recipient);
CREATE INDEX IF NOT EXISTS idx_payouts_status ON payouts(status);
