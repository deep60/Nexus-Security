-- Initial migration for bounty-manager
-- This is a placeholder migration file to satisfy the sqlx::migrate! macro

CREATE TABLE IF NOT EXISTS bounties (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
