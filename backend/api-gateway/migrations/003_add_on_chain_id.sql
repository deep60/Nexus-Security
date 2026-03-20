-- Migration 003: Add on_chain_id to bounties for correct chain ID mapping
-- The contract uses incremental bountyCounter (1,2,3...) not UUIDs.
-- We must store this value after parsing the BountyCreated event.

ALTER TABLE bounties ADD COLUMN IF NOT EXISTS on_chain_id BIGINT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_bounties_on_chain_id ON bounties(on_chain_id) WHERE on_chain_id IS NOT NULL;
