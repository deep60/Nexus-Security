-- Migration: add wallet_balances table for balance reconciliation worker

CREATE TABLE IF NOT EXISTS wallet_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address VARCHAR(42) NOT NULL UNIQUE,
    balance VARCHAR(78) NOT NULL DEFAULT '0',
    token_address VARCHAR(42) NOT NULL DEFAULT '0x0000000000000000000000000000000000000000',
    tracked BOOLEAN NOT NULL DEFAULT true,
    last_reconciled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_wallet_balances_address ON wallet_balances(address);
CREATE INDEX IF NOT EXISTS idx_wallet_balances_tracked ON wallet_balances(tracked) WHERE tracked = true;
